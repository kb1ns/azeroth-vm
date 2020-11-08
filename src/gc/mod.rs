use crate::{interpreter::thread::*, mem::*};

use lazy_static::lazy_static;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;

lazy_static! {
    static ref ROOTS: Arc<RwLock<Vec<Ref>>> = Arc::new(RwLock::new(vec![]));
}

pub static mut GC_TX: Option<mpsc::Sender<u32>> = None;

#[macro_export]
macro_rules! gc {
    () => {
        unsafe {
            match gc::GC_TX {
                Some(ref sender) => sender.send(0).unwrap(),
                None => panic!("GC not initialized"),
            }
        }
    };
}

pub fn init() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let _ = rx.recv().unwrap();
        gc();
    });
    unsafe {
        GC_TX.replace(tx);
    }
}

fn gc() {
    let roots = ThreadGroup::collect_roots();
    println!("young gc, collecting tracing roots: {:2x?}", roots);
    ThreadGroup::notify_all();
}
