/// Compare the speed of the naive `shift_left` and `shift_right` implementations to the ones we implemented for
/// `BitVec` Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
mod naive;

use std::io::Write;
use utilities_rs::Stopwatch;

fn main() {
    type BV = BitVec<u8>;

    // Number of trials to run & how often to print progress.
    let n_trials = 1_000;
    let n_tick = n_trials / 20;

    // Length of the bit-vector to test & a shift amount.
    let len = 1_000_000;
    let shift = len / 2;

    // Space for the results which we will check match at the end.
    let mut sh_optimized = BV::ones(len);
    let mut sh_naive = BV::ones(len);

    // Set up a stopwatch.
    let sw = Stopwatch::default();

    // Run the trials for the optimized implementation.
    print!("Running {n_trials} trials of the optimized shift ");
    let mut dt_optimized = sw.elapsed();
    for n in 0..n_trials {
        sh_optimized = &sh_optimized >> shift;
        sh_optimized = &sh_optimized << shift;
        if n % n_tick == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
    println!(" done!");
    dt_optimized = sw.elapsed() - dt_optimized;

    // Run the trials for the naive implementation.
    print!("Running {n_trials} trials of the naive shift ");
    let mut dt_naive = sw.elapsed();
    for n in 0..n_trials {
        sh_naive = naive::shift_right(&sh_naive, shift);
        sh_naive = naive::shift_left(&sh_naive, shift);
        if n % n_tick == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
    println!(" done!");
    dt_naive = sw.elapsed() - dt_naive;

    // Check the two loops end up at the same (pointless) place ...
    assert_eq!(sh_naive, sh_optimized, "Oops the loops reached different outcomes!");

    // Print the timing results.
    println!();
    println!("Optimized shifts: {}", Stopwatch::format_seconds(dt_optimized));
    println!("Naive shifts:     {}", Stopwatch::format_seconds(dt_naive));
    println!("Speed-up factor:  {:.0}x", dt_naive / dt_optimized);
}
