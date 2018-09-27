use std::mem;
use super::*;

pub struct JavaStack<'r> {
    pub java_method: Vec<Frame<'r>>,
    pub java_pc: usize,
    pub max_stack_size: usize,
    pub stack_size: usize,
}

pub struct Frame<'r> {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub class_ref: &'r Class,
    pub method_name: String,
    pub descriptor_name: String,
}

impl<'r> Frame<'r> {
    pub fn new(class_ref: &'r Class, method_name: String, descriptor_name: String) -> Frame<'r> {
        Frame {
            // TODO deal with the method `Code` attribute
            locals: Vec::<Slot>::with_capacity(20),
            operands: Vec::<Slot>::with_capacity(20),
            class_ref: class_ref,
            method_name: method_name,
            descriptor_name: descriptor_name,
        }
    }

    pub fn get_frame_size(&self) -> usize {
        (self.operands.capacity() + self.locals.capacity()) * mem::size_of::<Slot>()
    }
}

impl<'r> JavaStack<'r> {
    pub fn allocate(max_stack_size: usize) -> JavaStack<'r> {
        JavaStack {
            java_method: Vec::<Frame>::new(),
            java_pc: 0,
            max_stack_size: max_stack_size,
            stack_size: 0,
        }
    }

    pub fn push(&mut self, class_ref: &'r Class, method_name: String, method_descriptor: String) {
        let f = Frame::new(class_ref, method_name, method_descriptor);
        if self.stack_size + f.get_frame_size() >= self.max_stack_size {
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

    pub fn top(&mut self) -> &'r mut Frame {
        match self.java_method.last_mut() {
            Some(frame) => frame,
            None => {
                panic!("fatal error: Stack out of range");
            }
        }
    }
}
