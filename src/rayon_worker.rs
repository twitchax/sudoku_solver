use crossbeam::channel::Sender;
use log::info;
use rayon::Scope;
use std::sync::{atomic::{Ordering, AtomicU64}, Arc};
use crate::sudoku::{
    Sudoku,
    SetResult
};
use crossbeam::channel;

pub fn solve(mut sudoku: Sudoku) -> (Sudoku, u64) {
    let total_ops = Arc::new(AtomicU64::new(0));
    let (success_tx, success_rx) = channel::bounded::<Sudoku>(1);
    let total_ops_clone = total_ops.clone();
    
    rayon::scope(move |s: &Scope<'_>| {
        test_position(&total_ops, &mut sudoku, s, 0, &success_tx);
    });
    
    let result = success_rx.recv().unwrap();
    let ops = total_ops_clone.fetch_or(0, Ordering::SeqCst);

    (result, ops)
}

pub fn test_position<'s>(
    total_ops: &Arc<AtomicU64>,
    sudoku: &mut Sudoku, 
    scope: &Scope<'s>, 
    pos: usize, 
    success_tx: &Sender<Sudoku>
) {
    // Increment counter.
    total_ops.fetch_add(1, Ordering::Relaxed);

    // Check for success criteria.
    if pos == 81 {
        info!("Found the solution!");
        let _ = success_tx.clone().send(sudoku.clone());
        return;
    }

    // If there is already a number in this square, then continue.
    if sudoku.has_value(pos) {
        return test_position(total_ops, sudoku, scope, pos + 1, success_tx);
    }

    // Iterate through the numbers in this position.
    for k in 1..10 {
        if sudoku.set(pos, k) == SetResult::Set {
            let total_ops_clone = total_ops.clone();
            let mut sudoku_clone = sudoku.clone();
            let success_tx_clone = success_tx.clone();

            scope.spawn(move |s1| {
                // Always start at position 0 since cloning the sudoku resorts the desired order.
                test_position(&total_ops_clone, &mut sudoku_clone, s1, 0, &success_tx_clone);
            });
        }
    }
}