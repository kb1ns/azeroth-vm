use crate::bytecode::method::Method;
use crate::interpreter;
use crate::mem::{klass::*, *};
use std::sync::Arc;

const DEFAULT_STACK_LEN: usize = 128 * 1024;

pub struct JavaStack {
    pub frames: Vec<JavaFrame>,
    pub max_stack_size: usize,
}

impl JavaStack {
    pub fn new() -> Self {
        JavaStack {
            frames: Vec::new(),
            // TODO
            max_stack_size: 0,
        }
    }

    pub fn has_next(&self, pc: usize) -> bool {
        match self.frames.last() {
            Some(ref frame) => pc < frame.method.get_code().expect("Illegal class file").2.len(),
            None => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn invoke(&mut self, mut frame: JavaFrame, pc: usize) -> usize {
        if !self.is_empty() {
            let (_, descriptor, access_flag) = &frame.method.get_name_and_descriptor();
            let (params, _) = interpreter::resolve_method_descriptor(descriptor);
            let mut slots: usize = params
                .into_iter()
                .map(|p| match p.as_ref() {
                    "D" | "J" => 2,
                    _ => 1,
                })
                .sum();
            if !frame.method.is_static() {
                slots = slots + 1;
            }
            if frame.method.is_native() {
                // TODO native invokcation not implemented yet, return the pc directly
                return pc;
            }
            let current = self.frames.last_mut().expect("Illegal stack");
            unsafe {
                let params = frame.locals.as_mut_ptr();
                current.ptr = current.ptr.sub(slots * PTR_SIZE);
                current.ptr.copy_to(params, slots * PTR_SIZE);
            }
            frame.pc = pc;
        }
        self.frames.push(frame);
        0
    }

    pub fn backtrack(&mut self) -> usize {
        let frame = self.frames.pop().expect("Illegal operands stack: ");
        if !self.is_empty() {
            let (_, descriptor, _) = &frame.method.get_name_and_descriptor();
            let (_, ret) = interpreter::resolve_method_descriptor(descriptor);
            let slots: usize = match ret.as_ref() {
                "D" | "J" => 2,
                "V" => 0,
                _ => 1,
            };
            let current = self.frames.last_mut().expect("");
            unsafe {
                let return_val = frame.ptr.sub(slots * PTR_SIZE);
                current.ptr.copy_from(return_val, slots * PTR_SIZE);
                current.ptr = current.ptr.add(slots * PTR_SIZE);
            }
        }
        frame.pc
    }

    pub fn code_at(&self, pc: usize) -> u8 {
        self.frames.last().expect("Illegal class file").code_at(pc)
    }

    pub fn load(&mut self, offset: usize, count: usize) {
        let current = self.frames.last_mut().expect("Illegal class file");
        unsafe {
            current.ptr.copy_from(
                current.locals[offset * PTR_SIZE..].as_ptr(),
                count * PTR_SIZE,
            );
            current.ptr = current.ptr.add(count * PTR_SIZE);
        }
    }

    pub fn store(&mut self, offset: usize, count: usize) {
        let current = self.frames.last_mut().expect("Illegal class file");
        unsafe {
            current.ptr = current.ptr.sub(count * PTR_SIZE);
            current.ptr.copy_to(
                current.locals[offset * PTR_SIZE..].as_mut_ptr(),
                count * PTR_SIZE,
            );
        }
    }

    pub fn get(&self, offset: usize) -> Slot {
        let mut data = NULL;
        let current = self.frames.last().expect("Illegal operands");
        &data[..].copy_from_slice(&current.locals[offset * PTR_SIZE..(offset + 1) * PTR_SIZE]);
        data
    }

    pub fn get_w(&self, offset: usize) -> WideSlot {
        let mut data = LONG_NULL;
        let current = self.frames.last().expect("Illegal operands");
        &data[..].copy_from_slice(&current.locals[offset * PTR_SIZE..(offset + 2) * PTR_SIZE]);
        data
    }

    pub fn set(&mut self, offset: usize, v: Slot) {
        let frame = self.frames.last_mut().expect("Illegal class file");
        &frame.locals[offset * PTR_SIZE..].copy_from_slice(&v[..]);
    }

    pub fn set_w(&mut self, offset: usize, v: WideSlot) {
        let frame = self.frames.last_mut().expect("Illegal class file");
        &frame.locals[offset * PTR_SIZE..].copy_from_slice(&v[..]);
    }

    pub fn push(&mut self, v: &[u8], len: usize) {
        let current = self.frames.last_mut().expect("Illegal class file");
        unsafe {
            current.ptr.copy_from(v.as_ptr(), len);
            current.ptr = current.ptr.add(len);
        }
    }

    pub fn pop(&mut self) -> Slot {
        let mut data = NULL;
        let current = self.frames.last_mut().expect("Illegal operands");
        unsafe {
            current.ptr = current.ptr.sub(PTR_SIZE);
            current.ptr.copy_to(data.as_mut_ptr(), PTR_SIZE);
        }
        data
    }

    pub fn pop_w(&mut self) -> WideSlot {
        let mut data = LONG_NULL;
        let current = self.frames.last_mut().expect("Illegal operands");
        unsafe {
            current.ptr = current.ptr.sub(PTR_SIZE * 2);
            current.ptr.copy_to(data.as_mut_ptr(), PTR_SIZE * 2);
        }
        data
    }

    pub fn bi_op<F>(&mut self, f: F)
    where
        F: Fn(Slot, Slot) -> Slot,
    {
        let left = self.pop();
        let right = self.pop();
        self.push(&f(left, right), PTR_SIZE);
    }

    pub fn bi_op_w<F>(&mut self, f: F)
    where
        F: Fn(WideSlot, WideSlot) -> WideSlot,
    {
        let left = self.pop_w();
        let right = self.pop_w();
        self.push(&f(left, right), 2 * PTR_SIZE);
    }

    pub fn top(&self) -> Slot {
        let mut v = NULL;
        unsafe {
            self.frames
                .last()
                .expect("Illegal operands")
                .ptr
                .sub(PTR_SIZE)
                .copy_to(v.as_mut_ptr(), PTR_SIZE);
        }
        v
    }

    pub fn top_w(&self) -> WideSlot {
        let mut v = LONG_NULL;
        unsafe {
            self.frames
                .last()
                .expect("Illegal operands")
                .ptr
                .sub(PTR_SIZE)
                .copy_to(v.as_mut_ptr(), 2 * PTR_SIZE);
        }
        v
    }

    pub fn fetch_heap(&mut self, addr: u32, offset: usize, len: usize) {
        let current = self.frames.last_mut().expect("Illegal operands");
        let heap_ptr = jvm_heap!().base;
        unsafe {
            let target = heap_ptr.add(addr as usize + OBJ_HEADER_LEN + offset);
            current.ptr.copy_from(target, len);
            if len <= PTR_SIZE {
                current.ptr = current.ptr.add(PTR_SIZE);
            } else {
                current.ptr = current.ptr.add(2 * PTR_SIZE);
            }
        }
    }

    pub fn set_heap_aligned(&mut self, addr: u32, offset: usize, len: usize) {
        let current = self.frames.last_mut().expect("Illegal operands");
        let heap_ptr = jvm_heap!().base;
        unsafe {
            let target = heap_ptr.add(addr as usize + OBJ_HEADER_LEN + offset);
            if len <= PTR_SIZE {
                current.ptr = current.ptr.sub(PTR_SIZE)
            } else {
                current.ptr = current.ptr.sub(2 * PTR_SIZE)
            }
            target.copy_from(current.ptr, len);
        }
    }

    pub fn current_class(&self) -> Arc<Klass> {
        self.frames.last().expect("Illegal stack").klass.clone()
    }
}
pub struct JavaFrame {
    pub locals: Vec<u8>,
    pub operands: Vec<u8>,
    pub ptr: *mut u8,
    pub klass: Arc<Klass>,
    pub method: Arc<Method>,
    pub pc: usize,
}

impl JavaFrame {
    pub fn new(klass: Arc<Klass>, method: Arc<Method>) -> JavaFrame {
        match method.get_code() {
            None => {
                if !method.is_native() {
                    panic!("Abstract method or interface not allow here");
                }
                // TODO native method
                let mut operands: Vec<u8> = vec![];
                let ptr = operands.as_mut_ptr();
                JavaFrame {
                    locals: vec![],
                    operands: operands,
                    ptr: ptr,
                    klass: klass,
                    method: Arc::clone(&method),
                    pc: 0,
                }
            }
            Some((stacks, locals, _, _, _)) => {
                let mut operands = vec![0u8; PTR_SIZE * stacks as usize];
                let ptr = operands.as_mut_ptr();
                JavaFrame {
                    locals: vec![0u8; PTR_SIZE * locals as usize],
                    operands: operands,
                    ptr: ptr,
                    klass: klass,
                    method: Arc::clone(&method),
                    pc: 0,
                }
            }
        }
    }

    pub fn code_at(&self, pc: usize) -> u8 {
        self.method.get_code().expect("").2[pc]
    }

    pub fn dump(&self, pc: usize) {
        let (name, descriptor, _) = self.method.get_name_and_descriptor();
        println!("current class: {:?}", self.klass.bytecode.get_name());
        println!("current method: {:?} {:?}", name, descriptor);
        println!("locals: {:02x?}", self.locals);
        println!("stacks: {:02x?}", self.operands);
        println!("base: {:x?}", self.operands.as_ptr());
        println!("ptr: {:x?}", self.ptr);
        println!("pc: {:?}", pc);
        println!(
            "instructions: {:02x?}\n",
            self.method.get_code().expect("").2
        );
    }
}

#[cfg(test)]
mod test {

    #[test]
    pub fn test_stack() {
        let java_lang_object = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";
        let class_vec = base64::decode(java_lang_object).unwrap();
        let bytecode = super::Class::from_vec(class_vec);
        let klass = super::Klass::new(bytecode, super::metaspace::Classloader::ROOT, None, vec![]);
        let klass = super::Arc::new(klass);
        let method = klass
            .bytecode
            .get_method("toString", "()Ljava/lang/String;")
            .unwrap();
        let locals = vec![
            0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 0xau8, 0xbu8, 0xcu8, 0xdu8, 0xeu8,
            0xfu8,
        ];
        let mut operands = vec![0u8; 32];
        let ptr = operands.as_mut_ptr();
        let init = super::JavaFrame {
            locals: locals,
            operands: operands,
            ptr: ptr,
            klass: klass,
            method: method,
            pc: 0,
        };
        let mut stack = super::JavaStack {
            frames: Vec::<super::JavaFrame>::with_capacity(10),
            max_stack_size: 4096,
        };
        stack.frames.push(init);
        // load, ptr: 0 -> 4
        stack.load(0, 1);
        assert_eq!(
            &stack.frames.last().unwrap().operands[..4],
            &[0u8, 1u8, 2u8, 3u8]
        );
        {
            let ptr = stack.frames.last().unwrap().ptr;
            assert_eq!(
                ptr,
                stack.frames.last_mut().unwrap().operands[4..].as_mut_ptr()
            );
        }
        // load, ptr: 4 -> 8
        stack.load(3, 1);
        assert_eq!(
            &stack.frames.last().unwrap().operands[4..8],
            &[0xcu8, 0xdu8, 0xeu8, 0xfu8]
        );
        {
            let ptr = stack.frames.last().unwrap().ptr;
            assert_eq!(
                ptr,
                stack.frames.last_mut().unwrap().operands[8..].as_mut_ptr()
            );
        }
        // store, ptr: 8 -> 4
        stack.store(1, 1);
        {
            let ptr = stack.frames.last().unwrap().ptr;
            assert_eq!(
                ptr,
                stack.frames.last_mut().unwrap().operands[4..].as_mut_ptr()
            );
        }
        assert_eq!(
            &stack.frames.last().unwrap().locals[4..8],
            &[0xcu8, 0xdu8, 0xeu8, 0xfu8]
        );
        // push, ptr: 4 -> 8
        stack.push(&super::NULL, super::PTR_SIZE);
        {
            let ptr = stack.frames.last().unwrap().ptr;
            assert_eq!(
                ptr,
                stack.frames.last_mut().unwrap().operands[8..].as_mut_ptr()
            );
        }
        assert_eq!(&stack.frames.last().unwrap().operands[4..8], &[0, 0, 0, 0]);
        // pop_w, ptr: 8 -> 0
        stack.pop_w();
        {
            let ptr = stack.frames.last().unwrap().ptr;
            assert_eq!(
                ptr,
                stack.frames.last_mut().unwrap().operands[..].as_mut_ptr()
            );
        }
    }
}
