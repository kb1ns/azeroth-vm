extern crate base64;

use self::base64::decode;
use bytecode::class::Class;
use mem::metaspace::Klass;
use mem::*;
use std::mem::{size_of, transmute};
use std::sync::Arc;

pub struct Heap {
    pub oldgen_ptr: *mut u8,
    pub oldgen: Vec<u8>,
    pub s0_ptr: *mut u8,
    pub s0: Vec<u8>,
    pub s1_ptr: *mut u8,
    pub s1: Vec<u8>,
    pub eden_ptr: *mut u8,
    pub eden: Vec<u8>,
}

impl Heap {
    pub fn init(old_size: usize, survivor_size: usize, eden_size: usize) {
        let mut oldgen = vec![0u8; old_size];
        let mut eden = vec![0u8; eden_size];
        let mut s0 = vec![0u8; survivor_size];
        let mut s1 = vec![0u8; survivor_size];
        unsafe {
            HEAP.replace(Heap {
                oldgen_ptr: oldgen.as_mut_ptr(),
                oldgen: oldgen,
                s0_ptr: s0.as_mut_ptr(),
                s0: s0,
                s1_ptr: s1.as_mut_ptr(),
                s1: s1,
                eden_ptr: eden.as_mut_ptr(),
                eden: eden,
            });
        }
    }
}

static mut HEAP: Option<Heap> = None;

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

pub fn allocate(klass: Arc<Klass>) -> (*mut u8, usize) {
    unsafe {
        let header = Klass::new_instance(klass);
        let ptr = transmute::<ObjectHeader, [u8; size_of::<ObjectHeader>()]>(header).as_ptr();
        // TODO
        ptr.copy_to(jvm_heap!().eden_ptr, size_of::<ObjectHeader>());
        (jvm_heap!().eden_ptr, size_of::<ObjectHeader>())
    }
}

#[test]
pub fn test() {
    let java_lang_object = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";
    let class_vec = decode(java_lang_object).unwrap();
    let bytecode = Class::from_vec(class_vec);
    let klass = Klass::new(bytecode, metaspace::Classloader::ROOT);
    let (_, size) = allocate(Arc::new(klass));
    assert_eq!(3, size);
}
