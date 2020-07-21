use crossbeam::channel::Sender;
use log::info;
use std::sync::{atomic::{Ordering, AtomicU64}, Arc};
use core_affinity;

use crate::beggar_pool::{
    BeggarPool,
    DonationResult
};
use crate::sudoku::{
    Sudoku,
    SetResult
};

const EPOCH_SIZE: u64 = 10;

pub fn start(to: &Arc<AtomicU64>, bp: &BeggarPool<Sudoku>, stx: &Sender<Sudoku>) {
    let total_ops = to.clone();
    let mut beggar_pool = bp.clone();
    let success_tx = stx.clone();

    let _ = std::thread::spawn(move || {
        // Register with the pool.
        let id = beggar_pool.register();
        //log::info!("{} Registered.", id);

        core_affinity::set_for_current(core_affinity::CoreId { id });

        

        // Outer loop asks for work.
        while let Some(mut sudoku) = beggar_pool.beg_work(id) {
            //info!("[{}] Got work.", id);
            // Start at 0 every time, and walk through the sudoku, if needed.
            test_position(id, &total_ops, &mut 0, &mut sudoku, &beggar_pool, 0, &success_tx);
        }
    });
}

// This is a workaround for the fact that recursive async functions in Rust require a `BoxFuture`.
// https://rust-lang.github.io/async-book/07_workarounds/05_recursion.html
pub fn test_position<'a>(
    id: usize,
    total_ops: &'a Arc<AtomicU64>,
    counter: &'a mut u64, 
    sudoku: &'a mut Sudoku, 
    beggar_pool: &'a BeggarPool<Sudoku>, 
    pos: usize, 
    success_tx: &'a Sender<Sudoku>
) {
    // Increment counters.
    *counter += 1;
    total_ops.fetch_add(1, Ordering::Relaxed);

    // Check for success criteria.
    if pos == 81 {
        info!("[{}] Found the solution!", id);
        let _ = success_tx.clone().send(sudoku.clone());
        return;
    }

    // If there is already a number in this square, then continue.
    if let Some(_) = *sudoku.get_at_ref(pos) {
        return test_position(id, total_ops, counter, sudoku, beggar_pool, pos + 1, success_tx);
    }

    // Iterate through the numbers in this position.
    for k in 1..10 {
        if sudoku.set_at(pos, k) == SetResult::Set {
            if *counter % EPOCH_SIZE != 1 || beggar_pool.donate_work(&sudoku) == DonationResult::NotDonated {
                //info!("[{}] Setting position {} to {}.", id, pos, k);
                test_position(id, total_ops, counter, sudoku, beggar_pool, pos + 1, success_tx);
            } else {
                //info!("[{}] Donated {} at position {}.", id, k, pos);
            }
        }
    }

    // If this path was not a success and was exhausted, set this position to none.
    sudoku.erase_at(pos);
}