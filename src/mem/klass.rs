use super::RefKey;
use crate::bytecode::{class::Class, method::Method};
use crate::mem::{metaspace::*, Ref, PTR_SIZE};
use std::collections::HashMap;
use std::mem::{size_of, transmute};
use std::sync::{atomic::AtomicBool, Arc, Mutex};

pub type MethodRef = (*const Class, *const Method);

pub struct Klass {
    pub bytecode: Option<Arc<Class>>,
    pub name: String,
    pub classloader: Ref,
    pub vtable: HashMap<RefKey, MethodRef>,
    pub itable: HashMap<RefKey, MethodRef>,
    pub layout: HashMap<RefKey, (usize, usize)>,
    pub len: usize,
    pub ref_len: usize,
    pub superclass: Option<Arc<Klass>>,
    pub superinterfaces: Vec<Arc<Klass>>,
    pub initialized: AtomicBool,
    pub mutex: Mutex<u8>,
}

#[derive(Clone)]
pub struct ObjHeader {
    pub mark: u32,
    pub size: Option<u32>,
    pub klass: *const Klass,
}

pub const OBJ_HEADER_SIZE: usize = size_of::<ObjHeader>();

pub const OBJ_HEADER_PADDING: usize = OBJ_HEADER_SIZE % PTR_SIZE;

pub type ObjHeaderRaw = [u8; OBJ_HEADER_SIZE];

const LOCK_STATE_MASK: u32 = 0x07;

const GC_STATE_MASK: u32 = 0x03;

const LOCK_FREE_FLAG: u32 = 0x01;

const GC_AGE_MASK: u32 = 0x78;

impl ObjHeader {

    pub fn is_lock_free(&self) -> bool {
        self.mark & LOCK_STATE_MASK == LOCK_FREE_FLAG
    }

    pub fn is_gc_status(&self) -> bool {
        self.mark & GC_STATE_MASK == GC_STATE_MASK
    }

    pub fn set_gc(&mut self) {
        self.mark &= GC_STATE_MASK;
    }

    pub fn get_gc_age(&self) -> u32 {
        (self.mark & GC_AGE_MASK) >> 3
    }

    pub fn is_instance(&self) -> bool {
        self.size.is_none()
    }

    pub fn incr_gc_age(&mut self) -> bool {
        if self.mark & GC_AGE_MASK == GC_AGE_MASK {
            return true;
        }
        self.mark += 0x08;
        return false;
    }

    pub fn new_instance(klass: *const Klass) -> Self {
        Self {
            mark: 0,
            size: None,
            klass: klass,
        }
    }

    pub fn new_array(klass: *const Klass, size: u32) -> Self {
        Self {
            mark: 0,
            size: Some(size),
            klass: klass,
        }
    }

    pub fn into_vm_raw(self) -> ObjHeaderRaw {
        unsafe { transmute::<Self, ObjHeaderRaw>(self) }
    }

    pub fn from_vm_raw(ptr: *const u8) -> Self {
        let mut obj_header_raw = [0u8; OBJ_HEADER_SIZE];
        let obj_header_ptr = obj_header_raw.as_mut_ptr();
        unsafe {
            obj_header_ptr.copy_from(ptr, OBJ_HEADER_SIZE);
            transmute::<ObjHeaderRaw, Self>(obj_header_raw)
        }
    }
}

impl Klass {
    pub fn new(
        bytecode: Arc<Class>,
        classloader: Ref,
        superclass: Option<Arc<Klass>>,
        interfaces: Vec<Arc<Klass>>,
    ) -> Self {
        let name = bytecode.get_name().to_owned();
        let mut klass = Klass {
            bytecode: Some(bytecode),
            name: name,
            classloader: classloader,
            vtable: HashMap::new(),
            itable: HashMap::new(),
            layout: HashMap::new(),
            len: 0,
            ref_len: PTR_SIZE,
            superclass: superclass,
            superinterfaces: interfaces,
            initialized: AtomicBool::new(false),
            mutex: Mutex::<u8>::new(0),
        };
        &klass.build_vtable();
        &klass.build_itable();
        &klass.build_layout();
        klass
    }

    pub fn new_phantom_klass(name: &str) -> Self {
        Klass {
            bytecode: None,
            name: name.to_owned(),
            classloader: ROOT_CLASSLOADER,
            vtable: HashMap::new(),
            itable: HashMap::new(),
            layout: HashMap::new(),
            len: match name {
                "I" | "F" | "Z" | "B" | "S" | "C" => PTR_SIZE,
                "D" | "J" => 2 * PTR_SIZE,
                _ => PTR_SIZE,
            },
            ref_len: match name {
                "I" | "F" | "Z" | "B" | "S" | "C" => PTR_SIZE,
                "D" | "J" => 2 * PTR_SIZE,
                _ => PTR_SIZE,
            },
            superclass: None,
            superinterfaces: vec![],
            initialized: AtomicBool::new(true),
            mutex: Mutex::<u8>::new(0),
        }
    }

