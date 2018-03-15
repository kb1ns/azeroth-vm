pub struct Heap {
    pub old: Vec<u8>,
    pub s0: Vec<u8>,
    pub s1: Vec<u8>,
    pub eden: Vec<u8>,
}

impl Heap {
    pub fn init(old_size: usize, survivor_size: usize, eden_size: usize) -> Heap {
        Heap {
            old: Vec::<u8>::with_capacity(old_size),
            s0: Vec::<u8>::with_capacity(survivor_size),
            s1: Vec::<u8>::with_capacity(survivor_size),
            eden: Vec::<u8>::with_capacity(eden_size),
        }
    }
}
