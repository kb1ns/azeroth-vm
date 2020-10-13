pub mod thread;

use self::thread::ThreadContext;
use crate::{
    bytecode,
    bytecode::{atom::*, constant_pool::ConstantItem},
    mem::{heap::Heap, klass::*, metaspace::*, strings::Strings, *},
};
// use crate::{
//     bytecode,
//     bytecode::{atom::*, constant_pool::ConstantItem},
// };
use std::sync::Arc;

use log::trace;

macro_rules! math_bi {
    ($l: tt, $r: tt, $op: tt) => {
        |a, b| ($l::from_le_bytes(a) $op $r::from_le_bytes(b)).to_le_bytes()
    }
}

macro_rules! math_un {
    ($t: tt, $op: tt) => {
        |a| ($op $t::from_le_bytes(a)).to_le_bytes()
    }
}

pub fn execute(context: &mut ThreadContext) {
    while context.stack.has_next(context.pc) {
        let pause = context.rx.try_recv();
        if pause.is_ok() {
            context.tx.send(context.roots()).unwrap();
            // waiting signal
            let _ = context.rx.recv().unwrap();
        }
        // handle_exception
        if context.exception_pending {
            handle_exception(context);
        }
        let instruction = context.stack.code_at(context.pc);
        context.stack.dump(context.pc);
        match instruction {
            // nop
            0x00 => {
                context.pc = context.pc + 1;
            }
            // aconst_null
            0x01 => {
                context.stack.push(&NULL);
                context.pc = context.pc + 1;
            }
            // iconst -1 ~ 5
            0x02..=0x08 => {
                let opr = context.stack.code_at(context.pc) as i32 - 3;
                context.stack.push(&opr.to_le_bytes());
                context.pc = context.pc + 1;
            }
            // lconst 0 ~ 1
            0x09..=0x0a => {
                let opr = context.stack.code_at(context.pc) as i64 - 9;
                context.stack.push_w(&opr.to_le_bytes());
                context.pc = context.pc + 1;
            }
            // fconst 0 ~ 2
            0x0b..=0x0d => {
                let opr = context.stack.code_at(context.pc) as f32 - 11.0;
                context.stack.push(&opr.to_le_bytes());
                context.pc = context.pc + 1;
            }
            // dconst 0 ~ 1
            0x0e..=0x0f => {
                let opr = context.stack.code_at(context.pc) as f64 - 14.0;
                context.stack.push_w(&opr.to_le_bytes());
                context.pc = context.pc + 1;
            }
            // bipush
            0x10 => {
                context
                    .stack
                    .push(&(context.stack.code_at(context.pc + 1) as i32).to_le_bytes());
                context.pc = context.pc + 2;
            }
            // sipush
            0x11 => {
                let opr = (context.stack.code_at(context.pc + 1) as i32) << 8
                    | (context.stack.code_at(context.pc + 2) as i32);
                context.stack.push(&opr.to_le_bytes());
                context.pc = context.pc + 3;
            }
            // ldc
            0x12 => {
                let v = match context
                    .stack
                    .class()
                    .constant_pool
                    .get(context.stack.code_at(context.pc + 1) as U2)
                {
                    ConstantItem::Float(f) => f.to_le_bytes(),
                    ConstantItem::Integer(i) => i.to_le_bytes(),
                    ConstantItem::String(r) => {
                        let utf8 = context.stack.class().constant_pool.get(*r).clone();
                        match utf8 {
                            ConstantItem::UTF8(s) => Strings::get(&s, context).to_le_bytes(),
                            _ => panic!(""),
                        }
                    }
                    _ => panic!(""),
                };
                context.stack.push(&v);
                context.pc = context.pc + 2;
            }
            // ldc2w
            0x14 => {
                let opr = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let v = match context.stack.class().constant_pool.get(opr) {
                    ConstantItem::Double(d) => d.to_le_bytes(),
                    ConstantItem::Long(l) => l.to_le_bytes(),
                    _ => panic!(""),
                };
                context.stack.push_w(&v);
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
            // iaload
            0x2e => {
                let array_idx = u32::from_le_bytes(context.stack.pop()) as usize;
                let arrayref = u32::from_le_bytes(context.stack.pop()) as usize;
                // let current = context.stack.mut_frame();
                unsafe {
                    let header = ArrayHeader::from_vm_raw(Heap::ptr(arrayref));
                    if array_idx >= header.size as usize {
                        context
                            .stack
                            .update(context.stack.operands().add(2 * PTR_SIZE));
                        throw_vm_exception(context, "java/lang/ArrayIndexOutOfBoundsException");
                        continue;
                    }
                    // FIXME heap -> stack
                    let offset = (&*header.klass).len * array_idx;
                    context.stack.operands().copy_from(
                        Heap::ptr(arrayref + ARRAY_HEADER_LEN + offset),
                        (&*header.klass).len,
                    );
                    context
                        .stack
                        .update(context.stack.operands().add((&*header.klass).len));
                }
                context.pc = context.pc + 1;
            }
            // istore/fstore/astore
            0x36 | 0x38 | 0x3a => {
                let opr = context.stack.code_at(context.pc + 1) as usize;
                context.stack.store(opr, 1);
                context.pc = context.pc + 2;
            }
            // lstore/dstore
            0x37 | 0x39 => {
                let opr = context.stack.code_at(context.pc + 1) as usize;
                context.stack.store(opr, 2);
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
                context.pc = context.pc + 1;
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
                context.pc = context.pc + 1;
            }
            // astore 0 ~ 3
            0x4b..=0x4e => {
                let opr = context.stack.code_at(context.pc) as usize - 0x4b;
                context.stack.store(opr, 1);
                context.pc = context.pc + 1;
            }
            // iastore
            0x4f => {
                let v = context.stack.pop();
                let array_idx = u32::from_le_bytes(context.stack.pop()) as usize;
                let arrayref = u32::from_le_bytes(context.stack.pop()) as usize;
                unsafe {
                    let header = ArrayHeader::from_vm_raw(Heap::ptr(arrayref));
                    if array_idx >= header.size as usize {
                        context.stack.upward(3);
                        throw_vm_exception(context, "java/lang/ArrayIndexOutOfBoundsException");
                        continue;
                    }
                    let offset = (&*header.klass).len * array_idx;
                    Heap::ptr(arrayref + ARRAY_HEADER_LEN + offset)
                        .copy_from(v.as_ptr(), (&*header.klass).len);
                }
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
                unsafe {
                    context
                        .stack
                        .operands()
                        .copy_from(context.stack.operands().sub(PTR_SIZE), PTR_SIZE);
                }
                context.stack.upward(1);
                context.pc = context.pc + 1;
            }
            // i/l/f/d +,-,*,/,%,<<,>>,>>>
            0x60..=0x83 => {
                let code = context.stack.code_at(context.pc);
                match code {
                    0x60 => context.stack.bi_op(math_bi!(i32, i32, +)),
                    0x61 => context.stack.bi_op_w(math_bi!(i64, i64, +)),
                    0x62 => context.stack.bi_op(math_bi!(f32, f32, +)),
                    0x63 => context.stack.bi_op_w(math_bi!(f64, f64, +)),
                    0x64 => context.stack.bi_op(math_bi!(i32, i32, -)),
                    0x65 => context.stack.bi_op_w(math_bi!(i64, i64, -)),
                    0x66 => context.stack.bi_op(math_bi!(f32, f32, -)),
                    0x67 => context.stack.bi_op_w(math_bi!(f64, f64, -)),
                    0x68 => context.stack.bi_op(math_bi!(i32, i32, *)),
                    0x69 => context.stack.bi_op_w(math_bi!(i64, i64, *)),
                    0x6a => context.stack.bi_op(math_bi!(f32, f32, *)),
                    0x6b => context.stack.bi_op_w(math_bi!(f64, f64, *)),
                    0x6c => context.stack.bi_op(math_bi!(i32, i32, /)),
                    0x6d => context.stack.bi_op_w(math_bi!(i64, i64, /)),
                    0x6e => context.stack.bi_op(math_bi!(f32, f32, /)),
                    0x6f => context.stack.bi_op_w(math_bi!(f64, f64, /)),
                    0x70 => context.stack.bi_op(math_bi!(i32, i32, %)),
                    0x71 => context.stack.bi_op_w(math_bi!(i64, i64, %)),
                    0x72 => context.stack.bi_op(math_bi!(f32, f32, %)),
                    0x73 => context.stack.bi_op_w(math_bi!(f64, f64, %)),
                    0x74 => context.stack.un_op(math_un!(i32, -)),
                    0x75 => context.stack.un_op_w(math_un!(i64, -)),
                    0x76 => context.stack.un_op(math_un!(f32, -)),
                    0x77 => context.stack.un_op_w(math_un!(f64, -)),
                    0x78 => context.stack.bi_op(math_bi!(i32, i32, <<)),
                    0x7a => context.stack.bi_op(math_bi!(u32, u32, >>)),
                    0x7c => context.stack.bi_op(math_bi!(i32, i32, >>)),
                    0x79 => {
                        let s = u32::from_le_bytes(context.stack.pop());
                        context
                            .stack
                            .un_op_w(|d| (i64::from_le_bytes(d) << s).to_le_bytes());
                    }
                    0x7b => {
                        let s = u32::from_le_bytes(context.stack.pop());
                        context
                            .stack
                            .un_op_w(|d| (u64::from_le_bytes(d) << s).to_le_bytes());
                    }
                    0x7d => {
                        let s = u32::from_le_bytes(context.stack.pop());
                        context
                            .stack
                            .un_op_w(|d| (i64::from_le_bytes(d) << s).to_le_bytes());
                    }
                    0x7e => context.stack.bi_op(math_bi!(i32, i32, &)),
                    0x7f => context.stack.bi_op_w(math_bi!(i64, i64, &)),
                    0x80 => context.stack.bi_op(math_bi!(i32, i32, |)),
                    0x81 => context.stack.bi_op_w(math_bi!(i64, i64, |)),
                    0x82 => context.stack.bi_op(math_bi!(i32, i32, ^)),
                    0x83 => context.stack.bi_op_w(math_bi!(i64, i64, ^)),
                    _ => unreachable!(),
                }
                context.pc = context.pc + 1;
            }
            // iinc
            0x84 => {
                let index = context.stack.code_at(context.pc + 1) as usize;
                let cst = context.stack.code_at(context.pc + 2) as i32;
                let new = i32::from_le_bytes(context.stack.get(index)) + cst;
                context.stack.set(index, &new.to_le_bytes());
                context.pc = context.pc + 3;
            }
            // i2l,i2f,i2d,l2i,l2f,l2d,f2i,f2l,f2d,d2i,d2l,d2f,i2b,i2c,i2s
            0x85..=0x93 => {
                let code = context.stack.code_at(context.pc);
                match code {
                    // i2f, l2d, f2i, d2l, i2b, i2c, i2s
                    0x86 | 0x8a | 0x8b | 0x8f | 0x91 | 0x92 | 0x93 => {}
                    // i2l
                    0x85 => {
                        let v = i32::from_le_bytes(context.stack.pop());
                        context.stack.push_w(&(v as i64).to_le_bytes());
                    }
                    // i2d
                    0x87 => {
                        let v = i32::from_le_bytes(context.stack.pop());
                        context.stack.push_w(&(v as f64).to_le_bytes());
                    }
                    // l2i
                    0x88 => {
                        let v = i64::from_le_bytes(context.stack.pop_w());
                        context.stack.push(&(v as i32).to_le_bytes());
                    }
                    // l2f
                    0x89 => {
                        let v = i64::from_le_bytes(context.stack.pop_w());
                        context.stack.push(&(v as f32).to_le_bytes());
                    }
                    // f2l
                    0x8c => {
                        let v = f32::from_le_bytes(context.stack.pop());
                        context.stack.push_w(&(v as i64).to_le_bytes());
                    }
                    // f2d
                    0x8d => {
                        let v = f32::from_le_bytes(context.stack.pop());
                        context.stack.push_w(&(v as f64).to_le_bytes());
                    }
                    // d2i
                    0x8e => {
                        let v = f64::from_le_bytes(context.stack.pop_w());
                        context.stack.push(&(v as i32).to_le_bytes());
                    }
                    // d2f
                    0x90 => {
                        let v = f64::from_le_bytes(context.stack.pop_w());
                        context.stack.push(&(v as f32).to_le_bytes());
                    }
                    _ => unreachable!(),
                }
                context.pc = context.pc + 1;
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
            // if_icmp eq,ne,lt,ge,gt,le
            0x9f..=0xa4 => {
                let (v1, v2) = {
                    let v2 = i32::from_le_bytes(context.stack.pop());
                    let v1 = i32::from_le_bytes(context.stack.pop());
                    (v1, v2)
                };
                let code = context.stack.code_at(context.pc);
                let offset = if code == 0x9f && v1 == v2
                    || code == 0xa0 && v1 != v2
                    || code == 0xa1 && v1 < v2
                    || code == 0xa2 && v1 >= v2
                    || code == 0xa3 && v1 > v2
                    || code == 0xa4 && v1 <= v2
                {
                    ((context.stack.code_at(context.pc + 1) as i16) << 8
                        | context.stack.code_at(context.pc + 2) as i16) as isize
                } else {
                    3
                };
                context.pc = (context.pc as isize + offset) as usize;
            }
            // if_acmpeq, if_acmpne
            0xa5 | 0xa6 => {
                // TODO reference as u32
                let (v1, v2) = {
                    let v2 = Ref::from_le_bytes(context.stack.pop());
                    let v1 = Ref::from_le_bytes(context.stack.pop());
                    (v1, v2)
                };
                let code = context.stack.code_at(context.pc);
                let offset = if code == 0xa5 && v1 == v2 || code == 0xa6 && v1 != v2 {
                    ((context.stack.code_at(context.pc + 1) as i16) << 8
                        | context.stack.code_at(context.pc + 2) as i16) as isize
                } else {
                    3
                };
                context.pc = (context.pc as isize + offset) as usize;
            }
            // goto
            0xa7 => {
                let offset = ((context.stack.code_at(context.pc + 1) as i16) << 8
                    | context.stack.code_at(context.pc + 2) as i16)
                    as i16;
                context.pc = (context.pc as isize + offset as isize) as usize;
            }
            // ireturn/lreturn/freturn/dreturn/areturn/return
            0xac..=0xb1 => {
                context.pc = context.stack.return_normal();
            }
            // getstatic
            0xb2 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let class = unsafe { context.stack.class_ptr().as_ref() }
                    .expect("stack_class_pointer_null");
                let (c, (f, t)) = class.constant_pool.get_javaref(field_idx);
                let found = ClassArena::load_class(c, context);
                if found.is_err() {
                    throw_vm_exception(context, "java/lang/ClassNotFoundException");
                    continue;
                }
                let (klass, initialized) = found.unwrap();
                if !initialized {
                    continue;
                }
                if let Some(ref field) = klass.bytecode.as_ref().unwrap().get_field(f, t) {
                    match &field.value.get() {
                        None => {
                            throw_vm_exception(context, "java/lang/IncompatibleClassChangeError");
                            continue;
                        }
                        Some(value) => match value {
                            Value::DWord(_) => context.stack.push_w(&Value::of_w(*value)),
                            _ => context.stack.push(&Value::of(*value)),
                        },
                    }
                    context.pc = context.pc + 3;
                } else {
                    throw_vm_exception(context, "java/lang/NoSuchFieldError");
                }
            }
            // putstatic
            0xb3 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (f, t)) = context.stack.class().constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                let found = ClassArena::load_class(&c, context);
                if found.is_err() {
                    throw_vm_exception(context, "java/lang/ClassNotFoundException");
                    continue;
                }
                let (klass, initialized) = found.unwrap();
                if !initialized {
                    continue;
                }
                if let Some(ref field) = klass.bytecode.as_ref().unwrap().get_field(&f, &t) {
                    &field.value.set(match t.as_ref() {
                        "D" | "J" => Some(Value::eval_w(context.stack.pop_w())),
                        _ => Some(Value::eval(context.stack.pop(), &t)),
                    });
                    context.pc = context.pc + 3;
                } else {
                    throw_vm_exception(context, "java/lang/NoSuchFieldError");
                }
            }
            // getfield
            0xb4 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (f, t)) = context.stack.class().constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                let found = ClassArena::load_class(&c, context);
                if found.is_err() {
                    throw_vm_exception(context, "java/lang/ClassNotFoundException");
                    continue;
                }
                let klass = found.unwrap().0;
                let objref = context.stack.pop();
                if objref == NULL {
                    throw_vm_exception(context, "java/lang/NullPointerException");
                    continue;
                }
                let objref = u32::from_le_bytes(objref) as usize;
                let found = klass.layout.get(&(c.as_ref(), f.as_ref(), t.as_ref()));
                if found.is_none() {
                    throw_vm_exception(context, "java/lang/NoSuchFieldError");
                    continue;
                }
                // FIXME heap -> stack
                let (offset, len) = found.unwrap();
                unsafe {
                    let target = Heap::ptr(objref + OBJ_HEADER_LEN + *offset);
                    context.stack.operands().copy_from(target, *len);
                    context.stack.upward(*len / PTR_SIZE);
                }
                context.pc = context.pc + 3;
            }
            // putfield
            0xb5 => {
                let field_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let (c, (f, t)) = context.stack.class().constant_pool.get_javaref(field_idx);
                let (c, f, t) = (c.to_string(), f.to_string(), t.to_string());
                let found = ClassArena::load_class(&c, context);
                if found.is_err() {
                    throw_vm_exception(context, "java/lang/ClassNotFoundException");
                    continue;
                }
                let klass = found.unwrap().0;
                let objref = context.stack.pop();
                if objref == NULL {
                    context.stack.upward(PTR_SIZE);
                    throw_vm_exception(context, "java/lang/NullPointerException");
                    continue;
                }
                let objref = u32::from_le_bytes(objref) as usize;
                let found = klass.layout.get(&(c.as_ref(), f.as_ref(), t.as_ref()));
                if found.is_none() {
                    throw_vm_exception(context, "java/lang/NoSuchFieldError");
                    continue;
                }
                // FIXME stack -> heap
                let (offset, len) = found.unwrap();
                unsafe {
                    let target = Heap::ptr(objref + OBJ_HEADER_LEN + *offset);
                    context.stack.downward(*len / PTR_SIZE);
                    target.copy_from(context.stack.operands(), *len);
                }
                context.pc = context.pc + 3;
            }
            // invokevirtual
            0xb6 => invoke_virtual(context),
            // invokespecial
            0xb7 => invoke_special(context),
            // invokestatic
            0xb8 => invoke_static(context),
            // invokeinterface
            0xb9 => invoke_interface(context),
            // new
            0xbb => {
                let class_index = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let class_name = context
                    .stack
                    .class()
                    .constant_pool
                    .get_str(class_index)
                    .to_owned();
                let found = ClassArena::load_class(&class_name, context);
                if found.is_err() {
                    throw_vm_exception(context, "java/lang/ClassNotFoundException");
                    continue;
                }
                let (klass, initialized) = found.unwrap();
                if !initialized {
                    continue;
                }
                let obj = Heap::allocate_object(&klass);
                let v = obj.to_le_bytes();
                context.stack.push(&v);
                trace!("allocate object, addr: {}", obj);
                context.pc = context.pc + 3;
            }
            // newarray
            0xbc => {
                let atype = match context.stack.code_at(context.pc + 1) {
                    4 => "[Z",
                    5 => "[C",
                    6 => "[F",
                    7 => "[D",
                    8 => "[B",
                    9 => "[S",
                    10 => "[I",
                    11 => "[J",
                    _ => unreachable!(),
                };
                let (klass, _) =
                    ClassArena::load_class(atype, context).expect("primitive_types_array");
                let size = u32::from_le_bytes(context.stack.pop());
                let array = Heap::allocate_array(&klass, size);
                let v = array.to_le_bytes();
                context.stack.push(&v);
                trace!("allocate array {}, addr:{}, size:{}", atype, array, size);
                context.pc = context.pc + 2;
            }
            // anewarray
            0xbd => {
                let class_index = (context.stack.code_at(context.pc + 1) as U2) << 8
                    | context.stack.code_at(context.pc + 2) as U2;
                let class_name =
                    "[".to_owned() + context.stack.class().constant_pool.get_str(class_index);
                let found = ClassArena::load_class(&class_name, context);
                if found.is_err() {
                    throw_vm_exception(context, "java/lang/ClassNotFoundException");
                    continue;
                }
                let (klass, initialized) = found.unwrap();
                if !initialized {
                    continue;
                }
                let size = u32::from_le_bytes(context.stack.pop());
                let array = Heap::allocate_array(&klass, size);
                let v = array.to_le_bytes();
                context.stack.push(&v);
                trace!(
                    "allocate array {}, addr:{}, size:{}",
                    class_name,
                    array,
                    size
                );
                context.pc = context.pc + 3;
            }
            // arraylength
            0xbe => {
                let addr = context.stack.pop();
                if addr == NULL {
                    throw_vm_exception(context, "java/lang/NullPointerException");
                    return;
                }
                let array = ArrayHeader::from_vm_raw(Heap::ptr(u32::from_le_bytes(addr) as usize));
                context.stack.push(&array.size.to_le_bytes());
                context.pc = context.pc + 1;
            }
            // athrow
            0xbf => {
                context.exception_pending = true;
                context.throwable_initialized = true;
            }
            _ => panic!(format!(
                "Instruction 0x{:2x?} not implemented yet.",
                instruction
            )),
        }
    }
}

