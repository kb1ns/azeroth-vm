pub struct Stack {
    data: Vec<u8>,
    pub base: *mut u8,
}

// pub struct JavaFrame {
//     pub locals: Vec<Slot>,
//     pub operands: Vec<Slot>,
//     pub klass: Arc<Klass>,
//     pub code: Arc<Vec<u8>>,
//     pub exception_handlers: Arc<Vec<ExceptionHandler>>,
//     pub current_method: (String, String, U2),
//     pub pc: usize,
// }

#[test]
pub fn test() {
    let mut x = Vec::<u8>::with_capacity(256);
    let p: *mut u8 = x.as_mut_ptr();
    let stack = Stack { data: x, base: p };
    let ptr: *const u8 = "fuck".as_ptr();
    unsafe {
        stack.base.copy_from(ptr, 5);
    }
    assert_eq!(0x66u8, unsafe { *stack.base.offset(0) });
    let arc = std::sync::Arc::new("hello".to_owned());
    let ptx = std::sync::Arc::into_raw(arc);
    let ptx = unsafe { std::mem::transmute::<*const String, *const u8>(ptx) };
    unsafe {
        stack
            .base
            .copy_from(ptx, std::mem::size_of::<std::sync::Arc<String>>());
    }
    let mut a = [0u8; std::mem::size_of::<std::sync::Arc<String>>()];
    let pta = a.as_mut_ptr();
    unsafe {
        stack
            .base
            .copy_to(pta, std::mem::size_of::<std::sync::Arc<String>>());
    }
    let reduction = unsafe { std::mem::transmute::<*const u8, *mut std::sync::Arc<String>>(pta) };
}
