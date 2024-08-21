use std::thread::{self, JoinHandle};

use crossbeam::channel::{Sender, Receiver};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    // (channal_cap), receiver is boss
    Source,
    Middle(usize),
    Sink(usize),
}

pub trait TPplNode: Send + 'static {
    type MsgType: Send + 'static;

    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType>;
    
    fn start(self: Box<Self>, receiver: Option<Receiver<Self::MsgType>>, sender: Option<Sender<Self::MsgType>>) -> JoinHandle<()> {
        thread::spawn(move || {
            if let Some(receiver) = receiver {
                for inp_v in receiver {
                    
                    if let Some(ref sender_) = sender {
                        sender_.send(self.as_ref().work_fn(Some(inp_v)).unwrap()).unwrap();
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
}

impl TPplNode for SourceNode {
    type MsgType = DummyMsg;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType> {
        let mut rng = rand::thread_rng();
        let v = rng.gen::<f32>();
        if v < 0.3 {
            return Some(DummyMsg { val: (v * 100.) as usize });
        } else {
            None
        }
        
    }
}

pub struct MiddleNode {
    
}

impl TPplNode for MiddleNode{
    type MsgType = DummyMsg;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType> {
        inp_v.and_then(|v| Some(DummyMsg{val: v.val * 10}))
    }

}

pub struct SinkNode {
}

impl TPplNode for SinkNode {
    type MsgType = DummyMsg ;
    fn work_fn(&self, inp_v: Option<Self::MsgType>) -> Option<Self::MsgType> {
        println!("v:{:?}", inp_v.unwrap());
        None
    }
}