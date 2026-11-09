use serde::{Deserialize, Serialize};

use super::object::ObjectData;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub root: ObjectData,
}


impl Scene {
    pub fn new(name: impl Into<String>, root: ObjectData) -> Self {
        Self {
            name: name.into(),
            root,
        }
    }
}
