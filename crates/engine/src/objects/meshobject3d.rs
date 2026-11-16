use crate::core::physics::Transform3D;
use crate::core::scene::{MeshShape, ObjectBehavior, ObjectId, TreeContext};

pub struct MeshObject3D {
    pub transform: Transform3D,
    pub shape: MeshShape,
    pub color: [f32; 4],
}

impl MeshObject3D {
    pub fn new(shape: MeshShape, color: [f32; 4]) -> Self {
        Self {
            transform: Transform3D::identity(),
            shape,
            color,
        }
    }

    pub fn cube(size: f32, color: [f32; 4]) -> Self {
        Self::new(MeshShape::Cube { size }, color)
    }

    pub fn sphere(radius: f32, color: [f32; 4]) -> Self {
        Self::new(MeshShape::Sphere { radius }, color)
    }

    /// Loads a `.gltf` or `.glb` model from your `assets/` folder.
    /// Automatically appends `#Scene0` if no sub-asset label is provided.
    pub fn gltf(path: &str) -> Self {
        let formatted_path = if path.contains('#') {
            path.to_string()
        } else {
            format!("{path}#Scene0")
        };

        Self::new(MeshShape::GltfPath(formatted_path), [1.0, 1.0, 1.0, 1.0])
    }

    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.transform.translate(x, y, z);
        self
    }
}


impl ObjectBehavior for MeshObject3D {
    fn start(&mut self, id: ObjectId, ctx: &mut TreeContext) {
        ctx.spawn_mesh(id, self.shape.clone(), self.color);
    }

    fn transform(&self) -> Option<&Transform3D> {
        Some(&self.transform)
    }
}
