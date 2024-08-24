use std::{collections::HashMap, sync::{Arc, Mutex}, thread::{self, JoinHandle}};

use crossbeam::channel::{Receiver, Sender};
use once_cell::sync::Lazy;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    // (channal_cap), sender is boss
    Source(usize),
    Middle(usize),
    Sink,
}

static CALLER_COUNTS: Lazy<Arc<Mutex<HashMap<String, usize>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// a PplNode means a single thread worker. if you want multiple threads to do the same job. 
/// you should build the sampe PplNode multiple times 
pub trait TPplNode: Send + 'static {
    type MsgType: Send + 'static;

    /// return:
    ///     for Source:
    ///         return None means job done
    ///         return Some means result of job
    ///     for Middle:
    ///         return None means send nothing
    ///         return Some means result of job
    ///     for Sink
    ///         you shuold alway return None 
    /// inp_v:
    ///     for Source: always none
    ///     for Middle: none means the previous job done
    ///     for Sink: none means the previous job done
    /// based on this . one can implement 
    ///     one to one
    ///     multi to one.  (need a buffer within a woker node)
    ///     one to multi
    ///     multi to multi
    fn work_fn(&mut self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>>;

    /// specify worker name
    fn name(&self) -> &str;
    
    /// ppl_scheduler will call this method to start the worker thread
    fn start(mut self: Box<Self>, receiver: Option<Receiver<Self::MsgType>>, sender: Option<Sender<Self::MsgType>>) -> JoinHandle<()> {
        let worker_idx = {
            let mut counts = CALLER_COUNTS.lock().unwrap();
            let counter = counts.entry(self.name().to_string()).or_insert(0);
            *counter += 1;
            *counter - 1
        };
        
        thread::Builder::new().name(format!("{}-{}", self.name(), worker_idx))
        .spawn(move || {
            if let Some(receiver) = receiver {
                for inp_v in receiver {
                    
                    if let Some(ref sender_) = sender { // middle
                        if let Some(vec_val) = self.as_mut().work_fn(Some(inp_v)) {
                            vec_val.into_iter().for_each(|val| sender_.send(val).unwrap());
                        }
                    } else { // for sink
                        self.as_mut().work_fn(Some(inp_v));
                    }
                
                }
            } else { // for source
                let sender = sender.as_ref().unwrap();
                loop {
                    if let Some(vec_val) = self.as_mut().work_fn(None) {
                        vec_val.into_iter().for_each(|val| sender.send(val).unwrap());
                    } else {
                        break;
                    }
                }
            }

            if let Some(ref sender_) = sender { // middle
                if let Some(vec_val) = self.as_mut().work_fn(None) {
                    vec_val.into_iter().for_each(|val| sender_.send(val).unwrap());
                }
            } else { // for sink
                self.as_mut().work_fn(None);
            }
            
        }).unwrap()
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
    fn work_fn(&mut self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>> {
        let mut v = self.counter.lock().unwrap();
        if *v == 0 {
            None
        } else {
            let old = *v;
            *v -= 1;
            Some(vec![DummyMsg { val: old}])
        }
        
    }
    fn name(&self) -> &str {
        "SourceNode"
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
    fn work_fn(&mut self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>> {
        inp_v.and_then(|v| Some(vec![DummyMsg{val: v.val * 10}]))
    }
    fn name(&self) -> &str {
        "MiddleNode"
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
    fn work_fn(&mut self, inp_v: Option<Self::MsgType>) -> Option<Vec<Self::MsgType>> {
        println!("v:{:?}", inp_v.unwrap());
        None
    }

    fn name(&self) -> &str {
        "SinkNode"
    }
}