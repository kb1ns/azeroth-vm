extern crate azerothvm;
extern crate regex;

use azerothvm::*;

fn main() {
    // TODO resovle args
    match std::env::var("JAVA_HOME") {
        Ok(home) => {
            start_vm("HelloWorld", "/tmp", &home);
        }
        Err(_) => {
            panic!("JAVA_HOME not set");
        }
    }
}

fn resolve_system_classpath(java_home: &str) -> Vec<String> {
    let mut java_home_dir = std::path::PathBuf::from(java_home);
    java_home_dir.push("jre/lib");
    let mut paths = Vec::<String>::new();
    if let Ok(sysjars) = std::fs::read_dir(&java_home_dir) {
        paths.append(&mut sysjars
            .map(|f| f.unwrap().path())
            .filter(|f| f.extension() == Some("jar".as_ref()))
            .map(|f| f.to_str().unwrap().to_string())
            .collect::<Vec<String>>());
        java_home_dir.push("ext");
        if let Ok(extjars) = std::fs::read_dir(&java_home_dir) {
            paths.append(&mut extjars
                .map(|f| f.unwrap().path())
                .filter(|f| f.extension() == Some("jar".as_ref()))
                .map(|f| f.to_str().unwrap().to_string())
                .collect::<Vec<String>>());
        }
        paths
    } else {
        panic!("JAVA_HOME not recognized");
    }
}

fn resolve_user_classpath(user_classpath: &str) -> Vec<String> {
    return user_classpath
        .split(":")
        .map(|p| p.to_string())
        .collect::<Vec<String>>();
}

fn start_vm(class_name: &str, user_classpath: &str, java_home: &str) {
    let system_paths = resolve_system_classpath(java_home);
    let user_paths = resolve_user_classpath(user_classpath);
    let class_arena = mem::metaspace::ClassArena::init(user_paths, system_paths);
    // TODO allocate heap
    // TODO GC thread
    let interpreter = interpreter::Interpreter {
        class_arena: std::sync::Arc::new(class_arena),
    };
    // TODO explicit lifetime
    let root_context = mem::stack::ThreadContext {};
    // TODO args
    interpreter.execute(class_name, "main", "([Ljava/lang/String;)V", &root_context);
}
