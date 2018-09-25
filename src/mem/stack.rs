pub struct Stack {
    pub jvm_method: Vec<Frame>,
    pub jvm_pc: usize,
    pub max_stack_size: usize,
}

pub struct Frame {
    locals: Vec<Slot>,
    operands: Vec<Slot>,
}

pub enum Slot {
    L1(u32),
    L2(u64),
}

impl Stack {

    pub fn allocate(max_stack_size: usize, pc: usize) -> Stack {
        Stack {
            jvm_method: Vec::<Frame>::new(),
            jvm_pc: pc,
            max_stack_size: max_stack_size,
        }
    }

    pub fn push(&mut self, max_locals: usize, max_op_stack_size: usize) {
        if self.jvm_method.len() >= self.max_stack_size {
            panic!("java.lang.StackOverflowError: {}", self.max_stack_size);
        }
        self.jvm_method.push(Frame {
            locals: Vec::<Slot>::with_capacity(max_locals),
            operands: Vec::<Slot>::with_capacity(max_op_stack_size),
        });
    }

    pub fn pop(&mut self) {
        self.jvm_method.pop();
    }
}
