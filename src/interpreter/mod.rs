use std;
use super::mem::stack::JavaStack;
use super::mem::stack::Frame;


pub fn execute(stack: &mut JavaStack) {}

unsafe fn run(stack: &mut JavaStack) {
    // TODO retrieve bytecode
    let directive: u8 = 0;
    match directive {
        // nop
        0x00 => {
            stack.java_pc = stack.java_pc + 1;
        }
        // aconst_null
        0x01 => {
            stack.top().operands.push([0x00, 0x00, 0x00, 0x00]);
            stack.java_pc = stack.java_pc + 1;
        }
        // iconst -1 ~ 5
        0x02 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(-1),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x03 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(0),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x04 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(1),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x05 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(2),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x06 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(3),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x07 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(4),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x08 => {
            stack.top().operands.push(
                std::mem::transmute::<i32, [u8; 4]>(5),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        // lconst 0 ~ 1
        // byteorder: higher first
        0x09 => {
            let b = std::mem::transmute::<i64, [u8; 8]>(0);
            let mut higher = [0u8; 4];
            higher.copy_from_slice(&b[0..4]);
            let mut lower = [0u8; 4];
            lower.copy_from_slice(&b[4..]);
            stack.top().operands.push(higher);
            stack.top().operands.push(lower);
            stack.java_pc = stack.java_pc + 1;
        }
        0x0a => {
            let b = std::mem::transmute::<i64, [u8; 8]>(1);
            let mut higher = [0u8; 4];
            higher.copy_from_slice(&b[0..4]);
            let mut lower = [0u8; 4];
            lower.copy_from_slice(&b[4..]);
            stack.top().operands.push(higher);
            stack.top().operands.push(lower);
            stack.java_pc = stack.java_pc + 1;
        }
        // fconst 0 ~ 2
        0x0b => {
            stack.top().operands.push(
                std::mem::transmute::<f32, [u8; 4]>(0.0),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x0c => {
            stack.top().operands.push(
                std::mem::transmute::<f32, [u8; 4]>(1.0),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        0x0d => {
            stack.top().operands.push(
                std::mem::transmute::<f32, [u8; 4]>(2.0),
            );
            stack.java_pc = stack.java_pc + 1;
        }
        // dconst 0 ~ 1
        0x0e => {
            let b = std::mem::transmute::<f64, [u8; 8]>(0.0);
            let mut higher = [0u8; 4];
            higher.copy_from_slice(&b[0..4]);
            let mut lower = [0u8; 4];
            lower.copy_from_slice(&b[4..]);
            stack.top().operands.push(higher);
            stack.top().operands.push(lower);
            stack.java_pc = stack.java_pc + 1;
        }
        0x0f => {
            let b = std::mem::transmute::<f64, [u8; 8]>(1.0);
            let mut higher = [0u8; 4];
            higher.copy_from_slice(&b[0..4]);
            let mut lower = [0u8; 4];
            lower.copy_from_slice(&b[4..]);
            stack.top().operands.push(higher);
            stack.top().operands.push(lower);
            stack.java_pc = stack.java_pc + 1;
        }
        // bipush
        0x10 => {

            stack.java_pc = stack.java_pc + 2;
        }
        0x11 => {}
        _ => {}
    }
}
