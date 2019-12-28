use bytecode::attribute::Attribute;
use bytecode::attribute::ExceptionHandler;
use bytecode::method::Method;
use mem::metaspace::Klass;
use mem::Slot;
use std::sync::Arc;

pub struct JavaStack<'a> {
    // thread
    pub frames: Vec<JavaFrame<'a>>,
    pub max_stack_size: usize,
    pub pc: u32,
}

impl<'a> JavaStack<'a> {
    // TODO
    pub fn new() -> JavaStack<'a> {
        JavaStack {
            frames: Vec::<JavaFrame<'a>>::new(),
            max_stack_size: 0,
            pc: 0,
        }
    }
}

pub struct JavaFrame<'class> {
    pub locals: Vec<Slot>,
    pub operands: Vec<Slot>,
    pub klass: Arc<Klass>,
    pub code: &'class [u8],
    pub exception_handlers: &'class [ExceptionHandler],
    // pub attributes: &'class Attributes,
    // pub class_name: &'class str,
    // pub method_name: &'class str,
    // pub descriptor: &'class str,
}

impl<'c> JavaFrame<'c> {
    pub fn new(class: Arc<Klass>, method: &'c Method) -> JavaFrame {
        if let Some(Attribute::Code(stacks, locals, ref code, ref exception_handlers, _)) =
            method.get_code()
        {
            return JavaFrame {
                locals: Vec::<Slot>::with_capacity(*locals as usize),
                operands: Vec::<Slot>::with_capacity(*stacks as usize),
                klass: class,
                code: code,
                exception_handlers: exception_handlers,
            };
        }
        panic!("Won't happend: abstract method or interface");
    }

    pub fn dump(&self) {}
}
