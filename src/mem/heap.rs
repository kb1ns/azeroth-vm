extern crate base64;

use self::base64::decode;
use mem::klass::{Klass, ObjectHeader};
use mem::*;
use std::mem::{size_of, transmute};
use std::sync::{Arc, RwLock};

pub struct Heap {
    _data: Vec<u8>,
    pub base: *mut u8,
    pub oldgen: Arc<RwLock<Region>>,
    pub s0: Arc<RwLock<Region>>,
    pub s1: Arc<RwLock<Region>>,
    pub eden: Arc<RwLock<Region>>,
}

pub struct Region {
    pub offset: Ref,
    pub limit: Ref,
}

impl Region {
    fn new(start: Ref, limit: Ref) -> Region {
        Region {
            offset: start,
            limit: limit,
        }
    }
}

impl Heap {
    pub fn init(old_size: u32, survivor_size: u32, eden_size: u32) {
        // TODO
        let mut data =
            Vec::<u8>::with_capacity((old_size + eden_size + survivor_size * 2) as usize);
        let ptr = data.as_mut_ptr();
        let eden = Region::new(0, eden_size);
        let s0 = Region::new(eden_size, survivor_size);
        let s1 = Region::new(eden_size + survivor_size, survivor_size);
        let oldgen = Region::new(eden_size + 2 * survivor_size, old_size);
        unsafe {
            HEAP.replace(Heap {
                _data: data,
                base: ptr,
                s0: Arc::new(RwLock::new(s0)),
                s1: Arc::new(RwLock::new(s1)),
                eden: Arc::new(RwLock::new(eden)),
                oldgen: Arc::new(RwLock::new(oldgen)),
            });
        }
    }

    pub fn allocate(&self, klass: &Arc<Klass>) -> (Ref, u32) {
        let header = Klass::new_instance(klass);
        let mut v = unsafe { transmute::<ObjectHeader, [u8; size_of::<ObjectHeader>()]>(header) };
        let ptr = v.as_mut_ptr();
        let mut eden = self.eden.write().unwrap();
        // TODO ensure enough space to allocate object
        unsafe {
            let eden_ptr = self.base.add(eden.offset as usize);
            eden_ptr.copy_from(ptr, size_of::<ObjectHeader>());
        }
        let addr = eden.offset;
        let instance_size = size_of::<ObjectHeader>() as u32 + (&klass).instance_size();
        eden.offset = eden.offset + instance_size;
        (addr, instance_size)
    }
}

pub static mut HEAP: Option<Heap> = None;

#[macro_export]
macro_rules! jvm_heap {
    () => {
        unsafe {
            match heap::HEAP {
                Some(ref heap) => heap,
                None => panic!("Heap not initialized"),
            }
        }
    };
}

#[test]
pub fn test() {
    Heap::init(10 * 1024 * 1024, 1024 * 1024, 1024 * 1024);
    let java_lang_object = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";
    let class_vec = decode(java_lang_object).unwrap();
    let bytecode = Class::from_vec(class_vec);
    let klass = Klass::new(bytecode, metaspace::Classloader::ROOT, None);
    let klass = Arc::new(klass);
    let (offset, size) = allocate(&klass);
    let payload_size = (&klass).instance_size();
    assert_eq!(size_of::<ObjectHeader>() as u32 + payload_size, size);
    let (offset, size) = allocate(&klass);
    assert_eq!(offset, size);
}
