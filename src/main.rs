#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![feature(test)]
//#![feature(core_intrinsics)]
//#![feature(vec_into_raw_parts)]

mod beggar_pool;
mod stealer_pool;
mod sudoku;
mod helpers;
mod worker;
mod pool;

use std::time::Instant;
use log::{error, info, LevelFilter};

use sudoku::Sudoku;
use helpers::{
    IntoError,
    Void,
    Res
};
use pool::Pool;

fn main() -> Void {
    let args: Vec<String> = std::env::args().collect();

    // Set the log level.
    simple_logger::init().unwrap();
    log::set_max_level(LevelFilter::Info);

    let sudoku = parse_sudoku_from_args(&args)?;
    info!("Entered ...\n\n{}", sudoku);

    let pool = Pool::new();
    
    pool.start(&sudoku);

    let start = Instant::now();
    let done_sudoku = pool.await_result()?;
    let elapsed = start.elapsed().as_millis();

    // Print!

    info!("Done!\n\n{}", done_sudoku);
    info!("Finished in {} ms using {} operations.", elapsed, pool.get_total_ops());

    Ok(())
}

fn parse_sudoku_from_args(args: &[String]) -> Res<Sudoku> {
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
    Ok(Sudoku::from_str(&sudoku_text))
}


#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_hard_solve(b: &mut Bencher) -> Void {
        b.iter(|| {
            let sudoku = parse_sudoku_from_args(&["dummy".to_owned(), "hard.txt".to_owned()])?;
            let pool = Pool::new();

            pool.start(&sudoku);
            pool.await_result()?;

            Ok(()) as Void
        });

        Ok(())
    }
}