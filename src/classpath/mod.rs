extern crate zip;

use super::regex::Regex;
use std;
use std::fs::File;
use std::io::Read;
use std::path::Path;

enum ClassEntry {
    Jar(String),
    Dir(String),
}

impl ClassEntry {
    // class_file format: java/lang/String.class
    fn find_class(&self, class_file: &str) -> Option<Vec<u8>> {
        match self {
            &ClassEntry::Dir(ref dir) => {
                let mut abs_path = std::path::PathBuf::from(&dir);
                abs_path.push(class_file);
                if abs_path.exists() && abs_path.is_file() {
                    let mut f = File::open(abs_path).unwrap();
                    let meta = f.metadata().unwrap();
                    let mut buf = Vec::<u8>::with_capacity(meta.len() as usize);
                    f.read_to_end(&mut buf).unwrap();
                    trace!("find class {} from {}", class_file, dir);
                    Some(buf)
                } else {
                    None
                }
            }
            &ClassEntry::Jar(ref jar) => {
                let jar_file = File::open(&jar).unwrap();
                let mut archive = zip::ZipArchive::new(&jar_file).unwrap();
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    if file.name() == class_file {
                        let mut buf = Vec::<u8>::with_capacity(file.size() as usize);
                        file.read_to_end(&mut buf).unwrap();
                        trace!("find class {} from {}", class_file, jar);
                        return Some(buf);
                    }
                }
                None
            }
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

    pub fn find_bootstrap_class(&self, class_name: &str) -> Option<Vec<u8>> {
        Classpath::find_class(&self.bootstrap, class_name)
    }

    pub fn find_ext_class(&self, class_name: &str) -> Option<Vec<u8>> {
        Classpath::find_class(&self.ext, class_name)
    }

    pub fn find_app_class(&self, class_name: &str) -> Option<Vec<u8>> {
        Classpath::find_class(&self.app, class_name)
    }

    pub fn find_resource(&self, resource_name: &str) -> Option<File> {
        for entry in &self.app {
            match entry {
                &ClassEntry::Dir(ref dir) => match std::fs::read_dir(dir) {
                    Err(_) => panic!("bootstrap classpath read error."),
                    Ok(paths) => {
                        let f = paths
                            .map(|f| f.unwrap().path())
                            .filter(|f| f.ends_with(resource_name))
                            .map(|f| File::open(f).unwrap())
                            .find(|_| true);
                        if let Some(res) = f {
                            return Some(res);
                        }
                    }
                },
                &ClassEntry::Jar(_) => {}
            }
        }
        None
    }

    fn find_class(entries: &Vec<ClassEntry>, class_name: &str) -> Option<Vec<u8>> {
        let mut class_file = Regex::new(r"\.")
            .unwrap()
            .replace_all(class_name, "/")
            .into_owned();
        class_file.push_str(".class");
        for entry in entries {
            match entry.find_class(&class_file) {
                None => {}
                Some(class) => {
                    return Some(class);
                }
            }
        }
        None
    }

    pub fn get_classpath(&self) -> String {
        let mut cp = String::new();
        for e in &self.bootstrap {
            match e {
                &ClassEntry::Jar(ref s) => {
                    cp = cp + ":" + s;
                }
                &ClassEntry::Dir(ref s) => {
                    cp = cp + ":" + s;
                }
            }
        }
        for e in &self.ext {
            match e {
                &ClassEntry::Jar(ref s) => {
                    cp = cp + ":" + s;
                }
                &ClassEntry::Dir(ref s) => {
                    cp = cp + ":" + s;
                }
            }
        }
        for e in &self.app {
            match e {
                &ClassEntry::Jar(ref s) => {
                    cp = cp + ":" + s;
                }
                &ClassEntry::Dir(ref s) => {
                    cp = cp + ":" + s;
                }
            }
        }
        cp
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
            entries.push(ClassEntry::Dir(path_str));
        } else if path.extension() == Some("jar".as_ref()) {
            entries.push(ClassEntry::Jar(path_str));
        }
    }
}
