//rusturashi
pub use bevy;
pub use bevy_rapier3d;

pub mod core;
pub mod objects;

use std::sync::Mutex;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::core::bevy_bridge::{fixed_tick_tree, sync_tree_with_bevy, GameTree};
use crate::core::scene::ObjectTree;

fn disable_global_gravity(mut rapier_config: Single<&mut RapierConfiguration>) {
    rapier_config.gravity = Vec3::ZERO;
}

pub struct EnginePlugin {
    initial_tree: Mutex<Option<ObjectTree>>,
}

impl EnginePlugin {
    pub fn new(tree: ObjectTree) -> Self {
        Self {
            initial_tree: Mutex::new(Some(tree)),
        }
    }
}

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        let tree = self
            .initial_tree
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(ObjectTree::new);

        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(GameTree::new(tree))
            .add_systems(Startup, disable_global_gravity)
            .add_systems(Update, sync_tree_with_bevy)
            .add_systems(FixedUpdate, fixed_tick_tree);
    }
}

/// Convenient re-exports of the most commonly used engine types
pub mod prelude {
    pub use crate::core::bevy_bridge::{sync_tree_with_bevy, GameTree};
    pub use crate::core::physics::{ColliderShape, PhysicsBodyType, Transform3D, CHUNK_SIZE};
    pub use crate::core::scene::*;
    pub use crate::objects::*;
    pub use crate::EnginePlugin;
    pub use crate::core::input::InputSnapshot;
}