fn invoke_virtual(context: &mut ThreadContext) {
    let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
        | context.stack.code_at(context.pc + 2) as U2;
    let (_, (m, t)) = context.stack.class().constant_pool.get_javaref(method_idx);
    // 0 for indicating non-static method
    let (_, slots, _) = bytecode::resolve_method_descriptor(t, 0);

    let addr = *context.stack.top_n(slots);
    if addr == NULL {
        throw_vm_exception(context, "java/lang/NullPointerException");
        return;
    }
    let obj = ObjectHeader::from_vm_raw(Heap::ptr(u32::from_le_bytes(addr) as usize));
    let klass = unsafe { obj.klass.as_ref() }.expect("obj_klass_pointer_null");
    if let Some(method) = klass.get_method_in_vtable(m, t) {
        context.pc = context
            .stack
            .invoke((*method).0, (*method).1, context.pc + 3, slots);
        return;
    }
    if let Some(method) = klass.bytecode.as_ref().unwrap().get_method(m, t) {
        if !method.is_final() {
            throw_vm_exception(context, "java/lang/IncompatibleClassChangeError");
            return;
        }
        let class = Arc::as_ptr(&klass.bytecode.as_ref().unwrap());
        let method = Arc::as_ptr(&method);
        context.pc = context.stack.invoke(class, method, context.pc + 3, slots);
        return;
    }
    throw_vm_exception(context, "java/lang/NoSuchMethodError");
}

