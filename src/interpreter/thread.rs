use crate::mem::{heap::*, klass::*, metaspace::*, stack::*, *};

pub struct ThreadContext {
    pub pc: usize,
    pub stack: JavaStack,
    // TODO thread
}
