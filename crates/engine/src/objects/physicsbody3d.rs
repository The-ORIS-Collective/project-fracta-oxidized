use crate::core::physics::{ColliderShape, PhysicsBodyType, Transform3D};
use crate::core::scene::{ObjectBehavior, ObjectId, TreeContext};

/// StaticBody3D (simulated by Rapier's Fixed RigidBody).
pub struct StaticBody3D {
    pub transform: Transform3D,
    pub collider: ColliderShape,
    pub restitution: f32,
}

impl StaticBody3D {
    pub fn new(collider: ColliderShape) -> Self {
        Self {
            transform: Transform3D::identity(),
            collider,
            restitution: 0.2,
        }
    }
}

impl ObjectBehavior for StaticBody3D {
    fn start(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        ctx.spawn_physics_body(
            id,
            PhysicsBodyType::Static,
            self.collider.clone(),
            0.0,
            self.restitution,
        );
    }

    fn transform(&self) -> Option<&Transform3D> {
        Some(&self.transform)
    }

    fn transform_mut(&mut self) -> Option<&mut Transform3D> {
        Some(&mut self.transform)
    }
}

/// RigidBody3D (simulated by Rapier gravity & collisions).
pub struct RigidBody3D {
    pub transform: Transform3D,
    pub collider: ColliderShape,
    pub mass: f32,
    pub restitution: f32,
    pub frozen: bool,
}

impl RigidBody3D {
    pub fn new(collider: ColliderShape, mass: f32) -> Self {
        Self {
            transform: Transform3D::identity(),
            collider,
            mass,
            restitution: 0.4,
            frozen: false,
        }
    }

    pub fn set_frozen(&mut self, id: ObjectId, frozen: bool, ctx: &mut TreeContext) {
        if self.frozen != frozen {
            self.frozen = frozen;
            let body_type = if frozen {
                PhysicsBodyType::Static
            } else {
                PhysicsBodyType::Dynamic
            };
            ctx.set_body_type(id, body_type);
        }
    }
}

impl ObjectBehavior for RigidBody3D {
    fn start(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        let initial_type = if self.frozen {
            PhysicsBodyType::Static
        } else {
            PhysicsBodyType::Dynamic
        };

        ctx.spawn_physics_body(
            id,
            initial_type,
            self.collider.clone(),
            self.mass,
            self.restitution,
        );
    }

    fn transform(&self) -> Option<&Transform3D> {
        Some(&self.transform)
    }

    fn transform_mut(&mut self) -> Option<&mut Transform3D> {
        Some(&mut self.transform)
    }
}

