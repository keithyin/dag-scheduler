use std::thread::{JoinHandle, self};

use crossbeam::channel::{Receiver, Sender};



pub trait TNode {
    type Value;
    fn name(&self) -> &String;
    fn num_workers(&self) -> usize;
    fn channel_caps(&self) -> &Vec<usize>;

    fn work(&self, receivers: Vec<Receiver<Self::Value>>, senders: Vec<Sender<Self::Value>>);

}

pub struct CommonNode<Ctx> {
    name: String,
    num_workers: usize,
    channel_caps: Vec<usize>,
    work_core: fn(receivers: Vec<Receiver<Ctx>>, senders: Vec<Sender<Ctx>>)
}

impl<Ctx: Send> CommonNode <Ctx> {
    pub fn new(name: &str, num_workers: usize, channel_caps: &[usize], 
        work_core: fn(receivers: Vec<Receiver<Ctx>>, senders: Vec<Sender<Ctx>>)) -> Self {

        CommonNode { 
            name: name.to_string(), 
            num_workers,
            channel_caps: channel_caps.to_vec(),
            work_core
        }
    }
}

impl <Ctx: Send> TNode for CommonNode<Ctx> {
    type Value = Ctx;
    fn name(&self) -> &String {
        &self.name    
    }

    fn num_workers(&self) -> usize {
        self.num_workers
    }
    fn channel_caps(&self) -> &Vec<usize> {
        &self.channel_caps
    }

    fn work(&self, receivers: Vec<Receiver<Self::Value>>, senders: Vec<Sender<Self::Value>>) {
        (self.work_core)(receivers, senders);
    }

}