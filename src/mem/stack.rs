pub struct Stack {
    pub jvm_method: Vec<Frame>,
    pub jvm_pc: usize,
    pub max_stack_size: usize,
}

pub struct Frame {
    locals: Vec<u8>,
    operands: Vec<u8>,
}

impl Stack {

    pub fn allocate(max_stack_size: usize, pc: usize) -> Stack {
        Stack {
            jvm_method: Vec::<Frame>::new(),
            jvm_pc: pc,
            max_stack_size: max_stack_size,
        }
    }

}
