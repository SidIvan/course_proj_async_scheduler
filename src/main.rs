use std::fmt::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::task::{Context, Poll};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use futures::executor::block_on;
use futures::task::noop_waker;

// use std::future::Future;
// use std::io::Read;
// use std::net::TcpListener;
// use std::pin::Pin;
// use std::sync::{Arc, mpsc, Mutex};
// use std::sync::mpsc::{Receiver, Sender, sync_channel, SyncSender};
// use std::task::{Context, Poll, Waker};
// use std::thread;
// use std::time::Duration;
// use futures::future::BoxFuture;
// use futures::FutureExt;
// use futures::task::{ArcWake, noop_waker, waker_ref};
// use tokio::task;
//
// pub struct TimerFuture {
//     shared_state: Arc<Mutex<SharedState>>,
// }
//
// pub struct SocketReadFuture {
//     shared_state: Arc<Mutex<SharedState>>
// }
//
// impl Future for SocketReadFuture {
//     type Output = ();
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let mut shared_state = self.shared_state.lock().unwrap();
//         if shared_state.completed {
//             Poll::Ready(())
//         } else {
//             shared_state.waker = Some(cx.waker().clone());
//             Poll::Pending
//         }
//     }
// }
//
// impl SocketReadFuture {
//     pub fn new(portno: &str) -> Self {
//         let shared_state = Arc::new(Mutex::new(SharedState {
//             completed: false,
//             waker: None,
//         }));
//         for _i in 0..2 {
//             let thread_shared_state = shared_state.clone();
//             let listener = TcpListener::bind("127.0.0.1:".to_owned() + portno).unwrap();
//             let (mut stream, _) = listener.accept().unwrap();
//             let mut buf = [0; 1];
//             stream.read(&mut buf);
//             println!("Received byte: {}", buf[0]);
//             let mut xshared_state = thread_shared_state.lock().unwrap();
//             xshared_state.completed = true;
//             if let Some(waker) = xshared_state.waker.take() {
//                 waker.wake()
//             }
//     }
//         SocketReadFuture { shared_state }
//     }
// }
//
// struct SharedState {
//     completed: bool,
//     waker: Option<Waker>,
// }
//
// impl Future for TimerFuture {
//     type Output = ();
//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let mut shared_state = self.shared_state.lock().unwrap();
//         if shared_state.completed {
//             Poll::Ready(())
//         } else {
//             shared_state.waker = Some(cx.waker().clone());
//             Poll::Pending
//         }
//     }
// }
//
// impl TimerFuture {
//     pub fn new(duration: Duration) -> Self {
//         let shared_state = Arc::new(Mutex::new(SharedState {
//             completed: false,
//             waker: None,
//         }));
//         let thread_shared_state = shared_state.clone();
//         thread::spawn(move || {
//             thread::sleep(duration);
//             let mut shared_state = thread_shared_state.lock().unwrap();
//             shared_state.completed = true;
//             if let Some(waker) = shared_state.waker.take() {
//                 waker.wake()
//             }
//         });
//         TimerFuture { shared_state }
//     }
// }
// struct Executor {
//     ready_queue: Receiver<Arc<Task>>,
// }
//
// impl Executor {
//     fn run(&self) {
//         while let Ok(task) = self.ready_queue.recv() {
//             let mut future_slot = task.future.lock().unwrap();
//             if let Some(mut future) = future_slot.take() {
//                 let waker = waker_ref(&task);
//                 let context = &mut Context::from_waker(&waker);
//                 if future.as_mut().poll(context).is_pending() {
//                     *future_slot = Some(future);
//                 }
//             }
//         }
//     }
// }
//
// // #[derive(Clone)]
// struct Spawner {
//     task_sender: SyncSender<Arc<Task>>,
// }
//
// /// A future that can reschedule itself to be polled by an `Executor`.
// struct Task {
//     future: Mutex<Option<BoxFuture<'static, ()>>>,
//     task_sender: SyncSender<Arc<Task>>,
// }
//
// fn new_executor_and_spawner() -> (Executor, Spawner) {
//     const MAX_QUEUED_TASKS: usize = 3;
//     let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
//     (Executor { ready_queue }, Spawner { task_sender })
// }
//
//
// impl Spawner {
//     fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
//         let future = future.boxed();
//         let task = Arc::new(Task {
//             future: Mutex::new(Some(future)),
//             task_sender: self.task_sender.clone(),
//         });
//         self.task_sender.send(task).expect("too many tasks queued");
//     }
// }
//
//
// impl ArcWake for Task {
//     fn wake_by_ref(arc_self: &Arc<Self>) {
//         let cloned = arc_self.clone();
//         arc_self
//             .task_sender
//             .send(cloned)
//             .expect("too many tasks queued");
//     }
// }
//
//
//
//
//
// // fn main() {
// //     let (executor, spawner) = new_executor_and_spawner();
// //
// //     // Spawn a task to print before and after waiting on a timer.
// //     // for i in 0..3 {
// //         spawner.spawn(
// //             async {
// //             println!("howdy!");
// //             // Wait for our timer future to complete after two seconds.
// //             // TimerFuture::new(Duration::new(2, 0)).await;
// //             SocketReadFuture::new("10000");
// //             println!("done!");
// //         });
// //         spawner.spawn(
// //             async {
// //                 println!("howdy!");
// //                 // Wait for our timer future to complete after two seconds.
// //                 // TimerFuture::new(Duration::new(2, 0)).await;
// //                 SocketReadFuture::new("10001");
// //                 println!("done!");
// //             });
// //     // }
// //
// //     // Drop the spawner so that our executor knows it is finished and won't
// //     // receive more incoming tasks to run.
// //     drop(spawner);
// //
// //     // Run the executor until the task queue is empty.
// //     // This will print "howdy!", pause, and then print "done!".
// //     executor.run();
// // }
//
//

