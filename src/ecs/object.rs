pub struct Object {
    pub components: Vec<usize>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            components: vec![],
        }
    }
}
