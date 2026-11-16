use crate::core::physics::Transform3D;
use crate::core::scene::{ObjectBehavior, ObjectId, TreeContext};

pub struct Camera3D {
    pub transform: Transform3D,
    pub fov_degrees: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub is_current: bool,
}

impl Camera3D {
    pub fn new(fov_degrees: f32) -> Self {
        Self {
            transform: Transform3D::identity(),
            fov_degrees,
            near_plane: 0.1,
            far_plane: 100_000.0,
            is_current: true,
        }
    }
}

impl ObjectBehavior for Camera3D {
    fn start(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        ctx.spawn_camera(id, self.fov_degrees, self.near_plane, self.far_plane);
    }

    fn end(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        ctx.despawn_visual(id);
    }

    fn transform(&self) -> Option<&Transform3D> {
        Some(&self.transform)
    }
}
