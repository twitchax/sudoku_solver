#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![feature(portable_simd)]
#![feature(test)]
//#![feature(core_intrinsics)]
//#![feature(vec_into_raw_parts)]

mod rayon_worker;
//mod beggar_pool;
mod sudoku;
mod helpers;
//mod worker;
//mod pool;

use std::time::Instant;
use log::{error, info, LevelFilter};

use sudoku::Sudoku;
use helpers::{
    IntoError,
    Void,
    Res
};

fn main() -> Void {
    let args: Vec<String> = std::env::args().collect();
    rayon::ThreadPoolBuilder::new().num_threads(32).build_global().unwrap();

    // Set the log level.
    //simple_logger::init().unwrap();
    //log::set_max_level(LevelFilter::Info);

    let sudoku = parse_sudoku_from_args(&args)?;
    println!("Entered ...\n\n{}", sudoku);

    let start = Instant::now();
    let (done_sudoku, total_ops) = rayon_worker::solve(sudoku);

    let elapsed = start.elapsed().as_micros();

    // Print!

    println!("Done!\n\n{}", done_sudoku);
    println!("Finished in {} μs using {} operations.", elapsed, total_ops);

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
            
            let _ = rayon_worker::solve(sudoku);

            Ok(()) as Void
        });

        Ok(())
    }
}