use crate::{mem::*, interpreter::thread::*};

use std::sync::{Arc, RwLock};
use std::thread;
use lazy_static::lazy_static;


lazy_static!{
    static ref ROOTS: Arc<RwLock<Vec<Ref>>> = Arc::new(RwLock::new(vec![]));
}

pub fn init_gc() {
    thread::spawn(|| {

    });
}


pub fn young_gc() {
    thread::spawn(|| {
        println!("young gc, collecting tracing roots: {:2x?}", ThreadGroup::collect_roots());
    });
}

pub fn old_gc() {
    
}
