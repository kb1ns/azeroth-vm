use super::bytecode::*;
use super::mem::metaspace::*;
use super::mem::stack::*;
use super::mem::*;
use bytecode::atom::*;
use std;
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
    pub stacktrace: Vec<String>,
}

fn fire_exception(class: &str, method: &str, line: isize, message: &str) -> JavaError {
    JavaError {
        message: message.to_string(),
        stacktrace: vec![],
    }
}

fn find_and_init_class(stack: &mut JavaStack, class_name: &str) -> Arc<Klass> {
    let class = unsafe {
        if let Some(ref classes) = metaspace::CLASSES {
            classes.clone().find_class(class_name)
        } else {
            panic!("won't happend: ClassArena not initialized");
        }
    };
    // TODO
    let class = class.expect("ClassNotFoundException");
    if !class.initialized.load(Ordering::Relaxed) {
        if let Ok(_) = class.mutex.try_lock() {
            class.initialized.store(true, Ordering::Relaxed);
            let clinit = class
                .clone()
                .bytecode
                .get_method("<clinit>", "()V")
                .expect("clinit must exist");
            let frame = stack::JavaFrame::new(class.clone(), clinit);
            invoke(stack, frame);
        }
    }
    class
}

pub fn invoke(stack: &mut JavaStack, current: JavaFrame) -> Result<Return, JavaError> {
    stack.frames.push(current);
    let mut pc: U4 = 0;
    let code_len = &stack.top().expect("Won't happend").code.len();
    while pc < *code_len as U4 {
        unsafe {
            let instruction = {
                let current = &stack.top().expect("Won't happend");
                current.dump(pc as usize);
                current.code[pc as usize]
            };
            match instruction {
                // nop
                0x00 => {
                    pc = pc + 1;
                }
                // aconst_null
                0x01 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    frame.operands.push(NULL);
                    pc = pc + 1;
                }
                // iconst -1 ~ 5
                0x02..=0x08 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let opr = frame.code[pc as usize] as i32 - 3;
                    frame.operands.push(opr.memorized());
                    pc = pc + 1;
                }
                // lconst 0 ~ 1
                // byteorder: higher first
                0x09..=0x0a => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let opr = frame.code[pc as usize] as i64 - 9;
                    let (lower, higher) = opr.memorized();
                    frame.operands.push(higher);
                    frame.operands.push(lower);
                    pc = pc + 1;
                }
                // fconst 0 ~ 2
                0x0b..=0x0d => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let opr = frame.code[pc as usize] as f32 - 11.0;
                    frame.operands.push(opr.memorized());
                    pc = pc + 1;
                }
                // dconst 0 ~ 1
                0x0e..=0x0f => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let opr = frame.code[pc as usize] as f64 - 14.0;
                    let (lower, higher) = opr.memorized();
                    frame.operands.push(higher);
                    frame.operands.push(lower);
                    pc = pc + 1;
                }
                // bipush
                0x10 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    frame
                        .operands
                        .push((frame.code[(pc + 1) as usize] as i32).memorized());
                    pc = pc + 2;
                }
                // sipush
                0x11 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    frame.operands.push(
                        ((frame.code[(pc + 1) as usize] as i32) << 8
                            | (frame.code[(pc + 2) as usize] as i32))
                            .memorized(),
                    );
                    pc = pc + 3;
                }
                // iload 0 ~ 3
                0x1a..=0x1d => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let opr = frame.code[pc as usize] as usize - 0x1a;
                    frame.operands.push(frame.locals[opr]);
                    pc = pc + 1;
                }
                // istore 0 ~ 3
                0x3b..=0x3e => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    if let Some(i) = frame.operands.pop() {
                        let opr = frame.code[pc as usize] as usize - 0x3b;
                        frame.locals[opr] = i;
                        pc = pc + 1;
                    } else {
                        panic!("invalid frame: locals");
                    }
                }
                // iadd
                0x60 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    if let Some(left) = frame.operands.pop() {
                        if let Some(right) = frame.operands.pop() {
                            // TODO
                            let v1 = std::mem::transmute::<Slot, i32>(left);
                            let v2 = std::mem::transmute::<Slot, i32>(right);
                            frame
                                .operands
                                .push(std::mem::transmute::<i32, Slot>(v1 + v2));
                            frame.operands.push((v1 + v2).memorized());
                            pc = pc + 1;
                            continue;
                        }
                    }
                    panic!("invalid frame: locals");
                }
                // iinc
                0x84 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let index = frame.code[(pc + 1) as usize] as usize;
                    let cst = frame.code[(pc + 2) as usize] as i32;
                    let new = std::mem::transmute::<Slot, i32>(frame.locals[index]) + cst;
                    frame.locals[index] = new.memorized();
                    pc = pc + 3;
                }
                // if_icmpge
                0xa2 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    let size = frame.operands.len();
                    let v1 = std::mem::transmute::<Slot, i32>(frame.operands[size - 2]);
                    let v2 = std::mem::transmute::<Slot, i32>(frame.operands[size - 1]);
                    if v1 >= v2 {
                        pc = (frame.code[(pc + 1) as usize] as U4) << 8
                            | frame.code[(pc + 2) as usize] as U4;
                    } else {
                        pc = pc + 3;
                    }
                }
                // goto
                0xa7 => {
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    pc = (frame.code[(pc + 1) as usize] as U4) << 8
                        | frame.code[(pc + 2) as usize] as U4;
                }
                0xb1 => {
                    return Ok(Return::Void);
                }
                // getstatic
                0xb2 => {
                    let frame = &stack.top().expect("Won't happend");
                    let field_idx = (frame.code[(pc + 1) as usize] as U2) << 8
                        | frame.code[(pc + 2) as usize] as U2;
                    let klass = frame.klass.clone();
                    let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                    // TODO load class `c`, push `f` to operands according to the type `t`
                    let class = find_and_init_class(stack, c);
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    if let Some(ref field) = class.bytecode.get_field(f, t) {
                        match &field.value.get() {
                            None => {
                                panic!("");
                            }
                            Some(value) => match value {
                                Value::Word(v) => {
                                    frame.operands.push(*v);
                                }
                                Value::DWord(lower, higher) => {
                                    frame.operands.push(*higher);
                                    frame.operands.push(*lower);
                                }
                            },
                        }
                    } else {
                        // TODO
                        return Err(fire_exception("", "", -1, "NoSuchFieldError"));
                    }
                    pc = pc + 3;
                }
                // putstatic
                0xb3 => {
                    let frame = &stack.top().expect("Won't happend");
                    let field_idx = (frame.code[(pc + 1) as usize] as U2) << 8
                        | frame.code[(pc + 2) as usize] as U2;
                    let klass = frame.klass.clone();
                    let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                    // TODO load class `c`, push `f` to operands according to the type `t`
                    let class = find_and_init_class(stack, c);
                    let frame = &mut stack.top_mut().expect("Won't happend");
                    if let Some(ref field) = class.bytecode.get_field(f, t) {
                        match t {
                            "D" | "J" => {
                                let lower = frame.operands.pop().expect("Illegal locals: ");
                                let higher = frame.operands.pop().expect("Illegal locals: ");
                                &field.value.set(Some(Value::DWord(lower, higher)));
                            }
                            _ => {
                                let v = frame.operands.pop().expect("Illegal locals: ");
                                &field.value.set(Some(Value::Word(v)));
                            }
                        }
                    } else {
                        // TODO
                        return Err(fire_exception("", "", -1, "NoSuchFieldError"));
                    }
                    pc = pc + 3;
                }
                // invokestatic
                0xb8 => {
                    // let method_idx =
                    //     (code[(pc + 1) as usize] as U2) << 8 | code[(pc + 2) as usize] as U2;
                    // let class = load_class(c)?;
                    // let (c, (m, t)) = klass.bytecode.constant_pool.get_javaref(method_idx);
                    // if let Some(ref method) = class.bytecode.get_method(m, t) {}
                }
                _ => {
                    pc = pc + 1;
                }
            }
        }
    }
    stack.frames.pop();
    Ok(Return::Void)
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
}
