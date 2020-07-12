use crate::mem::{heap::*, klass::*, metaspace::*, stack::*, *};


pub struct ThreadContext {
    pc: usize,
    stack: JavaStack,
    // TODO thread
}
