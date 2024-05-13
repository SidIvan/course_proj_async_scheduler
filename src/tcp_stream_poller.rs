use std::collections::HashMap;
use mio::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError};
use std::time::Duration;
use mio::{Events, Interest, Poll, Token};
use crate::SchedulerQueue;
use crate::socket_read_future::SocketReadFuture;
use mio::event::Source;
use crate::socket_write_future::{SocketWriteFuture, WriteFutureInfo};

pub(crate) struct TcpStreamPoller {
    poll: Poll,
    events: Events,
    streams: HashMap<Token, Arc<Mutex<TcpStream>>>,
    w_streams: HashMap<Token, Arc<WriteFutureInfo>>,
    queue: Arc<Mutex<dyn SchedulerQueue + Send>>,
    read_receiver: Receiver<Arc<Mutex<TcpStream>>>,
    read_sender: Arc<Mutex<SyncSender<Arc<Mutex<TcpStream>>>>>,
    write_receiver: Receiver<Arc<WriteFutureInfo>>,
    write_sender: Arc<Mutex<SyncSender<Arc<WriteFutureInfo>>>>,
}

impl TcpStreamPoller {
    pub(crate) fn new(queue: Arc<Mutex<dyn SchedulerQueue + Send>>, read_receiver: Receiver<Arc<Mutex<TcpStream>>>, read_sender: Arc<Mutex<SyncSender<Arc<Mutex<TcpStream>>>>>,
                      write_receiver: Receiver<Arc<WriteFutureInfo>>, write_sender: Arc<Mutex<SyncSender<Arc<WriteFutureInfo>>>>) -> Self {
        let poll = Poll::new().unwrap();
        let events = Events::with_capacity(1024);

        Self {
            poll,
            events,
            streams: HashMap::new(),
            w_streams: HashMap::new(),
            queue,
            read_receiver,
            read_sender,
            write_receiver,
            write_sender,
        }
    }

    pub(crate) fn add_read_stream(&mut self, stream: Arc<Mutex<TcpStream>>) {
        let token = Token(self.streams.len());
        let stream1 = Arc::clone(&stream);
        self.streams.insert(token, stream1);
        let mut stream = Arc::clone(&stream);
        self.poll.registry().register(&mut *stream.lock().unwrap(), token, Interest::READABLE).unwrap();
    }

    pub(crate) fn add_write_stream(&mut self, stream: Arc<WriteFutureInfo>) {
        let token = Token(self.streams.len());
        let stream1 = Arc::clone(&stream);
        self.w_streams.insert(token, stream1);
        let mut stream = Arc::clone(&stream.tcp_stream);
        self.poll.registry().register(&mut *stream.lock().unwrap(), token, Interest::WRITABLE).unwrap();
    }

    pub(crate) fn process(&mut self) {
        loop {
            loop {
                match self.read_receiver.try_recv() {
                    Ok(stream) => {
                        self.add_read_stream(stream);
                        break;
                    }
                    Err(err) => {
                        match err {
                            TryRecvError::Empty => {
                                break;
                            }
                            TryRecvError::Disconnected => {
                                return;
                            }
                        }
                    }
                }
            }
            loop {
                match self.write_receiver.try_recv() {
                    Ok(stream) => {
                        self.add_write_stream(stream);
                        break;
                    }
                    Err(err) => {
                        match err {
                            TryRecvError::Empty => {
                                break;
                            }
                            TryRecvError::Disconnected => {
                                return;
                            }
                        }
                    }
                }
            }
            self.poll.poll(&mut self.events, Some(Duration::new(0, 1000000))).unwrap();
            for event in self.events.iter() {
                if event.is_readable() && !event.is_read_closed() {
                    println!("IXIXIXIXIX");
                    let token = event.token();
                    let tmp = self.streams.get(&token).unwrap();
                    let cpy = Arc::clone(tmp);
                    cpy.lock().unwrap().deregister(self.poll.registry());
                    let x = Arc::clone(&self.read_sender);
                    let cpy = Arc::clone(tmp);
                    self.queue.lock().unwrap().repush(Box::pin(SocketReadFuture::new(cpy, x, false)));
                }
                if event.is_writable() && !event.is_write_closed() {
                    let token = event.token();
                    let tmp = self.w_streams.get(&token).unwrap();
                    let cpy = Arc::clone(tmp);
                    cpy.tcp_stream.lock().unwrap().deregister(self.poll.registry());
                    let x = Arc::clone(&self.write_sender);
                    let cpy = Arc::clone(tmp);
                    let stream = Arc::clone(&cpy.tcp_stream);
                    let data = Arc::clone(&cpy.data);
                    self.queue.lock().unwrap().repush(Box::pin(SocketWriteFuture::new(stream, data, x)));
                }
            }
        }
    }
}
