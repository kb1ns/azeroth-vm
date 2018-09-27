use std;
use super::mem::*;
use super::mem::stack::JavaStack;


pub unsafe fn run<'r>(stack: &'r mut JavaStack<'r>) -> usize {
    // TODO retrieve bytecode
    let directive: u8 = 0;
    match directive {
        // nop
        0x00 => 1,
        // aconst_null
        0x01 => {
            stack.top().operands.push(NULL);
            1
        }
        // iconst -1 ~ 5
        0x02 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(-1));
            1
        }
        0x03 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(0));
            1
        }
        0x04 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(1));
            1
        }
        0x05 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(2));
            1
        }
        0x06 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(3));
            1
        }
        0x07 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(4));
            1
        }
        0x08 => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<i32, Slot>(5));
            1
        }
        // lconst 0 ~ 1
        // byteorder: higher first
        0x09 => {
            let b = std::mem::transmute::<i64, Slot2>(0);
            let (higher, lower) = split_slot2(b);
            let f = stack.top();
            f.operands.push(higher);
            f.operands.push(lower);
            1
        }
        0x0a => {
            let b = std::mem::transmute::<i64, Slot2>(0);
            let (higher, lower) = split_slot2(b);
            let f = stack.top();
            f.operands.push(higher);
            f.operands.push(lower);
            1
        }
        // fconst 0 ~ 2
        0x0b => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<f32, Slot>(0.0));
            1
        }
        0x0c => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<f32, Slot>(1.0));
            1
        }
        0x0d => {
            stack
                .top()
                .operands
                .push(std::mem::transmute::<f32, Slot>(2.0));
            1
        }
        // dconst 0 ~ 1
        0x0e => {
            let b = std::mem::transmute::<f64, Slot2>(0.0);
            let (higher, lower) = split_slot2(b);
            let f = stack.top();
            f.operands.push(higher);
            f.operands.push(lower);
            1
        }
        0x0f => {
            let b = std::mem::transmute::<f64, Slot2>(1.0);
            let (higher, lower) = split_slot2(b);
            let f = stack.top();
            f.operands.push(higher);
            f.operands.push(lower);
            1
        }
        // bipush
        0x10 => 2,
        0x11 => 0,
        _ => 0,
    }
}
