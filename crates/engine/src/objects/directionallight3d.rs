use crate::core::physics::Transform3D;
use crate::core::scene::{ObjectBehavior, ObjectId, TreeContext};

pub struct DirectionalLight3D {
    pub transform: Transform3D,
    pub color: [f32; 4],
    pub illuminance: f32,
    pub shadows_enabled: bool,
}

impl DirectionalLight3D {
    pub fn new(illuminance: f32) -> Self {
        Self {
            transform: Transform3D::identity(),
            color: [1.0, 1.0, 1.0, 1.0],
            illuminance,
            shadows_enabled: true,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b, 1.0];
        self
    }

    pub fn with_shadows(mut self, enabled: bool) -> Self {
        self.shadows_enabled = enabled;
        self
    }
}

impl ObjectBehavior for DirectionalLight3D {
    fn start(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        ctx.spawn_directional_light(id, self.color, self.illuminance, self.shadows_enabled);
    }

    fn transform(&self) -> Option<&Transform3D> {
        Some(&self.transform)
    }
}
