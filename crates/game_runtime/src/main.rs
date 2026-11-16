mod player;

use engine::bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use engine::bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use engine::bevy::prelude::*;
use engine::bevy::render::{
    RenderPlugin,
    render_resource::WgpuFeatures,
    settings::{RenderCreation, WgpuSettings},
};
use engine::bevy::time::common_conditions::on_timer;
use engine::bevy::window::{CursorGrabMode, CursorOptions, PresentMode};
use engine::bevy_rapier3d::render::{DebugRenderContext, RapierDebugRenderPlugin};
use engine::prelude::*;
use player::Player6DoF;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Default, PartialEq, Eq)]
enum RenderMode {
    #[default]
    Lit,
    Unshaded,
    Wireframe,
    WireframeOnly,
}

impl RenderMode {
    fn cycle(&mut self) {
        *self = match self {
            Self::Lit => Self::Unshaded,
            Self::Unshaded => Self::Wireframe,
            Self::Wireframe => Self::WireframeOnly,
            Self::WireframeOnly => Self::Lit,
        };
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Lit => "Lit",
            Self::Unshaded => "Unshaded",
            Self::Wireframe => "Wireframe",
            Self::WireframeOnly => "Wireframe Only",
        }
    }
}

struct OriginalMeshRenderState {
    visibility: Option<Visibility>,
    material: Option<Handle<StandardMaterial>>,
    unshaded_material: Option<Handle<StandardMaterial>>,
    wireframe_only_material: Option<Handle<StandardMaterial>>,
}

fn setup_fps_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: --"),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::srgb(0.2, 1.0, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        Name::new("fps_hud"),
        Visibility::Hidden,
    ));

    commands.spawn((
        Text::new("Origin Chunk: [0, 0, 0]\nCamera Chunk: [0, 0, 0]"),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgb(0.2, 1.0, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(42.0),
            left: Val::Px(12.0),
            ..default()
        },
        Name::new("chunk_hud"),
        Visibility::Hidden,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Name::new("render_mode_hud"),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Lit"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Name::new("render_mode_text"),
            ));
        });

    commands.spawn((
        Text::new("PROJECT FRACTA  /  INSPECTOR"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            right: Val::Px(16.0),
            width: Val::Px(360.0),
            max_height: Val::Vh(85.0),
            padding: UiRect::all(Val::Px(14.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.045, 0.065, 0.94)),
        BorderColor::all(Color::srgba(0.25, 0.55, 0.75, 0.8)),
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.35),
            Val::Px(0.0),
            Val::Px(4.0),
            Val::Px(0.0),
            Val::Px(12.0),
        ),
        Name::new("scene_tree_hud"),
        Visibility::Hidden,
    ));
}

fn toggle_debug_ui(keys: Res<ButtonInput<KeyCode>>, mut ui: Query<(&Name, &mut Visibility)>) {
    for (name, mut visibility) in &mut ui {
        let toggle_key = match name.as_str() {
            "fps_hud" | "chunk_hud" => KeyCode::F1,
            "scene_tree_hud" => KeyCode::F2,
            _ => continue,
        };

        if keys.just_pressed(toggle_key) {
            *visibility = match *visibility {
                Visibility::Visible => Visibility::Hidden,
                Visibility::Hidden | Visibility::Inherited => Visibility::Visible,
            };
        }
    }
}

