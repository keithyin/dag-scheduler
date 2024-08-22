use std::thread::JoinHandle;

use super::ppl_node::{NodeType, TPplNode};
use crossbeam::channel::{self, Sender, Receiver};


pub struct PplScheduler<Ctx>{
    // use Option. because one can take the inner behind the &mut self.
    nodes: Option<Vec<Vec<Box<dyn TPplNode<MsgType=Ctx> + Sync>>>>,
    send_recvs: Vec<(Option<Sender<Ctx>>, Option<Receiver<Ctx>>)>,
    node_types: Vec<NodeType>,
    prev_node_type: Option<NodeType>,
}

impl <Ctx: Send + 'static > PplScheduler<Ctx> {
    pub fn new() -> Self {
        Self { 
            nodes: Some(vec![]), 
            send_recvs: vec![], 
            node_types: vec![], 
            prev_node_type: None
        }
    }

    pub fn add_single_worker_nodes(&mut self, node_type: NodeType, nodes: Vec<Box<dyn TPplNode<MsgType = Ctx> + Sync>>) {
        match node_type {
            NodeType::Source(cap) => {
                assert!(self.nodes.as_ref().unwrap().len() == 0, "only the first node can be source");
                self.nodes.as_mut().unwrap().push(nodes);
                let (send, recv) = channel::bounded::<Ctx>(cap);
                self.send_recvs.push((Some(send), Some(recv)));
            },

            NodeType::Middle(cap) => {
                if self.prev_node_type.is_some() {
                    match self.prev_node_type.as_ref().unwrap() {
                        NodeType::Sink => panic!("the precedent node of the MiddleNode can't be Sink"),
                        _ => 1,
                    };
                }
                
                self.nodes.as_mut().unwrap().push(nodes);
                let (send, recv) = channel::bounded::<Ctx>(cap);
                self.send_recvs.push((Some(send), Some(recv)));
            },

            NodeType::Sink => {
                assert!(self.nodes.as_ref().unwrap().len() > 0, "sink node need a prev node");
                self.nodes.as_mut().unwrap().push(nodes);
                self.send_recvs.push((None, None));

            }
        };

        self.prev_node_type = Some(node_type);
        self.node_types.push(node_type);
    }


    /// start the pipeline. return the JoinHandle and last_recv
    /// 1. if no Source node at the begining of the pipeline. first_recv.is_some() == true
    /// 2. if no sink node at the end of the pipeline. last_recv.is_some() == true
    pub fn start(&mut self, first_recv: Option<Receiver<Ctx>>) -> (Vec<Vec<JoinHandle<()>>>, Option<Receiver<Ctx>>){

        let handles = self.nodes.take().unwrap()
        .into_iter()
        .enumerate()
        .map(|(idx, single_work_nodes)| {
            match &self.node_types[idx] {
                NodeType::Source(_) => {
                    let sender = self.send_recvs[idx].0.clone();
                    single_work_nodes.into_iter()
                        .map(|node| node.start(None, sender.clone()))
                        .collect::<Vec<JoinHandle<()>>>()

                },
                NodeType::Middle(_) => {
                    let recv  = if idx > 0 {
                        self.send_recvs[idx - 1].1.clone()
                    } else {
                        assert!(first_recv.is_some());
                        first_recv.clone()
                    };
                    let sender = self.send_recvs[idx].0.clone();
                    single_work_nodes.into_iter()
                        .map(|node| node.start(recv.clone(), sender.clone()))
                        .collect::<Vec<JoinHandle<()>>>()

                }, 
                NodeType::Sink => {
                    let recv  = if idx > 0 {
                        self.send_recvs[idx - 1].1.clone()
                    } else {
                        assert!(first_recv.is_some());
                        first_recv.clone()
                    };

                    single_work_nodes.into_iter()
                        .map(|node| node.start(recv.clone(), None))
                        .collect::<Vec<JoinHandle<()>>>()
                }
            }
        }).collect::<Vec<_>>();
        let last_recv = match self.node_types.last().unwrap() {
            NodeType::Sink => None, 
            _ => self.send_recvs.last().unwrap().1.clone()
        };
        self.send_recvs = vec![];
        (handles, last_recv)

    }
    
}


#[cfg(test)]
mod test {
    use crate::pipeline_v2::ppl_node::{MiddleNode, NodeType, SinkNode, SourceNode};

    use super::PplScheduler;


    #[test]
    fn test_pipeline() {

        let source_nodes = SourceNode::new(2);
        let middle_nodes = MiddleNode::new(3);
        let sink_nodes = SinkNode::new(4);
        let mut pipeline = PplScheduler::new();
        pipeline.add_single_worker_nodes(NodeType::Source(2), source_nodes);

    }
}