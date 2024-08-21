use std::thread::{self, JoinHandle};

use crossbeam::channel::{Sender, Receiver};

pub trait TPplNode {
    type Value;
    
    fn start(self: Box<Self>) -> JoinHandle<()>;

    fn get_sender(&self) -> &Option<Sender<Self::Value>>;
    fn get_receiver(&self) -> &Option<Receiver<Self::Value>>;
    fn set_sender(&mut self, sender: Option<Sender<Self::Value>>);
    fn set_receiver(&mut self, receiver: Option<Receiver<Self::Value>>);

}

#[derive(Debug)]
pub struct DummyCtx {

}

pub struct SourceNode{
    sender: Sender<DummyCtx>
}

impl TPplNode for SourceNode {
    type Value = DummyCtx;
    fn start(self: Box<Self>) -> JoinHandle<()> {
        thread::spawn(move|| {
            self.sender.send(DummyCtx {  }).unwrap();
        })

    }
}

pub struct MiddleNode {
    sender: Sender<DummyCtx>,
    receiver: Receiver<DummyCtx>
}

impl TPplNode for MiddleNode{
    type Value = DummyCtx;
    fn start(self: Box<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            for ctx in self.receiver {
                self.sender.send(ctx).unwrap();
            }
        })
    }
}

pub struct SinkNode {
    receiver: Receiver<DummyCtx>
}

impl TPplNode for SinkNode {
    type Value = DummyCtx;
    fn start(self: Box<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            for ctx in self.receiver {
                println!("{:?}", ctx);
            }
        })
    }
}