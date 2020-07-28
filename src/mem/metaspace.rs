use crate::mem::{klass::Klass, stack::*, *};
use crate::interpreter::thread::ThreadContext;
use log::trace;
use std::sync::{Arc, Mutex};

pub struct ClassArena {
    pub cp: Classpath,
    // TODO allow a class been loaded by different classloader instances
    pub classes: CHashMap<String, Arc<Klass>>,
    mutex: Mutex<u32>,
}

pub enum Classloader {
    ROOT,
    EXT,
    APP(Ref),
}

pub static mut CLASSES: Option<Arc<ClassArena>> = None;

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
        let mut arena = ClassArena {
            cp: cp,
            classes: CHashMap::new(),
            mutex: Mutex::new(0),
        };
        arena.classes.insert("I".to_owned(), Arc::new(Klass::new_phantom_klass("I")));
        arena.classes.insert("J".to_owned(), Arc::new(Klass::new_phantom_klass("J")));
        arena.classes.insert("F".to_owned(), Arc::new(Klass::new_phantom_klass("F")));
        arena.classes.insert("D".to_owned(), Arc::new(Klass::new_phantom_klass("D")));
        arena.classes.insert("S".to_owned(), Arc::new(Klass::new_phantom_klass("S")));
        arena.classes.insert("C".to_owned(), Arc::new(Klass::new_phantom_klass("C")));
        arena.classes.insert("Z".to_owned(), Arc::new(Klass::new_phantom_klass("Z")));
        arena.classes.insert("B".to_owned(), Arc::new(Klass::new_phantom_klass("B")));
        arena.classes.insert("V".to_owned(), Arc::new(Klass::new_phantom_klass("V")));
        unsafe {
            CLASSES.replace(Arc::new(arena));
        }
    }

    fn parse_class(&self, class_name: &str) -> Option<Class> {
        if let Some(bytecode) = self.cp.find_app_class(class_name) {
            return Some(Class::from_vec(bytecode));
        }
        if let Some(bytecode) = self.cp.find_bootstrap_class(class_name) {
            return Some(Class::from_vec(bytecode));
        }
        if let Some(bytecode) = self.cp.find_ext_class(class_name) {
            return Some(Class::from_vec(bytecode));
        }
        None
    }

    pub fn load_class(
        &self,
        class_name: &str,
        context: &mut ThreadContext,
    ) -> Result<(Arc<Klass>, bool), String> {
        let class_name = Regex::new(r"\.")
            .unwrap()
            .replace_all(class_name, "/")
            .into_owned();
        match self.classes.get(&class_name) {
            Some(klass) => Ok((Arc::clone(&klass), true)),
            None => {
                let _ = self.mutex.lock().unwrap();
                if let Some(loaded) = self.classes.get(&class_name) {
                    return Ok((loaded.clone(), true));
                }
                let class = match self.parse_class(&class_name) {
                    Some(class) => Arc::new(class),
                    None => {
                        return Err(class_name.to_owned());
                    }
                };
                let superclass = if !class.get_super_class().is_empty() {
                    Some(self.load_class(class.get_super_class(), context)?.0)
                } else {
                    None
                };
                let mut interfaces: Vec<Arc<Klass>> = vec![];
                for interface in class.get_interfaces() {
                    interfaces.push(self.load_class(interface, context)?.0);
                }
                initialize_class(&class, &mut context.stack, context.pc);
                // TODO classloader
                let klass = Arc::new(Klass::new(class, Classloader::ROOT, superclass, interfaces));
                self.classes.insert(class_name, klass.clone());
                Ok((klass, false))
            }
        }
    }
}

fn initialize_class(class: &Arc<Class>, stack: &mut JavaStack, pc: usize) {
    trace!("initializing class {}", class.get_name());
    match class.get_method("<clinit>", "()V") {
        Some(clinit) => {
            let frame = JavaFrame::new(Arc::as_ptr(&class), Arc::as_ptr(&clinit));
            stack.invoke(frame, pc);
        }
        None => {}
    }
}
