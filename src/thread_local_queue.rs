use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::thread::ThreadId;
use rand::Rng;
use crate::single_queue::SingleGlobalQueue;
use crate::queue::{PushStrategy, SchedulerQueue};

pub(crate) struct ThreadLocalQueue {
    push_strategy: PushStrategy,
    queues: Mutex<Vec<SingleGlobalQueue>>,
    thread_nos: Vec<ThreadId>
}

impl ThreadLocalQueue {
    pub(crate) fn arc(queues_no: i32, push_strategy: PushStrategy) -> Arc<Mutex<ThreadLocalQueue>> {
        return Arc::new(Mutex::new(Self::new(queues_no, push_strategy)));
    }
    pub(crate) fn new(queues_no: i32, push_strategy: PushStrategy) -> ThreadLocalQueue {
        let mut queues: Vec<SingleGlobalQueue> = Vec::new();
        for _i in 0..queues_no {
            queues.push(SingleGlobalQueue::new());
        }
        return ThreadLocalQueue{
            push_strategy,
            queues: Mutex::new(queues),
            thread_nos: vec![],
        }
    }
}

impl SchedulerQueue for ThreadLocalQueue {
    fn push(&mut self, future: Pin<Box<dyn Future<Output=()> + Send>>) {
        match self.push_strategy {
            PushStrategy::RANDOM => {
                let mut queues = self.queues.lock().unwrap();
                let queue_ind = rand::thread_rng().gen_range(0..queues.len());
                let mut queue = queues.get_mut(queue_ind).unwrap();
                queue.push(future);

            }
            PushStrategy::SHORTEST => {
                let mut queues = self.queues.lock().unwrap();
                let mut j = 0;
                let mut min_len = queues.get(0).unwrap().num_tasks.load(Ordering::SeqCst);
                for i in 1..queues.len() {
                    let cur_len = queues.get(i).unwrap().num_tasks.load(Ordering::SeqCst);
                    if cur_len < min_len {
                        j = i;
                        min_len = cur_len;
                    }
                }
                queues.get_mut(j).unwrap().push(future);
            }
        }
    }

    fn repush(&mut self, future: Pin<Box<dyn Future<Output=()> + Send>>) {
        match self.push_strategy {
            PushStrategy::RANDOM => {
                let mut queues = self.queues.lock().unwrap();
                let queue_ind = rand::thread_rng().gen_range(0..queues.len());
                let mut queue = queues.get_mut(queue_ind).unwrap();
                queue.repush(future);

            }
            PushStrategy::SHORTEST => {
                let mut queues = self.queues.lock().unwrap();
                let mut j = 0;
                let mut min_len = queues.get(0).unwrap().num_tasks.load(Ordering::SeqCst);
                for i in 1..queues.len() {
                    let cur_len = queues.get(i).unwrap().num_tasks.load(Ordering::SeqCst);
                    if cur_len < min_len {
                        j = i;
                        min_len = cur_len;
                    }
                }
                queues.get_mut(j).unwrap().repush(future);
            }
        }
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
                queues.get_mut(i).unwrap().done();
                return;
            }
        }
    }

    fn can_stop(&mut self) -> bool {
        let thread_id = thread::current().id();
        let mut queues = self.queues.lock().unwrap();
        for i in 0..self.thread_nos.len() {
            if *self.thread_nos.get_mut(i).unwrap() == thread_id {
                return queues.get_mut(i).unwrap().can_stop();
            }
        }
        panic!("Can not find queue for thread {:?}", thread_id);
    }
}