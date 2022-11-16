use std::{
    collections::LinkedList,
    sync::{atomic::AtomicUsize, mpsc::Sender, Arc, Mutex},
    thread::{JoinHandle, Scope, Thread},
};

type ThreadFunc = fn(*const std::ffi::c_void);

struct Work {
    func: ThreadFunc,
    arg: *const std::ffi::c_void,
    next: *mut Work,
}

struct ScopedThreadPoolPrivate<'scope> {
    scope: Scope<'scope, 'scope>,
    workMutex: Arc<Mutex<bool>>,
}

pub struct ScopedThreadPool<'scope> {
    pool: Arc<Mutex<ScopedThreadPoolPrivate<'scope>>>,
}
