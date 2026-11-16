use std::collections::HashMap;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::core::input::InputSnapshot;
use crate::core::physics::{CHUNK_SIZE, PhysicsBodyType, Transform3D};
use crate::core::scene::{MeshShape, ObjectId, ObjectTree, RenderCommand};

#[derive(Resource)]
pub struct GameTree {
    pub tree: ObjectTree,
    pub entity_map: HashMap<ObjectId, Entity>,
    spawn_commands: HashMap<ObjectId, RenderCommand>,
    physics_velocities: HashMap<ObjectId, Velocity>,
    pub current_input: InputSnapshot,
    pub origin_chunk: [i64; 3],
}

impl GameTree {
    pub fn new(tree: ObjectTree) -> Self {
        Self {
            tree,
            entity_map: HashMap::new(),
            spawn_commands: HashMap::new(),
            physics_velocities: HashMap::new(),
            current_input: InputSnapshot::default(),
            origin_chunk: [0, 0, 0]
        }
    }
}

#[derive(Component)]
pub struct LinkedObject(pub ObjectId);

/// Marker component distinguishing Rapier-driven dynamic bodies from tree-driven objects.
#[derive(Component)]
pub struct DynamicPhysicsBody;


pub fn fixed_tick_tree(
    mut game: ResMut<GameTree>,
    fixed_time: Res<Time<Fixed>>,
) {
    let input = game.current_input.clone();
    game.tree.fixed_update(fixed_time.delta_secs(), &input);
}

