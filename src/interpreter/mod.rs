use super::bytecode::method::*;
use super::bytecode::*;
use super::mem::metaspace::*;
use super::mem::stack::*;
use super::mem::*;
use bytecode::atom::*;
use bytecode::attribute::*;
use std;
use std::sync::atomic::*;

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

fn load_class(class_name: &str) -> Result<std::sync::Arc<Klass>, JavaError> {
    unsafe {
        if let Some(classes) = &metaspace::CLASSES {
            return match classes.clone().find_class(class_name) {
                Some(k) => {
                    if !k.initialized.load(Ordering::Relaxed) {
                        // init class
                        if let Ok(_) = k.mutex.try_lock() {
                            k.initialized.store(true, Ordering::Relaxed);
                            if let Some(ref clinit) = k.bytecode.get_method("<clinit>", "()V") {
                                if let Err(mut e) = call(&k, clinit, vec![]) {
                                    return Err(e);
                                }
                            }
                        }
                    }
                    Ok(k)
                }
                None => Err(fire_exception(
                    class_name,
                    "",
                    -1,
                    "java.lang.ClassNotFoundException",
                )),
            };
        }
    }
    panic!("ClassArena not initialized.");
}

pub fn invoke<'a>(stack: &mut JavaStack<'a>, frame: JavaFrame<'a>) -> Result<Return, JavaError> {
    stack.frames.push(frame);
    let frame = &mut stack.frames.first_mut().expect("Won't happend");
    let mut pc: U4 = 0;
    let code = frame.code;
    while pc < frame.code.len() as U4 {
        unsafe {
            match code[pc as usize] {
                // nop
                0x00 => {
                    pc = pc + 1;
                }
                // aconst_null
                0x01 => {
                    frame.operands.push(NULL);
                    pc = pc + 1;
                }
                // iconst -1 ~ 5
                0x02..=0x08 => {
                    let opr = code[pc as usize] as i32 - 3;
                    frame.operands.push(opr.memorized());
                    pc = pc + 1;
                }
                // lconst 0 ~ 1
                // byteorder: higher first
                0x09..=0x0a => {
                    let opr = code[pc as usize] as i64 - 9;
                    let (lower, higher) = opr.memorized();
                    frame.operands.push(higher);
                    frame.operands.push(lower);
                    pc = pc + 1;
                }
                // fconst 0 ~ 2
                0x0b..=0x0d => {
                    let opr = code[pc as usize] as f32 - 11.0;
                    frame.operands.push(opr.memorized());
                    pc = pc + 1;
                }
                // dconst 0 ~ 1
                0x0e..=0x0f => {
                    let opr = code[pc as usize] as f64 - 14.0;
                    let (lower, higher) = opr.memorized();
                    frame.operands.push(higher);
                    frame.operands.push(lower);
                    pc = pc + 1;
                }
                // bipush
                0x10 => {
                    frame
                        .operands
                        .push((code[(pc + 1) as usize] as i32).memorized());
                    pc = pc + 2;
                }
                // sipush
                0x11 => {
                    frame.operands.push(
                        ((code[(pc + 1) as usize] as i32) << 8 | (code[(pc + 2) as usize] as i32))
                            .memorized(),
                    );
                    pc = pc + 3;
                }
                // iload 0 ~ 3
                0x1a..=0x1d => {
                    let opr = code[pc as usize] as usize - 0x1a;
                    frame.operands.push(frame.locals[opr]);
                    pc = pc + 1;
                }
                // istore 0 ~ 3
                0x3b..=0x3e => {
                    if let Some(i) = frame.operands.pop() {
                        let opr = code[pc as usize] as usize - 0x3b;
                        frame.locals[opr] = i;
                        pc = pc + 1;
                    } else {
                        panic!("invalid frame: locals");
                    }
                }
                // iadd
                0x60 => {
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
                    let index = code[(pc + 1) as usize] as usize;
                    let cst = code[(pc + 2) as usize] as i32;
                    let new = std::mem::transmute::<Slot, i32>(frame.locals[index]) + cst;
                    frame.locals[index] = new.memorized();
                    pc = pc + 3;
                }
                // if_icmpge
                0xa2 => {
                    let size = frame.operands.len();
                    let v1 = std::mem::transmute::<Slot, i32>(frame.operands[size - 2]);
                    let v2 = std::mem::transmute::<Slot, i32>(frame.operands[size - 1]);
                    if v1 >= v2 {
                        pc = (code[(pc + 1) as usize] as U4) << 8 | code[(pc + 2) as usize] as U4;
                    } else {
                        pc = pc + 3;
                    }
                }
                // goto
                0xa7 => {
                    pc = (code[(pc + 1) as usize] as U4) << 8 | code[(pc + 2) as usize] as U4;
                }
                0xb1 => {
                    return Ok(Return::Void);
                }
                // getstatic
                0xb2 => {
                    let field_idx =
                        (code[(pc + 1) as usize] as U2) << 8 | code[(pc + 2) as usize] as U2;
                    let klass = frame.klass.clone();
                    let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                    // TODO load class `c`, push `f` to operands according to the type `t`
                    let class = load_class(c)?;
                    if let Some(ref field) = class.bytecode.get_field(f, t) {
                        match &field.value.get() {
                            // expect static field
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
                    let field_idx =
                        (code[(pc + 1) as usize] as U2) << 8 | code[(pc + 2) as usize] as U2;
                    let klass = frame.klass.clone();
                    let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                    // TODO load class `c`, push `f` to operands according to the type `t`
                    let class = load_class(c)?;
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

fn call(klass: &Klass, method: &Method, mut args: Vec<Slot>) -> Result<Return, JavaError> {
    println!(
        "execute method {}.{}",
        &klass.bytecode.this_class_name, &method.name
    );
    if let Some(&Attribute::Code(stacks, locals, ref code, ref exception_handler, ref attributes)) =
        method.get_code()
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
                    0x02..=0x08 => {
                        let opr = code[pc as usize] as i32 - 3;
                        operands.push(opr.memorized());
                        pc = pc + 1;
                    }
                    // lconst 0 ~ 1
                    // byteorder: higher first
                    0x09..=0x0a => {
                        let opr = code[pc as usize] as i64 - 9;
                        let (lower, higher) = opr.memorized();
                        operands.push(higher);
                        operands.push(lower);
                        pc = pc + 1;
                    }
                    // fconst 0 ~ 2
                    0x0b..=0x0d => {
                        let opr = code[pc as usize] as f32 - 11.0;
                        operands.push(opr.memorized());
                        pc = pc + 1;
                    }
                    // dconst 0 ~ 1
                    0x0e..=0x0f => {
                        let opr = code[pc as usize] as f64 - 14.0;
                        let (lower, higher) = opr.memorized();
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
                    0x1a..=0x1d => {
                        let opr = code[pc as usize] as usize - 0x1a;
                        operands.push(locals[opr]);
                        pc = pc + 1;
                    }
                    // istore 0 ~ 3
                    0x3b..=0x3e => {
                        if let Some(i) = operands.pop() {
                            let opr = code[pc as usize] as usize - 0x3b;
                            locals[opr] = i;
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
                        pc = (code[(pc + 1) as usize] as U4) << 8 | code[(pc + 2) as usize] as U4;
                    }
                    0xb1 => {
                        return Ok(Return::Void);
                    }
                    // getstatic
                    0xb2 => {
                        let field_idx =
                            (code[(pc + 1) as usize] as U2) << 8 | code[(pc + 2) as usize] as U2;
                        let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                        // TODO load class `c`, push `f` to operands according to the type `t`
                        let class = load_class(c)?;
                        if let Some(ref field) = class.bytecode.get_field(f, t) {
                            match &field.value.get() {
                                // non-static
                                None => {
                                    panic!("");
                                }
                                Some(value) => match value {
                                    Value::Word(v) => {
                                        operands.push(*v);
                                    }
                                    Value::DWord(lower, higher) => {
                                        operands.push(*higher);
                                        operands.push(*lower);
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
                        let field_idx =
                            (code[(pc + 1) as usize] as U2) << 8 | code[(pc + 2) as usize] as U2;
                        let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                        // TODO load class `c`, push `f` to operands according to the type `t`
                        let class = load_class(c)?;
                        if let Some(ref field) = class.bytecode.get_field(f, t) {
                            // TODO params validation
                            if t == "D" || t == "J" {
                                if let Some(lower) = operands.pop() {
                                    if let Some(higher) = operands.pop() {
                                        &field.value.set(Some(Value::DWord(lower, higher)));
                                    }
                                }
                            } else {
                                if let Some(v) = operands.pop() {
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
                        panic!(format!(
                            "directive {:?} not implemented at present",
                            code[pc as usize]
                        ));
                    }
                }
            }
        }
        // TODO
        Ok(Return::Void)
    } else {
        // TODO
        Err(fire_exception("", "", -1, "AbstractMethodError"))
    }
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
