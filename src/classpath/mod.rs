extern crate regex;

use self::regex::Regex;
use std::fs;
use std::path::Path;
use super::bytecode::class::*;

enum ClassEntry {
    JAR(String),
    DIR(String),
}

impl ClassEntry {
    fn find_class(&self, class_file: &str) -> Option<Class> {
        match self {
            &ClassEntry::DIR(ref dir) => {
                let mut abs_path = dir.clone();
                abs_path.push_str(class_file);
                let classfile = Path::new(&abs_path);
                if classfile.exists() {
                    //TODO read
                }
                None
            },
            &ClassEntry::JAR(ref jar) => {

                None
            },
        }
    }
}

pub struct Classpath {
    bootstrap: Vec<ClassEntry>,
    ext: Vec<ClassEntry>,
    app: Vec<ClassEntry>,
}

impl Classpath {
    pub fn init() -> Classpath {
        Classpath {
            bootstrap: Vec::<ClassEntry>::new(),
            ext: Vec::<ClassEntry>::new(),
            app: Vec::<ClassEntry>::new(),
        }
    }

    pub fn find_bootstrap_class(&self, class_name: String) -> Option<Class> {
        Classpath::find_class(&self.bootstrap, class_name)
    }

    pub fn find_ext_class(&self, class_name: String) -> Option<Class> {
        Classpath::find_class(&self.ext, class_name)
    }

    pub fn find_app_class(&self, class_name: String) -> Option<Class> {
        Classpath::find_class(&self.app, class_name)
    }

    fn find_class(entries: &Vec<ClassEntry>, class_name: String) -> Option<Class> {
        let mut class_file = Regex::new(r"\.")
            .unwrap()
            .replace_all(&class_name, "/")
            .into_owned();
        class_file.push_str(".class");
        for entry in entries {
            match entry.find_class(&class_file) {
                None => {},
                Some(class) => {
                    return Some(class);
                }
            }
        }
        None
    }

    pub fn append_bootstrap_classpath(&mut self, path: String) {
        Classpath::append_classpath(&mut self.bootstrap, path);
    }

    pub fn append_ext_classpath(&mut self, path: String) {
        Classpath::append_classpath(&mut self.ext, path);
    }

    pub fn append_app_classpath(&mut self, path: String) {
        Classpath::append_classpath(&mut self.app, path);
    }

    fn append_classpath(entries: &mut Vec<ClassEntry>, path_str: String) {
        let s = &path_str.clone();
        let path = Path::new(s);
        if path.is_dir() {
            entries.push(ClassEntry::DIR(path_str));
            match fs::read_dir(&path) {
                Err(_) => panic!("bootstrap classpath needed."),
                Ok(paths) => paths
                    .map(|f| f.unwrap().path())
                    .filter(|f| f.ends_with(".jar"))
                    .map(|f| f.to_str().unwrap().to_string())
                    .for_each(|f| entries.push(ClassEntry::JAR(f))),
            }
        } else if path.ends_with(".jar") {
            entries.push(ClassEntry::JAR(path_str));
        }
    }
}