pub fn sync_tree_with_bevy(
    mut commands: Commands,
    mut game: ResMut<GameTree>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut bevy_transforms: Query<(
        &mut Transform,
        Option<&DynamicPhysicsBody>,
        Option<&Velocity>,
    )>,
) {
    let input = InputSnapshot {
        pressed_keys: keys.get_pressed().copied().collect(),
        just_pressed_keys: keys.get_just_pressed().copied().collect(),
        pressed_mouse: mouse_buttons.get_pressed().copied().collect(),
        just_pressed_mouse: mouse_buttons.get_just_pressed().copied().collect(),
        mouse_delta: mouse_motion.delta,
        scroll_delta: mouse_scroll.delta.y,
    };

    game.current_input = input.clone();
    game.tree.update(time.delta_secs(), &input);

    // 2. Process any SpawnCamera, SpawnMesh, SpawnDirectionalLight, SpawnPhysicsBody, or DespawnVisual commands
    let render_cmds: Vec<RenderCommand> = game.tree.pending_render_commands.drain(..).collect();
    for cmd in render_cmds {
        let spawn_object_id = match &cmd {
            RenderCommand::SpawnCamera { object_id, .. }
            | RenderCommand::SpawnMesh { object_id, .. }
            | RenderCommand::SpawnDirectionalLight { object_id, .. }
            | RenderCommand::SpawnPhysicsBody { object_id, .. } => Some(*object_id),
            _ => None,
        };
        if let Some(object_id) = spawn_object_id {
            game.spawn_commands.insert(object_id, cmd.clone());
        }

        match cmd {
            RenderCommand::SpawnCamera {
                object_id,
                fov_degrees,
                near,
                far,
            } => {
                let entity = commands
                    .spawn((
                        Camera3d::default(),
                        Projection::Perspective(PerspectiveProjection {
                            fov: fov_degrees.to_radians(),
                            near,
                            far,
                            ..default()
                        }),
                        Transform::IDENTITY,
                        LinkedObject(object_id),
                    ))
                    .id();
                game.entity_map.insert(object_id, entity);
            }

            RenderCommand::SpawnMesh {
                object_id,
                shape,
                color,
            } => {
                let entity = match shape {
                    MeshShape::GltfPath(path) => commands
                        .spawn((
                            WorldAssetRoot(asset_server.load(path)),
                            Transform::IDENTITY,
                            LinkedObject(object_id),
                        ))
                        .id(),

                    primitive => {
                        let mesh_handle = match primitive {
                            MeshShape::Cube { size } => meshes.add(Cuboid::from_length(size)),
                            MeshShape::Sphere { radius } => meshes.add(Sphere::new(radius)),
                            MeshShape::Plane { size } => {
                                meshes.add(Plane3d::default().mesh().size(size, size))
                            }
                            MeshShape::GltfPath(_) => unreachable!(),
                        };

                        let material_handle = materials.add(StandardMaterial {
                            base_color: Color::srgba(color[0], color[1], color[2], color[3]),
                            ..default()
                        });

                        commands
                            .spawn((
                                Mesh3d(mesh_handle),
                                MeshMaterial3d(material_handle),
                                Transform::IDENTITY,
                                LinkedObject(object_id),
                            ))
                            .id()
                    }
                };
                game.entity_map.insert(object_id, entity);
            }

            RenderCommand::SpawnDirectionalLight {
                object_id,
                color,
                illuminance,
                shadows_enabled,
            } => {
                let entity = commands
                    .spawn((
                        DirectionalLight {
                            color: Color::srgba(color[0], color[1], color[2], color[3]),
                            illuminance,
                            shadow_maps_enabled: shadows_enabled,
                            ..default()
                        },
                        Transform::IDENTITY,
                        LinkedObject(object_id),
                    ))
                    .id();
                game.entity_map.insert(object_id, entity);
            }

            RenderCommand::SpawnPhysicsBody {
                object_id,
                body_type,
                collider,
                mass,
                restitution,
            } => {
                let rapier_body = match body_type {
                    PhysicsBodyType::Static => RigidBody::Fixed,
                    PhysicsBodyType::Dynamic => RigidBody::Dynamic,
                    PhysicsBodyType::Kinematic => RigidBody::KinematicPositionBased,
                };

                // Place the physics body at its initial floating-origin offset immediately
                let origin = Transform3D::from_chunk(game.origin_chunk);
                let initial_tf = game
                    .tree
                    .get_global_transform(object_id)
                    .map(|tf| {
                        let [dx, dy, dz] = origin.delta_to(&tf);
                        Transform {
                            translation: Vec3::new(dx as f32, dy as f32, dz as f32),
                            rotation: Quat::from_array(tf.rotation),
                            scale: Vec3::from_array(tf.scale),
                        }
                    })
                    .unwrap_or(Transform::IDENTITY);
                let velocity = game
                    .physics_velocities
                    .remove(&object_id)
                    .unwrap_or_else(Velocity::zero);

                let mut entity_cmds = commands.spawn((
                    rapier_body,
                    collider.to_rapier_collider(),
                    Restitution::coefficient(restitution),
                    initial_tf,
                    LinkedObject(object_id),
                    velocity,
                ));

                if body_type == PhysicsBodyType::Dynamic {
                    entity_cmds.insert((
                        AdditionalMassProperties::Mass(mass),
                        DynamicPhysicsBody,
                    ));
                }

                game.entity_map.insert(object_id, entity_cmds.id());
            }

            RenderCommand::SetBodyType {
                object_id,
                body_type,
            } => {
                if let Some(&entity) = game.entity_map.get(&object_id) {
                    let rapier_body = match body_type {
                        PhysicsBodyType::Static => RigidBody::Fixed,
                        PhysicsBodyType::Dynamic => RigidBody::Dynamic,
                        PhysicsBodyType::Kinematic => RigidBody::KinematicPositionBased,
                    };

                    commands.entity(entity).insert(rapier_body);

                    if body_type == PhysicsBodyType::Dynamic {
                        commands.entity(entity).insert(DynamicPhysicsBody);
                    } else {
                        commands.entity(entity).remove::<DynamicPhysicsBody>();
                    }
                }
            }

            RenderCommand::SetVelocity {
                object_id,
                linvel,
                angvel,
            } => {
                if let Some(&entity) = game.entity_map.get(&object_id) {
                    commands.entity(entity).insert(Velocity {
                        linear: linvel,
                        angular: angvel,
                    });
                }
            }

            RenderCommand::DespawnVisual(object_id) => {
                game.spawn_commands.remove(&object_id);
                game.physics_velocities.remove(&object_id);
                if let Some(entity) = game.entity_map.remove(&object_id) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    
    let previous_origin = Transform3D::from_chunk(game.origin_chunk);

    let GameTree {
        tree,
        entity_map,
        spawn_commands,
        physics_velocities,
        origin_chunk,
        current_input: _,
    } = &mut *game;

    for (&obj_id, &entity) in entity_map.iter() {
        if let Ok((bevy_tf, is_dynamic, _)) = bevy_transforms.get(entity) {
            if is_dynamic.is_some() {
                if let Some(obj_tf) = tree.get_transform_mut(obj_id) {
                    *obj_tf = Transform3D::from_bevy_relative_to(bevy_tf, &previous_origin);
                }
            }
        }
    }

    let active_cam_tf = tree.find_active_camera_transform().unwrap_or_default();
    let current_origin = Transform3D::from_chunk(*origin_chunk);
    let [dx, dy, dz] = current_origin.delta_to(&active_cam_tf);

    let shift_threshold = (CHUNK_SIZE * 0.5 + (CHUNK_SIZE / 16.0)) as f64;
    let chunk_changed = dx.abs() > shift_threshold
        || dy.abs() > shift_threshold
        || dz.abs() > shift_threshold;
    
    if chunk_changed {
        *origin_chunk = active_cam_tf.chunk;
    }

    let new_origin = Transform3D::from_chunk(*origin_chunk);

    for (&obj_id, &entity) in entity_map.iter() {
        if let Ok((mut bevy_tf, is_dynamic, _)) = bevy_transforms.get_mut(entity) {
            if chunk_changed || is_dynamic.is_none() {
                if let Some(obj_transform) = tree.get_global_transform(obj_id) {
                    let target_tf = obj_transform.to_bevy_relative_to(&new_origin);
                    if *bevy_tf != target_tf {
                        *bevy_tf = target_tf;
                    }
                }
            }
        }
    }

    let entities_outside_current_chunk: Vec<_> = entity_map
        .iter()
        .filter_map(|(&obj_id, &entity)| {
            let obj_transform = tree.get_global_transform(obj_id)?;
            (!active_cam_tf.is_within_chunk_radius(&obj_transform, 0))
                .then_some((obj_id, entity))
        })
        .collect();

    for (obj_id, entity) in entities_outside_current_chunk {
        if let Ok((_, _, Some(velocity))) = bevy_transforms.get(entity) {
            physics_velocities.insert(obj_id, *velocity);
        }
        entity_map.remove(&obj_id);
        commands.entity(entity).despawn();
    }

    let entities_to_respawn: Vec<_> = spawn_commands
        .iter()
        .filter_map(|(&obj_id, spawn_command)| {
            if entity_map.contains_key(&obj_id) {
                return None;
            }

            let obj_transform = tree.get_global_transform(obj_id)?;
            active_cam_tf
                .is_within_chunk_radius(&obj_transform, 0)
                .then_some(spawn_command.clone())
        })
        .collect();
    tree.pending_render_commands.extend(entities_to_respawn);

}
