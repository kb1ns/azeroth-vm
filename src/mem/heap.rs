use crate::mem::{
    klass::{Klass, ObjectHeader, OBJ_HEADER_LEN},
    *,
};
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
        let mut data = Vec::<u8>::with_capacity(
            PTR_SIZE + (old_size + eden_size + survivor_size * 2) as usize,
        );
        let ptr = data.as_mut_ptr();
        let eden = Region::new(PTR_SIZE as u32, eden_size);
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

    pub fn allocate_object(&self, klass: &Arc<Klass>) -> Ref {
        let instance_len = OBJ_HEADER_LEN + klass.len;
        let mut eden = self.eden.write().unwrap();
        // ensure enough space to allocate object
        if eden.offset + instance_len as u32 >= eden.limit {
            // TODO gc
            panic!("OutOfMemoryError");
        }
        let obj_header = ObjectHeader::new(klass);
        unsafe {
            let eden_ptr = self.base.add(eden.offset as usize);
            // copy object header
            let obj_header_ptr = obj_header.into_vm_raw().as_ptr();
            eden_ptr.copy_from(obj_header_ptr, OBJ_HEADER_LEN);
            let addr = eden.offset;
            eden.offset = eden.offset + instance_len as u32;
            addr
        }
    }

    // pub fn allocate_array(&self, klass: &Arc<Klass>, size: usize) -> (Ref, u32){

    // }
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

#[cfg(test)]
mod test {

    use crate::mem::heap;

    #[test]
    pub fn test() {
        super::Heap::init(10 * 1024 * 1024, 1024 * 1024, 1024 * 1024);
        let java_lang_object = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";
        let class_vec = base64::decode(java_lang_object).unwrap();
        let bytecode = super::Class::from_vec(class_vec);
        let klass = super::Klass::new(bytecode, super::metaspace::Classloader::ROOT, None, vec![]);
        let klass = super::Arc::new(klass);
        let obj0 = jvm_heap!().allocate_object(&klass);
        assert_eq!(super::PTR_SIZE as u32, obj0);
        let obj1 = jvm_heap!().allocate_object(&klass);
        assert_eq!(
            super::OBJ_HEADER_LEN + klass.len + obj0 as usize,
            obj1 as usize
        );

        let obj0_ptr = unsafe { jvm_heap!().base.add(obj0 as usize) };
        let obj_header: super::ObjectHeader = super::klass::ObjectHeader::from_vm_raw(obj0_ptr);
        let java_lang_object_klass = unsafe { &*obj_header.klass };
        assert_eq!(
            "java/lang/Object",
            java_lang_object_klass.bytecode.this_class_name
        );
    }
}
