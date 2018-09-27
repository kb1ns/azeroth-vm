extern crate azerothvm;
extern crate regex;

use azerothvm::*;
use regex::Regex;

fn main() {
    // TODO resovle args
    match std::env::var("JAVA_HOME") {
        Ok(home) => {
            start_vm("java.lang.String", "", &home);
        }
        Err(_) => {
            panic!("JAVA_HOME not set");
        }
    }
}

fn start_vm(class_name: &str, app_classpth: &str, java_home: &str) {
    let mut java_home_dir = std::path::PathBuf::from(java_home);
    java_home_dir.push("jre/lib");
    if let Ok(files) = std::fs::read_dir(java_home_dir) {
        let bootstrap_paths = files
            .map(|f| f.unwrap().path())
            .filter(|f| f.extension() == Some("jar".as_ref()))
            .map(|f| f.to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let app_paths = app_classpth
            .split(":")
            .map(|p| p.to_string())
            .collect::<Vec<String>>();
        let mut arena = mem::metaspace::ClassArena::init(app_paths, bootstrap_paths);
        let main_class = Regex::new(r"\.").unwrap().replace_all(class_name, "/");
        if let Some(klass) = arena.find_class(main_class.as_ref()) {
            println!("execute main class {}", klass.bytecode.get_class_name());
            // TODO allocate the main-thread stack to run class
            let _main_stack = mem::stack::JavaStack::allocate(128 * 1024);
            return;
        } else {
            panic!("java.lang.ClassNotFoundException");
        }
    } else {
        panic!("JAVA_HOME not recognized");
    }
}
