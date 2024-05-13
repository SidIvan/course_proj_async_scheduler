mod single_queue;
mod queue;
mod thread_local_queue;
mod single_thread_executor;
mod multi_thread_executor;
mod socket_read_future;
mod tcp_stream_poller;
mod socket_write_future;

// use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, sync_channel, SyncSender};
use std::task::{Context, Poll};
use std::thread;
use futures::executor::block_on;
use mio::net::{TcpStream, TcpListener};
use rand::Rng;
use crate::multi_thread_executor::MultiThreadExecutor;
use crate::queue::{PushStrategy, SchedulerQueue, TypeOfQueue};
use crate::single_thread_executor::SingleThreadExecutor;
use crate::socket_read_future::SocketReadFuture;
use crate::socket_write_future::{SocketWriteFuture, WriteFutureInfo};
use crate::tcp_stream_poller::TcpStreamPoller;


async fn some_async_foo() {
    println!("Задача x выполняется");
}

fn read_socket_test() {
    // let executor = SingleThreadExecutor::new(TypeOfQueue::SimpleGlobal);
    let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::SimpleGlobal);
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::ThreadUnique(4, PushStrategy::RANDOM));
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::ThreadUnique(4, PushStrategy::SHORTEST));

    // for i in 0..10000 {
    //     let  y = i;
    //     executor.spawn(async move {
    //         // sleep(Duration::new(0, 1000000000));
    //         // some_async_foo().await;
    //         println!("Задача {} выполняется", &y);
    //     });
    // }
    let tmp = TcpListener::bind("127.0.0.1:8080".parse().unwrap()).unwrap();
    let tmp1 = Arc::clone(&executor.queue);
    let (snd, rcv): (SyncSender<Arc<Mutex<TcpStream>>>, Receiver<Arc<Mutex<TcpStream>>>) = sync_channel(1024);
    let arc_snd = Arc::new(Mutex::new(snd));
    let snd_clone = Arc::clone(&arc_snd);
    let (w_snd, w_rcv): (SyncSender<Arc<WriteFutureInfo>>, Receiver<Arc<WriteFutureInfo>>) = sync_channel(1024);
    let w_arc_snd = Arc::new(Mutex::new(w_snd));
    let w_snd_clone = Arc::clone(&w_arc_snd);
    let mut tcp_stream_poller = TcpStreamPoller::new(tmp1, rcv, snd_clone, w_rcv, w_snd_clone);
    thread::spawn(move || {
        tcp_stream_poller.process();
    });
    let snd_clone = Arc::clone(&arc_snd);
    loop {
        match tmp.accept() {
            Ok((tcp_stream, _)) => {
                let fut = SocketReadFuture::new(Arc::new(Mutex::new(tcp_stream)), snd_clone);
                executor.spawn(fut);
                break;
            }
            Err(_) => {}
        }
    }
    let tmp = TcpListener::bind("127.0.0.1:8081".parse().unwrap()).unwrap();
    let snd_clone = Arc::clone(&arc_snd);
    loop {
        match tmp.accept() {
            Ok((tcp_stream, _)) => {
                let fut = SocketReadFuture::new(Arc::new(Mutex::new(tcp_stream)), snd_clone);
                executor.spawn(fut);
                break;
            }
            Err(_) => {}
        }
    }
    executor.wait();
}

fn write_socket_test() {
    let executor = SingleThreadExecutor::new(TypeOfQueue::SimpleGlobal);
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::SimpleGlobal);
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::ThreadUnique(4, PushStrategy::RANDOM));
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::ThreadUnique(4, PushStrategy::SHORTEST));

    // for i in 0..10000 {
    //     let  y = i;
    //     executor.spawn(async move {
    //         // sleep(Duration::new(0, 1000000000));
    //         // some_async_foo().await;
    //         println!("Задача {} выполняется", &y);
    //     });
    // }
    let tmp1 = Arc::clone(&executor.queue);
    let (snd, rcv): (SyncSender<Arc<Mutex<TcpStream>>>, Receiver<Arc<Mutex<TcpStream>>>) = sync_channel(1024);
    let arc_snd = Arc::new(Mutex::new(snd));
    let snd_clone = Arc::clone(&arc_snd);
    let (w_snd, w_rcv): (SyncSender<Arc<WriteFutureInfo>>, Receiver<Arc<WriteFutureInfo>>) = sync_channel(1024);
    let w_arc_snd = Arc::new(Mutex::new(w_snd));
    let w_snd_clone = Arc::clone(&w_arc_snd);
    let mut tcp_stream_poller = TcpStreamPoller::new(tmp1, rcv, snd_clone, w_rcv, w_snd_clone);
    thread::spawn(move || {
        tcp_stream_poller.process();
    });
    let snd_clone = Arc::clone(&w_arc_snd);
    loop {
        match TcpStream::connect("127.0.0.1:8080".parse().unwrap()) {
            Ok((tcp_stream)) => {
                let data: [u8; 3] = [49, 49, 49];
                let fut = SocketWriteFuture::new(Arc::new(Mutex::new(tcp_stream)), Arc::new(data), snd_clone);
                executor.spawn(fut);
                break;
            }
            Err(_) => {}
        }
    }
    println!("DONE");
    let snd_clone = Arc::clone(&w_arc_snd);
    loop {
        match TcpStream::connect("127.0.0.1:8081".parse().unwrap()) {
            Ok((tcp_stream)) => {
                let data: [u8; 4] = [50, 50, 50, 50];
                let fut = SocketWriteFuture::new(Arc::new(Mutex::new(tcp_stream)), Arc::new(data), snd_clone);
                executor.spawn(fut);
                break;
            }
            Err(_) => {}
        }
    }
    executor.wait();
}

async fn async_main() {
    fibonacci_test();
}

fn fibonacci(n: i32) -> i32 {
    if n < 1 {
        panic!("Wrong number of fibonacci number")
    }
    if n <= 2 {
        return 1;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

async fn fibonacci_future(n: i32) {
    println!("{}-th fibonacci number is {}", n, fibonacci(n))
}

fn fibonacci_test() {
    // let executor = SingleThreadExecutor::new(TypeOfQueue::SimpleGlobal);
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::SimpleGlobal);
    // let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::ThreadUnique(4, PushStrategy::RANDOM));
    let executor: MultiThreadExecutor = MultiThreadExecutor::new(4, TypeOfQueue::ThreadUnique(4, PushStrategy::TYPE_SPLIT));
    for i in 1..20 {
        executor.spawn(fibonacci_future(42));
    }
    executor.wait();
}

fn main() {
    block_on(async_main());
}

pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}