fn invoke_interface(context: &mut ThreadContext) {
    let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
        | context.stack.code_at(context.pc + 2) as U2;
    let (c, (m, t)) = context.stack.class().constant_pool.get_javaref(method_idx);
    let (_, slots, _) = bytecode::resolve_method_descriptor(t, 0);
    let addr = *context.stack.top_n(slots);
    if addr == NULL {
        throw_vm_exception(context, "java/lang/NullPointerException");
        return;
    }
    let addr = u32::from_le_bytes(addr);
    let obj = ObjectHeader::from_vm_raw(Heap::ptr(addr as usize));
    let klass = unsafe { obj.klass.as_ref() }.expect("obj_klass_pointer_null");
    if let Some(method) = klass.get_method_in_itable(c, m, t) {
        context.pc = context
            .stack
            .invoke((*method).0, (*method).1, context.pc + 5, slots);
        return;
    }
    throw_vm_exception(context, "java/lang/NoSuchMethodError");
}

fn invoke_static(context: &mut ThreadContext) {
    let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
        | context.stack.code_at(context.pc + 2) as U2;
    let class = unsafe { context.stack.class_ptr().as_ref() }.expect("class_pointer_null");
    let (c, (m, t)) = class.constant_pool.get_javaref(method_idx);
    let found = ClassArena::load_class(c, context);
    if found.is_err() {
        throw_vm_exception(context, "java/lang/ClassNotFoundException");
        return;
    }
    let (klass, initialized) = found.unwrap();
    if !initialized {
        return;
    }
    let method = klass.bytecode.as_ref().unwrap().get_method(m, t);
    if method.is_none() {
        throw_vm_exception(context, "java/lang/NoSuchMethodError");
        return;
    }
    let method = method.unwrap();
    if !method.is_static() {
        throw_vm_exception(context, "java/lang/IncompatibleClassChangeError");
        return;
    }
    let (_, desc, access_flag) = method.get_name_and_descriptor();
    let (_, slots, _) = bytecode::resolve_method_descriptor(desc, access_flag);
    let class = Arc::as_ptr(&klass.bytecode.as_ref().unwrap());
    let method = Arc::as_ptr(&method);
    context.pc = context.stack.invoke(class, method, context.pc + 3, slots);
}