trait SchedulerQueue {
    fn push<F>(&self, future: F)
    where F: Future<Output=()> + Send + 'static;

    fn pop(&self) -> Result<Pin<Box<dyn Future<Output = ()> + Send>>, RecvError>;
}

struct ChannelAsyncQueue {
    head: Receiver<Pin<Box<dyn Future<Output = ()> + Send>>>,
    tail: Sender<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl SchedulerQueue for ChannelAsyncQueue {
    fn push<F>(&self, future: F)
    where F : Future<Output=()> + Send + 'static
    {
        self.tail.send(Box::pin(future)).unwrap()
    }

    fn pop(&self) -> Result<Pin<Box<dyn Future<Output = ()> + Send>>, RecvError>{
        return self.head.recv();
    }
}

struct SingleThreadExecutor {
    queue: Sender<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

impl SingleThreadExecutor {
    fn new() -> SingleThreadExecutor {
        let (sender, receiver): (Sender<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>, Receiver<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>) = mpsc::channel();

        // Запускаем поток, который опрашивает задачи из очереди
        thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                let mut pinned = task;
                let y = noop_waker();
                let mut cx = Context::from_waker(&y);

                loop {
                    match pinned.as_mut().poll(&mut cx) {
                        Poll::Ready((z)) => {println!("{:?}",z);break},
                        Poll::Pending => {}
                    }
                }
            }
        });

        SingleThreadExecutor { queue: sender }

    }

    fn spawn<F>(&self, future: F)
        where
            F: Future<Output = ()> + Send + 'static,
    {
        self.queue.send(Box::pin(future)).unwrap();
    }
}

async fn some_async_foo() {
    println!("Задача x выполняется");
}

fn main() {
    let executor = SingleThreadExecutor::new();

    // Запускаем несколько асинхронных задач с помощью исполнителя
    executor.spawn(async {
        println!("Задача 1 выполняется");
    });
    executor.spawn(async {
        println!("Задача 2 выполняется");
    });
    executor.spawn(async {
        println!("Задача 3 выполняется");
    });
    executor.spawn(some_async_foo());
    sleep(Duration::new(5, 0))

}

// // use tokio::net::TcpListener;
// //
// // #[tokio::main]
// // async fn main() {
// //     let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
// //
// //     loop {
// //         let (socket, _) = listener.accept().await.unwrap();
// //         // A new task is spawned for each inbound socket. The socket is
// //         // moved to the new task and processed there.
// //         tokio::spawn(async move {
// //             process(socket).await;
// //         });
// //     }
// // }
// fn poll_future(future: &mut (dyn Future<Output = ()> + Send + 'static), cx: &mut Context<'_>) -> Poll<()> {
//     loop {
//         match future.poll(cx) {
//             Poll::Ready(()) => break,
//             Poll::Pending => {}
//         }
//     }
// }
// async fn my_future() -> i32 {
//     2
// }
//
// async fn async_main() {
//     let x = my_future();
//     let mut ctx = Context::from_waker(&noop_waker());
//     x.poll(&ctx);
//     let y = x.await;
// }
//
// fn main() {
//     block_on(async_main())
//     //
//     // let mut future = my_future();
//     // let mut cx = Context::from_waker(&noop_waker());
//     // poll_future(&mut future, &mut cx);
//
// }

// async fn my_future() -> i32 {
//     2
// }
//
// fn main() {
//     let mut future = Box::pin(my_future());
//     let binding = noop_waker();
//     let mut cx = Context::from_waker(&binding);
//
//     match future.as_mut().poll(&mut cx) {
//         Poll::Ready((i32)) => println!("{}", i32),
//         Poll::Pending => {}
//     }
// }
