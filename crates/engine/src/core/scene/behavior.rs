use std::any::Any;
use crate::core::physics::Transform3D;
use crate::core::scene::{ObjectId, TreeContext};

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait ObjectBehavior: AsAny + Send + Sync {
    fn start(&mut self, _id: ObjectId, _ctx: &mut TreeContext) {}
    fn end(&mut self, _id: ObjectId, _ctx: &mut TreeContext) {}
    fn frame_tick(&mut self, _id: ObjectId, _delta: f32, _ctx: &mut TreeContext) {}
    fn fixed_tick(&mut self, _id: ObjectId, _delta: f32, _ctx: &mut TreeContext) {}

    fn transform(&self) -> Option<&Transform3D> {
        None
    }

    fn transform_mut(&mut self) -> Option<&mut Transform3D> {
        None
    }
}
