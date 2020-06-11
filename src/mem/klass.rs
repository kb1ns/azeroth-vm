use crate::bytecode::class::Class;
use crate::mem::{metaspace::*, Ref};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub superclass: Option<Arc<Klass>>,
    pub initialized: AtomicBool,
    pub mutex: Mutex<u8>,
}

#[derive(Clone)]
pub struct ObjectHeader {
    pub mark: Ref,
    pub klass: Arc<Klass>,
    pub array_len: Option<usize>,
}

impl ObjectHeader {
    pub fn new_instance(klass: &Arc<Klass>) -> ObjectHeader {
        ObjectHeader {
            mark: 0,
            klass: Arc::clone(klass),
            array_len: None,
        }
    }

    pub fn new_array(klass: &Arc<Klass>, array_len: usize) -> ObjectHeader {
        ObjectHeader {
            mark: 0,
            klass: Arc::clone(klass),
            array_len: Some(array_len),
        }
    }
}

pub struct Instance {
    pub header: ObjectHeader,
    payload: *mut u8,
    len: usize,
    pub location: u32,
}

impl Instance {
    pub fn new(header: ObjectHeader, payload: *mut u8, len: usize, location: u32) -> Instance {
        Instance {
            header: header,
            payload: payload,
            len: len,
            location: location,
        }
    }

    // pub fn get_instance(header: &ObjectHeader) -> Instance {
    //     match header.array_len {
    //         Some(array_len) => ,
    //         None =>
    //     }
    // }
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

    // TODO
    pub fn payload_len(&self) -> usize {
        let size = self.bytecode.fields.iter().map(|x| x.memory_size()).sum();
        // FIXME override field
        match &self.superclass {
            Some(klass) => klass.payload_len() + size,
            None => size,
        }
    }
}
