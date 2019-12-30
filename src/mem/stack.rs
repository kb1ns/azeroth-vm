use bytecode::atom::*;
use bytecode::attribute::Attribute;
use bytecode::attribute::ExceptionHandler;
use bytecode::method::Method;
use bytecode::*;
use interpreter;
use mem::metaspace::Klass;
use mem::Slot;
use mem::NULL;
use std::sync::Arc;

pub struct JavaStack {
    // TODO thread
    frames: Vec<JavaFrame>,
    pub max_stack_size: usize,
}

impl JavaStack {
    // TODO
    pub fn new() -> JavaStack {
        JavaStack {
            frames: Vec::<JavaFrame>::new(),
            max_stack_size: 0,
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

    pub fn has_next(&self, pc: usize) -> bool {
        match self.top() {
            Some(ref frame) => pc < frame.code.len(),
            None => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn push(&mut self, mut frame: JavaFrame, pc: usize) {
        if !self.is_empty() {
            let (_, ref descriptor, access_flag) = &frame.current_method;
            let (params, _) = interpreter::resolve_method_descriptor(descriptor);
            let mut i = 0;
            if access_flag & METHOD_ACC_STATIC != METHOD_ACC_STATIC {
                // TODO put `this` to local[0]
                frame.locals[0] = NULL;
                i = 1;
            }
            let current = self.top_mut().expect("Won't happend");
            for param in params.iter() {
                match param.as_ref() {
                    "D" | "J" => {
                        let lower = current.operands.pop().expect("Illegal operands stack: ");
                        let higher = current.operands.pop().expect("Illegal operands stack: ");
                        frame.locals[i] = higher;
                        frame.locals[i + 1] = lower;
                        i = i + 2;
                    }
                    _ => {
                        let v = current.operands.pop().expect("Illegal operands stack: ");
                        frame.locals[i] = v;
                        i = i + 1;
                    }
                }
            }
            frame.pc = pc;
        }
        self.frames.push(frame);
    }

    pub fn pop(&mut self) -> usize {
        let mut frame = self.frames.pop().expect("Illegal operands stack: ");
        if !self.is_empty() {
            let (_, ref descriptor, _) = &frame.current_method;
            let (_, ret) = interpreter::resolve_method_descriptor(descriptor);
            match ret.as_ref() {
                "D" | "J" => {
                    let lower = frame.operands.pop().expect("Illegal operands stack: ");
                    let higher = frame.operands.pop().expect("Illegal operands stack: ");
                    self.top_mut().expect("Won't happend").operands.push(higher);
                    self.top_mut().expect("Won't happend").operands.push(lower);
                }
                "V" => {}
                _ => {
                    let v = frame.operands.pop().expect("Illegal operands stack: ");
                    self.top_mut().expect("Won't happend").operands.push(v);
                }
            }
        }
        frame.pc
    }
}

pub struct JavaFrame {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub klass: Arc<Klass>,
    pub code: Arc<Vec<u8>>,
    pub exception_handlers: Arc<Vec<ExceptionHandler>>,
    pub current_method: (String, String, U2),
    pub pc: usize,
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
                pc: 0,
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
