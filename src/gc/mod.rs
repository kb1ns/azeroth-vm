use crate::mem::*;

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

fn mutate() {

}

pub fn young_gc() {
    
}

pub fn old_gc() {
    
}
