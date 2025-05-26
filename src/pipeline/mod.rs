use crossbeam::channel::{Receiver, Sender};
use std::{
    cell::Cell,
    marker::PhantomData,
    thread::{self, JoinHandle},
};

pub struct Source(PhantomData<*const ()>);
pub struct Sink(PhantomData<*const ()>);

pub trait TSourceWork: Clone + Send + 'static {
    type SendType;

    fn process(&self) -> Option<Self::SendType>;
}

pub trait TSinkWork: Clone + Send + 'static {
    type RecvType;
    fn process(&self, recv: Self::RecvType);
}

pub trait TIntermediateWork: Clone + Send + 'static {
    type RecvType;
    type SendType;
    fn process(&self, recv: Self::RecvType) -> Self::SendType;
}

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

    pub fn add_source_stage<S>(
        self,
        name: &str,
        threads: usize,
        handler: impl FnOnce(Sender<S>) + Clone + Send + 'static,
        cap: usize,
    ) -> Pipeline<S>
    where
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) = crossbeam::channel::bounded(cap);
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
    pub fn add_source_work_stage<S, H>(
        self,
        name: &str,
        threads: usize,
        handler: H,
        cap: usize,
    ) -> Pipeline<S>
    where
        H: TSourceWork<SendType = S>,
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) = crossbeam::channel::bounded(cap);
        for idx in 0..threads {
            let send_ = next_send.clone();
            let tname = format!("{}_{}", name, idx);
            let handler_ = handler.clone();
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    while let Some(v) = handler_.process() {
                        send_.send(v).unwrap();
                    }
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
        cap: usize,
    ) -> Pipeline<S>
    where
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) = crossbeam::channel::bounded(cap);

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

    pub fn add_work_stage<S, H>(
        self,
        name: &str,
        threads: usize,
        handler: H,
        cap: usize,
    ) -> Pipeline<S>
    where
        H: TIntermediateWork<RecvType = T, SendType = S>,
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) = crossbeam::channel::bounded(cap);

        for idx in 0..threads {
            let handler_ = handler.clone();
            let recv_ = self.receiver.clone();
            let send_ = next_send.clone();
            let tname = format!("{}_{}", name, idx);
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    recv_.unwrap().iter().for_each(|v| {
                        let send_v = handler_.process(v);
                        send_.send(send_v).unwrap();
                    });
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

    pub fn add_sink_work_stage<H>(self, name: &str, threads: usize, handler: H) -> Pipeline<Sink>
    where
        H: TSinkWork<RecvType = T>,
    {
        let mut handlers = self.join_handlers.take();

        for idx in 0..threads {
            let handler_ = handler.clone();
            let recv_ = self.receiver.clone();
            let tname = format!("{}_{}", name, idx);
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    recv_.unwrap().iter().for_each(|v| {
                        handler_.process(v);
                    });
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

    use std::sync::{Arc, Mutex};

    use crossbeam::channel::{Receiver, Sender};

    use super::{Pipeline, TIntermediateWork, TSinkWork, TSourceWork};

    #[test]
    fn test_pipeline() {
        let ppl = Pipeline::new();
        let ppl = ppl.add_source_stage(
            "source",
            4,
            move |s: Sender<i32>| {
                s.send(100).unwrap();
            },
            10,
        );

        let ppl = ppl.add_stage(
            "Multiply",
            2,
            move |r: Receiver<i32>, s: Sender<i32>| {
                for v in r {
                    let _ = s.send(v * 10);
                }
            },
            10,
        );

        let _ppl = ppl.add_sink_stage("sink", 1, move |r: Receiver<i32>| {
            let mut sum = 0;
            for v in r {
                sum += v;
            }
            println!("sum_result: {}", sum);
        });
    }

    #[derive(Clone)]
    pub struct SourceWork(pub Arc<Mutex<Vec<i32>>>);
    impl TSourceWork for SourceWork {
        type SendType = i32;

        fn process(&self) -> Option<Self::SendType> {
            if self.0.lock().unwrap().is_empty() {
                return None;
            }
            let v = self.0.lock().unwrap().pop();
            if v.is_none() {
                return None;
            }
            return Some(v.unwrap());
        }
    }

    #[derive(Debug, Clone)]
    pub struct IntermidiateWork;
    impl TIntermediateWork for IntermidiateWork {
        type RecvType = i32;
        type SendType = i32;

        fn process(&self, r: i32) -> Self::SendType {
            r * 10
        }
    }

    #[derive(Debug, Clone)]
    pub struct SinkWork;
    impl TSinkWork for SinkWork {
        type RecvType = i32;

        fn process(&self, r: i32) {
            println!("sink: {}", r);
        }
    }
    #[test]
    fn test_pipeline_with_work() {
        let ppl = Pipeline::new();
        let ppl = ppl.add_source_work_stage(
            "source",
            4,
            SourceWork(Arc::new(Mutex::new(vec![1, 2, 3, 4]))),
            10,
        );
        let ppl = ppl.add_work_stage("Multiply", 2, IntermidiateWork, 10);
        let _ppl = ppl.add_sink_work_stage("sink", 1, SinkWork);
    }
}
