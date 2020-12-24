use crate::interpreter::thread::ThreadContext;
use crate::mem::{klass::Klass, *};
use log::trace;
use std::sync::{Arc, Mutex};

pub struct ClassArena {
    pub cp: Classpath,
    // TODO allow a class been loaded by different classloader instances
    pub classes: CHashMap<String, Arc<Klass>>,
    mutex: Mutex<u32>,
}

// pub enum Classloader {
//     ROOT,
//     EXT,
//     APP(Ref),
// }

pub const ROOT_CLASSLOADER: Ref = 0;

static mut CLASSES: Option<Arc<ClassArena>> = None;

#[macro_export]
macro_rules! class_arena {
    () => {
        unsafe {
            match CLASSES {
                Some(ref classes) => classes,
                None => panic!("ClassArena not initialized"),
            }
        }
    };
}

impl ClassArena {
    pub fn init(app_paths: Vec<String>, bootstrap_paths: Vec<String>) {
        let mut cp = Classpath::init();
        for path in bootstrap_paths {
            cp.append_bootstrap_classpath(path);
        }
        for path in app_paths {
            cp.append_app_classpath(path);
        }
        let classes = CHashMap::new();
        classes.insert("I".to_owned(), Arc::new(Klass::new_phantom_klass("I")));
        classes.insert("J".to_owned(), Arc::new(Klass::new_phantom_klass("J")));
        classes.insert("F".to_owned(), Arc::new(Klass::new_phantom_klass("F")));
        classes.insert("D".to_owned(), Arc::new(Klass::new_phantom_klass("D")));
        classes.insert("S".to_owned(), Arc::new(Klass::new_phantom_klass("S")));
        classes.insert("C".to_owned(), Arc::new(Klass::new_phantom_klass("C")));
        classes.insert("Z".to_owned(), Arc::new(Klass::new_phantom_klass("Z")));
        classes.insert("B".to_owned(), Arc::new(Klass::new_phantom_klass("B")));
        classes.insert("V".to_owned(), Arc::new(Klass::new_phantom_klass("V")));
        let arena = ClassArena {
            cp: cp,
            classes: classes,
            mutex: Mutex::new(0),
        };
        unsafe { CLASSES.replace(Arc::new(arena)) };
    }

    fn parse_class(class_name: &str) -> Option<Class> {
        if let Some(bytecode) = class_arena!().cp.find_app_class(class_name) {
            return Some(Class::from_vec(bytecode));
        }
        if let Some(bytecode) = class_arena!().cp.find_bootstrap_class(class_name) {
            return Some(Class::from_vec(bytecode));
        }
        if let Some(bytecode) = class_arena!().cp.find_ext_class(class_name) {
            return Some(Class::from_vec(bytecode));
        }
        None
    }

    pub fn load_class(
        class_name: &str,
        context: &mut ThreadContext,
    ) -> Result<(Arc<Klass>, bool), String> {
        let class_name = Regex::new(r"\.")
            .unwrap()
            .replace_all(class_name, "/")
            .into_owned();
        match class_arena!().classes.get(&class_name) {
            Some(klass) => Ok((Arc::clone(&klass), true)),
            None => {
                let _ = class_arena!().mutex.lock().unwrap();
                if let Some(loaded) = class_arena!().classes.get(&class_name) {
                    return Ok((loaded.clone(), true));
                }
                if &class_name[..1] == "[" {
                    let (_, initialized) = Self::load_class(&class_name[1..], context)?;
                    let array_klass = Arc::new(Klass::new_phantom_klass(&class_name));
                    class_arena!()
                        .classes
                        .insert(class_name, array_klass.clone());
                    return Ok((array_klass, initialized));
                }
                let class = match Self::parse_class(&class_name) {
                    Some(class) => Arc::new(class),
                    None => {
                        return Err(class_name.to_owned());
                    }
                };
                let superclass = if !class.get_super_class().is_empty() {
                    Some(Self::load_class(class.get_super_class(), context)?.0)
                } else {
                    None
                };
                let mut interfaces: Vec<Arc<Klass>> = vec![];
                for interface in class.get_interfaces() {
                    interfaces.push(Self::load_class(interface, context)?.0);
                }
                initialize_class(&class, context);
                let klass = Arc::new(Klass::new(class, ROOT_CLASSLOADER, superclass, interfaces));
                class_arena!().classes.insert(class_name, klass.clone());
                Ok((klass, false))
            }
        }
    }
}

fn initialize_class(class: &Arc<Class>, context: &mut ThreadContext) {
    trace!("initializing class {}", class.get_name());
    match class.get_method("<clinit>", "()V") {
        Some(clinit) => {
            context.pc =
                context
                    .stack
                    .invoke(Arc::as_ptr(&class), Arc::as_ptr(&clinit), context.pc, 0);
        }
        None => {}
    }
}
