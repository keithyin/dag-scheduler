use std::{collections::HashMap, thread};
use crossbeam::channel::{Receiver, Sender};

use crate::nodes;

pub struct Scheduler <Ctx>{

    nodes: HashMap<String, Box<dyn nodes::TNode<Value = Ctx> + Sync>>,
    nodes_inputs: HashMap<String, Vec<Receiver<Ctx>>>,
    nodes_outputs: HashMap<String, Vec<Receiver<Ctx>>>,
    nodes_senders: HashMap<String, Vec<Sender<Ctx>>>,
    ready_to_start: bool

}

impl<Ctx: Send> Scheduler <Ctx>{
    
    pub fn new() -> Self {
        Scheduler { 
            nodes: HashMap::new(),
            nodes_inputs: HashMap::new(),
            nodes_outputs: HashMap::new(),
            nodes_senders: HashMap::new(),
            ready_to_start: false
        }
    }

    /// 一个node可以依赖前序多个节点的输入
    pub fn add_node(&mut self, last: bool, pre_name_idxs: &Vec<(&str, usize)>, mut node: Box<dyn nodes::TNode<Value = Ctx> + Sync>) -> Option<Vec<Receiver<Ctx>>>{
        let mut final_receivers = None;
        for (pre_name, pre_idx) in pre_name_idxs {
            
            let pre_dep = &self.nodes_outputs.get(*pre_name).unwrap()[*pre_idx];

            self.nodes_inputs.entry(node.name().to_string())
                .and_modify(|v| v.push(pre_dep.clone()))
                .or_insert(vec![pre_dep.clone()]);
        }
        self.nodes_inputs.entry(node.name().to_string()).or_insert(vec![]);

        let mut senders = vec![];
        let mut receivers = vec![];

        for cap in node.channel_caps() {
            let (s, r) = crossbeam::channel::bounded(*cap);
            senders.push(s);
            receivers.push(r);
        }

        self.nodes_senders.insert(node.name().to_string(), senders);
        self.nodes_outputs.insert(node.name().to_string(), receivers);

        if last {
            final_receivers = Some(self.nodes_outputs.get(node.name()).unwrap()
                .iter()
                .map(|v| v.clone())
                .collect::<Vec<Receiver<Ctx>>>());
        }

        self.nodes.insert(node.name().to_string(), node);

        final_receivers
    }

    pub fn seal(&mut self) {
        self.ready_to_start = true;
        // self.nodes_inputs.clear();
        // self.nodes_outputs.clear();
        // self.nodes_senders.clear();
    }

    pub fn start(&mut self) {
        assert!(self.ready_to_start, "call seal() first");

        thread::scope(|s| {

            for (_, node) in & self.nodes {

                let senders = self.nodes_senders.get(node.name()).unwrap();
                let inputs = self.nodes_inputs.get(node.name()).unwrap();
                for _ in 0..node.num_workers() {
                    let senders = senders.iter()
                        .map(|v| v.clone())
                        .collect::<Vec<Sender<Ctx>>>();
                    let inputs = inputs.iter()
                        .map(|v| v.clone())
                        .collect::<Vec<Receiver<Ctx>>>();
                    s.spawn(move || {
                        node.work(inputs, senders);
                    });
                }
                self.nodes_senders.get_mut(node.name()).unwrap().clear();
                self.nodes_outputs.get_mut(node.name()).unwrap().clear();
                self.nodes_inputs.get_mut(node.name()).unwrap().clear();
        }

        })

    }

}