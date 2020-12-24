use crate::{
    interpreter::thread::ThreadContext,
    mem::{heap::Heap, metaspace::ClassArena, Ref},
};
use std::{collections::HashMap, sync::RwLock};

pub struct Strings(RwLock<HashMap<String, Ref>>);

static mut STRINGS: Option<Strings> = None;

#[macro_export]
macro_rules! strings {
    () => {
        unsafe {
            match STRINGS {
                Some(ref v) => v,
                None => panic!("StringConstants not initialized"),
            }
        }
    };
}

impl Strings {
    pub fn init() {
        unsafe {
            STRINGS.replace(Strings(RwLock::new(HashMap::with_capacity(4096))));
        }
    }

    pub fn get(constant: &str, context: &mut ThreadContext) -> Ref {
        {
            let constants = strings!().0.read().unwrap();
            if constants.contains_key(constant) {
                return *constants.get(constant).unwrap();
            }
        }
        let mut constants = strings!().0.write().unwrap();
        if constants.contains_key(constant) {
            return *constants.get(constant).unwrap();
        }
        let (klass, _) =
            ClassArena::load_class("java/lang/String", context).expect("jre_not_found");
        let obj = Heap::allocate_object_directly(&klass);
        // let bytearray = Heap::allocate_bytes_directly(constant);
        // let (content_field, len) = klass.layout.get(&("java/lang/String", "value", "[B")).unwrap();
        // unsafe {
        //     let target = Heap::ptr(obj as usize + OBJ_HEADER_SIZE + *content_field);
        //     target.copy_from(bytearray, *len);
        // }
        constants.insert(constant.to_owned(), obj);
        obj
    }
}
