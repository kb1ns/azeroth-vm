use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use mem::*;

pub struct ClassArena {
    pub cp: Classpath,
    // TODO allow a class been loaded by different classloader instances
    pub classes: CHashMap<String, Arc<Klass>>,
}

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub initialized: AtomicBool,
    pub mutex: Mutex<u8>,
}

impl Klass {
    pub fn new(bytecode: Class, classloader: Classloader) -> Klass {
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            initialized: AtomicBool::new(false),
            mutex: Mutex::<u8>::new(0),
        }
    }

    fn instance_size(&self) -> usize {
        0
    }

    pub fn new_instance(klass: Arc<Klass>) -> ObjectHeader {
        // TODO
        ObjectHeader {
            head: NULL,
            klass: klass,
            array_info: None,
        }
    }
}

pub enum Classloader {
    ROOT,
    EXT,
    APP(Word),
}

pub static mut CLASSES: Option<Arc<ClassArena>> = None;

macro_rules! find_class {
    ($x:expr) => {
        unsafe {
            match CLASSES {
                Some(ref classes) => classes.find_class($x),
                None => {
                    panic!("ClassArena not initialized");
                }
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

        unsafe {
            CLASSES.replace(Arc::new(ClassArena {
                cp: cp,
                classes: CHashMap::new(),
            }));
        }
    }

    fn define_class(&self, class_name: &str, classloader: Classloader) -> Option<Klass> {
        if let Some(bytecode) = self.cp.find_bootstrap_class(class_name) {
            return Some(Klass::new(Class::from_vec(bytecode), classloader));
        }
        if let Some(bytecode) = self.cp.find_ext_class(class_name) {
            return Some(Klass::new(Class::from_vec(bytecode), classloader));
        }
        if let Some(bytecode) = self.cp.find_app_class(class_name) {
            return Some(Klass::new(Class::from_vec(bytecode), classloader));
        }
        None
    }

    pub fn find_class(&self, class: &str) -> Option<Arc<Klass>> {
        let class_name = Regex::new(r"\.")
            .unwrap()
            .replace_all(class, "/")
            .into_owned();
        match self.classes.get(&class_name) {
            None => {
                // TODO classloader
                match self.define_class(&class_name, Classloader::ROOT) {
                    None => None,
                    Some(k) => {
                        let name = class_name.clone();
                        let instance = Arc::new(k);
                        self.classes.insert_new(class_name, instance.clone());
                        Some(instance)
                    }
                }
            }
            Some(ptr) => Some(ptr.clone()),
        }
    }
}
