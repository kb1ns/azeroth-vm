extern crate rjvm;

fn main() {
    let mut cp = rjvm::classpath::Classpath::init();
    cp.append_bootstrap_classpath("/Library/Java/JavaVirtualMachines/jdk1.8.0_151.jdk/Contents/Home/jre/lib/rt.jar".to_string());
    cp.find_bootstrap_class("java.util.concurrent.TimeUnit".to_string());
}
