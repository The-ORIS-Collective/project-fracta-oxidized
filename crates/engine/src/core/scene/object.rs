use crate::core::scene::ObjectBehavior;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId(pub usize);

pub struct SceneObject {
    pub name: String,
    pub parent: Option<ObjectId>,
    pub children: Vec<ObjectId>,
    pub behavior: Box<dyn ObjectBehavior>,
}
