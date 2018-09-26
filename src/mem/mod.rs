pub mod heap;
pub mod stack;
pub mod classloader;
pub mod metaspace;

pub const NULL: stack::Slot = [0x00, 0x00, 0x00, 0x00];