fn invoke_special(context: &mut ThreadContext) {
    let method_idx = (context.stack.code_at(context.pc + 1) as U2) << 8
        | context.stack.code_at(context.pc + 2) as U2;
    let class = unsafe { context.stack.class_ptr().as_ref() }.expect("class_pointer_null");
    let (c, (m, t)) = class.constant_pool.get_javaref(method_idx);
    let found = ClassArena::load_class(c, context);
    if found.is_err() {
        throw_vm_exception(context, "java/lang/ClassNotFoundException");
        return;
    }
    let (klass, initialized) = found.unwrap();
    // redundant check
    if !initialized {
        return;
    }
    // TODO subclass check
    let method = klass.bytecode.as_ref().unwrap().get_method(m, t);
    if method.is_none() {
        throw_vm_exception(context, "java/lang/NoSuchMethodError");
        return;
    }
    let method = method.unwrap();
    let (_, desc, access_flag) = method.get_name_and_descriptor();
    let (_, slots, _) = bytecode::resolve_method_descriptor(desc, access_flag);
    let class = Arc::as_ptr(&klass.bytecode.as_ref().unwrap());
    let method = Arc::as_ptr(&method);
    context.pc = context.stack.invoke(class, method, context.pc + 3, slots);
}

fn throw_vm_exception(context: &mut ThreadContext, error_class: &str) {
    let (error, initialized) = ClassArena::load_class(error_class, context).expect("jre_not_found");
    if !initialized {
        return;
    }
    let exception = Heap::allocate_object(&error).to_le_bytes();
    // FIXME stack resize?
    context.stack.push(&exception);
    context.exception_pending = true;
    context.throwable_initialized = false;
}

fn handle_exception(context: &mut ThreadContext) {
    // FIXME args of throwable <init>
    if !context.throwable_initialized {
        // TODO init throwable
        context.throwable_initialized = true;
    }
    if context.stack.is_empty() {
        return;
    }
    let error_ref = Ref::from_le_bytes(*context.stack.top());
    let header = ObjectHeader::from_vm_raw(Heap::ptr(error_ref as usize));
    let error_klass = unsafe { &*header.klass };
    match context.stack.match_exception_table(context.pc, error_klass) {
        Some(pc) => {
            context.pc = pc;
            context.exception_pending = false;
        }
        None => context.pc = context.stack.fire_exception(),
    }
}

#[test]
pub fn test_shift() {
    let u = 1i32;
    assert_eq!(0, u >> 1);
    assert_eq!(2, u << 1);
    let nu = i32::MIN;
    let sfr = unsafe { std::mem::transmute::<u32, i32>(std::mem::transmute::<i32, u32>(nu) >> 1) };
    let test: u32 = 0x80000001 >> 1;
    assert_eq!(test as i32, sfr);
    assert_eq!(0x80000000, i32::MIN as u32);
}
