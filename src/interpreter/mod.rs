pub mod thread;

use crate::bytecode::{atom::*, constant_pool::ConstantItem, *};
use crate::mem::{heap::*, klass::*, metaspace::*, stack::*, *};
use std::sync::Arc;
use thread::ThreadContext;

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

pub fn execute_frame(context: &mut ThreadContext) {}

pub fn execute(context: &mut ThreadContext) {
    if context.stack.is_empty() {
        return;
    }
    // let mut context.pc: usize = context.stack.frames.last().expect("Won't happend").context.pc;
    while context.stack.has_next(context.pc) {
        let instruction = {
            let frame = context.stack.frames.last().expect("Won't happend");
            // TODO if debug
            frame.dump(context.pc);
            context.stack.code_at(context.pc)
        };
        match instruction {
            // nop
            0x00 => {
                context.pc = context.pc + 1;
            }
            // aconst_null
            0x01 => {
                context.stack.push(&NULL, PTR_SIZE);
                context.pc = context.pc + 1;
            }
            // iconst -1 ~ 5
            0x02..=0x08 => {
                let opr = context.stack.code_at(context.pc) as i32 - 3;
                context.stack.push(&opr.to_le_bytes(), PTR_SIZE);
                context.pc = context.pc + 1;
            }
            // lconst 0 ~ 1
            // byteorder: higher first
            0x09..=0x0a => {
                let opr = context.stack.code_at(context.pc) as i64 - 9;
                context.stack.push(&opr.to_le_bytes(), 2 * PTR_SIZE);
                context.pc = context.pc + 1;
            }
            // fconst 0 ~ 2
            0x0b..=0x0d => {
                let opr = context.stack.code_at(context.pc) as f32 - 11.0;
                context.stack.push(&opr.to_le_bytes(), PTR_SIZE);
                context.pc = context.pc + 1;
            }
            // dconst 0 ~ 1
            0x0e..=0x0f => {
                let opr = context.stack.code_at(context.pc) as f64 - 14.0;
                context.stack.push(&opr.to_le_bytes(), 2 * PTR_SIZE);
                context.pc = context.pc + 1;
            }
            // bipush
            0x10 => {
                context.stack.push(
                    &(context.stack.code_at(context.pc + 1) as i32).to_le_bytes(),
                    PTR_SIZE,
                );
                context.pc = context.pc + 2;
            }
            // sipush
            0x11 => {
                let opr = (context.stack.code_at(context.pc + 1) as i32) << 8
                    | (context.stack.code_at(context.pc + 2) as i32);
                context.stack.push(&opr.to_le_bytes(), PTR_SIZE);
                context.pc = context.pc + 3;
            }
            // ldc
            0x12 => {
                let class = context.stack.class();
                let v = match context
                    .stack
                    .class()
                    .constant_pool
                    .get(context.stack.code_at(context.pc + 1) as U2)
                {
                    ConstantItem::Float(f) => f.to_le_bytes(),
                    ConstantItem::Integer(i) => i.to_le_bytes(),
                    // TODO String
                    _ => panic!("Illegal class file"),
                };
                context.stack.push(&v, PTR_SIZE);
                context.pc = context.pc + 2;
            }
            // ldc2w
            0x14 => {
                let opr = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let v = match context.stack.class().constant_pool.get(opr) {
                    ConstantItem::Double(d) => d.to_le_bytes(),
                    ConstantItem::Long(l) => l.to_le_bytes(),
                    _ => panic!("Illegal class file"),
                };
                context.stack.push(&v, 2 * PTR_SIZE);
                context.pc = context.pc + 3;
            }
            // iload/fload
            0x15 | 0x17 => {
                let opr = context.stack.code_at(context.pc + 1) as usize;
                context.stack.load(opr, 1);
                context.pc = context.pc + 2;
            }
            // lload/dload
            0x16 | 0x18 => {
                let opr = context.stack.code_at(context.pc + 1) as usize;
                context.stack.load(opr, 2);
                context.pc = context.pc + 2;
            }
            // iload 0 ~ 3
            0x1a..=0x1d => {
                let opr = context.stack.code_at(context.pc) as usize - 0x1a;
                context.stack.load(opr, 1);
                context.pc = context.pc + 1;
            }
            // lload 0 ~ 3
            0x1e..=0x21 => {
                let opr = context.stack.code_at(context.pc) as usize - 0x1e;
                context.stack.load(opr, 2);
                context.pc = context.pc + 1;
            }
            // fload 0 ~ 3
            0x22..=0x25 => {
                let opr = context.stack.code_at(context.pc) as usize - 0x22;
                context.stack.load(opr, 1);
                context.pc = context.pc + 1;
            }
            // dload 0 ~ 3
            0x26..=0x29 => {
                let opr = context.stack.code_at(context.pc) as usize - 0x26;
                context.stack.load(opr, 2);
                context.pc = context.pc + 1;
            }
            // aload 0 ~ 3
            0x2a..=0x2d => {
                let opr = context.stack.code_at(context.pc) as usize - 0x2a;
                context.stack.load(opr, 1);
                context.pc = context.pc + 1;
            }
            // istore/fstore
            0x36 | 0x38 | 0x3a => {
                context
                    .stack
                    .store(context.stack.code_at(context.pc + 1) as usize, 1);
                context.pc = context.pc + 2;
            }
            // lstore/dstore
            0x37 | 0x39 => {
                context
                    .stack
                    .store(context.stack.code_at(context.pc + 1) as usize, 2);
                context.pc = context.pc + 2;
            }
            // istore 0 ~ 3
            0x3b..=0x3e => {
                let opr = context.stack.code_at(context.pc) as usize - 0x3b;
                context.stack.store(opr, 1);
                context.pc = context.pc + 1;
            }
            // lstore 0 ~ 3
            0x3f..=0x42 => {
                let opr = context.stack.code_at(context.pc) as usize - 0x3f;
                context.stack.store(opr, 2);
                context.pc = context.pc + 2;
            }
            // fstore 0 ~ 3
            0x43..=0x46 => {
                let opr = context.stack.code_at(context.pc) as usize - 0x43;
                context.stack.store(opr, 1);
                context.pc = context.pc + 1;
            }
            // dstore 0 ~ 3
            0x47..=0x4a => {
                let opr = context.stack.code_at(context.pc) as usize - 0x47;
                context.stack.store(opr, 2);
                context.pc = context.pc + 2;
            }
            // astore 0 ~ 3
            0x4b..=0x4e => {
                let opr = context.stack.code_at(context.pc) as usize - 0x4b;
                context.stack.store(opr, 1);
                context.pc = context.pc + 1;
            }
            // pop
            0x57 => {
                context.stack.pop();
                context.pc = context.pc + 1;
            }
            // pop2
            0x58 => {
                context.stack.pop_w();
                context.pc = context.pc + 1;
            }
            // dup
            0x59 => {
                let current = context
                    .stack
                    .frames
                    .last_mut()
                    .expect("Illegal operands context.stack:");
                unsafe {
                    let dup = current.ptr.sub(PTR_SIZE);
                    current.ptr.copy_from(dup, PTR_SIZE);
                    current.ptr = current.ptr.add(PTR_SIZE);
                }
                context.pc = context.pc + 1;
            }
            // iadd
            0x60 => {
                context
                    .stack
                    .bi_op(|a, b| (i32::from_le_bytes(a) + i32::from_le_bytes(b)).to_le_bytes());
                context.pc = context.pc + 1;
            }
            0x64 => {
                context
                    .stack
                    .bi_op(|a, b| (i32::from_le_bytes(a) + i32::from_le_bytes(b)).to_le_bytes());
                context.pc = context.pc + 1;
            }
            // lsub
            0x65 => {
                context
                    .stack
                    .bi_op_w(|a, b| (i64::from_le_bytes(a) + i64::from_le_bytes(b)).to_le_bytes());
                context.pc = context.pc + 1;
            }
            // fmul
            0x6a => {
                context
                    .stack
                    .bi_op(|a, b| (f32::from_le_bytes(a) * f32::from_le_bytes(b)).to_le_bytes());
                context.pc = context.pc + 1;
            }
            // ddiv
            0x6f => {
                context
                    .stack
                    .bi_op_w(|a, b| (f64::from_le_bytes(a) / f64::from_le_bytes(b)).to_le_bytes());
                context.pc = context.pc + 1;
            }
            // iinc
            0x84 => {
                let index = context.stack.code_at(context.pc + 1) as usize;
                let cst = context.stack.code_at(context.pc + 2) as i32;
                let new = i32::from_le_bytes(context.stack.get(index)) + cst;
                context.stack.set(index, new.to_le_bytes());
                context.pc = context.pc + 3;
            }
            // ifeq, ifne, iflt, ifge, ifgt, ifle
            0x99..=0x9e => {
                let opr = i32::from_le_bytes(context.stack.pop());
                if opr == 0 && instruction == 0x99
                    || opr != 0 && instruction == 0x9a
                    || opr < 0 && instruction == 0x9b
                    || opr >= 0 && instruction == 0x9c
                    || opr > 0 && instruction == 0x9d
                    || opr <= 0 && instruction == 0x9e
                {
                    let jump = (context.stack.code_at(context.pc + 1) as U2) << 8
                        | context.stack.code_at(context.pc + 2) as U2;
                    context.pc = jump as usize;
                } else {
                    context.pc = context.pc + 3;
                }
            }
            // if_icmpge
            0xa2 => {
                let (v1, v2) = {
                    let v2 = i32::from_le_bytes(context.stack.pop());
                    let v1 = i32::from_le_bytes(context.stack.pop());
                    (v1, v2)
                };
                if v1 >= v2 {
                    let offset = ((context.stack.code_at(context.pc + 1) as i16) << 8
                        | context.stack.code_at(context.pc + 2) as i16)
                        as isize;
                    context.pc = (context.pc as isize + offset) as usize;
                } else {
                    context.pc = context.pc + 3;
                }
            }
            // goto
            0xa7 => {
                let offset = ((context.stack.code_at(context.pc + 1) as i16) << 8
                    | context.stack.code_at(context.pc + 2) as i16)
                    as i16;
                context.pc = (context.pc as isize + offset as isize) as usize;
            }
            // return
            0xb1 => {
                context.pc = context.stack.backtrack();
            }
            // getstatic
            0xb2 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let class = context.stack.class();
                let (c, (f, t)) = class.constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                // TODO load class `c`, push `f` to operands according to the type `t`
                let (klass, initialized) = class_arena!()
                    .load_class(&c, context)
                    .expect("ClassNotFoundException");
                if !initialized {
                    continue;
                }
                if let Some(ref field) = klass.bytecode.get_field(&f, &t) {
                    match &field.value.get() {
                        None => panic!(""),
                        Some(value) => match value {
                            Value::DWord(_) => {
                                context.stack.push(&Value::of_w(*value), 2 * PTR_SIZE)
                            }
                            _ => context.stack.push(&Value::of(*value), PTR_SIZE),
                        },
                    }
                } else {
                    // TODO
                    // handle_exception();
                    panic!("NoSuchFieldException");
                }
                context.pc = context.pc + 3;
            }
            // putstatic
            0xb3 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (f, t)) = context.stack.class().constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                // TODO load class `c`, push `f` to operands according to the type `t`
                let (klass, initialized) = class_arena!()
                    .load_class(&c, context)
                    .expect("ClassNotFoundException");
                if !initialized {
                    continue;
                }
                if let Some(ref field) = klass.bytecode.get_field(&f, &t) {
                    &field.value.set(match t.as_ref() {
                        "D" | "J" => Some(Value::eval_w(context.stack.pop_w())),
                        _ => Some(Value::eval(context.stack.pop(), &t)),
                    });
                } else {
                    // TODO
                    panic!("NoSuchFieldException");
                }
                context.pc = context.pc + 3;
            }
            // getfield
            0xb4 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                // let frame = context.stack.frames.last().expect("Illegal class file");
                // let klass = frame.klass.clone();
                let (c, (f, t)) = context.stack.class().constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                // TODO load class `c`, push `f` to operands according to the type `t`
                let klass = class_arena!()
                    .load_class(&c, context)
                    .expect("ClassNotFoundException")
                    .0;
                let objref = context.stack.pop();
                if objref == NULL {
                    // TODO
                    panic!("NullPointerException");
                }
                let objref = u32::from_le_bytes(objref);
                // TODO
                let (offset, len) = klass
                    .layout
                    .get(&(c.as_ref(), f.as_ref(), t.as_ref()))
                    .expect("NoSuchFieldException");
                context.stack.fetch_heap(objref, *offset, *len);
                context.pc = context.pc + 3;
            }
            // putfield
            0xb5 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (f, t)) = context.stack.class().constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                // TODO load class `c`, push `f` to operands according to the type `t`
                let klass = class_arena!()
                    .load_class(&c, context)
                    .expect("ClassNotFoundException")
                    .0;
                let objref = context.stack.pop();
                if objref == NULL {
                    // TODO
                    panic!("NullPointerException");
                }
                let objref = u32::from_le_bytes(objref);
                // TODO
                let (offset, len) = klass
                    .layout
                    .get(&(c.as_ref(), f.as_ref(), t.as_ref()))
                    .expect("NoSuchFieldException");
                context.stack.set_heap_aligned(objref, *offset, *len);
                context.pc = context.pc + 3;
            }
            // invokevirtual
            0xb6 => {
                let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let class = context.stack.class();
                let (c, (m, t)) = class.constant_pool.get_javaref(method_idx);
                let (c, m, t) = (c.to_string(), m.to_string(), t.to_string());
                let addr = u32::from_le_bytes(context.stack.top());
                let heap_ptr = jvm_heap!().base;
                let klass = unsafe {
                    let obj_header = ObjectHeader::from_vm_raw(heap_ptr.add(addr as usize));
                    Arc::from_raw(obj_header.klass)
                };
                if let Some(method_ref) = klass.get_method_in_vtable(&m, &t) {
                    let new_frame = JavaFrame::new((*method_ref).0, (*method_ref).1);
                    context.pc = context.stack.invoke(new_frame, context.pc + 3);
                } else if let Some(method) = klass.bytecode.get_method(&m, &t) {
                    if !method.is_final() {
                        panic!("ClassVerifyError");
                    }
                    let new_frame =
                        JavaFrame::new(Arc::as_ptr(&klass.bytecode), Arc::as_ptr(&method));
                    context.pc = context.stack.invoke(new_frame, context.pc + 3);
                } else {
                    panic!("NoSuchMethodError");
                }
            }
            // invokespecial
            0xb7 => {
                let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (m, t)) = context.stack.class().constant_pool.get_javaref(method_idx);
                let (c, m, t) = (c.to_string(), m.to_string(), t.to_string());
                // TODO
                let klass = class_arena!()
                    .load_class(&c, context)
                    .expect("ClassNotFoundException")
                    .0;
                if let Some(method) = klass.bytecode.get_method(&m, &t) {
                    let new_frame =
                        JavaFrame::new(Arc::as_ptr(&klass.bytecode), Arc::as_ptr(&method));
                    context.pc = context.stack.invoke(new_frame, context.pc + 3);
                } else {
                    // let mut current = klass;
                    // loop {
                    //     match &current.superclass {
                    //         Some(superclass) => {
                    //             if let Some(method) = superclass.bytecode.get_method(m, t) {
                    //                 let new_frame = JavaFrame::new(current, method);
                    //                 context.pc = context.stack.invoke(new_frame, context.pc + 3);
                    //                 break;
                    //             }
                    //             current = Arc::clone(superclass);
                    //         }
                    //         None => panic!("NoSuchMethodError"),
                    //     }
                    // }
                }
            }
            // invokestatic
            0xb8 => {
                let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (m, t)) = context.stack.class().constant_pool.get_javaref(method_idx);
                // TODO
                let (c, m, t) = (c.to_string(), m.to_string(), t.to_string());
                let (klass, initialized) = class_arena!()
                    .load_class(&c, context)
                    .expect("ClassNotFoundException");
                if !initialized {
                    continue;
                }
                if let Some(method) = klass.bytecode.get_method(&m, &t) {
                    // TODO
                    if !method.is_static() {
                        panic!("");
                    }
                    let new_frame =
                        JavaFrame::new(Arc::as_ptr(&klass.bytecode), Arc::as_ptr(&method));
                    context.pc = context.stack.invoke(new_frame, context.pc + 3);
                } else {
                    // TODO
                    panic!("NoSuchMethodException");
                }
            }
            // invokeinterface
            0xb9 => {
                let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (m, t)) = context.stack.class().constant_pool.get_javaref(method_idx);
                let addr = u32::from_le_bytes(context.stack.top());
                let heap_ptr = jvm_heap!().base;
                let klass = unsafe {
                    let obj_header = ObjectHeader::from_vm_raw(heap_ptr.add(addr as usize));
                    Arc::from_raw(obj_header.klass)
                };
                if let Some(method) = klass.bytecode.get_method(m, t) {
                    let new_frame =
                        JavaFrame::new(Arc::as_ptr(&klass.bytecode), Arc::as_ptr(&method));
                    context.pc = context.stack.invoke(new_frame, context.pc + 5);
                } else if let Some(method_ref) = klass.get_method_in_itable(c, m, t) {
                    let new_frame = JavaFrame::new((*method_ref).0, (*method_ref).1);
                    context.pc = context.stack.invoke(new_frame, context.pc + 5);
                } else {
                    panic!("NoSuchMethodError");
                }
            }
            // new
            0xbb => {
                let class_index = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let class_name = context
                    .stack
                    .class()
                    .constant_pool
                    .get_str(class_index)
                    .to_string();
                // TODO
                let (klass, initialized) = class_arena!()
                    .load_class(&class_name, context)
                    .expect("ClassNotFoundException");
                if !initialized {
                    continue;
                }
                let obj = jvm_heap!().allocate_object(&klass);
                let v = obj.to_le_bytes();
                println!("allocate object, addr: {}", obj);
                context.stack.push(&v, PTR_SIZE);
                context.pc = context.pc + 3;
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
