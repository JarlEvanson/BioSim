use std::alloc;
use std::alloc::Layout;
use std::ffi::c_void;
use std::mem;
use std::sync::{Arc, Condvar, Mutex};

type ThreadFunc = fn(*const c_void);

struct Work {
    func: ThreadFunc,
    next: *mut Work,
    ptr: [u8; 0],
}

struct Body {
    workFront: *mut Work,
    workLast: *mut Work,
    workIsAvailable: Condvar,
    noThreadsWorking: Condvar,
}

impl Body {
    fn new() -> Body {
        Body {
            workFront: 0 as *mut Work,
            workLast: 0 as *mut Work,
            workIsAvailable: Condvar::new(),
            noThreadsWorking: Condvar::new(),
        }
    }
}

struct ThreadPool {
    body: Arc<Mutex<Body>>,
}

impl ThreadPool {
    pub fn new(count: usize) -> ThreadPool {
        let body = Body::new();

        ThreadPool {
            body: Arc::new(Mutex::new(body)),
        }
    }
}
