use bytecode::atom::*;
use bytecode::attribute::{Attribute, ExceptionHandler};
use bytecode::method::Method;
use bytecode::*;
use interpreter;
use mem::metaspace::Klass;
use mem::{Slot, NULL, PTR_SIZE};
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

    pub fn has_next(&self, pc: usize) -> bool {
        match self.top() {
            Some(ref frame) => pc < frame.code.len(),
            None => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn invoke(&mut self, mut frame: JavaFrame, pc: usize) {
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

    pub fn backtrack(&mut self) -> usize {
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

    pub fn get_code(&self, pc: usize) -> u8 {
        self.frames.last().expect("Illegal class file").code[pc]
    }

    pub fn load(&mut self, offset: usize, count: usize) {
        let current = self.frames.last_mut().expect("Illegal class file");
        current
            .operands_ptr
            .copy_from(current.locals[offset * PTR_SIZE..].as_ptr(), count);
        current.operands_ptr = current.operands_ptr.add(count * PTR_SIZE);
    }

    pub fn store(&mut self, offset: usize, count: usize) {
        let current = self.top_mut().expect("Illegal class file");
        current.operands_ptr = current.operands_ptr.sub(count * PTR_SIZE);
        current.operands_ptr.copy_to(&current.locals[offset * PTR_SIZE..].as_mut_ptr(), count * PTR_SIZE);
    }

    pub fn get(&self, offset: usize) -> Slot {
        let mut data = NULL;
        let current = self.frames.last_mut().expect("Illegal operands");
        &data[..].copy_from_slice(currentl.locals[offset * PTR_SIZE..(offset + 1) * PTR_SIZE]);
        data
    }

    pub fn get_w(&self, offset: usize) -> (Slot, Slot) {
        let mut (data0, data1) = (NULL, NULL);
        let current = self.frames.last_mut().expect("Illegal operands");
        &data0[..].copy_from_slice(currentl.locals[offset * PTR_SIZE..(offset + 1) * PTR_SIZE]);
        &data1[..].copy_from_slice(currentl.locals[(offset + 1) * PTR_SIZE..(offset + 2) * PTR_SIZE]);
        (data0, data1)
    }

    pub fn set(&mut self, offset: usize, v: Slot) {
        let frame = self.frames.last_mut().expect("Illegal class file");
        &frame.locals[offset * PTR_SIZE..].copy_from_slice(&v[..]);
    }

    pub fn set_w(&mut self, offset: usize, v: (Slot, Slot)) {
        let frame = self.frames.last_mut().expect("Illegal class file");
        &frame.locals[offset * PTR_SIZE..].copy_from_slice(&v.0[..]);
        &frame.locals[(offset + 1) * PTR_SIZE].copy_from_slice(&v.1[..]);
        // frame.locals[offset] = v.0;
        // frame.locals[offset + 1] = v.1;
    }

    pub fn push(&mut self, v: Slot) {
        let current = self.frames.last_mut().expect("Illegal class file");
        current.operands_ptr.copy_from(&v.as_ptr(), PTR_SIZE);
        current.operands_ptr = top.operands_ptr.add(PTR_SIZE);
    }

    pub fn pop(&mut self) -> Slot {
        let mut data = NULL;
        let current = self.frames.last_mut().expect("Illegal operands");
        current.operands_ptr = current.operands_ptr.sub(PTR_SIZE);
        current.operands_ptr.copy_to(&mut data.as_mut_ptr(), PTR_SIZE);
        data
    }
}

pub struct JavaFrame {
    pub locals: Vec<u8>,
    locals_ptr: *mut u8,
    operands: Vec<u8>,
    pub operands_ptr: *mut u8,
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
            let mut locals = vec![0u8; locals as usize];
            let locals_ptr = locals.as_mut_ptr();
            let mut operands = Vec::<u8>::with_capacity(PTR_SIZE * stacks as usize);
            let operands_ptr = operands.as_mut_ptr();
            return JavaFrame {
                locals: locals,
                locals_ptr: locals_ptr,
                operands: operands,
                operands_ptr: operands_ptr,
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
