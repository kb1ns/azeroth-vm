use mem::*;

pub struct ClassArena {
    pub cp: Classpath,
    // TODO allow a class been loaded by different classloader instances
    pub classes: CHashMap<String, std::sync::Arc<Klass>>,
}

pub struct Klass {
    pub bytecode: Class,
    pub classloader: Classloader,
    pub initialized: std::sync::atomic::AtomicBool,
    pub mutex: std::sync::Mutex<u8>,
}

impl Klass {
    fn new(bytecode: Class, classloader: Classloader) -> Klass {
        Klass {
            bytecode: bytecode,
            classloader: classloader,
            initialized: std::sync::atomic::AtomicBool::new(false),
            mutex: std::sync::Mutex::<u8>::new(0),
        }
    }

    fn instance_size(&self) -> usize {
        // self.bytecode.fields
        0
    }
}

pub enum Classloader {
    ROOT,
    EXT,
    APP(Word),
}

pub static mut CLASSES: Option<std::sync::Arc<ClassArena>> = None;

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
            CLASSES.replace(std::sync::Arc::new(ClassArena {
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

    pub fn find_class(&self, class: &str) -> Option<std::sync::Arc<Klass>> {
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
                        self.classes.insert_new(class_name, std::sync::Arc::new(k));
                        // we can't return k.clone() directly
                        match self.classes.get(&name) {
                            None => panic!("won't happend"),
                            Some(kwg) => Some(kwg.clone()),
                        }
                    }
                }
            }
            Some(ptr) => Some(ptr.clone()),
        }
    }
}
