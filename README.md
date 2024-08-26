a scheduler of dag computation graph.

* dag scheduler and nodes. each node of a dag graph will launch multi threads for computation.
  * src/scheduler.rs
  * scr/nodes.rs
* pipline scheduler and nodes. num of nodes means num of threads in pipeline
  * pipeline_v2/ppl_scheduler.rs
  * pipeline_v2/ppl_node.rs

pipeline_scheduler

 ```rust

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

let source_nodes = SourceNode::new(5);
let middle_nodes = MiddleNode::new(3);
let sink_nodes = SinkNode::new(4);
let mut pipeline = PplScheduler::new();
pipeline.add_single_worker_nodes(NodeType::Source(2), source_nodes);
pipeline.add_single_worker_nodes(NodeType::Middle(2), middle_nodes);
pipeline.add_single_worker_nodes(NodeType::Sink, sink_nodes);
join_all_nodes_handles(pipeline.start(None).0);

 ```