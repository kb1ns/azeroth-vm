use crate::gc;
use crate::jvm_heap;
use crate::mem::{
    klass::{Klass, ObjHeader, OBJ_HEADER_SIZE},
    *,
};
use std::sync::{Arc, RwLock};

pub struct Heap {
    _data: Vec<u8>,
    pub base: *mut u8,
    pub oldgen: Arc<RwLock<Region>>,
    pub from: Arc<RwLock<Region>>,
    pub to: Arc<RwLock<Region>>,
    pub eden: Arc<RwLock<Region>>,
    pub young_capacity: u32,
    pub eden_size: u32,
    pub survivor_size: u32,
    pub old_size: u32,
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
        let mut data =
            Vec::<u8>::with_capacity(PTR_SIZE + (old_size + eden_size + survivor_size) as usize);
        let ptr = data.as_mut_ptr();
        let mut offset = PTR_SIZE as u32;
        let eden = Region::new(offset, eden_size);
        offset += eden_size;
        let from = Region::new(offset, survivor_size);
        offset += survivor_size;
        let to = Region::new(offset, survivor_size);
        offset += survivor_size;
        let oldgen = Region::new(offset, old_size);
        unsafe {
            HEAP.replace(Heap {
                _data: data,
                base: ptr,
                from: Arc::new(RwLock::new(from)),
                to: Arc::new(RwLock::new(to)),
                eden: Arc::new(RwLock::new(eden)),
                oldgen: Arc::new(RwLock::new(oldgen)),
                young_capacity: offset,
                eden_size: eden_size,
                survivor_size: survivor_size,
                old_size: old_size,
            });
        }
    }

    pub fn is_young_object(addr: Ref) -> bool {
        addr < jvm_heap!().young_capacity
    }

    pub fn is_null(addr: Ref) -> bool {
        addr == 0
    }

    pub fn copy_object_to_region(obj_ref: *mut Ref, obj: &mut ObjHeader, region: &mut Region) {
        // let native_ptr = Heap::ptr(addr as usize);
        // let mut obj = ObjHeader::from_vm_raw(native_ptr);
        if obj.incr_gc_age() {
            // TODO copy to old generation
        }
        // array and instance
        let instance_len = match obj.is_instance() {
            true => unsafe { &*obj.klass }.len as usize,
            false => obj.size.unwrap() as usize * unsafe { &*obj.klass }.len,
        } + OBJ_HEADER_SIZE;
        let free = Heap::ptr(region.offset as usize);
        unsafe { free.copy_from(Heap::ptr(*obj_ref as usize), instance_len) };
        let addr = region.offset;
        region.offset = addr + instance_len as u32;
        unsafe { obj_ref.write(addr) };
    }

    pub fn swap_from_and_to() {
        let mut from = jvm_heap!().from.write().unwrap();
        let mut to = jvm_heap!().to.write().unwrap();
        let mut eden = jvm_heap!().eden.write().unwrap();
        eden.offset = PTR_SIZE as u32;
        let offset = from.offset;
        from.offset = to.offset;
        to.offset = offset;
    }

    pub fn allocate_object(klass: &Arc<Klass>) -> Ref {
        if let Some(addr) = Self::allocate_object_in_region(klass, &jvm_heap!().eden) {
            return addr;
        } else if let Some(addr) = Self::allocate_object_in_region(klass, &jvm_heap!().oldgen) {
            return addr;
        }
        panic!("OutOfMemoryError");
    }

    pub fn allocate_object_directly(klass: &Arc<Klass>) -> Ref {
        if let Some(addr) = Self::allocate_object_in_region(klass, &jvm_heap!().oldgen) {
            return addr;
        }
        panic!("OutOfMemoryError");
    }

    fn allocate_object_in_region(klass: &Arc<Klass>, region: &Arc<RwLock<Region>>) -> Option<Ref> {
        let mut region = region.write().unwrap();
        let instance_len = OBJ_HEADER_SIZE + klass.len;
        if region.offset + instance_len as u32 >= region.limit {
            return None;
        }
        let obj_header = ObjHeader::new_instance(Arc::as_ptr(klass));
        let obj_ptr = obj_header.into_vm_raw().as_ptr();
        let free = Heap::ptr(region.offset as usize);
        unsafe { free.copy_from(obj_ptr, OBJ_HEADER_SIZE) };
        let addr = region.offset;
        region.offset = addr + instance_len as u32;
        Some(addr)
    }

    fn allocate_array_in_region(
        klass: &Arc<Klass>,
        region: &Arc<RwLock<Region>>,
        size: u32,
    ) -> Option<Ref> {
        let mut region = region.write().unwrap();
        let array_len = OBJ_HEADER_SIZE + klass.ref_len * size as usize;
        if region.offset + array_len as u32 >= region.limit {
            return None;
        }
        let array_header = ObjHeader::new_array(Arc::as_ptr(klass), size);
        let array_ptr = array_header.into_vm_raw().as_ptr();
        let free = Heap::ptr(region.offset as usize);
        unsafe { free.copy_from(array_ptr, OBJ_HEADER_SIZE) };
        let addr = region.offset;
        region.offset = region.offset + array_len as u32;
        Some(addr)
    }

    pub fn allocate_array(klass: &Arc<Klass>, size: u32) -> Option<Ref> {
        let array_len = OBJ_HEADER_SIZE + klass.ref_len * size as usize;
        let mut eden = jvm_heap!().eden.write().unwrap();
        // ensure enough space to allocate object
        if eden.offset + array_len as u32 >= eden.limit {
            return None;
        }
        let array_header = ObjHeader::new_array(Arc::as_ptr(klass), size);
        unsafe {
            let eden_ptr = jvm_heap!().base.add(eden.offset as usize);
            let array_header_ptr = array_header.into_vm_raw().as_ptr();
            eden_ptr.copy_from(array_header_ptr, OBJ_HEADER_SIZE);
            let addr = eden.offset;
            eden.offset = eden.offset + array_len as u32;
            Some(addr)
        }
    }

    pub fn ptr(offset: usize) -> *mut u8 {
        unsafe { jvm_heap!().base.add(offset) }
    }

    pub fn as_obj(addr: Ref) -> ObjHeader {
        ObjHeader::from_vm_raw(Heap::ptr(addr as usize))
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

#[cfg(test)]
mod test {

    use crate::mem::heap;
    use std::sync::Arc;

    #[test]
    pub fn test() {
        super::Heap::init(10 * 1024 * 1024, 1024 * 1024, 1024 * 1024);
        let java_lang_object = "yv66vgAAADQATgcAMQoAAQAyCgARADMKADQANQoAAQA2CAA3CgARADgKADkAOgoAAQA7BwA8CAA9CgAKAD4DAA9CPwgAPwoAEQBACgARAEEHAEIBAAY8aW5pdD4BAAMoKVYBAARDb2RlAQAPTGluZU51bWJlclRhYmxlAQAPcmVnaXN0ZXJOYXRpdmVzAQAIZ2V0Q2xhc3MBABMoKUxqYXZhL2xhbmcvQ2xhc3M7AQAJU2lnbmF0dXJlAQAWKClMamF2YS9sYW5nL0NsYXNzPCo+OwEACGhhc2hDb2RlAQADKClJAQAGZXF1YWxzAQAVKExqYXZhL2xhbmcvT2JqZWN0OylaAQANU3RhY2tNYXBUYWJsZQEABWNsb25lAQAUKClMamF2YS9sYW5nL09iamVjdDsBAApFeGNlcHRpb25zBwBDAQAIdG9TdHJpbmcBABQoKUxqYXZhL2xhbmcvU3RyaW5nOwEABm5vdGlmeQEACW5vdGlmeUFsbAEABHdhaXQBAAQoSilWBwBEAQAFKEpJKVYBAAhmaW5hbGl6ZQcARQEACDxjbGluaXQ+AQAKU291cmNlRmlsZQEAC09iamVjdC5qYXZhAQAXamF2YS9sYW5nL1N0cmluZ0J1aWxkZXIMABIAEwwAFwAYBwBGDABHACUMAEgASQEAAUAMABsAHAcASgwASwBMDAAkACUBACJqYXZhL2xhbmcvSWxsZWdhbEFyZ3VtZW50RXhjZXB0aW9uAQAZdGltZW91dCB2YWx1ZSBpcyBuZWdhdGl2ZQwAEgBNAQAlbmFub3NlY29uZCB0aW1lb3V0IHZhbHVlIG91dCBvZiByYW5nZQwAKAApDAAWABMBABBqYXZhL2xhbmcvT2JqZWN0AQAkamF2YS9sYW5nL0Nsb25lTm90U3VwcG9ydGVkRXhjZXB0aW9uAQAeamF2YS9sYW5nL0ludGVycnVwdGVkRXhjZXB0aW9uAQATamF2YS9sYW5nL1Rocm93YWJsZQEAD2phdmEvbGFuZy9DbGFzcwEAB2dldE5hbWUBAAZhcHBlbmQBAC0oTGphdmEvbGFuZy9TdHJpbmc7KUxqYXZhL2xhbmcvU3RyaW5nQnVpbGRlcjsBABFqYXZhL2xhbmcvSW50ZWdlcgEAC3RvSGV4U3RyaW5nAQAVKEkpTGphdmEvbGFuZy9TdHJpbmc7AQAVKExqYXZhL2xhbmcvU3RyaW5nOylWACEAEQAAAAAAAAAOAAEAEgATAAEAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAAlAQoAFgATAAABEQAXABgAAQAZAAAAAgAaAQEAGwAcAAAAAQAdAB4AAQAUAAAALgACAAIAAAALKiumAAcEpwAEA6wAAAACABUAAAAGAAEAAACVAB8AAAAFAAIJQAEBBAAgACEAAQAiAAAABAABACMAAQAkACUAAQAUAAAAPAACAAEAAAAkuwABWbcAAiq2AAO2AAS2AAUSBrYABSq2AAe4AAi2AAW2AAmwAAAAAQAVAAAABgABAAAA7AERACYAEwAAAREAJwATAAABEQAoACkAAQAiAAAABAABACoAEQAoACsAAgAUAAAAcgAEAAQAAAAyHwmUnAANuwAKWRILtwAMvx2bAAkdEg2kAA27AApZEg63AAy/HZ4ABx8KYUAqH7YAD7EAAAACABUAAAAiAAgAAAG/AAYBwAAQAcMAGgHEACQByAAoAckALAHMADEBzQAfAAAABgAEEAkJBwAiAAAABAABACoAEQAoABMAAgAUAAAAIgADAAEAAAAGKgm2AA+xAAAAAQAVAAAACgACAAAB9gAFAfcAIgAAAAQAAQAqAAQALAATAAIAFAAAABkAAAABAAAAAbEAAAABABUAAAAGAAEAAAIrACIAAAAEAAEALQAIAC4AEwABABQAAAAgAAAAAAAAAAS4ABCxAAAAAQAVAAAACgACAAAAKQADACoAAQAvAAAAAgAw";
        let class_vec = base64::decode(java_lang_object).unwrap();
        let bytecode = super::Class::from_vec(class_vec);
        let klass = super::Klass::new(
            Arc::new(bytecode),
            super::metaspace::ROOT_CLASSLOADER,
            None,
            vec![],
        );
        let klass = super::Arc::new(klass);
        let obj0 = super::Heap::allocate_object(&klass);
        assert_eq!(super::PTR_SIZE as u32, obj0);
        let obj1 = super::Heap::allocate_object(&klass);
        assert_eq!(
            super::OBJ_HEADER_SIZE + klass.len + obj0 as usize,
            obj1 as usize
        );

        let obj0_ptr = unsafe { jvm_heap!().base.add(obj0 as usize) };
        let obj_header: super::ObjHeader = super::klass::ObjHeader::from_vm_raw(obj0_ptr);
        let java_lang_object_klass = unsafe { &*obj_header.klass };
        assert_eq!("java/lang/Object", java_lang_object_klass.name);
    }
}
