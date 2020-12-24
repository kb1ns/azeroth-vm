// #![feature(weak_into_raw)]
use azerothvm::{
    gc,
    interpreter::thread::ThreadGroup,
    mem::{
        heap::Heap,
        metaspace::{ClassArena, *},
        strings::Strings,
    },
};

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
                    Ok(home) => start_vm(&main_class, &cp, &home),
                    Err(_) => panic!("JAVA_HOME not set"),
                }
            }
        }
        Err(_) => panic!("can't read file"),
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
    ClassArena::init(user_paths, system_paths);
    Heap::init(10 * 1024 * 1024, 1024 * 1024, 1024 * 1024);
    Strings::init();
    ThreadGroup::init();
    gc::init();
    ThreadGroup::new_thread(
        ROOT_CLASSLOADER,
        class_name,
        "main",
        "([Ljava/lang/String;)V",
        true,
    );
}
