pub mod mesh;
pub mod instance;

use std::any::Any;

pub trait Component {
    fn as_any(&self) -> &dyn Any;
}
