use crate::bytecode::{atom::*, constant_pool::ConstantItem, *};
use crate::mem::{heap::*, klass::*, metaspace::*, stack::*, *};
use std::sync::atomic::*;
use std::sync::Arc;

use log::trace;

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
                trace!("initialize class {}", klass.bytecode.get_name());
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
        if pc == 0 {
            let klass = stack.current_class();
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
                stack.push(&NULL, PTR_SIZE);
                pc = pc + 1;
            }
            // iconst -1 ~ 5
            0x02..=0x08 => {
                let opr = stack.code_at(pc) as i32 - 3;
                stack.push(&opr.to_le_bytes(), PTR_SIZE);
                pc = pc + 1;
            }
            // lconst 0 ~ 1
            // byteorder: higher first
            0x09..=0x0a => {
                let opr = stack.code_at(pc) as i64 - 9;
                stack.push(&opr.to_le_bytes(), 2 * PTR_SIZE);
                pc = pc + 1;
            }
            // fconst 0 ~ 2
            0x0b..=0x0d => {
                let opr = stack.code_at(pc) as f32 - 11.0;
                stack.push(&opr.to_le_bytes(), PTR_SIZE);
                pc = pc + 1;
            }
            // dconst 0 ~ 1
            0x0e..=0x0f => {
                let opr = stack.code_at(pc) as f64 - 14.0;
                stack.push(&opr.to_le_bytes(), 2 * PTR_SIZE);
                pc = pc + 1;
            }
            // bipush
            0x10 => {
                stack.push(&(stack.code_at(pc + 1) as i32).to_le_bytes(), PTR_SIZE);
                pc = pc + 2;
            }
            // sipush
            0x11 => {
                let opr = (stack.code_at(pc + 1) as i32) << 8 | (stack.code_at(pc + 2) as i32);
                stack.push(&opr.to_le_bytes(), PTR_SIZE);
                pc = pc + 3;
            }
            // ldc
            0x12 => {
                let klass = stack.current_class();
                match klass
                    .bytecode
                    .constant_pool
                    .get(stack.code_at(pc + 1) as U2)
                {
                    ConstantItem::Float(f) => stack.push(&f.to_le_bytes(), PTR_SIZE),
                    ConstantItem::Integer(i) => stack.push(&i.to_le_bytes(), PTR_SIZE),
                    // TODO String
                    _ => panic!("Illegal class file"),
                }
                pc = pc + 2;
            }
            // ldc2w
            0x14 => {
                let opr = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack.current_class();
                match klass.bytecode.constant_pool.get(opr) {
                    ConstantItem::Double(d) => stack.push(&d.to_le_bytes(), 2 * PTR_SIZE),
                    ConstantItem::Long(l) => stack.push(&l.to_le_bytes(), 2 * PTR_SIZE),
                    _ => panic!("Illegal class file"),
                }
                pc = pc + 3;
            }
            // iload/fload
            0x15 | 0x17 => {
                let opr = stack.code_at(pc + 1) as usize;
                stack.load(opr, 1);
                pc = pc + 2;
            }
            // lload/dload
            0x16 | 0x18 => {
                let opr = stack.code_at(pc + 1) as usize;
                stack.load(opr, 2);
                pc = pc + 2;
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
            // istore/fstore
            0x36 | 0x38 | 0x3a => {
                stack.store(stack.code_at(pc + 1) as usize, 1);
                pc = pc + 2;
            }
            // lstore/dstore
            0x37 | 0x39 => {
                stack.store(stack.code_at(pc + 1) as usize, 2);
                pc = pc + 2;
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
            // pop
            0x57 => {
                stack.pop();
                pc = pc + 1;
            }
            // pop2
            0x58 => {
                stack.pop_w();
                pc = pc + 1;
            }
            // dup
            0x59 => {
                let current = stack.frames.last_mut().expect("Illegal operands stack:");
                unsafe {
                    let dup = current.ptr.sub(PTR_SIZE);
                    current.ptr.copy_from(dup, PTR_SIZE);
                    current.ptr = current.ptr.add(PTR_SIZE);
                }
                pc = pc + 1;
            }
            // iadd
            0x60 => {
                stack.bi_op(|a, b| (i32::from_le_bytes(a) + i32::from_le_bytes(b)).to_le_bytes());
                pc = pc + 1;
            }
            // lsub
            0x65 => {
                stack.bi_op_w(|a, b| (i64::from_le_bytes(a) + i64::from_le_bytes(b)).to_le_bytes());
                pc = pc + 1;
            }
            // fmul
            0x6a => {
                stack.bi_op(|a, b| (f32::from_le_bytes(a) * f32::from_le_bytes(b)).to_le_bytes());
                pc = pc + 1;
            }
            // ddiv
            0x6f => {
                stack.bi_op_w(|a, b| (f64::from_le_bytes(a) / f64::from_le_bytes(b)).to_le_bytes());
                pc = pc + 1;
            }
            // iinc
            0x84 => {
                let index = stack.code_at(pc + 1) as usize;
                let cst = stack.code_at(pc + 2) as i32;
                let new = i32::from_le_bytes(stack.get(index)) + cst;
                stack.set(index, new.to_le_bytes());
                pc = pc + 3;
            }
            // ifeq, ifne, iflt, ifge, ifgt, ifle
            0x99..=0x9e => {
                let opr = i32::from_le_bytes(stack.pop());
                if opr == 0 && instruction == 0x99
                    || opr != 0 && instruction == 0x9a
                    || opr < 0 && instruction == 0x9b
                    || opr >= 0 && instruction == 0x9c
                    || opr > 0 && instruction == 0x9d
                    || opr <= 0 && instruction == 0x9e
                {
                    let jump = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                    pc = jump as usize;
                } else {
                    pc = pc + 3;
                }
            }
            // if_icmpge
            0xa2 => {
                let (v1, v2) = {
                    let v2 = i32::from_le_bytes(stack.pop());
                    let v1 = i32::from_le_bytes(stack.pop());
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
                let field_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack.current_class();
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
                            Value::DWord(_) => stack.push(&Value::of_w(*value), 2 * PTR_SIZE),
                            _ => stack.push(&Value::of(*value), PTR_SIZE),
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
                let klass = stack.current_class();
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
            // getfield
            0xb4 => {
                let field_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let frame = stack.frames.last().expect("Illegal class file");
                let klass = frame.klass.clone();
                let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                // TODO load class `c`, push `f` to operands according to the type `t`
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                let objref = stack.pop();
                println!("{:2x?}", objref);
                if objref == NULL {
                    // TODO
                    panic!("NullPointerException");
                }
                let objref = u32::from_le_bytes(objref);
                // TODO
                let (offset, len) = klass.layout.get(&(c, f, t)).expect("NoSuchFieldException");
                stack.fetch_heap(objref, *offset, *len);
                pc = pc + 3;
            }
            // putfield
            0xb5 => {
                let field_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let frame = stack.frames.last().expect("Illegal class file");
                let klass = frame.klass.clone();
                let (c, (f, t)) = klass.bytecode.constant_pool.get_javaref(field_idx);
                // TODO load class `c`, push `f` to operands according to the type `t`
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                let objref = stack.pop();
                println!("{:2x?}", objref);
                if objref == NULL {
                    // TODO
                    panic!("NullPointerException");
                }
                let objref = u32::from_le_bytes(objref);
                // TODO
                let (offset, len) = klass.layout.get(&(c, f, t)).expect("NoSuchFieldException");

                pc = pc + 3;
            }
            // invokevirtual
            0xb6 => {
                let method_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack.current_class();
                let (c, (m, t)) = klass.bytecode.constant_pool.get_javaref(method_idx);
                // TODO
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(method) = klass.get_method_in_vtable(m, t) {
                    let new_frame = JavaFrame::new(klass, method);
                    pc = stack.invoke(new_frame, pc + 3);
                } else if let Some(method) = klass.bytecode.get_method(m, t) {
                    if !method.is_final() {
                        panic!("ClassVerifyError");
                    }
                    let new_frame = JavaFrame::new(klass, method);
                    pc = stack.invoke(new_frame, pc + 3);
                } else {
                    panic!("NoSuchMethodError");
                }
            }
            // invokespecial
            0xb7 => {
                let method_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack.current_class();
                let (c, (m, t)) = klass.bytecode.constant_pool.get_javaref(method_idx);
                // TODO
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(method) = klass.bytecode.get_method(m, t) {
                    let new_frame = JavaFrame::new(klass, method);
                    pc = stack.invoke(new_frame, pc + 3);
                } else {
                    let mut current = klass;
                    loop {
                        match &current.superclass {
                            Some(superclass) => {
                                if let Some(method) = superclass.bytecode.get_method(m, t) {
                                    let new_frame = JavaFrame::new(current, method);
                                    pc = stack.invoke(new_frame, pc + 3);
                                    break;
                                }
                                current = Arc::clone(superclass);
                            }
                            None => panic!("NoSuchMethodError"),
                        }
                    }
                }
            }
            // invokestatic
            0xb8 => {
                let method_idx = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack.current_class();
                let (c, (m, t)) = klass.bytecode.constant_pool.get_javaref(method_idx);
                // TODO
                let klass = find_class!(c).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                if let Some(ref method) = klass.bytecode.get_method(m, t) {
                    // TODO
                    if !method.is_static() {
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
                let class_index = (stack.code_at(pc + 1) as U2) << 8 | stack.code_at(pc + 2) as U2;
                let klass = stack.current_class();
                let class_name = klass.bytecode.constant_pool.get_str(class_index);
                // TODO
                let klass = find_class!(class_name).expect("ClassNotFoundException");
                if !ensure_initialized(stack, klass.clone(), pc) {
                    pc = 0;
                    continue;
                }
                let obj = jvm_heap!().allocate_object(&klass);
                let v = obj.to_le_bytes();
                println!("allocate object, addr: {}", obj);
                stack.push(&v, PTR_SIZE);
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
