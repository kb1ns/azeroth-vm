use bytecode::atom::*;
use bytecode::*;
use mem::heap::*;
use mem::klass::*;
use mem::metaspace::*;
use mem::stack::*;
use mem::*;
use std::mem::transmute;
use std::sync::atomic::*;
use std::sync::Arc;

pub enum Return {
    Word(Slot),
    DWord(WideSlot),
    Void,
}

pub struct JavaError {
    // TODO exception class
    pub message: String,
    pub line: i32,
    pub throwable: String,
}

fn ensure_initialized(stack: &mut JavaStack, klass: Arc<Klass>, pc: usize) -> bool {
    if !klass.initialized.load(Ordering::Relaxed) {
        if let Ok(_) = klass.mutex.try_lock() {
            if !klass.initialized.load(Ordering::Relaxed) {
                klass.initialized.store(true, Ordering::Relaxed);
                println!("init class {}", klass.bytecode.get_name());
                let initialized = match klass.bytecode.get_method("<clinit>", "()V") {
                    Some(clinit) => {
                        let frame = JavaFrame::new(klass.clone(), clinit);
                        stack.invoke(frame, pc);
                        false
                    }
                    None => true,
                };
                let superclass = klass.bytecode.get_super_class();
                if !superclass.is_empty() {
                    // TODO
                    let superclass = find_class!(superclass).expect("ClassNotFoundException");
                    ensure_initialized(stack, superclass, 0);
                }
                return initialized;
            }
        }
    }
    true
}

