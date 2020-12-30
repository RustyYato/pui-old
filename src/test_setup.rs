use std::{
    any::Any,
    boxed::Box,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    sync::{Arc, Barrier, Condvar, Mutex},
    thread::{spawn, JoinHandle},
    vec::Vec,
};

pub struct ThreadGroup(Arc<Packet>);

pub struct WaitOnDrop(Arc<Packet>);

enum ThreadInfo {
    Done,
    Empty,
    Panic(Box<dyn Any + Send>),
}

struct Packet {
    size: usize,
    barrier: Barrier,
    waiter: Barrier,
    threads: Mutex<Vec<ThreadInfo>>,
    cv: Condvar,
}

impl ThreadGroup {
    pub fn new(size: usize) -> ThreadGroup {
        Self(Arc::new(Packet {
            size,
            barrier: Barrier::new(1 + size),
            waiter: Barrier::new(size),
            threads: Mutex::new(Vec::with_capacity(size)),
            cv: Condvar::new(),
        }))
    }

    pub fn spawn<F: 'static + Send + FnOnce(usize, WaitOnDrop)>(&self, f: F) -> JoinHandle<()> {
        let packet = self.0.clone();
        let id;

        {
            let mut threads = packet.threads.lock().unwrap();
            id = threads.len();

            assert!(id < packet.size);

            threads.push(ThreadInfo::Empty);
        }

        spawn(move || {
            packet.barrier.wait();
            let panic = catch_unwind(AssertUnwindSafe(|| f(id, WaitOnDrop(packet.clone()))));

            let mut threads = packet.threads.lock().unwrap();

            if let Err(e) = panic {
                threads[id] = ThreadInfo::Panic(e);
            } else {
                threads[id] = ThreadInfo::Done;
            }

            packet.cv.notify_one();
        })
    }
}

impl WaitOnDrop {
    pub fn wait(&self) { self.0.waiter.wait(); }
}

impl Drop for WaitOnDrop {
    fn drop(&mut self) { self.0.waiter.wait(); }
}

impl Drop for ThreadGroup {
    fn drop(&mut self) {
        self.0.barrier.wait();

        let packet = &self.0;

        let mut threads = packet.threads.lock().unwrap();

        loop {
            let mut done = 0;

            for group in threads.iter_mut() {
                match std::mem::replace(group, ThreadInfo::Done) {
                    ThreadInfo::Done => done += 1,
                    ThreadInfo::Empty => (),
                    ThreadInfo::Panic(e) => resume_unwind(e),
                }
            }

            if done == packet.size {
                break
            } else {
                threads = packet.cv.wait(threads).unwrap();
            }
        }
    }
}
