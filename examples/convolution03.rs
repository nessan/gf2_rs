/// Convolution test for profiling & benchmarking.
/// Run in release mode for realistic timings.
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

    // Randomly generate two bit-vectors that we will convolve.
    let lhs_len = 5_000;
    let rhs_len = 5_000;
    let lhs: BV = BV::random(lhs_len);
    let rhs: BV = BV::random(rhs_len);

    // Set up a stopwatch.
    let sw = Stopwatch::default();

    // Do the convolutions ...
    print!("Running {n_trials} trials of the optimized convolution ");
    let mut dt_optimized = sw.elapsed();
    for n in 0..n_trials {
        let _ = lhs.convolved_with(&rhs);
        if n % n_tick == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
    println!(" done!");
    dt_optimized = sw.elapsed() - dt_optimized;

    // Print the timing results.
    println!();
    println!("Convolution loop time: {}", Stopwatch::format_seconds(dt_optimized));
}
