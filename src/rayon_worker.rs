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
        test_position(&total_ops, &mut sudoku, s, &success_tx);
    });
    
    let result = success_rx.recv().unwrap();
    let ops = total_ops_clone.fetch_or(0, Ordering::SeqCst);

    (result, ops)
}

pub fn test_position<'s>(
    total_ops: &Arc<AtomicU64>,
    sudoku: &mut Sudoku, 
    scope: &Scope<'s>,
    success_tx: &Sender<Sudoku>
) {
    // Increment counter.
    total_ops.fetch_add(1, Ordering::Relaxed);

    // Check for success criteria.
    // The sudoku positions are _always_ sorted in most constrained to
    // least constrained order.  If the 0th position is set, then we're done!
    if sudoku.has_value(0) {
        info!("Found the solution!");
        let _ = success_tx.clone().send(sudoku.clone());
        return;
    }

    // Iterate through the numbers in this position.
    let can_place = sudoku.can_place_what(0);

    for num in can_place[1..10].iter().filter(|n| **n != 0) {
        let total_ops_clone = total_ops.clone();
        let mut sudoku_clone = sudoku.clone();
        let success_tx_clone = success_tx.clone();

        sudoku_clone.unchecked_set(0, *num);
        sudoku_clone.update_constraint_order();

        scope.spawn(move |s1| {
            // Always start at position 0 since cloning the sudoku to the desired order.
            test_position(&total_ops_clone, &mut sudoku_clone, s1, &success_tx_clone);
        });
    }
}