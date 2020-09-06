#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
//#![feature(vec_into_raw_parts)]

mod beggar_pool;
mod sudoku;
mod helpers;
mod worker;

use log::{error, info, LevelFilter};
use crossbeam::channel;

use sudoku::Sudoku;
use beggar_pool::{
    BeggarPool,
    DonationResult
};
use helpers::{
    IntoError,
    Void
};
use std::{sync::{atomic::{Ordering, AtomicU64}, Arc}, time::Instant};

fn main() -> Void {
    let args: Vec<String> = std::env::args().collect();

    // Set the log level.
    simple_logger::init().unwrap();
    log::set_max_level(LevelFilter::Info);

    let sudoku_text = if args.len() == 2 {
        let sudoku_file = args[1].to_owned();
        let sudoku_file_data = std::fs::read(sudoku_file)?;
        
        std::str::from_utf8(&sudoku_file_data)?.to_owned()
    } else {
        let e = "A sudoku file must be passed in.";
        error!("{}", e);
        return e.into_error();
    };

    // Parse original sudoku.
    let sudoku = Sudoku::from_str(&sudoku_text);
    info!("Entered ...\n\n{}", sudoku);

    // Create a beggar pool.
    let num_cores = core_affinity::get_core_ids().unwrap().len();
    let beggar_pool = BeggarPool::<Sudoku>::new(num_cores);
    let (success_tx, success_rx) = channel::bounded::<Sudoku>(1);
    let total_ops = Arc::new(AtomicU64::new(0));

    // Spawn the workers.
    for _ in 0..num_cores {
        worker::start(&total_ops, &beggar_pool, &success_tx);
    }

    // Send the initial sudoku to the beggar pool.
    // Need to loop since work must be donated to a specific "ready" beggar.
    while beggar_pool.donate_work(&sudoku) == DonationResult::NotDonated {}

    let start = Instant::now();

    // Await success.
    let done_sudoku = success_rx.recv()?;

    // Print!

    info!("Done!\n\n{}", done_sudoku);
    info!("Finished in {} ms using {} operations.", start.elapsed().as_millis(), total_ops.fetch_or(0, Ordering::Relaxed));

    Ok(())
}