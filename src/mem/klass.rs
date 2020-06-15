use crate::bytecode::{class::Class, method::Method};
use crate::mem::{metaspace::*, Ref};
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub vtable: HashMap<String, Arc<Method>>,
    pub itable: HashMap<String, Arc<Method>>,
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
    pub fn new(
        bytecode: Class,
        classloader: Classloader,
        superclass: Option<Arc<Klass>>,
        interfaces: Vec<Arc<Klass>>,
    ) -> Klass {
        let vtable = Klass::build_vtable(&bytecode, &superclass);
        let itable = Klass::build_itable(&bytecode, &superclass, &interfaces);
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            vtable: vtable,
            itable: itable,
            superclass: superclass,
            initialized: AtomicBool::new(false),
            mutex: Mutex::<u8>::new(0),
        }
    }

    pub fn get_method_in_vtable(&self, name: &str, desc: &str) -> Option<Arc<Method>> {
        if let Some(ref m) = self.vtable.get(&(name.to_string() + ":" + desc)) {
            Some(Arc::clone(m))
        } else {
            None
        }
    }

    fn build_vtable(
        current: &Class,
        superclass: &Option<Arc<Klass>>,
    ) -> HashMap<String, Arc<Method>> {
        let mut vtable = HashMap::<String, Arc<Method>>::new();
        match superclass {
            Some(klass) => {
                for (k, v) in &klass.vtable {
                    vtable.insert(k.clone(), Arc::clone(&v));
                }
                for (k, v) in &current.methods.0 {
                    if (v.is_public() || v.is_protected()) && !v.is_final() && !v.is_static() {
                        vtable.insert(k.clone(), Arc::clone(&v));
                    }
                }
            }
            None => {
                for (k, v) in &current.methods.0 {
                    if (v.is_public() || v.is_protected()) && !v.is_final() && !v.is_static() {
                        vtable.insert(k.clone(), Arc::clone(&v));
                    }
                }
            }
        }
        vtable
    }

    fn build_itable(
        current: &Class,
        superclass: &Option<Arc<Klass>>,
        interfaces: &Vec<Arc<Klass>>,
    ) -> HashMap<String, Arc<Method>> {
        let mut itable = HashMap::<String, Arc<Method>>::new();
        match superclass {
            Some(klass) => {
                for (k, v) in &klass.itable {
                    itable.insert(k.clone(), Arc::clone(&v));
                }
            }
            None => {}
        }
        for ifs in interfaces {
            for (k, v) in &ifs.bytecode.methods.0 {
                itable.insert(ifs.bytecode.get_name().to_string() + &k, Arc::clone(&v));
            }
        }
        itable
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
