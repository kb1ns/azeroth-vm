use crate::mem::{heap::*, klass::*, metaspace::*, stack::*, *};

pub struct ThreadContext {
    pub pc: usize,
    pub stack: JavaStack,
    pub exception_pending: bool,
    pub throwable_initialized: bool,
    // TODO thread
}

impl ThreadContext {
    pub fn new() -> Self {
        Self {
            pc: 0,
            stack: JavaStack::new(),
            exception_pending: false,
            throwable_initialized: false,
        }
    }
}
