pub struct Heap {
    pub oldgen_ptr: *mut [u8],
    pub oldgen_size: usize,
    pub s0_ptr: *mut [u8],
    pub s0_size: usize,
    pub s1_ptr: *mut [u8],
    pub s1_size: usize,
    pub eden_ptr: *mut [u8],
    pub eden_size: usize,
}

impl Heap {
    pub unsafe fn init(old_size: usize, survivor_size: usize, eden_size: usize) -> Heap {
        let oldgen = vec![0u8; old_size];
        let edengen = vec![0u8; eden_size];
        let s0 = vec![0u8; survivor_size];
        let s1 = vec![0u8; survivor_size];
        Heap {
            oldgen_ptr: Box::into_raw(oldgen.into_boxed_slice()),
            oldgen_size: old_size,
            s0_ptr: Box::into_raw(s0.into_boxed_slice()),
            s0_size: survivor_size,
            s1_ptr: Box::into_raw(s1.into_boxed_slice()),
            s1_size: survivor_size,
            eden_ptr: Box::into_raw(edengen.into_boxed_slice()),
            eden_size: eden_size,
        }
    }
}
