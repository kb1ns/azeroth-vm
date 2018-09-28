use std::mem;
use super::*;

pub struct JvmStack<'r> {
    pub frames: Vec<Frame<'r>>,
    // we declare locals and operands in JvmStack rather than Frame
    // because of the rust mutable borrow limitation
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub pc: usize,
    pub max_stack_size: usize,
    pub stack_size: usize,
}

pub struct Frame<'r> {
    pub class_ref: &'r Class,
    pub method_name: String,
    pub descriptor_name: String,
}

impl<'r> JvmStack<'r> {
    pub fn allocate(max_stack_size: usize) -> JvmStack<'r> {
        JvmStack {
            frames: Vec::<Frame>::new(),
            locals: Vec::<Slot>::new(),
            operands: Vec::<Slot>::new(),
            pc: 0,
            max_stack_size: max_stack_size,
            stack_size: 0,
        }
    }

    pub fn push(&mut self, class_ref: &'r Class, method_name: String, method_descriptor: String) {
        let f = Frame::new(class_ref, method_name, method_descriptor);
        // if self.stack_size + f.get_frame_size() >= self.max_stack_size {
        //     panic!("java.lang.StackOverflowError: {}", self.max_stack_size);
        // }
        // self.stack_size = self.stack_size + f.get_frame_size();
        self.frames.push(f);
    }

    pub fn pop(&mut self) {
        // if let Some(ref f) = self.frames.pop() {
        //     self.stack_size = self.stack_size - f.get_frame_size();
        // }
    }

    pub fn top(&self) -> &'r Frame {
        match self.frames.last() {
            Some(frame) => frame,
            None => {
                panic!("fatal error: Stack out of range");
            }
        }
    }
}

impl<'r> Frame<'r> {
    pub fn new(class_ref: &'r Class, method_name: String, descriptor_name: String) -> Frame<'r> {
        Frame {
            class_ref: class_ref,
            method_name: method_name,
            descriptor_name: descriptor_name,
        }
    }
}
