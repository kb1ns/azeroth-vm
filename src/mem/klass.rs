use bytecode::class::Class;
use mem::metaspace::Classloader;
use mem::Ref;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub initialized: AtomicBool,
    pub mutex: Mutex<u8>,
}

pub struct ObjectHeader {
    pub mark: Ref,
    pub klass: Arc<Klass>,
}

impl Klass {
    pub fn new(bytecode: Class, classloader: Classloader) -> Klass {
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            initialized: AtomicBool::new(false),
            mutex: Mutex::<u8>::new(0),
        }
    }

    pub fn is_array(&self) -> bool {
        false
    }

    pub fn instance_size(&self) -> u32 {
        0
    }

    pub fn new_instance(klass: &Arc<Klass>) -> ObjectHeader {
        ObjectHeader {
            mark: 0,
            klass: Arc::clone(klass),
        }
    }
}
