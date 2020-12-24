use crate::{interpreter::thread::*, jvm_heap, mem::heap::Heap, mem::*};

use lazy_static::lazy_static;
use std::collections::HashMap;
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

fn copy_young(roots: &mut Vec<*mut Ref>) {
    let mut to = jvm_heap!().to.write().unwrap();
    let mut forwarding = HashMap::<Ref, Ref>::new();
    while !roots.is_empty() {
        let obj_ref = roots.pop().unwrap();
        let origin_addr = unsafe { *obj_ref };
        let mut obj = Heap::as_obj(origin_addr);
        if Heap::is_young_object(origin_addr) && !Heap::is_null(origin_addr) {
            Heap::copy_object_to_region(obj_ref, &mut obj, &mut to);
            obj.set_gc();
            forwarding.insert(origin_addr, unsafe { *obj_ref });
            let refs = unsafe { &*obj.klass }.get_holding_refs(unsafe { *obj_ref });
            for r in refs {
                if Heap::is_young_object(unsafe { *r }) && !Heap::is_null(unsafe { *r }) {
                    let gc_forwarding = forwarding.get(&unsafe { *r });
                    if gc_forwarding.is_some() {
                        unsafe { r.write(*gc_forwarding.unwrap()) };
                    } else {
                        roots.push(r);
                        break;
                    }
                }
            }
        }
    }
    Heap::swap_from_and_to();
}

pub fn gc() {
    let mut roots = ThreadGroup::collect_roots();
    copy_young(&mut roots);
    ThreadGroup::notify_all();
}
