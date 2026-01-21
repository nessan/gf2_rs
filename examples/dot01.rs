/// Dot products u.v for profiling & benchmarking.
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
    let n_trials: usize = 1_000_000;
    let n_tick = (n_trials / 20).max(1);

    // Vector size.
    let n: usize = 1_000_000;

    // Two random vectors
    type Word = usize;
    let mut u: BitVec<Word> = BitVec::random(n);
    let v: BitVec<Word> = BitVec::random(n);

    // To do something in the loop, we count how often the dot product is 1.
    let mut count: usize = 0;

    print!("Running {} trials of u.v of size {} ", n_trials.pretty(), n.pretty());
    io::stdout().flush().ok();

    // Start a stop-watch
    let sw = Stopwatch::new();
    for trial in 0..n_trials {
        if trial % n_tick == 0 {
            print!(".");
            io::stdout().flush().ok();
        }

        if u.dot(&v) {
            count += 1;
        }

        // Change the input a bit for the next trial.
        u.set(trial % n, true);
    }
    println!(" done.");
    println!("Loop time: {}.", sw);
    println!("Counter:   {}.", count.pretty());
}
