use std::future::{Future};
use std::io::{Read, Write};
use mio::net::{TcpStream};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::SyncSender;
use std::task::{Context, Poll};

pub(crate) struct SocketWriteFuture {
    socket: Arc<Mutex<TcpStream>>,
    data: Arc<[u8]>,
    poller: Arc<Mutex<SyncSender<Arc<WriteFutureInfo>>>>,
}

pub(crate) struct WriteFutureInfo {
    pub(crate) tcp_stream: Arc<Mutex<TcpStream>>,
    pub(crate) data: Arc<[u8]>,
}

impl SocketWriteFuture {
    pub(crate) fn new(socket: Arc<Mutex<TcpStream>>, data: Arc<[u8]>, poller: Arc<Mutex<SyncSender<Arc<WriteFutureInfo>>>>) -> SocketWriteFuture {
        return  SocketWriteFuture{ socket, data, poller }
    }
}

impl Future for SocketWriteFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut y = self.socket.lock().unwrap();
        let mut data_copy = Arc::clone(&self.data);
        let x = match y.write_all(&mut data_copy) {
            Ok(_) => {
                Poll::Ready(())
            }
            Err(x) => {
                let tmp = Arc::clone(&self.socket);
                drop(y);
                let data_copy = Arc::clone(&self.data);
                self.poller.lock().unwrap().send(Arc::new(WriteFutureInfo{ tcp_stream: tmp, data: data_copy }));
                Poll::Pending
            }
        };
        x
    }
}