fn update_render_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut render_mode: Local<RenderMode>,
    mut original_states: Local<HashMap<Entity, OriginalMeshRenderState>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut mode_hud: Query<
        (&Name, Option<&mut Text>, Option<&mut Visibility>),
        Without<Mesh3d>,
    >,
    meshes: Query<
        (
            Entity,
            Option<&MeshMaterial3d<StandardMaterial>>,
            Option<&Visibility>,
        ),
        With<Mesh3d>,
    >,
) {
    if keys.just_pressed(KeyCode::F3) {
        render_mode.cycle();
    }

    wireframe_config.global = matches!(
        *render_mode,
        RenderMode::Wireframe | RenderMode::WireframeOnly
    );
    original_states.retain(|entity, _| meshes.get(*entity).is_ok());

    for (entity, current_material, current_visibility) in &meshes {
        let original = original_states
            .entry(entity)
            .or_insert_with(|| OriginalMeshRenderState {
                visibility: current_visibility.copied(),
                material: current_material.map(|material| material.0.clone()),
                unshaded_material: None,
                wireframe_only_material: None,
            });

        let target_material = match *render_mode {
            RenderMode::Lit | RenderMode::Wireframe => original.material.clone(),
            RenderMode::Unshaded => {
                if original.unshaded_material.is_none() {
                    if let Some(base_material) = original
                        .material
                        .as_ref()
                        .and_then(|material| materials.get(material))
                    {
                        let mut unshaded = base_material.clone();
                        unshaded.unlit = true;
                        original.unshaded_material = Some(materials.add(unshaded));
                    }
                }
                original.unshaded_material.clone()
            }
            RenderMode::WireframeOnly => {
                if original.wireframe_only_material.is_none() {
                    if let Some(base_material) = original
                        .material
                        .as_ref()
                        .and_then(|material| materials.get(material))
                    {
                        let mut wireframe_only = base_material.clone();
                        wireframe_only.base_color.set_alpha(0.0);
                        wireframe_only.alpha_mode = AlphaMode::Blend;
                        wireframe_only.unlit = true;
                        original.wireframe_only_material = Some(materials.add(wireframe_only));
                    }
                }
                original.wireframe_only_material.clone()
            }
        };

        if let Some(target_material) = target_material {
            if current_material.is_none_or(|current| current.0 != target_material) {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(target_material));
            }
        } else if original.material.is_none() && current_material.is_some() {
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }

        if let Some(visibility) = original.visibility {
            if current_visibility != Some(&visibility) {
                commands.entity(entity).insert(visibility);
            }
        } else if current_visibility.is_some() {
            commands.entity(entity).remove::<Visibility>();
        }
    }

    for (name, text, visibility) in &mut mode_hud {
        match name.as_str() {
            "render_mode_hud" => {
                if let Some(mut visibility) = visibility {
                    *visibility = if *render_mode == RenderMode::Lit {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    };
                }
            }
            "render_mode_text" => {
                if let Some(mut text) = text {
                    **text = render_mode.label().to_owned();
                }
            }
            _ => {}
        }
    }
}

fn toggle_rapier_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    if keys.just_pressed(KeyCode::F4) {
        rapier_debug.enabled = !rapier_debug.enabled;
    }
}

fn update_fps_ui(diagnostics: Res<DiagnosticsStore>, mut hud_text: Query<(&mut Text, &Name)>) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    for (mut hud_text, name) in &mut hud_text {
        if name.as_str() == "fps_hud" {
            **hud_text = format!("FPS: {:.0}", fps);
        }
    }
}

fn update_chunk_ui(game: Res<GameTree>, mut hud_text: Query<(&mut Text, &Name)>) {
    let [cx, cy, cz] = game.origin_chunk;
    let chunk_text = if let Some(cam_tf) = game.tree.find_active_camera_transform() {
        format!(
            "Origin Chunk: [{cx}, {cy}, {cz}]\nCamera Chunk: [{}, {}, {}] ({:.1}, {:.1}, {:.1})",
            cam_tf.chunk[0],
            cam_tf.chunk[1],
            cam_tf.chunk[2],
            cam_tf.position[0],
            cam_tf.position[1],
            cam_tf.position[2],
        )
    } else {
        format!("Origin Chunk: [{cx}, {cy}, {cz}]")
    };

    for (mut hud_text, name) in &mut hud_text {
        if name.as_str() == "chunk_hud" {
            **hud_text = chunk_text.clone();
        }
    }
}

