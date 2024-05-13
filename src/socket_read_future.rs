use std::future::{Future};
use std::io::Read;
use mio::net::{TcpStream};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::SyncSender;
use std::task::{Context, Poll};

pub(crate) struct SocketReadFuture {
    socket: Arc<Mutex<TcpStream>>,
    poller: Arc<Mutex<SyncSender<Arc<Mutex<TcpStream>>>>>,
}

impl SocketReadFuture {
    pub(crate) fn new(socket: Arc<Mutex<TcpStream>>, poller: Arc<Mutex<SyncSender<Arc<Mutex<TcpStream>>>>>) -> SocketReadFuture {
        println!("AXAXA");
        return  SocketReadFuture{ socket: socket, poller: poller}
    }
}

impl Future for SocketReadFuture {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut buf= [0; 1024];
        let mut y = self.socket.lock().unwrap();
        let x = match y.read_exact(&mut buf) {
           Ok(_) => {
               Poll::Ready(())
           }
           Err(x) => {
               let tmp = Arc::clone(&self.socket);
               drop(y);
               self.poller.lock().unwrap().send(tmp);
               Poll::Pending
           }
       };
        println!("aaa{:?}", buf);
        x
    }
}