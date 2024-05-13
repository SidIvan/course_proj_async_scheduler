use std::future::Future;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;
use std::task::{Context, Poll};
use std::{io, thread};
use std::thread::JoinHandle;
use std::time::Duration;
use futures::task::noop_waker;
use mio::event::Source;
use mio::{Interest, Registry, Token};
use mio::net::TcpStream;
use crate::queue::SchedulerQueue;
use crate::{queue, TypeOfQueue};
use crate::tcp_stream_poller::TcpStreamPoller;


pub(crate) struct SingleThreadExecutor {
    pub(crate) queue: Arc<Mutex<dyn SchedulerQueue + Send>>,
    shutdown: Arc<AtomicBool>,
    join_handle: JoinHandle<()>,
}

impl SingleThreadExecutor {
    pub(crate) fn new(type_of_queue: TypeOfQueue) -> SingleThreadExecutor {
        let queue = queue::create_queue(type_of_queue);
        let q = Arc::clone(&queue);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_copy = Arc::clone(&shutdown);
        let join_handle = thread::spawn(move || {
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
                            Poll::Pending => {
                            }
                        }
                    }
                    Err(err_val) => {
                        match err_val {
                            TryRecvError::Empty => {
                                if q.lock().unwrap().can_stop() && shutdown_copy.load(Ordering::SeqCst) {
                                    return;
                                }
                                continue;
                            }
                            TryRecvError::Disconnected => {return;}
                        }
                    }
                }
            }
        });

        SingleThreadExecutor { queue, shutdown, join_handle }

    }

    pub(crate) fn spawn<F>(&self, future: F)
        where
            F: Future<Output = ()> + Send + 'static,
    {
        self.queue.lock().unwrap().push(Box::pin(future));
    }

    pub(crate) fn wait(self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.join_handle.join();
    }
}