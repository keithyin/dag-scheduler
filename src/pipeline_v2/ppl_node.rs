use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle}};

use crossbeam::{channel::{Receiver, Sender}, epoch::Atomic};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    // (channal_cap), sender is boss
    Source(usize),
    Middle(usize),
    Sink,
}

pub trait TPplNode: Send + 'static {
    type MsgType: Send + 'static;

    /// default behavior. 
    ///     return None. 
    ///         for source: job DONE
    ///         for middle: send nothing
    ///         for sink: return anything you want, but the returned value will be dropped immediately
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType>;
    
    fn start(self: Box<Self>, receiver: Option<Receiver<Self::MsgType>>, sender: Option<Sender<Self::MsgType>>) -> JoinHandle<()> {
        thread::spawn(move || {
            if let Some(receiver) = receiver {
                for inp_v in receiver {
                    
                    if let Some(ref sender_) = sender { // middle
                        if let Some(val) = self.as_ref().work_fn(Some(inp_v)) {
                            sender_.send(val).unwrap();
                        }
                    } else { // for sink
                        self.as_ref().work_fn(Some(inp_v));
                    }
                
                }
            } else { // for source
                let sender = sender.unwrap();
                loop {
                    if let Some(oup_v) = self.as_ref().work_fn(None) {
                        sender.send(oup_v).unwrap();
                    } else {
                        break;
                    }
                }
            }
            
        })
    }
    
}

#[derive(Debug)]
pub struct DummyMsg {
    pub val: usize,
}

pub struct SourceNode{
    counter: Arc<Mutex<usize>>
}

impl SourceNode {
    pub fn new(num: usize) -> Vec<Box<dyn TPplNode<MsgType = DummyMsg> + Sync>> {
        (0..num).into_iter()
            .map(|_| Box::new(SourceNode{counter: Arc::new(Mutex::new(10))}) as Box<dyn TPplNode<MsgType = DummyMsg> + Sync>)
            .collect()
    }
}

impl TPplNode for SourceNode {
    type MsgType = DummyMsg;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType> {
        let v = self.counter.lock().unwrap();
        if *v == 0 {
            None
        } else {
            Some(DummyMsg { val: *v})
        }
        
    }
}

pub struct MiddleNode {
    
}

impl MiddleNode {
    pub fn new(num: usize) -> Vec<Box<dyn TPplNode<MsgType = DummyMsg> + Sync>> {
        (0..num).into_iter()
            .map(|_| Box::new(MiddleNode{}) as Box<dyn TPplNode<MsgType = DummyMsg> + Sync>)
            .collect()
    }
}

impl TPplNode for MiddleNode{
    type MsgType = DummyMsg;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType> {
        inp_v.and_then(|v| Some(DummyMsg{val: v.val * 10}))
    }

}

pub struct SinkNode {
}

impl SinkNode {
    pub fn new(num: usize) -> Vec<Box<dyn TPplNode<MsgType = DummyMsg> + Sync>> {
        (0..num).into_iter()
            .map(|_| Box::new(SinkNode{}) as Box<dyn TPplNode<MsgType = DummyMsg> + Sync>)
            .collect()
    }
}

impl TPplNode for SinkNode {
    type MsgType = DummyMsg ;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType> {
        println!("v:{:?}", inp_v.unwrap());
        None
    }
}