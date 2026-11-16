use bevy::prelude::Vec3;
use bevy_rapier3d::prelude::Collider;

#[derive(Clone, Debug)]
pub enum ColliderShape {
    Cuboid { half_extents: Vec3 },
    Ball { radius: f32 },
    CapsuleY { half_height: f32, radius: f32 },
}

impl ColliderShape {
    pub fn to_rapier_collider(&self) -> Collider {
        match *self {
            ColliderShape::Cuboid { half_extents } => {
                Collider::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
            ColliderShape::Ball { radius } => Collider::ball(radius),
            ColliderShape::CapsuleY { half_height, radius } => {
                Collider::capsule_y(half_height, radius)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsBodyType {
    Static,    // StaticBody3D -> Rapier's RigidBody::Fixed
    Dynamic,   // RigidBody3D -> Rapier's RigidBody::Dynamic
    Kinematic, // AnimatableBody3D -> RigidBody::KinematicPositionBased
}
