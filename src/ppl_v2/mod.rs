use crossbeam::channel::{Receiver, Sender};
use std::{
    cell::Cell,
    marker::PhantomData,
    thread::{self, JoinHandle},
};

pub struct Source(PhantomData<*const ()>);
pub struct Sink(PhantomData<*const ()>);

pub struct Pipeline<T> {
    receiver: Option<Receiver<T>>, // receiver of last worker
    join_handlers: Cell<Vec<JoinHandle<()>>>,
}

impl Pipeline<Source> {
    pub fn new() -> Self {
        Self {
            receiver: None,
            join_handlers: Cell::new(vec![]),
        }
    }

    pub fn add_init_stage<S>(
        self,
        name: &str,
        threads: usize,
        handler: impl FnOnce(Sender<S>) + Clone + Send + 'static,
        cap: Option<usize>,
    ) -> Pipeline<S>
    where
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) =
            crossbeam::channel::bounded(cap.expect("cap can't be None when is_sink==false"));
        for idx in 0..threads {
            let handler_ = handler.clone();
            let send_ = next_send.clone();
            let tname = format!("{}_{}", name, idx);
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    handler_(send_);
                })
                .unwrap();
            handlers.push(join_handler);
        }

        Pipeline {
            receiver: Some(next_recv),
            join_handlers: Cell::new(handlers),
        }
    }
}

impl<T> Pipeline<T>
where
    T: Send + 'static,
{
    pub fn add_stage<S>(
        self,
        name: &str,
        threads: usize,
        handler: impl FnOnce(Receiver<T>, Sender<S>) + Clone + Send + 'static,
        cap: Option<usize>,
    ) -> Pipeline<S>
    where
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) =
            crossbeam::channel::bounded(cap.expect("cap can't be None when is_sink==false"));

        for idx in 0..threads {
            let handler_ = handler.clone();
            let recv_ = self.receiver.clone();
            let send_ = next_send.clone();
            let tname = format!("{}_{}", name, idx);
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    handler_(recv_.unwrap(), send_);
                })
                .unwrap();
            handlers.push(join_handler);
        }

        Pipeline {
            receiver: Some(next_recv),
            join_handlers: Cell::new(handlers),
        }
    }

    pub fn add_sink_stage(
        self,
        name: &str,
        threads: usize,
        handler: impl FnOnce(Receiver<T>) + Clone + Send + 'static,
    ) -> Pipeline<Sink> {
        let mut handlers = self.join_handlers.take();

        for idx in 0..threads {
            let handler_ = handler.clone();
            let recv_ = self.receiver.clone();
            let tname = format!("{}_{}", name, idx);
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    handler_(recv_.unwrap());
                })
                .unwrap();
            handlers.push(join_handler);
        }

        Pipeline {
            receiver: None,
            join_handlers: Cell::new(handlers),
        }
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<T>> {
        self.receiver.take()
    }
}

impl<T> Drop for Pipeline<T>
where
    T: Sized,
{
    fn drop(&mut self) {
        self.join_handlers.take().into_iter().for_each(|handler| {
            handler.join().unwrap();
        });
    }
}

#[cfg(test)]
mod test {

    use crossbeam::channel::{Receiver, Sender};

    use super::Pipeline;

    #[test]
    fn test_pipeline() {
        let ppl = Pipeline::new();
        let ppl = ppl.add_init_stage(
            "source",
            4,
            move |s: Sender<i32>| {
                s.send(100).unwrap();
            },
            Some(10),
        );

        let ppl = ppl.add_stage(
            "Multiply",
            2,
            move |r: Receiver<i32>, s: Sender<i32>| {
                for v in r {
                    let _ = s.send(v * 10);
                }
            },
            Some(10),
        );

        let _ppl = ppl.add_sink_stage("sink", 1, move |r: Receiver<i32>| {
            let mut sum = 0;
            for v in r {
                sum += v;
            }
            println!("sum_result: {}", sum);
        });
    }
}
