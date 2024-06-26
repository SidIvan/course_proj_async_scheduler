use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::thread::ThreadId;
use rand::Rng;
use crate::single_queue::SingleGlobalQueue;
use crate::queue::SchedulerQueue;

pub(crate) struct StealingQueue {
    queues: Mutex<Vec<SingleGlobalQueue>>,
    thread_nos: Vec<ThreadId>,
    num_tasks: AtomicU64,
}

impl StealingQueue {
    pub(crate) fn arc(queues_no: i32) -> Arc<Mutex<StealingQueue>> {
        return Arc::new(Mutex::new(Self::new(queues_no)));
    }
    pub(crate) fn new(queues_no: i32) -> StealingQueue {
        let mut queues: Vec<SingleGlobalQueue> = Vec::new();
        for _i in 0..queues_no {
            queues.push(SingleGlobalQueue::new());
        }
        return StealingQueue{
            queues: Mutex::new(queues),
            thread_nos: vec![],
            num_tasks: AtomicU64::new(0),
        }
    }
}

impl SchedulerQueue for StealingQueue {
    fn push(&mut self, future: Pin<Box<dyn Future<Output=()> + Send>>) {
        let mut queues = self.queues.lock().unwrap();
        let queue_ind = rand::thread_rng().gen_range(0..queues.len());
        let mut queue = queues.get_mut(queue_ind).unwrap();
        queue.push(future);
        self.num_tasks.fetch_add(1, Ordering::SeqCst);
    }

    fn repush(&mut self, future: Pin<Box<dyn Future<Output=()> + Send>>) {
        let mut queues = self.queues.lock().unwrap();
        let queue_ind = rand::thread_rng().gen_range(0..queues.len());
        let mut queue = queues.get_mut(queue_ind).unwrap();
        queue.repush(future);
    }

    fn pop(&mut self) -> Result<Pin<Box<dyn Future<Output=()> + Send>>, TryRecvError> {
        let thread_id = thread::current().id();
        let mut queues = self.queues.lock().unwrap();
        for i in 0..self.thread_nos.len() {
            if *self.thread_nos.get_mut(i).unwrap() == thread_id {
                return queues.get_mut(i).unwrap().pop();
            }
        }
        self.thread_nos.push(thread_id);
        return queues.last_mut().unwrap().pop()
    }

    fn done(&mut self) {
        let thread_id = thread::current().id();
        let mut queues = self.queues.lock().unwrap();
        for i in 0..self.thread_nos.len() {
            if *self.thread_nos.get_mut(i).unwrap() == thread_id {
                self.num_tasks.fetch_sub(1, Ordering::SeqCst);
                queues.get_mut(i).unwrap().done();
                return;
            }
        }
        self.num_tasks.fetch_sub(1, Ordering::SeqCst);
    }

    fn can_stop(&mut self) -> bool {
        return self.num_tasks.load(Ordering::SeqCst) == 0;
    }
}