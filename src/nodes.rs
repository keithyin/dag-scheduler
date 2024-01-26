use std::thread::{JoinHandle, self};

use crossbeam::channel::{Receiver, Sender};



pub trait TNode {
    type Comm;
    fn name(&self) -> &str;
    fn outputs(&self) -> &Vec<Receiver<Self::Comm>>;
    fn outputs_mut(&mut self) -> &mut Vec<Receiver<Self::Comm>>;
    
    fn add_input(&mut self, recv: &Receiver<Self::Comm>);

    fn inputs(&self) -> &Vec<Receiver<Self::Comm>>;
    fn inputs_mut(&mut self) -> &Vec<Receiver<Self::Comm>>;
    
    fn senders(&self) -> &Vec<Sender<Self::Comm>>;
    fn senders_mut(&self) -> &mut Vec<Sender<Self::Comm>>;

    fn work(&self, receivers: Vec<Receiver<Self::Comm>>, senders: Vec<Sender<Self::Comm>>);
    fn num_workers(&self) -> usize;

    fn start(&self) -> Vec<JoinHandle<()>>{
        let mut handles = vec![];
        let senders = self.senders();
        let inputs = self.inputs();
        for _ in 0..self.num_workers() {
            let senders = senders.iter()
                .map(|v| v.clone())
                .collect::<Vec<Sender<Self::Comm>>>();
            let inputs = inputs.iter()
                .map(|v| v.clone())
                .collect::<Vec<Receiver<Self::Comm>>>();
            let handler = thread::spawn(move || {
                self.work(inputs, senders);
            });
        }

        handles
    }
}