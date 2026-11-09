use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectData {
    pub id: ObjectId,
    pub name: String,
    pub object_type: String,
    pub children: Vec<ObjectData>,
}


impl ObjectData {
    pub fn new(
        id: ObjectId,
        name: impl Into<String>,
        object_type: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            object_type: object_type.into(),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: NodeData) {
        self.children.push(child);
    }
}
