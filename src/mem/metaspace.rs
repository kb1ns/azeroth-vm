use std::collections::BTreeMap;

pub struct ClassArena {
    cp: super::Classpath,
    // TODO
    classes: BTreeMap<String, Klass>,
}

pub struct Klass {
    bytecode: super::Class,
    classloader: Classloader,
}

pub enum Classloader {
    ROOT,
    EXT,
    // TODO
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
            classes: BTreeMap::new(),
        }
    }

    fn define_class(&mut self, bytecode: Vec<u8>, classloader: Classloader) -> &Klass {
        let class = super::Class::from_vec(bytecode);
        let klass = Klass {
            bytecode: class,
            classloader: classloader,
        };
        let klass_name = klass.bytecode.get_class_name().to_string();
        self.classes.insert(klass_name.clone(), klass);
        if let Some(k) = self.classes.get(&klass_name) {
            return k;
        }
        panic!("java.lang.NoClassDefFoundError");
    }

    // TODO make this function thread-safe
    pub fn find_class(&mut self, class_name: &str) -> Option<&Klass> {
        if self.classes.contains_key(class_name) {
            return self.classes.get(class_name);
        }
        if let Some(bytecode) = self.cp.find_bootstrap_class(class_name) {
            return Some(self.define_class(bytecode, Classloader::ROOT));
        }
        if let Some(bytecode) = self.cp.find_ext_class(class_name) {
            return Some(self.define_class(bytecode, Classloader::EXT));
        }
        if let Some(bytecode) = self.cp.find_app_class(class_name) {
            // TODO classloader instance
            return Some(self.define_class(bytecode, Classloader::ROOT));
        }
        None
    }
}