fn update_scene_tree_ui(game: Res<GameTree>, mut hud_text: Query<(&mut Text, &Name)>) {
    let scene_nodes = game.tree.scene_tree_snapshot();
    let scene_names: std::collections::HashMap<_, _> = scene_nodes
        .iter()
        .map(|(id, _, name)| (*id, name.clone()))
        .collect();

    let mut entities: Vec<_> = game
        .entity_map
        .iter()
        .filter_map(|(&object_id, &entity)| {
            let name = scene_names.get(&object_id)?;
            Some((entity, name))
        })
        .collect();
    entities.sort_by_key(|(entity, _)| entity.to_bits());

    let mut inspector_text = format!(
        "PROJECT FRACTA  /  INSPECTOR\n\
         --------------------------------\n\
         SCENE TREE  ({})\n",
        scene_nodes.len()
    );
    if scene_nodes.is_empty() {
        inspector_text.push_str("  (empty)\n");
    } else {
        for (id, depth, name) in scene_nodes {
            inspector_text.push_str(&format!("{}+ {}  [#{}]\n", "  ".repeat(depth), name, id.0));
        }
    }

    inspector_text.push_str(&format!("\nBEVY ENTITIES  ({})\n", entities.len()));
    for (entity, name) in entities {
        inspector_text.push_str(&format!("- {:?}  {name}\n", entity));
    }

    for (mut hud_text, name) in &mut hud_text {
        if name.as_str() == "scene_tree_hud" {
            **hud_text = inspector_text.clone();
        }
    }
}

fn main() {
    let mut tree = ObjectTree::new();

    // Player and attached camera
    let mut player = Player6DoF::new();
    player.body.transform.translate(0.0, 2.0, 6.0);
    let player_id = tree.add_object("Player", None, Box::new(player));

    tree.add_object("MainCamera", Some(player_id), Box::new(Camera3D::new(90.0)));

    // Directional light
    let mut sun = DirectionalLight3D::new(10_000.0);
    sun.transform.translate(4.0, 8.0, 4.0);
    sun.transform.look_at(&Transform3D::identity(), Vec3::Y);
    tree.add_object("SunLight", None, Box::new(sun));

    // Floor and matching mesh
    let mut floor = StaticBody3D::new(ColliderShape::Cuboid {
        half_extents: Vec3::new(10.0, 0.25, 10.0),
    });
    floor.transform.translate(0.0, -2.0, 0.0);
    let floor_id = tree.add_object("Floor", None, Box::new(floor));

    let mut floor_mesh = MeshObject3D::new(MeshShape::Plane { size: 20.0 }, [0.35, 0.35, 0.4, 1.0]);
    floor_mesh.transform.translate(0.0, 0.25, 0.0);
    tree.add_object("FloorMesh", Some(floor_id), Box::new(floor_mesh));

    // Falling cube and attached mesh
    let mut cube_body = RigidBody3D::new(
        ColliderShape::Cuboid {
            half_extents: Vec3::splat(0.75),
        },
        1.0,
    );
    cube_body.transform.translate(0.0, 4.0, 0.0);
    cube_body.transform.rotate_x(0.3);
    cube_body.transform.rotate_z(0.2);
    let cube_id = tree.add_object("FallingCube", None, Box::new(cube_body));

    let cube_mesh = MeshObject3D::cube(1.5, [0.2, 0.7, 0.9, 1.0]);
    tree.add_object("CubeMesh", Some(cube_id), Box::new(cube_mesh));

    // Additional floating sphere
    let mut sphere = MeshObject3D::sphere(1.5, [0.2, 0.7, 0.9, 1.0]);
    sphere.transform.translate(4.0, 4.0, 4.0);
    tree.add_object("TestSphere", None, Box::new(sphere));

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Project Fracta".into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    primary_cursor_options: Some(CursorOptions {
                        visible: false,
                        grab_mode: CursorGrabMode::Locked,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    })),
                    ..default()
                }),
            WireframePlugin::default(),
            RapierDebugRenderPlugin {
                enabled: false,
                ..default()
            },
            FrameTimeDiagnosticsPlugin::default(),
            EnginePlugin::new(tree),
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE.into(),
            ..default()
        })
        .add_systems(Startup, setup_fps_ui)
        .add_systems(
            Update,
            (
                toggle_debug_ui,
                update_render_mode,
                toggle_rapier_debug,
                update_chunk_ui,
                update_fps_ui.run_if(on_timer(Duration::from_millis(250))),
                update_scene_tree_ui.run_if(on_timer(Duration::from_millis(250))),
            ),
        )
        .run();
}
