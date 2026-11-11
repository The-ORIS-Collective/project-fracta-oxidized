pub mod core;
pub mod objects;

use bevy::prelude::*;

use crate::core::scene::{
    ObjectRegistry,
    register_builtin_objects,
};

pub fn setup_engine(app: &mut App) -> Result<(), String> {
    let mut object_registry = ObjectRegistry::new();

    register_builtin_objects(&mut object_registry)?;

    app.insert_resource(object_registry);

    Ok(())
}