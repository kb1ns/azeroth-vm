use crate::{
    bytecode,
    bytecode::{class::Class, method::Method},
    mem::{klass::*, *},
};
use std::collections::HashSet;

const DEFAULT_STACK_LEN: usize = 128 * 1024;

pub struct JavaStack {
    data: Vec<u8>,
    frames: Vec<JavaFrame>,
    max_stack_size: usize,
}

pub struct JavaFrame {
    locals: *mut u8,
    operands: *mut u8,
    class: *const Class,
    method: *const Method,
    pc: usize,
    max_locals: usize,
    active_refs: HashSet<Ref>,
}

impl JavaStack {
    pub fn new() -> Self {
        Self {
            data: vec![0u8; DEFAULT_STACK_LEN],
            frames: Vec::<JavaFrame>::with_capacity(256),
            max_stack_size: DEFAULT_STACK_LEN,
        }
    }

    pub fn frame(&self) -> &JavaFrame {
        self.frames.last().expect("empty_stack")
    }

    pub fn mut_frame(&mut self) -> &mut JavaFrame {
        self.frames.last_mut().expect("empty_stack")
    }

    pub fn operands(&self) -> *mut u8 {
        self.frame().operands
    }

    pub fn locals(&self) -> *mut u8 {
        self.frame().locals
    }

    pub fn update(&mut self, operands: *mut u8) {
        self.mut_frame().operands = operands;
    }

    pub fn upward(&mut self, n: usize) {
        unsafe {
            self.update(self.operands().add(n * PTR_SIZE));
        }
    }

    pub fn downward(&mut self, n: usize) {
        unsafe {
            self.update(self.operands().sub(n * PTR_SIZE));
        }
    }

    pub fn has_next(&self, pc: usize) -> bool {
        match self.frames.last() {
            None => false,
            Some(ref f) => {
                pc < unsafe { &*f.method }
                    .get_code()
                    .expect("null_code_attribute")
                    .2
                    .len()
            }
        }
    }

    pub fn method(&self) -> &Method {
        unsafe { &*self.frame().method }
    }

    pub fn class(&self) -> &Class {
        unsafe { &*self.frame().class }
    }

    pub fn method_ptr(&self) -> *const Method {
        self.frame().method
    }

