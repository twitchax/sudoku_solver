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
pub struct BeggarPool<T>
where
    T: Clone
{
    num_workers: usize,
    workers_registered: Arc<AtomicUsize>,
    beggar_senders: Arc<Vec<Sender<T>>>,
    beggar_receivers: Arc<Vec<Receiver<T>>>,
    beggar_queue: Arc<ArrayQueue<usize>>
}

impl<T> BeggarPool<T>
where
    T: Clone
{
    pub fn new(num_workers: usize) -> Self
    {
        let mut beggar_senders = Vec::with_capacity(num_workers);
        let mut beggar_receivers = Vec::with_capacity(num_workers);
        let beggar_queue = Arc::new(ArrayQueue::new(num_workers));

        for _ in 0..num_workers {
            let (sender, receiver) = channel::bounded(1);

            beggar_senders.push(sender);
            beggar_receivers.push(receiver);
        }

        Self { num_workers, workers_registered: Arc::new(AtomicUsize::new(0)), beggar_senders: Arc::new(beggar_senders), beggar_receivers: Arc::new(beggar_receivers), beggar_queue }
    }

    pub fn register(&mut self) -> usize {
        self.workers_registered.fetch_add(1, Ordering::Relaxed)
    }

    pub fn beg_work(&self, id: usize) -> Option<T> {
        match self.beggar_queue.push(id) {
            Ok(_) => self.beggar_receivers[id].recv().ok(),
            _ => None
        }
    }

    pub fn donate_work(&self, work: &T) -> DonationResult {
        match self.beggar_queue.pop().map(|id| self.beggar_senders[id].send(work.clone())) {
            Ok(_) => DonationResult::Donated,
            _ => DonationResult::NotDonated
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.beggar_queue.is_empty()
    }
}