use super::bytecode::atom::*;
use super::bytecode::attribute::Attribute;
use super::mem::metaspace::ClassArena;
use super::mem::stack::*;
use super::mem::*;
use std;

pub struct Interpreter {
    pub class_arena: std::sync::Arc<ClassArena>,
    // TODO heap
}

pub enum Return {
    Throwable(Vec<String>),
    Word(Slot),
    DWord(Slot2),
    Void,
}

impl Interpreter {
    // TODO locals
    pub fn execute(
        &self,
        class_name: &str,
        method_name: &str,
        method_descriptor: &str,
        mut args: Vec<Slot>,
    ) -> Return {
        if let Some(klass) = self.class_arena.find_class(class_name) {
            if let Ok(method) = klass.bytecode.get_method(method_name, method_descriptor) {
                if let Some(&Attribute::Code(
                    stacks,
                    locals,
                    ref code,
                    ref exception_handler,
                    ref attributes,
                )) = method.get_code()
                {
                    let mut pc: U4 = 0;
                    while args.len() != locals as usize {
                        args.push(NULL);
                    }
                    let mut locals = args;
                    let mut operands = Vec::<Slot>::with_capacity(stacks as usize);
                    while pc < code.len() as U4 {
                        println!("pc -> {}", pc);
                        println!("locals -> {:?}", locals);
                        println!("operands -> {:?}", operands);
                        unsafe {
                            match code[pc as usize] {
                                // nop
                                0x00 => {
                                    pc = pc + 1;
                                }
                                // aconst_null
                                0x01 => {
                                    operands.push(NULL);
                                    pc = pc + 1;
                                }
                                // iconst -1 ~ 5
                                0x02 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(-1));
                                    pc = pc + 1;
                                }
                                0x03 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(0));
                                    pc = pc + 1;
                                }
                                0x04 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(1));
                                    pc = pc + 1;
                                }
                                0x05 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(2));
                                    pc = pc + 1;
                                }
                                0x06 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(3));
                                    pc = pc + 1;
                                }
                                0x07 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(4));
                                    pc = pc + 1;
                                }
                                0x08 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(5));
                                    pc = pc + 1;
                                }
                                // lconst 0 ~ 1
                                // byteorder: higher first
                                0x09 => {
                                    let b = std::mem::transmute::<i64, Slot2>(0);
                                    let (higher, lower) = split_slot2(b);
                                    operands.push(higher);
                                    operands.push(lower);
                                    pc = pc + 1;
                                }
                                0x0a => {
                                    let b = std::mem::transmute::<i64, Slot2>(0);
                                    let (higher, lower) = split_slot2(b);
                                    operands.push(higher);
                                    operands.push(lower);
                                    pc = pc + 1;
                                }
                                // fconst 0 ~ 2
                                0x0b => {
                                    operands.push(std::mem::transmute::<f32, Slot>(0.0));
                                    pc = pc + 1;
                                }
                                0x0c => {
                                    operands.push(std::mem::transmute::<f32, Slot>(1.0));
                                    pc = pc + 1;
                                }
                                0x0d => {
                                    operands.push(std::mem::transmute::<f32, Slot>(2.0));
                                    pc = pc + 1;
                                }
                                // dconst 0 ~ 1
                                0x0e => {
                                    let b = std::mem::transmute::<f64, Slot2>(0.0);
                                    let (higher, lower) = split_slot2(b);
                                    operands.push(higher);
                                    operands.push(lower);
                                    pc = pc + 1;
                                }
                                0x0f => {
                                    let b = std::mem::transmute::<f64, Slot2>(1.0);
                                    let (higher, lower) = split_slot2(b);
                                    operands.push(higher);
                                    operands.push(lower);
                                    pc = pc + 1;
                                }
                                // bipush
                                0x10 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(
                                        code[(pc + 1) as usize] as i32,
                                    ));
                                    pc = pc + 2;
                                }
                                // sipush
                                0x11 => {
                                    operands.push(std::mem::transmute::<i32, Slot>(
                                        (code[(pc + 1) as usize] as i32) << 8
                                            | (code[(pc + 2) as usize] as i32),
                                    ));
                                    pc = pc + 3;
                                }
                                // iload 0 ~ 3
                                0x1a => {
                                    operands.push(locals[0]);
                                    pc = pc + 1;
                                }
                                0x1b => {
                                    operands.push(locals[2]);
                                    pc = pc + 1;
                                }
                                0x1c => {
                                    operands.push(locals[2]);
                                    pc = pc + 1;
                                }
                                0x1d => {
                                    operands.push(locals[3]);
                                    pc = pc + 1;
                                }
                                // istore 0 ~ 3
                                0x3b => {
                                    if let Some(i) = operands.pop() {
                                        locals[0] = i;
                                        pc = pc + 1;
                                    } else {
                                        panic!("invalid frame: locals");
                                    }
                                }
                                0x3c => {
                                    if let Some(i) = operands.pop() {
                                        locals[1] = i;
                                        pc = pc + 1;
                                    } else {
                                        panic!("invalid frame: locals");
                                    }
                                }
                                0x3d => {
                                    if let Some(i) = operands.pop() {
                                        locals[2] = i;
                                        pc = pc + 1;
                                    } else {
                                        panic!("invalid frame: locals");
                                    }
                                }
                                0x3e => {
                                    if let Some(i) = operands.pop() {
                                        locals[3] = i;
                                        pc = pc + 1;
                                    } else {
                                        panic!("invalid frame: locals");
                                    }
                                }
                                // iadd
                                0x60 => {
                                    if let Some(left) = operands.pop() {
                                        if let Some(right) = operands.pop() {
                                            let v1 = std::mem::transmute::<Slot, i32>(left);
                                            let v2 = std::mem::transmute::<Slot, i32>(right);
                                            operands
                                                .push(std::mem::transmute::<i32, Slot>(v1 + v2));
                                            pc = pc + 1;
                                            continue;
                                        }
                                    }
                                    panic!("invalid frame: locals");
                                }
                                // iinc
                                0x84 => {
                                    let index = code[(pc + 1) as usize] as usize;
                                    let cst = code[(pc + 2) as usize] as i32;
                                    let new = std::mem::transmute::<Slot, i32>(locals[index]) + cst;
                                    locals[index] = std::mem::transmute::<i32, Slot>(new);
                                    pc = pc + 3;
                                }
                                // if_icmpge
                                0xa2 => {
                                    let size = operands.len();
                                    let v1 = std::mem::transmute::<Slot, i32>(operands[size - 2]);
                                    let v2 = std::mem::transmute::<Slot, i32>(operands[size - 1]);
                                    if v1 >= v2 {
                                        pc = (code[(pc + 1) as usize] as U4) << 8
                                            | code[(pc + 2) as usize] as U4;
                                    } else {
                                        pc = pc + 3;
                                    }
                                }
                                // goto
                                0xa7 => {
                                    pc = (code[(pc + 1) as usize] as U4) << 8
                                        | code[(pc + 2) as usize] as U4;
                                }
                                0xb1 => {
                                    return Return::Void;
                                }
                                _ => {
                                    pc = pc + 1;
                                }
                            }
                        }
                    }
                    // TODO
                    return Return::Void;
                } else {
                    // TODO
                    panic!("Method is abstract");
                }
            } else {
                // TODO
                panic!("java.lang.NoSuchMethodError");
            }
        } else {
            // TODO
            panic!("java.lang.ClassNotFoundException");
        }
    }
}
