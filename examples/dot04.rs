/// Dot products M.N for profiling & benchmarking.
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use std::io::{
    self,
    Write,
};
use utilities_rs::{
    Pretty,
    Stopwatch,
};

fn main() {
    // Number of trials & progress ticks.
    let n_trials: usize = 1_000;
    let n_tick = (n_trials / 20).max(1);

    // Matrix size.
    let n: usize = 1_000;

    // Random matrices
    type Word = usize;
    let mut m0: BitMat<Word> = BitMat::random(n, n);
    let m1: BitMat<Word> = BitMat::random(n, n);

    // To do something in the loop, we count how often the top-right element from the product is 1.
    let mut count: usize = 0;

    print!("Running {} trials of M.N where the matrices are {} x {} ", n_trials.pretty(), n.pretty(), n.pretty());
    io::stdout().flush().ok();

    // Start a stop-watch
    let sw = Stopwatch::new();
    for trial in 0..n_trials {
        if trial % n_tick == 0 {
            print!(".");
            io::stdout().flush().ok();
        }

        if m0.dot_matrix(&m1).get(0, 0) {
            count += 1;
        }

        // Change the input a bit for the next trial.
        let i = trial % n;
        m0.set(i, i, true);
    }
    println!(" done.");
    println!("Loop time: {}.", sw);
    println!("Counter:   {}.", count.pretty());
}