    pub fn class_ptr(&self) -> *const Class {
        self.frame().class
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn append_ref_to_roots(&mut self, reference: Ref) {
        self.mut_frame().active_refs.insert(reference);
    }

    pub fn collect_tracing_roots(&self) -> HashSet<Ref> {
        self.frames
            .iter()
            .flat_map(|f| f.active_refs.iter())
            .map(|r| *r)
            .collect::<HashSet<_>>()
    }

    pub fn invoke(
        &mut self,
        class: *const Class,
        method: *const Method,
        pc: usize,
        locals: usize,
    ) -> usize {
        let m = unsafe { &*method };
        let (_, desc, access_flag) = m.get_name_and_descriptor();
        let (params, slots, ret) = bytecode::resolve_method_descriptor(desc, access_flag);
        // -------- drop this after implements native method -----------
        if m.is_native() {
            let ret: usize = match ret.as_ref() {
                "D" | "J" => 2,
                "V" => 0,
                _ => 1,
            };
            self.downward(slots - ret);
            return pc;
        }
        // -------------------------------------------------------------
        let locals = unsafe {
            if self.is_empty() {
                self.data.as_mut_ptr().add(locals * PTR_SIZE)
            } else {
                self.operands().sub(locals * PTR_SIZE)
            }
        };
        let mut active_refs = HashSet::new();
        let mut index = 0usize;
        for p in params {
            if p.starts_with("L") || p.starts_with("[") {
                let reference = unsafe { *locals.add(index * PTR_SIZE).cast::<Slot>() };
                if reference != NULL {
                    active_refs.insert(Ref::from_slot(reference));
                }
            }
            if p == "D" || p == "j" {
                index += 2;
            } else {
                index += 1;
            }
        }
        let method_ref = unsafe { &*method };
        match method_ref.get_code() {
            None => panic!("AbstractMethod"),
            Some((_, max_locals, _, _, _)) => self.frames.push(JavaFrame {
                locals: locals,
                operands: unsafe { locals.add(max_locals as usize * PTR_SIZE) },
                class: class,
                method: method,
                pc: pc,
                max_locals: max_locals as usize,
                active_refs: active_refs,
            }),
        }
        0
    }

    pub fn return_normal(&mut self) -> usize {
        let frame = self.frames.pop().expect("empty_stack");
        if !self.is_empty() {
            let (_, descriptor, access_flag) = &self.method().get_name_and_descriptor();
            let (_, _, ret) = bytecode::resolve_method_descriptor(descriptor, *access_flag);
            let slots: usize = match ret.as_ref() {
                "D" | "J" => 2,
                "V" => 0,
                _ => 1,
            };
            unsafe {
                let val = frame.operands.sub(slots * PTR_SIZE);
                self.operands().copy_from(val, slots * PTR_SIZE);
                self.update(self.operands().add(slots * PTR_SIZE));
            }
        }
        frame.pc
    }

    pub fn fire_exception(&mut self) -> usize {
        let frame = self.frames.pop().expect("empty_stack");
        if !self.is_empty() {
            unsafe {
                let error = frame.operands.sub(PTR_SIZE);
                self.operands().copy_from(error, PTR_SIZE);
                self.update(self.operands().add(PTR_SIZE));
            }
        }
        frame.pc
    }

    pub fn match_exception_table(&self, pc: usize, klass: &Klass) -> Option<usize> {
        let exception_table = self.method().get_code().unwrap().3;
        for handler in exception_table.as_slice() {
            if pc >= handler.start_pc as usize && pc < handler.end_pc as usize {
                match &handler.catch_type {
                    Some(exception_type) => {
                        if is_subclass(klass, &exception_type) {
                            return Some(handler.handler_pc as usize);
                        }
                    }
                    None => return Some(handler.handler_pc as usize),
                }
            }
        }
        None
    }

    pub fn code_at(&self, pc: usize) -> u8 {
        self.method().get_code().unwrap().2[pc]
    }

    pub fn load(&mut self, offset: usize, count: usize) {
        unsafe {
            self.operands()
                .copy_from(self.locals().add(offset * PTR_SIZE), count * PTR_SIZE);
            self.update(self.operands().add(count * PTR_SIZE));
        }
    }

    pub fn store(&mut self, offset: usize, count: usize) {
        unsafe {
            self.update(self.operands().sub(count * PTR_SIZE));
            self.locals()
                .add(offset * PTR_SIZE)
                .copy_from(self.operands(), count * PTR_SIZE);
        }
    }

    pub fn get(&self, offset: usize) -> Slot {
        unsafe { *self.locals().add(offset * PTR_SIZE).cast::<Slot>() }
    }

    pub fn get_w(&self, offset: usize) -> WideSlot {
        unsafe { *self.locals().add(offset * PTR_SIZE).cast::<WideSlot>() }
    }

    pub fn set(&self, offset: usize, v: &Slot) {
        unsafe {
            self.locals()
                .add(offset * PTR_SIZE)
                .copy_from(v.as_ptr(), PTR_SIZE);
        }
    }

    pub fn set_w(&self, offset: usize, v: &WideSlot) {
        unsafe {
            self.locals()
                .add(offset * PTR_SIZE)
                .copy_from(v.as_ptr(), PTR_SIZE * 2);
        }
    }

    pub fn push(&mut self, v: &Slot) {
        unsafe {
            self.operands().copy_from(v.as_ptr(), PTR_SIZE);
            self.update(self.operands().add(PTR_SIZE));
        }
    }

    pub fn push_w(&mut self, v: &WideSlot) {
        unsafe {
            self.operands().copy_from(v.as_ptr(), PTR_SIZE * 2);
            self.update(self.operands().add(PTR_SIZE * 2));
        }
    }

    pub fn pop(&mut self) -> Slot {
        unsafe {
            self.update(self.operands().sub(PTR_SIZE));
            *self.operands().cast::<Slot>()
        }
    }

    pub fn pop_w(&mut self) -> WideSlot {
        unsafe {
            self.update(self.operands().sub(PTR_SIZE * 2));
            *self.operands().cast::<WideSlot>()
        }
    }

    pub fn bi_op<F>(&mut self, f: F)
    where
        F: Fn(Slot, Slot) -> Slot,
    {
        let left = self.pop();
        let right = self.pop();
        self.push(&f(left, right));
    }

    pub fn bi_op_w<F>(&mut self, f: F)
    where
        F: Fn(WideSlot, WideSlot) -> WideSlot,
    {
        let left = self.pop_w();
        let right = self.pop_w();
        self.push_w(&f(left, right));
    }

    pub fn un_op<F>(&mut self, f: F)
    where
        F: Fn(Slot) -> Slot,
    {
        let opr = self.pop();
        self.push(&f(opr));
    }

    pub fn un_op_w<F>(&mut self, f: F)
    where
        F: Fn(WideSlot) -> WideSlot,
    {
        let opr = self.pop_w();
        self.push_w(&f(opr));
    }

    pub fn top(&self) -> &Slot {
        unsafe { &*self.operands().sub(PTR_SIZE).cast::<Slot>() }
    }

    pub fn top_w(&self) -> &WideSlot {
        unsafe { &*self.operands().sub(2 * PTR_SIZE).cast::<WideSlot>() }
    }

    pub fn top_n(&self, n: usize) -> &Slot {
        unsafe { &*self.operands().sub(PTR_SIZE * n).cast::<Slot>() }
    }

    pub fn dump(&self, pc: usize) {
        let (name, descriptor, _) = self.method().get_name_and_descriptor();
        println!("method layer: {}", self.frames.len() - 1);
        println!("current class: {:?}", self.class().get_name());
        println!("current method: {:?} {:?}", name, descriptor);
        let locals_offset = self.locals() as usize - self.data.as_ptr() as usize;
        println!(
            "locals: {:02x?}",
            &self.data[locals_offset..locals_offset + self.frame().max_locals * PTR_SIZE]
        );
        let operands_offset = self.operands() as usize - self.data.as_ptr() as usize;
        println!(
            "operands: {:02x?}",
            &self.data[locals_offset + self.frame().max_locals * PTR_SIZE..operands_offset]
        );
        println!("pc: {:?}", pc);
        println!("[pc]: {:02x?}", self.code_at(pc));
        println!("code: {:02x?}", self.method().get_code().unwrap().2);
        println!();
    }
}

fn is_subclass(klass: &Klass, target: &str) -> bool {
    let mut thisclass = klass;
    loop {
        if &thisclass.name == target {
            return true;
        }
        if thisclass.superclass.is_none() {
            break;
        }
        thisclass = thisclass.superclass.as_ref().unwrap();
    }
    false
}
