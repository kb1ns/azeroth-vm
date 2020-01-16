extern crate base64;

use self::base64::decode;
use bytecode::class::Class;
use mem::metaspace::Klass;
use mem::*;
use std::mem::{size_of, transmute};
use std::sync::{Arc, RwLock};

pub struct Heap {
    pub oldgen: Region,
    pub s0: Arc<RwLock<Region>>,
    pub s1: Arc<RwLock<Region>>,
    pub eden: Arc<RwLock<Region>>,
}

pub struct Region {
    pub data: Vec<u8>,
    pub base: *mut u8,
    pub offset: u32,
}

impl Region {
    fn new(size: usize) -> Region {
        let mut data = vec![0u8; size];
        let base = (&mut data).as_mut_ptr();
        Region {
            data: data,
            base: base,
            offset: 0,
        }
    }

    fn copy(&mut self, ptr: *mut u8, size: usize) -> bool {
        // TODO capacity
        unsafe {
            self.base.add(self.offset as usize).copy_from(ptr, size);
            self.offset = self.offset + size as u32;
            true
        }
    }
}

impl Heap {
    pub fn init(old_size: usize, survivor_size: usize, eden_size: usize) {
        unsafe {
            HEAP.replace(Heap {
                oldgen: Region::new(old_size),
                s0: Arc::new(RwLock::new(Region::new(survivor_size))),
                s1: Arc::new(RwLock::new(Region::new(survivor_size))),
                eden: Arc::new(RwLock::new(Region::new(eden_size))),
            });
        }
    }
}

static mut HEAP: Option<Heap> = None;

#[macro_export]
macro_rules! jvm_heap {
    () => {
        match heap::HEAP {
            Some(ref heap) => heap,
            None => {
                panic!("Heap not initialized");
            }
        }
    };
}

pub fn allocate(klass: Arc<Klass>) -> (Ref, usize) {
    unsafe {
        let header = Klass::new_instance(klass);
        let ptr = transmute::<ObjectHeader, [u8; size_of::<ObjectHeader>()]>(header).as_mut_ptr();
        let mut eden = jvm_heap!().eden.write().unwrap();
        eden.copy(ptr, size_of::<ObjectHeader>());
        (
            transmute::<u32, Ref>(eden.offset),
            size_of::<ObjectHeader>(),
        )
    }
}

#[test]
pub fn test() {
    Heap::init(10 * 1024 * 1024, 1024 * 1024, 1024 * 1024);
    let java_lang_object = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";
    let class_vec = decode(java_lang_object).unwrap();
    let bytecode = Class::from_vec(class_vec);
    let klass = Klass::new(bytecode, metaspace::Classloader::ROOT);
    let (ptr, size) = allocate(Arc::new(klass));
    assert_eq!(24, size);
    assert_eq!(24, unsafe{transmute::<Ref, u32>(ptr)});
}
