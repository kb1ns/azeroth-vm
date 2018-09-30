use std;
use super::mem::*;
use super::mem::stack::*;
use super::mem::metaspace::ClassArena;

pub struct Interpreter {
    pub class_arena: std::sync::Arc<ClassArena>,
    // TODO heap
}

impl Interpreter {
    pub fn execute(
        &self,
        class_name: &str,
        method_name: &str,
        method_descriptor: &str,
        context: &ThreadContext,
    ) {
        let mut stack = JvmStack::allocate(128 * 1024);
        // init first frame
        if let Some(klass) = self.class_arena.find_class(class_name) {
            let mut f = Frame::new(&klass.as_ref().bytecode, method_name, method_descriptor);
        } else {
            panic!("java.lang.ClassNotFoundException");
        }
    }

    // TODO
    unsafe fn run<'r>(stack: &'r mut JvmStack<'r>) {
        // let idx = stack.current_index();
        // let current = stack.frames[idx];
        // if let Ok(method) = class_ref.get_method(&method_name, &descriptor) {
        //     if let Some(&Attribute::Code(stacks, locals, _, _, _)) = method.get_code() {
        //     }
        // }
        let directive: u8 = 0;
        match directive {
            // nop
            0x00 => {
                stack.pc = stack.pc + 1;
            }
            // aconst_null
            // 0x01 => {
            //     let current = stack.current_index();
            //     stack.frames[current].operands.push(NULL);
            //     stack.pc = stack.pc + 1;
            // }
            // // iconst -1 ~ 5
            // 0x02 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(-1));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x03 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(0));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x04 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(1));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x05 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(2));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x06 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(3));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x07 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(4));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x08 => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<i32, Slot>(5));
            //     stack.pc = stack.pc + 1;
            // }
            // // lconst 0 ~ 1
            // // byteorder: higher first
            // 0x09 => {
            //     let b = std::mem::transmute::<i64, Slot2>(0);
            //     let (higher, lower) = split_slot2(b);
            //     let current = stack.current_index();
            //     stack.frames[current].operands.push(higher);
            //     stack.frames[current].operands.push(lower);
            //     stack.pc = stack.pc + 1;
            // }
            // 0x0a => {
            //     let b = std::mem::transmute::<i64, Slot2>(0);
            //     let (higher, lower) = split_slot2(b);
            //     let current = stack.current_index();
            //     stack.frames[current].operands.push(higher);
            //     stack.frames[current].operands.push(lower);
            //     stack.pc = stack.pc + 1;
            // }
            // // fconst 0 ~ 2
            // 0x0b => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<f32, Slot>(0.0));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x0c => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<f32, Slot>(1.0));
            //     stack.pc = stack.pc + 1;
            // }
            // 0x0d => {
            //     let current = stack.current_index();
            //     stack.frames[current]
            //         .operands
            //         .push(std::mem::transmute::<f32, Slot>(2.0));
            //     stack.pc = stack.pc + 1;
            // }
            // // dconst 0 ~ 1
            // 0x0e => {
            //     let b = std::mem::transmute::<f64, Slot2>(0.0);
            //     let (higher, lower) = split_slot2(b);
            //     let current = stack.current_index();
            //     stack.frames[current].operands.push(higher);
            //     stack.frames[current].operands.push(lower);
            //     stack.pc = stack.pc + 1;
            // }
            // 0x0f => {
            //     let b = std::mem::transmute::<f64, Slot2>(1.0);
            //     let (higher, lower) = split_slot2(b);
            //     let current = stack.current_index();
            //     stack.frames[current].operands.push(higher);
            //     stack.frames[current].operands.push(lower);
            //     stack.pc = stack.pc + 1;
            // }
            // bipush
            0x10 => {}
            0x11 => {}
            _ => {}
        }
    }
}
