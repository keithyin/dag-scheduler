use super::ppl_node::TPplNode;
use crossbeam::channel::{self, Sender, Receiver};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum NodeType {
    // usize: num_threads, usize: channel_capacity
    Source(usize), // source node don't need receiver
    // middle node need a receiver for receiving the output the previous node, 
    // and need a sender for sending the result of current node
    // usize for store the capacity of the corresponding channel
    Middle((usize, usize)), 

    Sink((usize, usize)), // sink node don't need sender

}

pub struct PplScheduler<Ctx>{
    nodes: Vec<Box<dyn TPplNode<Value=Ctx> + Sync>>,
    send_recvs: Vec<(Option<Sender<Ctx>>, Option<Receiver<Ctx>>)>,
    node_types: Vec<NodeType>,
    prev_node_type: Option<NodeType>,
}

impl <Ctx: Send> PplScheduler<Ctx> {
    pub fn new() -> Self {
        Self { 
            nodes: vec![], 
            send_recvs: vec![], 
            node_types: vec![], 
            prev_node_type: None
        }
    }

    pub fn add_node(&mut self, node_type: NodeType, node: Box<dyn TPplNode<Value = Ctx> + Sync>) {
        match node_type {
            NodeType::Source(_) => {
                
                assert!(self.nodes.len() == 0, "only the first node can be source");
                self.nodes.push(node);
                self.send_recvs.push((None, None));
            },

            NodeType::Middle((_, cap)) => {
                if self.prev_node_type.is_some() {
                    match self.prev_node_type.as_ref().unwrap() {
                        NodeType::Sink(_) => panic!("the precedent node of the MiddleNode can't be Sink"),
                        _ => 1,
                    };
                }
                
                self.nodes.push(node);
                let (send, recv) = channel::bounded::<Ctx>(cap);
                self.send_recvs.push((Some(send), Some(recv)));
            },

            NodeType::Sink((_, cap)) => {
                assert!(self.nodes.len() > 0, "sink node need a prev node");
                self.nodes.push(node);
                let (send, recv) = channel::bounded::<Ctx>(cap);
                self.send_recvs.push((Some(send), Some(recv)));

            }
        };

        self.prev_node_type = Some(node_type);
        self.node_types.push(node_type);
    }

    // if the last node is not a sink node, this func need to be run!!
    // pub fn add_final_sender(&mut self, final_sender: Sender<Ctx>) {
    //     assert!(self.nodes.len() > 0);
    //     match self.prev_node_type.as_ref().unwrap() {
    //         NodeType::Sink(_) => panic!("the last node of the pipeline is Sink, no need to call this func"),
    //         _ => 1,
    //     };
    //     self.send_recvs.push((Some(final_sender), None));
    // }

    // provide 
    pub fn run(&mut self) {
        match self.node_types.first().unwrap() {
            NodeType::Source(_) => 1,
            _ => panic!("first node in the pipeline must be Source"),
        };
        match self.node_types.last().unwrap() {
            NodeType::Sink(_) => 1,
            _ => panic!("last node in the pipeline must be Sink"),
        };

        thread::scope(|thread_scope| {

            self.nodes.iter().zip(self.node_types.iter()).enumerate().for_each(|(idx, (node, node_type))|{

                match node_type {
                    NodeType::Source(num_threads) => {
                        let sender = self.send_recvs[idx+1].0.take().unwrap();
                        for thread_idx in 0..*num_threads {
                            let send_ = sender.clone();
                            thread_scope.spawn(move|| node.work_source(thread_idx, send_));
                        }
                    },
    
                    NodeType::Middle((num_threads, _)) => {
                        let recv = self.send_recvs[idx].1.take().unwrap();
                        let sender = self.send_recvs[idx+1].0.take().unwrap();
                        for thread_idx in 0..*num_threads {
                            let recv_ = recv.clone();
                            let sender_ = sender.clone();
                            thread_scope.spawn(move || node.work_middle(thread_idx, recv_, sender_));
                        }
                    },
    
                    NodeType::Sink((num_threads, _)) => {
                        let recv = self.send_recvs[idx].1.take().unwrap();
                        for thread_idx in 0..*num_threads {
                            let recv_ = recv.clone();
                            thread_scope.spawn(move || node.work_sink(thread_idx, recv_));
                        }
                        
                    }     
                };
  
            })  
        });
    }
    


}

