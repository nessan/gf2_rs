/// Characteristic polynomial test for profiling & benchmarking.
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
    type Word = usize;
    type Mat = BitMat<Word>;

    // Number of trials & ticks.
    let n_trials: usize = 1_000;
    let n_tick = (n_trials / 20).max(1);

    // Matrix size.
    let n: usize = 100;

    print!("Running {} trials for random {} x {} bit-matrices ", n_trials.pretty(), n.pretty(), n.pretty());
    io::stdout().flush().ok();

    // Start a stop-watch
    let sw = Stopwatch::new();
    for trial in 0..n_trials {
        if trial % n_tick == 0 {
            print!(".");
            io::stdout().flush().ok();
        }

        let m: Mat = Mat::random(n, n);
        let p = m.characteristic_polynomial();
        //let pm = p.eval_matrix(&m);
        assert!(p.eval_matrix(&m).is_zero(), "Oops! p(M) != 0 for trial {}", trial.pretty());
    }
    println!(" done.");
    println!("Characteristic polynomial loop time: {}.", sw);
}
