use bytecode::class::Class;
use mem::metaspace::*;
use mem::Ref;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub superclass: Option<Arc<Klass>>,
    pub initialized: AtomicBool,
    pub mutex: Mutex<u8>,
}

pub struct ObjectHeader {
    pub mark: Ref,
    pub klass: Arc<Klass>,
}

impl Klass {
    pub fn new(bytecode: Class, classloader: Classloader, superclass: Option<Arc<Klass>>) -> Klass {
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            superclass: superclass,
            initialized: AtomicBool::new(false),
            mutex: Mutex::<u8>::new(0),
        }
    }

    pub fn is_array(&self) -> bool {
        false
    }

    pub fn instance_size(&self) -> u32 {
        let size = self.bytecode.fields.iter().map(|x| x.memory_size() as u32).sum();
        match &self.superclass {
            Some(klass) => klass.instance_size() + size,
            None => size,
        }
    }

    pub fn new_instance(klass: &Arc<Klass>) -> ObjectHeader {
        ObjectHeader {
            mark: 0,
            klass: Arc::clone(klass),
        }
    }
}
