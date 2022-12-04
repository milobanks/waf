use crate::ecs::object::Object;
use crate::ecs::component::Component;

pub struct Scene {
    pub objects: Vec<Object>,
    pub components: Vec<Box<dyn Component>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: vec![],
            components: vec![],
        }
    }
}

