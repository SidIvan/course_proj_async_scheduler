use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, mpsc, Mutex};
use crate::queue::SchedulerQueue;

pub(crate) struct SingleGlobalQueue {
    pub(crate) num_tasks: AtomicU64,
    head: Arc<Mutex<Receiver<Pin<Box<dyn Future<Output = ()> + Send>>>>>,
    tail: Sender<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl SingleGlobalQueue {
    pub(crate) fn new() -> SingleGlobalQueue {
        let (sender, receiver): (Sender<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>, Receiver<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>) = mpsc::channel();
        let queue = SingleGlobalQueue { num_tasks: AtomicU64::new(0), head: Arc::new(Mutex::new(receiver)), tail: sender };
        return queue;
    }

    pub(crate) fn arc() -> Arc<Mutex<SingleGlobalQueue>> {
        return Arc::new(Mutex::new(Self::new()));
    }
}
impl SchedulerQueue for SingleGlobalQueue {
    fn push(&mut self, future: Pin<Box<dyn Future<Output = ()> + Send>>)
    {
        self.repush(future);
        self.num_tasks.fetch_add(1, Ordering::SeqCst);
    }

    fn repush(&mut self, future: Pin<Box<dyn Future<Output=()> + Send>>) {
        self.tail.send(future).unwrap();
    }

    fn pop(&mut self) -> Result<Pin<Box<dyn Future<Output = ()> + Send>>, TryRecvError>{
        loop {
            return match self.head.lock().unwrap().try_recv() {
                Ok(future) => {
                    // println!("{}", self.num_tasks.load(Ordering::SeqCst));
                    Ok(future)
                }
                Err(val) => { Err(val) }
            }
        }
    }

    fn done(&mut self) {
        self.num_tasks.fetch_sub(1, Ordering::SeqCst);
    }

    fn can_stop(& mut self) -> bool {
        return self.num_tasks.load(Ordering::SeqCst) == 0;
    }
}
