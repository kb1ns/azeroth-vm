use super::*;
use std;

pub struct JvmStack<'r> {
    pub frames: Vec<Frame<'r>>,
    pub max_stack_size: usize,
    pub stack_size: usize,
    pub pc: u32,
}

pub struct Frame<'r> {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub class: &'r Class,
    pub code: &'r [u8],
    pub exception_handler: &'r [ExceptionHandler],
    pub attributes: &'r Attributes,
    pub method_name: &'r str,
    pub descriptor: &'r str,
}

pub struct ThreadContext {}

impl<'r> JvmStack<'r> {
    pub fn allocate(max_stack_size: usize) -> JvmStack<'r> {
        JvmStack {
            frames: Vec::<Frame>::new(),
            max_stack_size: max_stack_size,
            stack_size: 0,
            pc: 0,
        }
    }

    pub fn push(&mut self, class: &'r Class, method_name: &'r str, method_descriptor: &'r str) {
        let f = Frame::new(class, method_name, method_descriptor);
        // TODO check stack size
        self.frames.push(f);
    }

    pub fn pop(&mut self) {
        if let Some(f) = self.frames.pop() {
            if !self.frames.is_empty() {
                let current = self.frames.len() - 1;
                if let Some(ret_addr) = self.frames[current].locals.pop() {
                    unsafe {
                        self.pc = std::mem::transmute::<Slot, u32>(ret_addr);
                    }
                }
            }
        }
        panic!("pop empty stack");
    }

    pub fn current_index(&self) -> usize {
        self.frames.len() - 1
    }
}

impl<'r> Frame<'r> {
    pub fn new(class: &'r Class, method_name: &'r str, descriptor: &'r str) -> Frame<'r> {
        if let Ok(method) = class.get_method(&method_name, &descriptor) {
            if let Some(&Attribute::Code(
                stacks,
                locals,
                ref code,
                ref exception_handler,
                ref attributes,
            )) = method.get_code()
            {
                return Frame {
                    class: class,
                    code: code,
                    exception_handler: exception_handler,
                    attributes: attributes,
                    method_name: method_name,
                    descriptor: descriptor,
                    locals: Vec::<Slot>::with_capacity(locals as usize),
                    operands: Vec::<Slot>::with_capacity(stacks as usize),
                };
            }
            panic!("Method is abstract");
        }
        panic!("java.lang.NoSuchMethodError");
    }
}
