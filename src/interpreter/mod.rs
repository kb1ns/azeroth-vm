use super::bytecode::atom::*;
use super::bytecode::attribute::*;
use super::bytecode::class::*;
use super::bytecode::method::*;
use super::mem::metaspace::*;
use super::mem::*;
use std;

pub struct Interpreter {
    pub class_arena: std::sync::Arc<ClassArena>,
    // TODO heap
}

pub enum Return {
    Throwable(Vec<String>),
    Word(Slot),
    DWord(WideSlot),
    Void,
}

impl Interpreter {
    // TODO locals
    pub fn execute(
        &self,
        class_name: &str,
        method_name: &str,
        method_descriptor: &str,
        args: Vec<Slot>,
    ) -> Return {
        if let Some(klass) = self.class_arena.find_class(class_name) {
            if let Ok(method) = klass.bytecode.get_method(method_name, method_descriptor) {
                return self.call(&klass, method, args);
            } else {
                // TODO
                panic!("java.lang.NoSuchMethodError");
            }
        } else {
            // TODO classloader
            if let Some(klass) = self.class_arena.define_class(class_name, Classloader::ROOT) {
                println!("load class {}", class_name);
                let klass = std::sync::Arc::new(klass);
                let klass_1 = klass.clone();
                let mut map = self.class_arena.classes.write().unwrap();
                if let Ok(ref clinit) = klass.bytecode.get_method("<clinit>", "()V") {
                    self.call(&klass, clinit, vec![]);
                }
                map.insert(class_name.to_string(), klass);
                if let Ok(method) = klass_1.bytecode.get_method(method_name, method_descriptor) {
                    return self.call(&klass_1, method, args);
                } else {
                    // TODO
                    panic!("java.lang.NoSuchMethodError");
                }
            } else {
                panic!("java.lang.ClassNotFoundException");
            }
        }
    }

    fn call(&self, klass: &Klass, method: &Method, mut args: Vec<Slot>) -> Return {
        println!(
            "execute method {}.{}",
            &klass.bytecode.this_class_name, &method.name
        );
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
                            operands.push((-1 as i32).memorized());
                            pc = pc + 1;
                        }
                        0x03 => {
                            operands.push((0 as i32).memorized());
                            pc = pc + 1;
                        }
                        0x04 => {
                            operands.push((1 as i32).memorized());
                            pc = pc + 1;
                        }
                        0x05 => {
                            operands.push((2 as i32).memorized());
                            pc = pc + 1;
                        }
                        0x06 => {
                            operands.push((3 as i32).memorized());
                            pc = pc + 1;
                        }
                        0x07 => {
                            operands.push((4 as i32).memorized());
                            pc = pc + 1;
                        }
                        0x08 => {
                            operands.push((5 as i32).memorized());
                            pc = pc + 1;
                        }
                        // lconst 0 ~ 1
                        // byteorder: higher first
                        0x09 => {
                            let (lower, higher) = (0 as i64).memorized();
                            operands.push(higher);
                            operands.push(lower);
                            pc = pc + 1;
                        }
                        0x0a => {
                            let (lower, higher) = (1 as i64).memorized();
                            operands.push(higher);
                            operands.push(lower);
                            pc = pc + 1;
                        }
                        // fconst 0 ~ 2
                        0x0b => {
                            operands.push((0.0 as f32).memorized());
                            pc = pc + 1;
                        }
                        0x0c => {
                            operands.push((1.0 as f32).memorized());
                            pc = pc + 1;
                        }
                        0x0d => {
                            operands.push((2.0 as f32).memorized());
                            pc = pc + 1;
                        }
                        // dconst 0 ~ 1
                        0x0e => {
                            let (lower, higher) = (0.0 as f64).memorized();
                            operands.push(higher);
                            operands.push(lower);
                            pc = pc + 1;
                        }
                        0x0f => {
                            let (higher, lower) = (1.0 as f64).memorized();
                            operands.push(higher);
                            operands.push(lower);
                            pc = pc + 1;
                        }
                        // bipush
                        0x10 => {
                            operands.push((code[(pc + 1) as usize] as i32).memorized());
                            pc = pc + 2;
                        }
                        // sipush
                        0x11 => {
                            operands.push(
                                ((code[(pc + 1) as usize] as i32) << 8
                                    | (code[(pc + 2) as usize] as i32))
                                    .memorized(),
                            );
                            pc = pc + 3;
                        }
                        // iload 0 ~ 3
                        0x1a => {
                            operands.push(locals[0]);
                            pc = pc + 1;
                        }
                        0x1b => {
                            operands.push(locals[1]);
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
                                    // TODO
                                    let v1 = std::mem::transmute::<Slot, i32>(left);
                                    let v2 = std::mem::transmute::<Slot, i32>(right);
                                    operands.push(std::mem::transmute::<i32, Slot>(v1 + v2));
                                    operands.push((v1 + v2).memorized());
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
                            locals[index] = new.memorized();
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
                        // getstatic
                        0xb2 => {
                            let field_idx = (code[(pc + 1) as usize] as U2) << 8
                                | code[(pc + 2) as usize] as U2;
                            let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                            // TODO load class `c`, push `f` to operands according to the type `t`
                            pc = pc + 3;
                        }
                        _ => {
                            panic!(format!(
                                "directive {:?} not implemented at present",
                                code[pc as usize]
                            ));
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
    }
}
