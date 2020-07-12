#![feature(weak_into_raw)]

use azerothvm::{mem::metaspace::CLASSES, *};
use std::sync::Arc;

fn main() {
    match std::env::current_dir() {
        Ok(dir) => {
            if let Some(cp) = dir.to_str() {
                let mut main_class = String::new();
                let mut cp = cp.to_string();
                {
                    let mut args = argparse::ArgumentParser::new();
                    args.refer(&mut cp)
                        .add_option(&["--classpath"], argparse::Store, "");
                    args.refer(&mut main_class)
                        .add_argument("", argparse::Store, "");
                    args.parse_args_or_exit();
                }
                match std::env::var("JAVA_HOME") {
                    Ok(home) => {
                        start_vm(&main_class, &cp, &home);
                    }
                    Err(_) => {
                        panic!("JAVA_HOME not set");
                    }
                }
            }
        }
        Err(_) => {
            panic!("can't read file");
        }
    }
}

fn resolve_system_classpath(java_home: &str) -> Vec<String> {
    let mut java_home_dir = std::path::PathBuf::from(java_home);
    java_home_dir.push("jre/lib");
    let mut paths = Vec::<String>::new();
    if let Ok(sysjars) = std::fs::read_dir(&java_home_dir) {
        paths.append(
            &mut sysjars
                .map(|f| f.unwrap().path())
                .filter(|f| f.extension() == Some("jar".as_ref()))
                .map(|f| f.to_str().unwrap().to_string())
                .collect::<Vec<String>>(),
        );
        java_home_dir.push("ext");
        if let Ok(extjars) = std::fs::read_dir(&java_home_dir) {
            paths.append(
                &mut extjars
                    .map(|f| f.unwrap().path())
                    .filter(|f| f.extension() == Some("jar".as_ref()))
                    .map(|f| f.to_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
            );
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
    mem::metaspace::ClassArena::init(user_paths, system_paths);
    mem::heap::Heap::init(10 * 1024 * 1024, 1024 * 1024, 1024 * 1024);
    // TODO GC thread
    let mut main_thread_stack = mem::stack::JavaStack::new();
    let entry_class = match class_arena!().load_class(class_name, &mut main_thread_stack, 0) {
        Err(no_class) => panic!(format!("ClassNotFoundException: {}", no_class)),
        Ok(class) => class,
    };
    let main_method = entry_class
        .bytecode
        .get_method("main", "([Ljava/lang/String;)V")
        .expect("Main method not found");
    let main_method = mem::stack::JavaFrame::new(
        Arc::as_ptr(&entry_class.bytecode),
        Arc::as_ptr(&main_method),
    );
    &mut main_thread_stack.invoke(main_method, 0);
    interpreter::execute(&mut main_thread_stack);
}
