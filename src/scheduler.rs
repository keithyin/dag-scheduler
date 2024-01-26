use std::{collections::HashMap, thread};

use crate::nodes;

struct Scheduler <Comm>{

    nodes: HashMap<String, Box<dyn nodes::TNode<Comm = Comm>>>,
    ready_to_start: bool

}

impl<Comm> Scheduler <Comm>{
    
    pub fn new() -> Self {
        Scheduler { 
            nodes: HashMap::new(),
            ready_to_start: false
        }
    }

    /// 一个node可以依赖前序多个节点的输入
    pub fn add_node(&mut self, pre_name_idxs: &Vec<(&String, usize)>, mut node: Box<dyn nodes::TNode<Comm = Comm>>) {
        for (pre_name, pre_idx) in pre_name_idxs {
            let pre_node = self.nodes.get(*pre_name).unwrap();
            let pre_dep = &pre_node.outputs()[*pre_idx];
            node.add_input(pre_dep);
        }
    }

    pub fn seal(&mut self) {
        self.ready_to_start = true;
        for (_, node) in self.nodes.iter_mut() {
            node.outputs_mut().clear();
        }
    }

    pub fn start(&self) {
        assert!(self.ready_to_start, "call seal() first");


    }

}