use std;
use super::Regex;

pub struct ClassArena {
    pub cp: super::Classpath,
    // TODO allow a class been loaded by different classloader instance
    pub classes: std::sync::RwLock<std::collections::BTreeMap<String, std::sync::Arc<Klass>>>,
}

pub struct Klass {
    pub bytecode: super::Class,
    pub classloader: Classloader,
}

pub enum Classloader {
    ROOT,
    EXT,
    // TODO classloader instance
    APP([u8; 4]),
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
            classes: std::sync::RwLock::new(std::collections::BTreeMap::new()),
        }
    }

    fn define_class(&self, bytecode: Vec<u8>, classloader: Classloader) -> std::sync::Arc<Klass> {
        let class = super::Class::from_vec(bytecode);
        let klass = Klass {
            bytecode: class,
            classloader: classloader,
        };
        let klass = std::sync::Arc::new(klass);
        let klass_share = klass.clone();
        let klass_name = klass.bytecode.this_class_name.clone();
        let mut map = self.classes.write().unwrap();
        if let Some(old_class) = map.insert(klass_name.clone(), klass) {
            // TODO load a class twice
        }
        klass_share
    }

    pub fn find_class(&self, class: &str) -> Option<std::sync::Arc<Klass>> {
        let class_name = Regex::new(r"\.")
            .unwrap()
            .replace_all(class, "/")
            .into_owned();
        {
            let map = self.classes.read().unwrap();
            if map.contains_key(&class_name) {
                return match map.get(&class_name) {
                    None => None,
                    Some(ptr) => Some(ptr.clone()),
                };
            }
        }
        // TODO indicate classloader explicitly or use thread-context parent classloader
        if let Some(bytecode) = self.cp.find_bootstrap_class(&class_name) {
            return Some(self.define_class(bytecode, Classloader::ROOT));
        }
        if let Some(bytecode) = self.cp.find_ext_class(&class_name) {
            return Some(self.define_class(bytecode, Classloader::EXT));
        }
        if let Some(bytecode) = self.cp.find_app_class(&class_name) {
            // TODO classloader instance
            return Some(self.define_class(bytecode, Classloader::ROOT));
        }
        None
    }
}
