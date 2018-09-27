extern crate azerothvm;

use std::env;

fn main() {
    match env::var("JAVA_HOME") {
        Ok(home) => {
            let mut cp = azerothvm::classpath::Classpath::init();
            cp.append_bootstrap_classpath(home + "jre/lib/rt.jar");
            println!("{}", cp.get_classpath());
            if let Some(class) =
                cp.find_bootstrap_class("java.util.concurrent.TimeUnit".to_string())
            {
                class.debug_constants();
            }
        }
        Err(_) => {
            panic!("JAVA_HOME not set");
        }
    }
}
