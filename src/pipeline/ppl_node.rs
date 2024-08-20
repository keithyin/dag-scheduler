use crossbeam::channel::{Sender, Receiver};

pub trait TPplNode {
    type Value;



    fn work_middle(&self, thread_idx: usize, recv: Receiver<Self::Value>, send: Sender<Self::Value>) {}
    fn work_sink(&self, thread_idx: usize, recv: Receiver<Self::Value>) {
        
    }
    fn work_source(&self, thread_idx: usize, send: Sender<Self::Value>){}
}