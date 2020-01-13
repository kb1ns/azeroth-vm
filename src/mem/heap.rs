use mem::*;
use mem::metaspace::Klass;
use std::sync::Arc;
use std::mem::transmute;

pub struct Heap {
    pub oldgen_ptr: *mut u8,
    pub oldgen: Vec<u8>,
    pub s0_ptr: *mut u8,
    pub s0: Vec<u8>,
    pub s1_ptr: *mut u8,
    pub s1: Vec<u8>,
    pub eden_ptr: *mut u8,
    pub eden: Vec<u8>,
}

impl Heap {
    pub fn init(old_size: usize, survivor_size: usize, eden_size: usize) {
        let mut oldgen = vec![0u8; old_size];
        let mut eden = vec![0u8; eden_size];
        let mut s0 = vec![0u8; survivor_size];
        let mut s1 = vec![0u8; survivor_size];
        unsafe {
            HEAP.replace(Heap {
                oldgen_ptr: oldgen.as_mut_ptr(),
                oldgen: oldgen,
                s0_ptr: s0.as_mut_ptr(),
                s0: s0,
                s1_ptr: s1.as_mut_ptr(),
                s1: s1,
                eden_ptr: eden.as_mut_ptr(),
                eden: eden,
            });
        }
    }
}

static mut HEAP: Option<Heap> = None;

pub fn allocate(klass: Arc<Klass>) {

    // unsafe {
    //     transmute::<*mut u8, Word>()
    // }
}
