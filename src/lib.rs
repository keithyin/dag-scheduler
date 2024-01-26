pub mod scheduler;
pub mod nodes;


#[cfg(test)]
mod tests {
    use std::thread;

    use crossbeam::channel::{Receiver, Sender};

    use crate::nodes::CommonNode;

    use super::*;

    #[test]
    fn test_pipeline() {
        struct CtxV {
            pub a: i32
        } 

        fn init(_: Vec<Receiver<CtxV>>, outputs: Vec<Sender<CtxV>>) {
            for _ in 0..10000 {
                outputs[0].send(CtxV { a: 1 }).unwrap();
            }
        }

        fn add_one(inputs: Vec<Receiver<CtxV>>, outputs: Vec<Sender<CtxV>>)  {
            for mut ctx in inputs[0].iter() {
                ctx.a += 1;
                outputs[0].send(ctx).unwrap();
            }
        }

        fn add_two(inputs: Vec<Receiver<CtxV>>, outputs: Vec<Sender<CtxV>>)  {
            for mut ctx in inputs[0].iter() {
                ctx.a += 2;
                outputs[0].send(ctx).unwrap();
            }
        }

        let mut sche = scheduler::Scheduler::new();
        
        sche.add_node(false, &vec![], Box::new(CommonNode::new("first", 2, &[10], init)));
        sche.add_node(false, &vec![("first", 0)], Box::new(CommonNode::new("second", 2, &[10], add_one)));
        let outputs = sche.add_node(true, &vec![("second", 0)], Box::new(CommonNode::new("third", 2, &[10], add_two))).unwrap();

        let handler = thread::spawn(move|| {
            let mut result = vec![];
            for ctx in outputs[0].iter() {
                result.push(ctx.a);
            }
            return result;
        });

        sche.start();
        let result = handler.join().unwrap();
        println!("result:{result:?}");

    }
}
