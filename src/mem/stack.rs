use std::mem;
use super::*;

pub struct JavaStack {
    pub java_method: Vec<Frame>,
    pub java_pc: usize,
    pub max_stack_size: usize,
    stack_size: usize,
}

pub struct Frame {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
}

impl Frame {
    pub fn new(max_locals: usize, max_op_stack_size: usize) -> Frame {
        Frame {
            locals: Vec::<Slot>::with_capacity(max_locals),
            operands: Vec::<Slot>::with_capacity(max_op_stack_size),
        }
    }

    pub fn get_frame_size(&self) -> usize {
        (self.operands.capacity() + self.locals.capacity()) * mem::size_of::<Slot>()
    }
}

impl JavaStack {
    pub fn allocate(max_stack_size: usize, pc: usize) -> JavaStack {
        JavaStack {
            java_method: Vec::<Frame>::new(),
            java_pc: pc,
            max_stack_size: max_stack_size,
            stack_size: 0,
        }
    }

    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn push(&mut self, max_locals: usize, max_op_stack_size: usize) {
        let f = Frame::new(max_locals, max_op_stack_size);
        if self.get_stack_size() + f.get_frame_size() >= self.max_stack_size {
            panic!("java.lang.StackOverflowError: {}", self.max_stack_size);
        }
        self.stack_size = self.stack_size + f.get_frame_size();
        self.java_method.push(f);
    }

    pub fn pop(&mut self) {
        if let Some(ref f) = self.java_method.pop() {
            self.stack_size = self.stack_size - f.get_frame_size();
        }
    }

    pub fn top(&mut self) -> &mut Frame {
        match self.java_method.last_mut() {
            Some(frame) => frame,
            None => {
                panic!("fatal error: Stack out of range");
            }
        }
    }
}
