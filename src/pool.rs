use crossbeam::channel;
use std::{sync::{atomic::{Ordering, AtomicU64}, Arc}};

use crate::{
    beggar_pool::BeggarPool, 
    sudoku::Sudoku, 
    worker, 
    helpers::Res
};

pub struct Pool<T>
    where T: Clone
{
    beggar_pool: BeggarPool<T>,
    success_rx: channel::Receiver<T>,
    total_ops: Arc<AtomicU64>
}

impl Pool<Sudoku> {
    pub fn new() -> Self {
        let num_cores = core_affinity::get_core_ids().unwrap().len();
        let beggar_pool = BeggarPool::<Sudoku>::new(num_cores);
        let (success_tx, success_rx) = channel::bounded::<Sudoku>(1);
        let total_ops = Arc::new(AtomicU64::new(0));

        // Spawn the workers.
        for _ in 0..num_cores {
            worker::start(&total_ops, &beggar_pool, &success_tx);
        }

        while !beggar_pool.is_ready() {}

        Pool { beggar_pool, success_rx, total_ops }
    }

    pub fn start(&self, sudoku: &Sudoku) {
        self.beggar_pool.donate_work(sudoku);
    }

    pub fn get_total_ops(&self) -> u64 {
        self.total_ops.fetch_or(0, Ordering::Relaxed)
    }

    pub fn await_result(&self) -> Res<Sudoku> {
        Ok(self.success_rx.recv()?)
    }
}