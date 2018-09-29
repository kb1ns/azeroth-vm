use super::*;

pub struct JvmStack<'r> {
    pub frames: Vec<Frame<'r>>,
    pub max_stack_size: usize,
    pub stack_size: usize,
}

pub struct Frame<'r> {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    // tricky
    pub pc: usize,
    pub class_ref: &'r Class,
    pub method_name: String,
    pub descriptor: String,
}

impl<'r> JvmStack<'r> {
    pub fn allocate(max_stack_size: usize) -> JvmStack<'r> {
        JvmStack {
            frames: Vec::<Frame>::new(),
            max_stack_size: max_stack_size,
            stack_size: 0,
        }
    }

    pub fn push(&mut self, class_ref: &'r Class, method_name: String, method_descriptor: String) {
        let f = Frame::new(class_ref, method_name, method_descriptor);
        // TODO check stack size
        self.frames.push(f);
    }

    pub fn pop(&mut self) {
        if let Some(f) = self.frames.pop() {
            // TODO resolve return type

        }
        panic!("pop empty stack");
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
    pub fn new(class_ref: &'r Class, method_name: String, descriptor: String) -> Frame<'r> {
        if let Ok(method) = class_ref.get_method(&method_name, &descriptor) {
            if let Some(&Attribute::Code(stacks, locals, _, _, _)) = method.get_code() {
                return Frame {
                    class_ref: class_ref,
                    method_name: method_name,
                    descriptor: descriptor,
                    pc: 0,
                    locals: Vec::<Slot>::with_capacity(locals as usize),
                    operands: Vec::<Slot>::with_capacity(stacks as usize),
                };
            }
            panic!("Method is abstract");
        }
        panic!("java.lang.NoSuchMethodError");
    }
}
