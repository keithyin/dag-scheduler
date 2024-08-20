use std::thread::{self, JoinHandle};

use crossbeam::channel::{Sender, Receiver};

pub trait TPplNode {
    type Value;
    
    fn start(self: Box<Self>) -> JoinHandle<()>;
}


pub struct SourceNode<Ctx> {
    sender: Sender<Ctx>
}

impl <Ctx> TPplNode for SourceNode<Ctx> {
    type Value = Ctx;
    fn start(self: Box<Self>) -> JoinHandle<()> {
        thread::spawn(move|| {
            self.sender.send(Ctx::new()).unwrap();
        })

    }
}

pub struct MiddleNode<Ctx> {
    sender: Sender<Ctx>,
    receiver: Receiver<Ctx>
}

impl<Ctx> TPplNode for MiddleNode<Ctx> 
    where Ctx: Send + 'static,
{
    type Value = Ctx;
    fn start(self: Box<Self>) -> JoinHandle<()> {
        thread::spawn(move || {
            for ctx in self.receiver {
                self.sender.send(ctx).unwrap();
            }
        })
    }
}