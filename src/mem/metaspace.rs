use crate::mem::{klass::Klass, *};
use std::sync::Arc;

pub struct ClassArena {
    pub cp: Classpath,
    // TODO allow a class been loaded by different classloader instances
    pub classes: CHashMap<String, Arc<Klass>>,
}

pub enum Classloader {
    ROOT,
    EXT,
    APP(Ref),
}

pub static mut CLASSES: Option<Arc<ClassArena>> = None;

#[macro_export]
macro_rules! find_class {
    ($x:expr) => {
        unsafe {
            match CLASSES {
                Some(ref classes) => classes.load_class($x),
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

        unsafe {
            CLASSES.replace(Arc::new(ClassArena {
                cp: cp,
                classes: CHashMap::new(),
            }));
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

    pub fn load_class(&self, class: &str) -> Result<Arc<Klass>, String> {
        let class_name = Regex::new(r"\.")
            .unwrap()
            .replace_all(class, "/")
            .into_owned();
        match self.classes.get(&class_name) {
            None => match self.parse_class(&class_name) {
                None => Err(class.to_owned()),
                Some(k) => {
                    let superclass = (&k).get_super_class();
                    let superclass = if !superclass.is_empty() {
                        Some(self.load_class(superclass)?)
                    } else {
                        None
                    };
                    let ifs = (&k).get_interfaces();
                    let mut interfaces: Vec<Arc<Klass>> = vec![];
                    if !ifs.is_empty() {
                        for i in ifs {
                            interfaces.push(self.load_class(i)?);
                        }
                    }
                    // TODO classloader
                    let klass = Arc::new(Klass::new(k, Classloader::ROOT, superclass, interfaces));
                    self.classes.insert_new(class_name, klass.clone());
                    Ok(klass)
                }
            },
            Some(ptr) => Ok(ptr.clone()),
        }
    }
}
