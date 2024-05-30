use std::future::Future;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;
use std::task::{Context, Poll};
use std::thread;
use std::thread::JoinHandle;
use futures::task::noop_waker;
use crate::queue::SchedulerQueue;
use crate::{queue, TypeOfQueue};

pub(crate) struct MultiThreadExecutor {
    pub(crate) queue: Arc<Mutex<dyn SchedulerQueue + Send>>,
    shutdown: Arc<AtomicBool>,
    join_handles: Vec<JoinHandle<()>>,
}

impl MultiThreadExecutor {
    pub(crate) fn new(num_threads: i32, type_of_queue: TypeOfQueue) -> MultiThreadExecutor {
        let queue = queue::create_queue(type_of_queue);
        let mut join_handles: Vec<JoinHandle<()>> = Vec::new();
        let shutdown = Arc::new(AtomicBool::new(false));
        for _i in 0..num_threads {
            let q = Arc::clone(&queue);
            let shutdown_cpy = Arc::clone(&shutdown);
            join_handles.push(thread::spawn(move || {
                loop {
                    let mut q_guard = q.lock().unwrap();
                    let task = q_guard.pop();
                    drop(q_guard);
                    match task {
                        Ok(task) => {
                            let mut pinned = task;
                            let y = noop_waker();
                            let mut cx = Context::from_waker(&y);

                            match pinned.as_mut().poll(&mut cx) {
                                Poll::Ready(z) => {
                                    q.lock().unwrap().done();
                                },
                                Poll::Pending => {}
                            }
                        }
                        Err(err_val) => {
                            match err_val {
                                TryRecvError::Empty => {
                                    if q.lock().unwrap().can_stop() && shutdown_cpy.load(Ordering::SeqCst) {
                                        return;
                                    }
                                    continue;
                                }
                                TryRecvError::Disconnected => { return; }
                            }
                        }
                    }
                }
            }));
        }
        MultiThreadExecutor { queue, shutdown, join_handles }

    }

    pub(crate) fn spawn<F>(&self, future: F)
        where
            F: Future<Output = ()> + Send + 'static,
    {
        self.queue.lock().unwrap().push(Box::pin(future));
    }

    pub(crate) fn wait(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        while self.join_handles.len() != 0 {
            let join_handle = self.join_handles.pop().unwrap();
            println!("{}", self.join_handles.len());
            join_handle.join();
        }
    }
}