extern crate azerothvm;

use std::env;
use std::fs;

fn main() {
    // TODO resovle args
    match env::var("JAVA_HOME") {
        Ok(home) => {
            start_vm("", "", &home);
        }
        Err(_) => {
            panic!("JAVA_HOME not set");
        }
    }
}

fn start_vm(class_name: &str, app_classpth: &str, java_home: &str) {
    if let Ok(files) = fs::read_dir(java_home) {
        let bootstrap_paths = files.map(|f| f.unwrap().path())
            .filter(|f| f.ends_with(".jar"))
            .map(|f| f.to_str().unwrap().to_string())
            .collect::<Vec<String>>();
        let app_paths = app_classpth.split(":").map(|p| p.to_string())
            .collect::<Vec<String>>();
        let mut arena = azerothvm::mem::metaspace::ClassArena::init(app_paths, bootstrap_paths);
        if let Some(klass) = arena.find_class(class_name) {
            // TODO
            return;
        }
    } else {
        panic!("JAVA_HOME not recognized");
    }
    panic!("no main method found");
}
