use std::collections::HashSet;
use bevy::prelude::{KeyCode, MouseButton, Vec2};

#[derive(Default, Clone)]
pub struct InputSnapshot {
    pub pressed_keys: HashSet<KeyCode>,
    pub just_pressed_keys: HashSet<KeyCode>,
    pub pressed_mouse: HashSet<MouseButton>,
    pub just_pressed_mouse: HashSet<MouseButton>,
    pub mouse_delta: Vec2,
    pub scroll_delta: f32,
}

impl InputSnapshot {
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse.contains(&button)
    }

    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_mouse.contains(&button)
    }
}
