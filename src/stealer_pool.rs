use std::sync::{
    Arc, 
    atomic::{
        Ordering, 
        AtomicUsize
    }
};
use crossbeam::{
    queue::ArrayQueue,
    channel::{
        self,
        Sender,
        Receiver
    }
};

#[derive(PartialEq, Debug)]
pub enum DonationResult {
    Donated,
    NotDonated
}

#[derive(Clone)]
pub struct StealerPool<T>
where
    T: Clone
{
    num_workers: usize,
    workers_registered: Arc<AtomicUsize>,
    work_queues: Arc<Vec<ArrayQueue<T>>>
}

impl<T> StealerPool<T>
where
    T: Clone
{
    pub fn new(num_workers: usize) -> Self
    {
        let mut work_queues_vec = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            work_queues_vec.push(ArrayQueue::new(1024));
        } 

        Self { num_workers, workers_registered: Arc::new(AtomicUsize::new(0)), work_queues: Arc::new(work_queues_vec) }
    }

    pub fn register(&mut self) -> usize {
        self.workers_registered.fetch_add(1, Ordering::Relaxed)
    }

    pub fn steal_work(&self, id: usize) -> Option<T> {
        for k in 0..self.num_workers {
            if k == id {
                continue;
            }

            if let Ok(work) = self.work_queues[k].pop() {
                return Some(work);
            }
        }

        return None;
    }

    pub fn is_ready(&self) -> bool {
        self.num_workers == self.workers_registered.load(Ordering::Relaxed)
    }
}