pub fn execute(stack: &mut JavaStack) {
    if stack.is_empty() {
        return;
    }
    let mut pc: usize = stack.frames.last().expect("Won't happend").pc;
    while stack.has_next(pc) {
        // we must check if current class not be initialized
        let frame = stack.frames.last().expect("Won't happend");
        if pc == 0 {
            let klass = frame.klass.clone();
            if !ensure_initialized(stack, klass, pc) {
                pc = 0;
                continue;
            }
        }
        let instruction = {
            let frame = stack.frames.last().expect("Won't happend");
            // TODO if debug
            frame.dump(pc);
            stack.code_at(pc)
        };
        match instruction {
            // nop
            0x00 => {
                pc = pc + 1;
            }
            // aconst_null
            0x01 => {
                stack.push(&NULL);
                pc = pc + 1;
            }
            // iconst -1 ~ 5
            0x02..=0x08 => {
                let opr = stack.code_at(pc) as i32 - 3;
                stack.push(&opr.memorized());
                pc = pc + 1;
            }
            // lconst 0 ~ 1
            // byteorder: higher first
            0x09..=0x0a => {
                let opr = stack.code_at(pc) as i64 - 9;
                let long = opr.memorized();
                stack.push(&long[..4]);
                stack.push(&long[4..]);
                pc = pc + 1;
            }
            // fconst 0 ~ 2
            0x0b..=0x0d => {
                let opr = stack.code_at(pc) as f32 - 11.0;
                stack.push(&opr.memorized());
                pc = pc + 1;
            }
            // dconst 0 ~ 1
            0x0e..=0x0f => {
                let opr = stack.code_at(pc) as f64 - 14.0;
                let double = opr.memorized();
                stack.push(&double[..4]);
                stack.push(&double[4..]);
                pc = pc + 1;
            }
            // bipush
            0x10 => {
                stack.push(&(stack.code_at(pc + 1) as i32).memorized());
                pc = pc + 2;
            }
            // sipush
            0x11 => {
                stack.push(
                    &((stack.code_at(pc + 1) as i32) << 8 | (stack.code_at(pc + 2) as i32))
                        .memorized(),
                );
                pc = pc + 3;
            }
            // iload 0 ~ 3
            0x1a..=0x1d => {
                let opr = stack.code_at(pc) as usize - 0x1a;
                stack.load(opr, 1);
                pc = pc + 1;
            }
            // lload 0 ~ 3
            0x1e..=0x21 => {
                let opr = stack.code_at(pc) as usize - 0x1e;
                stack.load(opr, 2);
                pc = pc + 1;
            }
            // fload 0 ~ 3
            0x22..=0x25 => {
                let opr = stack.code_at(pc) as usize - 0x22;
                stack.load(opr, 1);
                pc = pc + 1;
            }
            // dload 0 ~ 3
            0x26..=0x29 => {
                let opr = stack.code_at(pc) as usize - 0x26;
                stack.load(opr, 2);
                pc = pc + 1;
            }
            // aload 0 ~ 3
            0x2a..=0x2d => {
                let opr = stack.code_at(pc) as usize - 0x2a;
                stack.load(opr, 1);
                pc = pc + 1;
            }
            // istore 0 ~ 3
            0x3b..=0x3e => {
                let opr = stack.code_at(pc) as usize - 0x3b;
                stack.store(opr, 1);
                pc = pc + 1;
            }
            // lstore 0 ~ 3
            0x3f..=0x42 => {
                let opr = stack.code_at(pc) as usize - 0x3f;
                stack.store(opr, 2);
                pc = pc + 2;
            }
            // fstore 0 ~ 3
            0x43..=0x46 => {
                let opr = stack.code_at(pc) as usize - 0x43;
                stack.store(opr, 1);
                pc = pc + 1;
            }
            // dstore 0 ~ 3
            0x47..=0x4a => {
                let opr = stack.code_at(pc) as usize - 0x47;
                stack.store(opr, 2);
                pc = pc + 2;
            }
            // astore 0 ~ 3
            0x4b..=0x4e => {
                let opr = stack.code_at(pc) as usize - 0x4b;
                stack.store(opr, 1);
                pc = pc + 1;
            }
            // dup
            0x59 => {
                let current = stack.frames.last_mut().expect("Illegal operands stack:");
                unsafe {
                    let ptr = current.operands_ptr.sub(PTR_SIZE);
                    current.operands_ptr.copy_from(ptr, PTR_SIZE);
                }
                pc = pc + 1;
            }
            // iadd
            0x60 => {
                let left = stack.pop();
                let right = stack.pop();
                stack.push(&(i32::from_ne_bytes(left) + i32::from_ne_bytes(right)).memorized());
                pc = pc + 1;
            }
            // iinc
            0x84 => {
                let index = stack.code_at(pc + 1) as usize;
                let cst = stack.code_at(pc + 2) as i32;
                let new = unsafe { transmute::<Slot, i32>(stack.get(index)) + cst };
                stack.set(index, new.memorized());
                pc = pc + 3;
            }
            // if_icmpge
            0xa2 => {
                let (v1, v2) = unsafe {
                    let v2 = transmute::<Slot, i32>(stack.pop());
                    let v1 = transmute::<Slot, i32>(stack.pop());
                    (v1, v2)
                };
                if v1 >= v2 {
                    let offset = ((stack.code_at(pc + 1) as i16) << 8
                        | stack.code_at(pc + 2) as i16) as isize;
                    pc = (pc as isize + offset) as usize;
                } else {
                    pc = pc + 3;
                }
            }
            // goto
            0xa7 => {
                let offset =
                    ((stack.code_at(pc + 1) as i16) << 8 | stack.code_at(pc + 2) as i16) as i16;
                pc = (pc as isize + offset as isize) as usize;
            }
            // return
            0xb1 => {
                pc = stack.backtrack();
            }
            // getstatic
            0xb2 => {
                let frame = &stack.frames.last_mut().expect("Won't happend");
                let field_idx = (frame.code_at(pc + 1) as U2) << 8 | frame.code_at(pc + 2) as U2;
                let klass = frame.klass.clone();
                let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                // TODO load class `c`, push `f` to operands according to the type `t`
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(ref field) = klass.bytecode.get_field(f, t) {
                    match &field.value.get() {
                        None => panic!(""),
                        Some(value) => match value {
                            Value::DWord(_) => stack.push(&Value::of_w(*value)),
                            _ => stack.push(&Value::of(*value)),
                        },
                    }
                } else {
                    // TODO
                    // handle_exception();
                    panic!("NoSuchFieldException");
                }
                pc = pc + 3;
            }
            // putstatic
            0xb3 => {
                let field_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack
                    .frames
                    .last()
                    .expect("Illegal class file")
                    .klass
                    .clone();
                let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                // TODO load class `c`, push `f` to operands according to the type `t`
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(ref field) = klass.bytecode.get_field(f, t) {
                    &field.value.set(match t {
                        "D" | "J" => Some(Value::eval_w(stack.pop_w())),
                        _ => Some(Value::eval(stack.pop(), t)),
                    });
                } else {
                    // TODO
                    panic!("NoSuchFieldException");
                }
                pc = pc + 3;
            }
            0xb7 => {
                let method_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack
                    .frames
                    .last()
                    .expect("Illegal class file")
                    .klass
                    .clone();
                let (c, (m, t)) = klass.bytecode.constant_pool.get_javaref(method_idx);
                // TODO
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(ref method) = klass.bytecode.get_method(m, t) {
                    let new_frame = JavaFrame::new(klass, Arc::clone(method));
                    pc = stack.invoke(new_frame, pc + 3);
                } else {
                    // TODO
                    panic!("NoSuchMethodException");
                }
            }
            // invokestatic
            0xb8 => {
                let method_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack
                    .frames
                    .last()
                    .expect("Illegal class file")
                    .klass
                    .clone();
                let (c, (m, t)) = klass.bytecode.constant_pool.get_javaref(method_idx);
                // TODO
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(ref method) = klass.bytecode.get_method(m, t) {
                    // TODO
                    if !method.is_static()  {
                        panic!("");
                    }
                    let new_frame = JavaFrame::new(klass, Arc::clone(method));
                    pc = stack.invoke(new_frame, pc + 3);
                } else {
                    // TODO
                    panic!("NoSuchMethodException");
                }
            }
            // new
            0xbb => {
                let class_index =
                    (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack
                    .frames
                    .last()
                    .expect("Illegal class file")
                    .klass
                    .clone();
                let class_name = klass.bytecode.constant_pool.get_str(class_index);
                // TODO
                let klass = find_class!(class_name).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                let (obj, size) = jvm_heap!().allocate(&klass);
                let v = unsafe { transmute::<u32, Slot>(obj) };
                stack.push(&v);
                pc = pc + 3;
            }
            _ => panic!(format!(
                "Instruction 0x{:2x?} not implemented yet.",
                instruction
            )),
        }
    }
}

// TODO
fn handle_exception(stack: &mut JavaStack, throwable: String, pc: usize) -> usize {
    pc
}

pub fn resolve_method_descriptor(descriptor: &str) -> (Vec<String>, String) {
    let t = descriptor
        .chars()
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut params: Vec<String> = vec![];
    let mut expect_type: bool = false;
    let mut expect_semicolon: bool = false;
    let mut token: String = String::new();
    for (i, ch) in descriptor.chars().enumerate() {
        if expect_semicolon {
            token.push(ch);
            if ch == ';' {
                expect_semicolon = false;
                expect_type = false;
                params.push(token.clone());
                token.clear();
            }
            continue;
        }
        match ch {
            '(' => {
                if expect_type {
                    panic!(format!("Illegal method descriptor: {}", descriptor));
                }
                continue;
            }
            ')' => {
                if expect_type {
                    panic!(format!("Illegal method descriptor: {}", descriptor));
                }
                return (params, t[i + 1..].join(""));
            }
            JVM_ARRAY => {
                expect_type = true;
                token.push('[');
            }
            JVM_REF => {
                expect_semicolon = true;
                token.push('L');
            }
            JVM_BYTE | JVM_CHAR | JVM_FLOAT | JVM_DOUBLE | JVM_INT | JVM_LONG | JVM_SHORT
            | JVM_BOOLEAN => {
                if expect_type {
                    token.push(ch);
                    params.push(token.clone());
                    token.clear();
                    expect_type = false;
                } else {
                    params.push(ch.to_string());
                }
            }
            _ => {
                if expect_semicolon {
                    token.push(ch);
                } else {
                    panic!(format!("Illegal method descriptor: {}", descriptor));
                }
            }
        }
    }
    panic!(format!("Illegal method descriptor: {}", descriptor));
}

#[test]
pub fn test_resolve_method() {
    let (params, ret) = resolve_method_descriptor("(Ljava/lang/String;IJ)V");
    assert_eq!(ret, "V");
    assert_eq!(params, vec!["Ljava/lang/String;", "I", "J"]);
    let (params, ret) = resolve_method_descriptor("([IJLjava/lang/String;)[Ljava/lang/String;");
    assert_eq!(ret, "[Ljava/lang/String;");
    assert_eq!(params, vec!["[I", "J", "Ljava/lang/String;"]);
    let (params, ret) = resolve_method_descriptor("([Ljava/lang/String;)V");
    assert_eq!(params, vec!["[Ljava/lang/String;"]);
    assert_eq!(ret, "V");
}
