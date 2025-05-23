use crossbeam::channel::{Receiver, Sender};
use std::{
    cell::Cell,
    marker::PhantomData,
    thread::{self, JoinHandle},
};

pub struct Source(PhantomData<*const ()>);
pub struct Sink(PhantomData<*const ()>);

pub struct Pipeline<T> {
    receiver: Option<Receiver<T>>,
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
        handler: impl FnOnce(Option<Sender<S>>) + Clone + Send + 'static,
        cap: Option<usize>,
    ) -> Pipeline<S>
    where
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) = {
            let (s, r) =
                crossbeam::channel::bounded(cap.expect("cap can't be None when is_sink==false"));
            (Some(s), Some(r))
        };
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
            receiver: next_recv,
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
        handler: impl FnOnce(Option<Receiver<T>>, Option<Sender<S>>) + Clone + Send + 'static,
        is_sink: bool,
        cap: Option<usize>,
    ) -> Pipeline<S>
    where
        S: Send + 'static,
    {
        let mut handlers = self.join_handlers.take();

        let (next_send, next_recv) = if is_sink {
            (None, None)
        } else {
            let (s, r) =
                crossbeam::channel::bounded(cap.expect("cap can't be None when is_sink==false"));
            (Some(s), Some(r))
        };
        for idx in 0..threads {
            let handler_ = handler.clone();
            let recv_ = self.receiver.clone();
            let send_ = next_send.clone();
            let tname = format!("{}_{}", name, idx);
            // println!("{}", tname);
            let join_handler = thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    handler_(recv_, send_);
                })
                .unwrap();
            handlers.push(join_handler);
        }

        Pipeline {
            receiver: next_recv,
            join_handlers: Cell::new(handlers),
        }
    }

    pub fn add_sink_stage(
        self,
        name: &str,
        threads: usize,
        handler: impl FnOnce(Option<Receiver<T>>) + Clone + Send + 'static,
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
                    handler_(recv_);
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

    use super::{Pipeline, Sink, Source};

    #[test]
    fn test_pipeline() {
        let ppl = Pipeline::new();
        let ppl = ppl.add_init_stage(
            "source",
            4,
            move |s: Option<Sender<i32>>| {
                s.unwrap().send(100).unwrap();
            },
            Some(10),
        );

        let _ppl = ppl.add_sink_stage("sink", 1, move |r: Option<Receiver<i32>>| {
            let mut sum = 0;
            for v in r.unwrap() {
                sum += v;
            }
            println!("sum_result: {}", sum);
        });
    }
}
