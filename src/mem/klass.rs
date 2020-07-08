use super::RefKey;
use crate::bytecode::{class::Class, field::Field, method::Method};
use crate::mem::{metaspace::*, Ref};
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub vtable: HashMap<RefKey, Arc<Method>>,
    pub itable: HashMap<RefKey, Arc<Method>>,
    pub layout: HashMap<RefKey, (usize, usize)>,
    pub len: usize,
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
        let (layout, len) = Klass::build_layout(&bytecode, &superclass);
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            vtable: vtable,
            itable: itable,
            layout: layout,
            len: len,
            superclass: superclass,
            initialized: AtomicBool::new(false),
            mutex: Mutex::<u8>::new(0),
        }
    }

    pub fn get_method_in_vtable(&self, name: &str, desc: &str) -> Option<Arc<Method>> {
        if let Some(ref m) = self.vtable.get(&("", name, desc)) {
            Some(Arc::clone(m))
        } else {
            None
        }
    }

    pub fn get_method_in_itable(&self, ifs: &str, name: &str, desc: &str) -> Option<Arc<Method>> {
        if let Some(ref m) = self.itable.get(&(ifs, name, desc)) {
            Some(Arc::clone(m))
        } else {
            None
        }
    }

    fn build_vtable(
        current: &Class,
        superclass: &Option<Arc<Klass>>,
    ) -> HashMap<RefKey, Arc<Method>> {
        let mut vtable = HashMap::<RefKey, Arc<Method>>::new();
        match superclass {
            Some(klass) => {
                for (k, v) in &klass.vtable {
                    vtable.insert(k.clone(), Arc::clone(&v));
                }
            }
            None => {}
        }
        for m in &current.methods {
            if (m.is_public() || m.is_protected()) && !m.is_final() && !m.is_static() {
                vtable.insert(
                    RefKey::new("".to_string(), m.name.clone(), m.descriptor.clone()),
                    Arc::clone(&m),
                );
            }
        }
        vtable
    }

    fn build_itable(
        current: &Class,
        superclass: &Option<Arc<Klass>>,
        interfaces: &Vec<Arc<Klass>>,
    ) -> HashMap<RefKey, Arc<Method>> {
        let mut itable = HashMap::<RefKey, Arc<Method>>::new();
        match superclass {
            Some(klass) => {
                for (k, v) in &klass.itable {
                    itable.insert(k.clone(), Arc::clone(&v));
                }
            }
            None => {}
        }
        for ifs in interfaces {
            for m in &ifs.bytecode.methods {
                for implements in &current.methods {
                    if m.name == implements.name && m.descriptor == implements.descriptor {
                        itable.insert(
                            RefKey::new(
                                ifs.bytecode.get_name().to_string(),
                                m.name.clone(),
                                m.descriptor.clone(),
                            ),
                            Arc::clone(&m),
                        );
                        break;
                    }
                }
            }
        }
        itable
    }

    fn build_layout(
        current: &Class,
        superclass: &Option<Arc<Klass>>,
    ) -> (HashMap<RefKey, (usize, usize)>, usize) {
        let mut layout = HashMap::<RefKey, (usize, usize)>::new();
        let (len, size) = match superclass {
            Some(klass) => {
                let mut max = 0usize;
                for (k, v) in &klass.layout {
                    layout.insert(k.clone(), (v.0, v.1));
                    max = std::cmp::max(max, v.0 + v.1);
                }
                (klass.len, max)
            }
            None => (0usize, 0usize),
        };
        let mut len = std::cmp::min(len, size);
        for f in &current.fields {
            if f.memory_size() > len % 4 && len % 4 != 0 {
                len = len + len % 4;
            }
            layout.insert(
                RefKey::new(
                    current.get_name().to_string(),
                    f.name.clone(),
                    f.descriptor.clone(),
                ),
                (len, f.memory_size()),
            );
            len = len + f.memory_size();
        }
        len = len + len % 4;
        (layout, len)
    }

    pub fn payload_len(&self) -> usize {
        self.len
    }
}
