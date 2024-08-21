use super::ppl_node::{NodeType, TPplNode};
use crossbeam::channel::{self, Sender, Receiver};


pub struct PplScheduler<Ctx>{
    nodes: Vec<Vec<Box<dyn TPplNode<MsgType=Ctx> + Sync>>>,
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

    pub fn add_single_worker_nodes(&mut self, node_type: NodeType, nodes: Vec<Box<dyn TPplNode<MsgType = Ctx> + Sync>>) {
        match node_type {
            NodeType::Source => {
                assert!(self.nodes.len() == 0, "only the first node can be source");
                self.nodes.push(nodes);
                self.send_recvs.push((None, None));
            },

            NodeType::Middle(cap) => {
                if self.prev_node_type.is_some() {
                    match self.prev_node_type.as_ref().unwrap() {
                        NodeType::Sink(_) => panic!("the precedent node of the MiddleNode can't be Sink"),
                        _ => 1,
                    };
                }
                
                self.nodes.push(nodes);
                let (send, recv) = channel::bounded::<Ctx>(cap);
                self.send_recvs.push((Some(send), Some(recv)));
            },

            NodeType::Sink(cap) => {
                assert!(self.nodes.len() > 0, "sink node need a prev node");
                self.nodes.push(nodes);
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
    pub fn start(&mut self) {
        match self.node_types.first().unwrap() {
            NodeType::Source => 1,
            _ => panic!("first node in the pipeline must be Source"),
        };
        match self.node_types.last().unwrap() {
            NodeType::Sink(_) => 1,
            _ => panic!("last node in the pipeline must be Sink"),
        };

        assert_eq!(self.nodes.len(), self.node_types.len());
        assert_eq!(self.nodes.len(), self.send_recvs.len());

        self.nodes.into_iter().enumerate().map(|(idx, single_work_nodes)| {
            
        });

    }
    
}

