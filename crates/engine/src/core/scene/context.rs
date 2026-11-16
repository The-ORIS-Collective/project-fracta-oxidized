use std::collections::HashMap;
use bevy::prelude::Vec3;
use crate::core::input::InputSnapshot;
use crate::core::physics::{ColliderShape, PhysicsBodyType};
use crate::core::scene::{ObjectBehavior, ObjectId, SceneObject};

pub enum TreeCommand {
    Remove(ObjectId),
    AddChild {
        parent: Option<ObjectId>,
        name: String,
        behavior: Box<dyn ObjectBehavior>,
    },
}


#[derive(Clone, Debug)]
pub enum MeshShape {
    Cube { size: f32 },
    Sphere { radius: f32 },
    Plane { size: f32 },
    GltfPath(String),
}


#[derive(Clone)]
pub enum BevyCommand {
    SpawnCamera {
        object_id: ObjectId,
        fov_degrees: f32,
        near: f32,
        far: f32,
    },
    SpawnMesh {
        object_id: ObjectId,
        shape: MeshShape,
        color: [f32; 4],
    },
    SpawnDirectionalLight {
        object_id: ObjectId,
        color: [f32; 4],
        illuminance: f32,
        shadows_enabled: bool,
    },
    SpawnPhysicsBody {
        object_id: ObjectId,
        body_type: PhysicsBodyType,
        collider: ColliderShape,
        mass: f32,
        restitution: f32,
    },
    SetBodyType {
        object_id: ObjectId,
        body_type: PhysicsBodyType,
    },
    SetVelocity {
        object_id: ObjectId,
        linvel: Vec3,
        angvel: Vec3,
    },
    DespawnVisual(ObjectId),
}

pub struct TreeContext<'a> {
    pub(crate) objects: &'a [Option<SceneObject>],
    pub(crate) physics_velocities: &'a HashMap<ObjectId, (Vec3, Vec3)>,
    pub(crate) commands: Vec<TreeCommand>,
    pub input: &'a InputSnapshot,
    pub(crate) render_commands: Vec<BevyCommand>,
}

impl<'a> TreeContext<'a> {
    pub(crate) fn new(
        objects: &'a [Option<SceneObject>],
        physics_velocities: &'a HashMap<ObjectId, (Vec3, Vec3)>,
        input: &'a InputSnapshot,
    ) -> Self {
        Self {
            objects,
            physics_velocities,
            commands: Vec::new(),
            input,
            render_commands: Vec::new(),
        }
    }

    // * --- Hierarchy Lookups ---

    pub fn get_parent(&self, id: ObjectId) -> Option<ObjectId> {
        self.objects.get(id.0)?.as_ref()?.parent
    }

    pub fn get_children(&self, id: ObjectId) -> &[ObjectId] {
        self.objects
            .get(id.0)
            .and_then(|o| o.as_ref())
            .map(|o| o.children.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_name(&self, id: ObjectId) -> Option<&str> {
        Some(self.objects.get(id.0)?.as_ref()?.name.as_str())
    }

    /// Downcasts another object's behavior to its concrete struct type (e.g., `ctx.get_object::<Player>(id)`).
    /// Note: Returns `None` if called on the currently ticking object since it is temporarily taken during `frame_tick`.
    pub fn get_object<T: 'static>(&self, id: ObjectId) -> Option<&T> {
        let obj = self.objects.get(id.0)?.as_ref()?;
        obj.behavior.as_any().downcast_ref::<T>()
    }

    // * --- Deferred Tree Mutations ---

    pub fn remove(&mut self, id: ObjectId) {
        self.commands.push(TreeCommand::Remove(id));
    }

    pub fn add_child(&mut self, parent: Option<ObjectId>, name: &str, behavior: Box<dyn ObjectBehavior>) {
        self.commands.push(TreeCommand::AddChild {
            parent,
            name: name.to_string(),
            behavior,
        });
    }

    // * --- Deferred Render Mutations ---

    pub fn spawn_camera(&mut self, object_id: ObjectId, fov_degrees: f32, near: f32, far: f32) {
        self.render_commands.push(BevyCommand::SpawnCamera {
            object_id,
            fov_degrees,
            near,
            far,
        });
    }

    pub fn spawn_mesh(&mut self, object_id: ObjectId, shape: MeshShape, color: [f32; 4]) {
        self.render_commands.push(BevyCommand::SpawnMesh {
            object_id,
            shape,
            color,
        });
    }

    pub fn spawn_directional_light(
        &mut self,
        object_id: ObjectId,
        color: [f32; 4],
        illuminance: f32,
        shadows_enabled: bool,
    ) {
        self.render_commands.push(BevyCommand::SpawnDirectionalLight {
            object_id,
            color,
            illuminance,
            shadows_enabled,
        });
    }

    pub fn spawn_physics_body(
        &mut self,
        object_id: ObjectId,
        body_type: PhysicsBodyType,
        collider: ColliderShape,
        mass: f32,
        restitution: f32,
    ) {
        self.render_commands.push(BevyCommand::SpawnPhysicsBody {
            object_id,
            body_type,
            collider,
            mass,
            restitution,
        });
    }

    pub fn set_body_type(&mut self, object_id: ObjectId, body_type: PhysicsBodyType) {
        self.render_commands.push(BevyCommand::SetBodyType {
            object_id,
            body_type,
        });
    }

    pub fn set_velocity(&mut self, object_id: ObjectId, linvel: Vec3, angvel: Vec3) {
        self.render_commands.push(BevyCommand::SetVelocity {
            object_id,
            linvel,
            angvel,
        });
    }

    pub fn get_velocity(&self, object_id: ObjectId) -> Option<(Vec3, Vec3)> {
        self.physics_velocities.get(&object_id).copied()
    }


    pub fn despawn_visual(&mut self, object_id: ObjectId) {
        self.render_commands.push(BevyCommand::DespawnVisual(object_id));
    }
}