    pub fn is_superclass(&self, target: &str) -> bool {
        let mut thisclass = self;
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

    pub fn get_method_in_vtable(&self, name: &str, desc: &str) -> Option<&MethodRef> {
        self.vtable.get(&("", name, desc))
    }

    pub fn get_method_in_itable(&self, ifs: &str, name: &str, desc: &str) -> Option<&MethodRef> {
        self.itable.get(&(ifs, name, desc))
    }

    pub fn get_holding_refs(&self, obj: Ref) -> Vec<*mut Ref> {
        self.layout.iter()
            .filter(|(k, _)| (&k.key.2).starts_with("L") || (&k.key.2).starts_with("["))
            .map(|(_, v)| v.0 as u32 + obj)
            .map(|mut r| &mut r as *mut u32)
            .collect::<_>()
    }

    fn build_vtable(&mut self) {
        match &self.superclass {
            Some(klass) => {
                for (k, v) in &klass.vtable {
                    self.vtable.insert(k.clone(), v.clone());
                }
            }
            None => {}
        }
        for m in &self.bytecode.as_ref().unwrap().methods {
            if (m.is_public() || m.is_protected())
                && !m.is_final()
                && !m.is_static()
                && m.name != "<init>"
            {
                self.vtable.insert(
                    RefKey::new("".to_string(), m.name.clone(), m.descriptor.clone()),
                    (
                        Arc::as_ptr(&self.bytecode.as_ref().unwrap()),
                        Arc::as_ptr(m),
                    ),
                );
            }
        }
    }

    fn build_itable(&mut self) {
        match &self.superclass {
            Some(klass) => {
                for (k, v) in &klass.itable {
                    self.itable.insert(k.clone(), v.clone());
                }
            }
            None => {}
        }
        let current = &*self.bytecode.as_ref().unwrap();
        for ifs in &self.superinterfaces {
            for m in &current.methods {
                if let Some(implement) = current.get_method(&m.name, &m.descriptor) {
                    self.itable.insert(
                        RefKey::new(ifs.name.clone(), m.name.clone(), m.descriptor.clone()),
                        (
                            Arc::as_ptr(&self.bytecode.as_ref().unwrap()),
                            Arc::as_ptr(&implement),
                        ),
                    );
                }
            }
        }
    }

    fn build_layout(&mut self) {
        let (len, size) = match &self.superclass {
            Some(klass) => {
                let mut max = 0usize;
                for (k, v) in &klass.layout {
                    self.layout.insert(k.clone(), (v.0, v.1));
                    max = std::cmp::max(max, v.0 + v.1);
                }
                (klass.len, max)
            }
            None => (0, 0),
        };
        let mut len = std::cmp::min(len, size);
        let current = &*self.bytecode.as_ref().unwrap();
        for f in &current.fields {
            self.layout.insert(
                RefKey::new(
                    current.get_name().to_string(),
                    f.name.clone(),
                    f.descriptor.clone(),
                ),
                (len, f.memory_size()),
            );
            len = len + f.memory_size();
        }
        self.len = len;
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
            Arc::new(bytecode),
            crate::mem::metaspace::ROOT_CLASSLOADER,
            None,
            vec![],
        );
        let java_lang_object_klass = Arc::new(java_lang_object_klass);
        assert_eq!(5, java_lang_object_klass.vtable.len());
        let bytecode = parse_class(DEFAULT_SIMPLE);
        let default_simple_klass = super::Klass::new(
            Arc::new(bytecode),
            crate::mem::metaspace::ROOT_CLASSLOADER,
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
        assert_eq!(
            true,
            std::ptr::eq((*to_string_method0).0, (*to_string_method1).0)
        );
        assert_eq!(
            true,
            std::ptr::eq((*to_string_method0).1, (*to_string_method1).1)
        );
        let bytecode = parse_class(DEFAULT_TEST);
        let default_test_klass = super::Klass::new(
            Arc::new(bytecode),
            crate::mem::metaspace::ROOT_CLASSLOADER,
            Some(java_lang_object_klass.clone()),
            vec![],
        );
        assert_eq!(6, default_test_klass.vtable.len());
        let to_string_method2 = default_test_klass
            .vtable
            .get(&("", "toString", "()Ljava/lang/String;"))
            .unwrap();
        assert_eq!(
            false,
            std::ptr::eq((*to_string_method0).0, (*to_string_method2).0)
        );
        assert_eq!(
            false,
            std::ptr::eq((*to_string_method0).1, (*to_string_method2).1)
        );
    }

    #[test]
    pub fn test_itable() {}

    #[test]
    pub fn test_layout() {
        let java_lang_object = parse_class(JAVA_LANG_OBJECT);
        let java_lang_object_klass = super::Klass::new(
            Arc::new(java_lang_object),
            crate::mem::metaspace::ROOT_CLASSLOADER,
            None,
            vec![],
        );
        assert_eq!(0, java_lang_object_klass.len);
        let java_lang_object_klass = Arc::new(java_lang_object_klass);
        let default_test = parse_class(DEFAULT_TEST);
        let default_test_klass = super::Klass::new(
            Arc::new(default_test),
            crate::mem::metaspace::ROOT_CLASSLOADER,
            Some(java_lang_object_klass.clone()),
            vec![],
        );
        assert_eq!(16, default_test_klass.len);
        assert_eq!(3, default_test_klass.layout.len());
        let default_extends_test = parse_class(DEFAULT_EXTENDS_TEST);
        let default_extends_test_klass = super::Klass::new(
            Arc::new(default_extends_test),
            crate::mem::metaspace::ROOT_CLASSLOADER,
            Some(Arc::new(default_test_klass)),
            vec![],
        );
        assert_eq!(40, default_extends_test_klass.len);
        assert_eq!(8, default_extends_test_klass.layout.len());
    }
}
