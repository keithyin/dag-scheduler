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
    /// return
    ///     one to one. Vec<Self::MsgType>.len() == 1
    ///     multiple to one. return None at some steps. then return Vec<Self::MsgType>.len() == 1
    ///     one to multi. Vec<Self::MsgType>.len() > 1
    /// inp: 
    ///     SourceNode: always None
    ///     Middle & Sink. this is the extra step. designed for send the last buffer in the work node.
    ///         if no buffer in the worker node. just return None
    ///         
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>>;
    
    fn start(self: Box<Self>, receiver: Option<Receiver<Self::MsgType>>, sender: Option<Sender<Self::MsgType>>) -> JoinHandle<()> {
        thread::spawn(move || {
            if let Some(receiver) = receiver {
                for inp_v in receiver {
                    
                    if let Some(ref sender_) = sender { // middle
                        if let Some(vec_val) = self.as_ref().work_fn(Some(inp_v)) {
                            vec_val.into_iter().for_each(|val| sender_.send(val).unwrap());
                        }
                    } else { // for sink
                        self.as_ref().work_fn(Some(inp_v));
                    }
                
                }
            } else { // for source
                let sender = sender.as_ref().unwrap();
                loop {
                    if let Some(vec_val) = self.as_ref().work_fn(None) {
                        vec_val.into_iter().for_each(|val| sender.send(val).unwrap());
                    } else {
                        break;
                    }
                }
            }

            if let Some(ref sender_) = sender { // middle
                if let Some(vec_val) = self.as_ref().work_fn(None) {
                    vec_val.into_iter().for_each(|val| sender_.send(val).unwrap());
                }
            } else { // for sink
                self.as_ref().work_fn(None);
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
        .map(|_| 
            Box::new(SourceNode{counter: Arc::new(Mutex::new(2))}) 
                as Box<dyn TPplNode<MsgType = DummyMsg> + Sync>)
        .collect()
    }
}

impl TPplNode for SourceNode {
    type MsgType = DummyMsg;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>> {
        let mut v = self.counter.lock().unwrap();
        if *v == 0 {
            None
        } else {
            let old = *v;
            *v -= 1;
            Some(vec![DummyMsg { val: old}])
        }
        
    }
}

pub struct MiddleNode {
    
}

impl MiddleNode {
    pub fn new(num: usize) -> Vec<Box<dyn TPplNode<MsgType = DummyMsg> + Sync>> {
        (0..num).into_iter()
        .map(|_| 
            Box::new(MiddleNode{}) 
                as Box<dyn TPplNode<MsgType = DummyMsg> + Sync>)
        .collect()
    }
}

impl TPplNode for MiddleNode{
    type MsgType = DummyMsg;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>> {
        inp_v.and_then(|v| Some(vec![DummyMsg{val: v.val * 10}]))
    }

}

pub struct SinkNode {
}

impl SinkNode {
    pub fn new(num: usize) -> Vec<Box<dyn TPplNode<MsgType = DummyMsg> + Sync>> {
        (0..num).into_iter()
        .map(|_| 
            Box::new(SinkNode{}) 
            as Box<dyn TPplNode<MsgType = DummyMsg> + Sync>)
        .collect()
    }
}

impl TPplNode for SinkNode {
    type MsgType = DummyMsg ;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>> {
        println!("v:{:?}", inp_v.unwrap());
        None
    }
}