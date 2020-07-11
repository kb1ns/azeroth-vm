use super::RefKey;
use crate::bytecode::{class::Class, method::Method};
use crate::mem::{metaspace::*, Ref, PTR_SIZE};
use std::collections::HashMap;
use std::mem::{size_of, transmute};
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
    pub klass: *const Klass,
}

pub const OBJ_HEADER_LEN: usize = size_of::<ObjectHeader>();

pub type ObjectHeaderRaw = [u8; OBJ_HEADER_LEN];

impl ObjectHeader {
    pub fn new(klass: Arc<Klass>) -> ObjectHeader {
        // The Arc counts are not affected.
        // The pointer is valid as long as the Arc has strong counts.
        // In another word, it is valid before the Klass unload.
        ObjectHeader {
            mark: 0,
            klass: Arc::into_raw(klass),
        }
    }

    pub fn into_vm_raw(self) -> ObjectHeaderRaw {
        unsafe { transmute::<Self, ObjectHeaderRaw>(self) }
    }

    pub fn from_vm_raw(ptr: *const u8) -> Self {
        let mut obj_header_raw = [0u8; OBJ_HEADER_LEN];
        let obj_header_ptr = obj_header_raw.as_mut_ptr();
        unsafe {
            obj_header_ptr.copy_from(ptr, OBJ_HEADER_LEN);
            transmute::<ObjectHeaderRaw, Self>(obj_header_raw)
        }
    }
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
            if (m.is_public() || m.is_protected())
                && !m.is_final()
                && !m.is_static()
                && m.name != "<init>"
            {
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
            if f.memory_size() > len % PTR_SIZE && len % PTR_SIZE != 0 {
                len = len + PTR_SIZE - len % PTR_SIZE;
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
        if len % PTR_SIZE != 0 {
            len = len + PTR_SIZE - len % PTR_SIZE;
        }
        (layout, len)
    }
}

#[cfg(test)]
pub mod test {

    use crate::bytecode::class::Class;
    use std::sync::Arc;

    const JAVA_LANG_OBJECT: &'static str = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";

    const DEFAULT_SIMPLE: &'static str = "yv66vgAAADQAEAoAAwANBwAOBwAPAQAGPGluaXQ+AQADKClWAQAEQ29kZQEAD0xpbmVOdW1iZXJUYWJsZQEABHRlc3QBAAgoSUpGRFopVgEADVN0YWNrTWFwVGFibGUBAApTb3VyY2VGaWxlAQALU2ltcGxlLmphdmEMAAQABQEABlNpbXBsZQEAEGphdmEvbGFuZy9PYmplY3QAIQACAAMAAAAAAAIAAQAEAAUAAQAGAAAAHQABAAEAAAAFKrcAAbEAAAABAAcAAAAGAAEAAAABAAkACAAJAAEABgAAAFUABAANAAAAGxoEYDYHHwplNwglDGo4ChUGmQAJGAQPbzkLsQAAAAIABwAAABoABgAAAAQABQAFAAoABgAPAAcAFAAIABoACgAKAAAACAAB/gAaAQQCAAEACwAAAAIADA==";

    const DEFAULT_TEST: &'static str = "yv66vgAAADQAGQoABAAVCAAWBwAXBwAYAQABaQEAAUkBAAFiAQABWgEAAWwBAAFKAQAGPGluaXQ+AQADKClWAQAEQ29kZQEAD0xpbmVOdW1iZXJUYWJsZQEACHRvU3RyaW5nAQAUKClMamF2YS9sYW5nL1N0cmluZzsBAAR0ZXN0AQAEKEkpVgEAClNvdXJjZUZpbGUBAA9UZXN0VlRhYmxlLmphdmEMAAsADAEAAAEAClRlc3RWVGFibGUBABBqYXZhL2xhbmcvT2JqZWN0ACEAAwAEAAAAAwAAAAUABgAAAAAABwAIAAAAAAAJAAoAAAADAAEACwAMAAEADQAAAB0AAQABAAAABSq3AAGxAAAAAQAOAAAABgABAAAAAgABAA8AEAABAA0AAAAbAAEAAQAAAAMSArAAAAABAA4AAAAGAAEAAAALAAEAEQASAAEADQAAABkAAAACAAAAAbEAAAABAA4AAAAGAAEAAAAQAAEAEwAAAAIAFA==";

