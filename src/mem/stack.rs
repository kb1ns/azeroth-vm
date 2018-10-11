use super::metaspace::Klass;
use super::*;
use std;

pub struct JvmStack {
    pub frames: Vec<Frame>,
    pub max_stack_size: usize,
    pub stack_size: usize,
    pub pc: u32,
}

pub struct Frame {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub klass: std::sync::Arc<Klass>,
    // pub code: &[u8],
    // pub exception_handler: &[ExceptionHandler],
    // pub attributes: &Attributes,
    // pub method_name: &str,
    pub descriptor: String,
}

impl JvmStack {
    // pub fn allocate(max_stack_size: usize) -> JvmStack {
    //     JvmStack {
    //         frames: Vec::<Frame>::new(),
    //         max_stack_size: max_stack_size,
    //         stack_size: 0,
    //         pc: 0,
    //     }
    // }

    // pub fn push(&mut self, frame: Frame) {
    //     // TODO check stack size
    //     self.frames.push(frame);
    // }

    // pub fn pop(&mut self) {
    //     if let Some(f) = self.frames.pop() {
    //         if !self.frames.is_empty() {
    //             let current = self.frames.len() - 1;
    //             if let Some(ret_addr) = self.frames[current].locals.pop() {
    //                 unsafe {
    //                     self.pc = std::mem::transmute::<Slot, u32>(ret_addr);
    //                 }
    //             }
    //         }
    //     }
    //     panic!("pop empty stack");
    // }

    // pub fn current_index(&self) -> usize {
    //     self.frames.len() - 1
    // }
}

// impl Frame {
//     pub fn new(klass: std::sync::Arc<Klass>, method_name: String, descriptor: String) -> Frame {
//         let klass_share = klass.clone();
//         if let Ok(method) = klass_share.bytecode.get_method(&method_name, &descriptor) {
//             if let Some(&Attribute::Code(
//                 stacks,
//                 locals,
//                 ref code,
//                 ref exception_handler,
//                 ref attributes,
//             )) = method.get_code()
//             {
//                 return Frame {
//                     klass: klass_share,
//                     code: code,
//                     exception_handler: exception_handler,
//                     attributes: attributes,
//                     method_name: method_name,
//                     descriptor: descriptor,
//                     locals: Vec::<Slot>::with_capacity(locals as usize),
//                     operands: Vec::<Slot>::with_capacity(stacks as usize),
//                 };
//             }
//             panic!("Method is abstract");
//         }
//         panic!("java.lang.NoSuchMethodError");
//     }
// }
