use super::*;
use std;

pub struct ClassArena {
    pub cp: super::Classpath,
    // TODO allow a class been loaded by different classloader instances
    pub classes: CHashMap<String, std::sync::Arc<Klass>>,
}

pub struct Klass {
    pub bytecode: super::Class,
    pub classloader: Classloader,
    pub initialized: std::sync::atomic::AtomicBool,
}

impl Klass {
    fn new(bytecode: Class, classloader: Classloader) -> Klass {
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

pub enum Classloader {
    ROOT,
    EXT,
    APP(Word),
}

impl ClassArena {
    pub fn init(app_paths: Vec<String>, bootstrap_paths: Vec<String>) -> ClassArena {
        let mut cp = super::Classpath::init();
        for path in bootstrap_paths {
            cp.append_bootstrap_classpath(path);
        }
        for path in app_paths {
            cp.append_app_classpath(path);
        }
        ClassArena {
            cp: cp,
            classes: CHashMap::new(),
        }
    }

    pub fn define_class(&self, class_name: &str, classloader: Classloader) -> Option<Klass> {
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

    pub fn find_class(&self, class: &str) -> Option<std::sync::Arc<Klass>> {
        let class_name = Regex::new(r"\.")
            .unwrap()
            .replace_all(class, "/")
            .into_owned();
        match self.classes.get(&class_name) {
            None => None,
            Some(ptr) => Some(ptr.clone()),
        }
    }
}
