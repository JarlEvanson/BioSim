#![warn(unused_imports)]

use std::{
    ffi::c_void,
    sync::{Arc, Condvar, Mutex},
    thread::{self},
};

type ThreadFunc = fn(*const std::ffi::c_void);

pub struct Work {
    func: ThreadFunc,
    arg: *const c_void,
    next: *mut Work,
}

struct ThreadPoolMutexData {
    workFirst: *mut Work,
    workLast: *mut Work,
    workingCnt: usize,
    threadCnt: usize,
    stop: bool,
}

impl ThreadPoolMutexData {
    pub fn addWork(&mut self, func: ThreadFunc, args: *const c_void) {
        let work = unsafe {
            let layout = std::alloc::Layout::new::<Work>();
            let work = std::alloc::alloc(layout) as *mut Work;

            (*work).arg = args;
            (*work).func = func;
            (*work).next = 0 as *mut Work;

            work
        };

        unsafe {
            if self.workFirst == 0 as *mut Work {
                self.workFirst = work;
                self.workLast = work;
            } else {
                (*self.workLast).next = work;
                self.workLast = work;
            }
        }
    }

    pub fn getWork(&mut self) -> Option<*mut Work> {
        let work = self.workFirst;

        if work == 0 as *mut Work {
            return None;
        }

        unsafe {
            if (*work).next == 0 as *mut Work {
                self.workFirst = 0 as *mut Work;
                self.workLast = 0 as *mut Work;
            } else {
                self.workFirst = (*work).next;
            }
        }

        Some(work)
    }
}

unsafe impl Send for ThreadPoolMutexData {}

struct ScopedThreadPoolPrivate {
    lock: Mutex<ThreadPoolMutexData>,
    workCond: std::sync::Condvar,
    workingCond: std::sync::Condvar,
}

impl ScopedThreadPoolPrivate {
    fn new(size: usize) -> ScopedThreadPoolPrivate {
        let data = ThreadPoolMutexData {
            workFirst: 0 as *mut Work,
            workLast: 0 as *mut Work,
            workingCnt: 0,
            threadCnt: size,
            stop: false,
        };

        let lock = Mutex::new(data);

        let workCond = Condvar::new();
        let workingCond = Condvar::new();

        ScopedThreadPoolPrivate {
            lock,
            workCond,
            workingCond,
        }
    }
}

#[derive(Clone)]
pub struct ScopedThreadPool {
    pool: Arc<ScopedThreadPoolPrivate>,
}

fn workerProcess(arg: ScopedThreadPool) {
    let mut work;

    loop {
        let mut data = arg
            .pool
            .workCond
            .wait_while(arg.pool.lock.lock().unwrap(), |data| {
                data.workFirst == 0 as *mut Work && !data.stop
            })
            .unwrap();

        if data.stop {
            break;
        }

        work = data.getWork();

        data.workingCnt += 1;

        std::mem::drop(data);

        if let Some(work) = work {
            unsafe { ((*work).func)((*work).arg) } //Calls function with args
            unsafe {
                std::alloc::dealloc(work as *mut u8, std::alloc::Layout::new::<Work>());
            }
        }

        let mut data = arg.pool.lock.lock().unwrap();
        data.workingCnt -= 1;

        if !data.stop && data.workingCnt == 0 && data.workFirst == 0 as *mut Work {
            arg.pool.workingCond.notify_all();
        }
    }

    let mut data = arg.pool.lock.lock().unwrap();
    data.threadCnt -= 1;
    arg.pool.workingCond.notify_all();
}

impl ScopedThreadPool {
    pub fn new(mut size: usize) -> ScopedThreadPool {
        if size == 0 {
            size = 2;
        }

        let pool = ScopedThreadPool {
            pool: Arc::new(ScopedThreadPoolPrivate::new(size)),
        };

        let mutex = pool.pool.lock.lock().unwrap();

        for _ in 0..size {
            let copy = pool.clone();
            thread::spawn(move || workerProcess(copy));
        }

        std::mem::drop(mutex);

        pool
    }

    pub fn addWork(&mut self, func: ThreadFunc, args: *const c_void) {
        let mut data = self.pool.lock.lock().unwrap();

        data.addWork(func, args);

        self.pool.workCond.notify_one();
    }

    pub fn getThreadCount(&self) -> usize {
        let mut data = self.pool.lock.lock().unwrap();

        data.threadCnt
    }
}

impl AsRef<Arc<ScopedThreadPoolPrivate>> for ScopedThreadPool {
    fn as_ref(&self) -> &Arc<ScopedThreadPoolPrivate> {
        &self.pool
    }
}
