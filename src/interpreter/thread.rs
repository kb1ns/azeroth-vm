use crate::interpreter;
use crate::mem::{metaspace::*, stack::*, *};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct ThreadGroup {
    threads:
        Arc<Mutex<BTreeMap<u32, (Rc<RefCell<ThreadContext>>, Sender<u32>, Receiver<HashSet<Ref>>)>>>,
}

static mut THREADS: Option<ThreadGroup> = None;

pub const THREAD_RUNNING: u32 = 1;
pub const THREAD_WAIT: u32 = 2;

#[macro_export]
macro_rules! jvm_threads {
    () => {
        unsafe {
            match THREADS {
                Some(ref threads) => threads,
                None => panic!("ThreadGroup not initialized"),
            }
        }
    };
}

impl ThreadGroup {
    pub fn init() {
        unsafe {
            THREADS.replace(ThreadGroup {
                threads: Arc::new(Mutex::new(BTreeMap::new())),
            });
        }
    }

    pub fn new_thread(class_name: &str, method_name: &str, method_descriptor: &str, init: bool) {
        let context = {
            let mut threads = jvm_threads!().threads.lock().unwrap();
            let id = *threads.deref().keys().last().unwrap_or(&0);
            let (sig_tx, sig_rx) = channel();
            let (col_tx, col_rx) = channel();
            let thread = ThreadContext::new(id, sig_rx, col_tx);
            threads
                .deref_mut()
                .insert(id, (Rc::new(RefCell::new(thread)), sig_tx, col_rx));
            threads.deref().get(&id).unwrap().0.clone()
        };
        let mut context = context.borrow_mut();
        let class = match ClassArena::load_class(class_name, &mut context) {
            Err(no_class) => panic!(format!("ClassNotFoundException: {}", no_class)),
            Ok((class, _)) => class,
        };
        if init {
            interpreter::execute(&mut context);
        }
        let method = class
            .bytecode
            .as_ref()
            .unwrap()
            .get_method(method_name, method_descriptor)
            .expect("Method not found");
        context.stack.invoke(
            Arc::as_ptr(&class.bytecode.as_ref().unwrap()),
            Arc::as_ptr(&method),
            0,
            1,
        );
        interpreter::execute(&mut context);
        Self::remove_thread(context.id);
    }

    pub fn remove_thread(id: u32) {
        let mut threads = jvm_threads!().threads.lock().unwrap();
        threads.deref_mut().remove(&id);
    }

    pub fn collect_roots() -> HashSet<Ref> {
        let threads = jvm_threads!().threads.lock().unwrap();
        threads
            .deref()
            .values()
            .map(|t| {
                t.1.send(THREAD_WAIT).unwrap();
                t.2.recv().unwrap()
            })
            .flatten()
            .collect::<HashSet<_>>()
    }

    pub fn notify_all() {
        let threads = jvm_threads!().threads.lock().unwrap();
        threads
            .deref()
            .values()
            .for_each(|t| t.1.send(THREAD_RUNNING).unwrap());
    }
}

pub struct ThreadContext {
    pub pc: usize,
    pub stack: JavaStack,
    pub exception_pending: bool,
    pub throwable_initialized: bool,
    pub status: AtomicU32,
    pub id: u32,
    pub rx: Receiver<u32>,
    pub tx: Sender<HashSet<Ref>>,
}

impl ThreadContext {
    fn new(id: u32, rx: Receiver<u32>, tx: Sender<HashSet<Ref>>) -> Self {
        Self {
            pc: 0,
            stack: JavaStack::new(),
            exception_pending: false,
            throwable_initialized: false,
            status: AtomicU32::new(THREAD_RUNNING),
            id: id,
            rx: rx,
            tx: tx,
        }
    }

    pub fn roots(&self) -> HashSet<Ref> {
        self.stack.collect_tracing_roots()
    }
}
