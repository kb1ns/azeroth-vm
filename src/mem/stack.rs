use bytecode::attribute::Attribute;
use bytecode::attribute::ExceptionHandler;
use bytecode::method::Method;
use mem::metaspace::Klass;
use mem::Slot;
use mem::NULL;
use std::sync::Arc;

pub struct JavaStack {
    // thread
    pub frames: Vec<JavaFrame>,
    pub max_stack_size: usize,
    pub pc: u32,
}

impl JavaStack {
    // TODO
    pub fn new() -> JavaStack {
        JavaStack {
            frames: Vec::<JavaFrame>::new(),
            max_stack_size: 0,
            pc: 0,
        }
    }

    pub fn top(&self) -> Option<&JavaFrame> {
        match self.frames.len() {
            0 => None,
            n => Some(&self.frames[n - 1]),
        }
    }

    pub fn top_mut(&mut self) -> Option<&mut JavaFrame> {
        match self.frames.len() {
            0 => None,
            n => Some(&mut self.frames[n - 1]),
        }
    }
}

pub struct JavaFrame {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub klass: Arc<Klass>,
    pub code: Arc<Vec<u8>>,
    pub exception_handlers: Arc<Vec<ExceptionHandler>>,
    // pub attributes: &'class Attributes,
    pub current_method: (String, String),
}

impl JavaFrame {
    pub fn new(class: Arc<Klass>, method: Arc<Method>) -> JavaFrame {
        let code_attribute = method
            .get_code()
            .expect("abstract method or interface not allowed");
        if let Attribute::Code(stacks, locals, ref code, ref exception, _) = code_attribute {
            let mut locals = Vec::<Slot>::with_capacity(locals as usize);
            for _i in 0..locals.capacity() {
                locals.push(NULL);
            }
            return JavaFrame {
                locals: locals,
                operands: Vec::<Slot>::with_capacity(stacks as usize),
                klass: class,
                code: Arc::clone(code),
                exception_handlers: Arc::clone(exception),
                current_method: method.get_name_and_descriptor(),
            };
        }
        panic!("won't happend");
    }

    // TODO
    pub fn dump(&self, pc: usize) {
        println!("current class: {:?}", self.klass.bytecode.get_name());
        println!(
            "current method: {:?} {:?}",
            self.current_method.0, self.current_method.1
        );
        println!("locals: {:x?}", self.locals);
        println!("stacks: {:x?}", self.operands);
        println!("pc: {:?}", pc);
        println!("instructions: {:x?}\n", &self.code);
    }
}
