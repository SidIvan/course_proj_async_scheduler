use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};
use crate::single_queue::SingleGlobalQueue;
use crate::thread_local_queue::ThreadLocalQueue;

pub(crate) trait SchedulerQueue {
    fn push(& mut self, future: Pin<Box<dyn Future<Output = ()> + Send>>);

    fn repush(& mut self, future: Pin<Box<dyn Future<Output = ()> + Send>>);

    fn pop(& mut self) -> Result<Pin<Box<dyn Future<Output = ()> + Send>>, TryRecvError>;

    fn done(& mut self);

    fn can_stop(& mut self) -> bool;
}

pub(crate) enum PushStrategy {
    RANDOM,
    SHORTEST,
    TYPE_SPLIT,
}

pub(crate) enum TypeOfQueue {
    SimpleGlobal,
    ThreadUnique(i32, PushStrategy),
    StealingQueue(i32, PushStrategy)
}

pub(crate) fn create_queue(type_of_queue: TypeOfQueue) -> Arc<Mutex<dyn SchedulerQueue + Send>> {
    return match type_of_queue {
        TypeOfQueue::SimpleGlobal => {
            SingleGlobalQueue::arc()
        }
        TypeOfQueue::ThreadUnique(num_threads, push_strategy) => {
            ThreadLocalQueue::arc(num_threads, push_strategy)
        }
        TypeOfQueue::StealingQueue(num_threads, push_strategy) => {
            todo!()
        }
    }
}