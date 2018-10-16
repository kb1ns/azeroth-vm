use super::metaspace::Klass;
use super::*;

pub struct JvmStack {
    pub frames: Vec<Frame>,
    pub max_stack_size: usize,
    pub stack_size: usize,
    pub pc: u32,
}

pub struct Frame {
    // pub locals: Vec<Slot>,
// pub operands: Vec<Slot>,
// pub klass: std::sync::Arc<Klass>,
// pub code: &[u8],
// pub exception_handler: &[ExceptionHandler],
// pub attributes: &Attributes,
// pub method_name: &str,
// pub descriptor: String,
}