    const DEFAULT_EXTENDS_TEST: &'static str = "yv66vgAAADQAFwoAAwAUBwAVBwAWAQABYQEAAUkBAAFiAQABWgEAAWMBAAFKAQABcwEAAVMBAANzdHIBABJMamF2YS9sYW5nL1N0cmluZzsBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAKU291cmNlRmlsZQEAFUV4dGVuZFRlc3RWVGFibGUuamF2YQwADgAPAQAQRXh0ZW5kVGVzdFZUYWJsZQEAClRlc3RWVGFibGUAIQACAAMAAAAFAAAABAAFAAAAAAAGAAcAAAAAAAgACQAAAAAACgALAAAAAAAMAA0AAAABAAEADgAPAAEAEAAAAB0AAQABAAAABSq3AAGxAAAAAQARAAAABgABAAAAAQABABIAAAACABM=";

    fn parse_class(bytecode: &str) -> Class {
        let class_vec = base64::decode(bytecode).unwrap();
        Class::from_vec(class_vec)
    }

    #[test]
    pub fn test_vtable() {
        let bytecode = parse_class(JAVA_LANG_OBJECT);
        let java_lang_object_klass = super::Klass::new(
            bytecode,
            crate::mem::metaspace::Classloader::ROOT,
            None,
            vec![],
        );
        let java_lang_object_klass = Arc::new(java_lang_object_klass);
        assert_eq!(5, java_lang_object_klass.vtable.len());
        let bytecode = parse_class(DEFAULT_SIMPLE);
        let default_simple_klass = super::Klass::new(
            bytecode,
            crate::mem::metaspace::Classloader::ROOT,
            Some(java_lang_object_klass.clone()),
            vec![],
        );
        assert_eq!(5, default_simple_klass.vtable.len());
        let to_string_method0 = java_lang_object_klass
            .vtable
            .get(&("", "toString", "()Ljava/lang/String;"))
            .unwrap();
        let to_string_method1 = default_simple_klass
            .vtable
            .get(&("", "toString", "()Ljava/lang/String;"))
            .unwrap();
        assert_eq!(true, Arc::ptr_eq(to_string_method0, to_string_method1));
        let bytecode = parse_class(DEFAULT_TEST);
        let default_test_klass = super::Klass::new(
            bytecode,
            crate::mem::metaspace::Classloader::ROOT,
            Some(java_lang_object_klass.clone()),
            vec![],
        );
        assert_eq!(6, default_test_klass.vtable.len());
        let to_string_method2 = default_test_klass
            .vtable
            .get(&("", "toString", "()Ljava/lang/String;"))
            .unwrap();
        assert_eq!(false, Arc::ptr_eq(to_string_method0, to_string_method2));
    }

    #[test]
    pub fn test_itable() {}

    #[test]
    pub fn test_layout() {
        let java_lang_object = parse_class(JAVA_LANG_OBJECT);
        let java_lang_object_klass = super::Klass::new(
            java_lang_object,
            crate::mem::metaspace::Classloader::ROOT,
            None,
            vec![],
        );
        assert_eq!(0, java_lang_object_klass.len);
        let java_lang_object_klass = Arc::new(java_lang_object_klass);
        let default_test = parse_class(DEFAULT_TEST);
        let default_test_klass = super::Klass::new(
            default_test,
            crate::mem::metaspace::Classloader::ROOT,
            Some(java_lang_object_klass.clone()),
            vec![],
        );
        assert_eq!(16, default_test_klass.len);
        assert_eq!(3, default_test_klass.layout.len());
        let default_extends_test = parse_class(DEFAULT_EXTENDS_TEST);
        let default_extends_test_klass = super::Klass::new(
            default_extends_test,
            crate::mem::metaspace::Classloader::ROOT,
            Some(Arc::new(default_test_klass)),
            vec![],
        );
        assert_eq!(40, default_extends_test_klass.len);
        assert_eq!(8, default_extends_test_klass.layout.len());
    }
}
