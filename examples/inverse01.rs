#![allow(non_snake_case)]
/// Create lots of "random" bit-matrices with 50% probability of a bit being set and check that if the matrix is
/// not singular then the inverse is correct.
///
/// We also check that the number of non-singular matrices is roughly as expected.
///
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use std::io::Write;
use utilities_rs::Stopwatch;

fn main() {
    type Mat = BitMat<u8>;

    let n = 500;

    // Number of trials to run & how often to print progress.
    let n_trials = 1_000;
    let n_tick = n_trials / 20;

    let mut singular = 0;
    print!("Running {n_trials} trials inverting {n} x {n} bit-matrices ");

    // Set up a stop-watch
    let sw = Stopwatch::default();
    let t0 = sw.elapsed();
    for n_trial in 0..n_trials {
        let A = Mat::random(n, n);
        if let Some(A_inv) = A.inverse() {
            assert_eq!(&A * &A_inv, Mat::identity(n), "Oops! A_inv * A != I");
        }
        else {
            singular += 1;
        }
        if n_trial % n_tick == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
    let dt = sw.elapsed() - t0;
    println!(" done!");

    // Print some statistics ...
    let expected_singular = Mat::probability_singular(n) * n_trials as f64;
    println!("Loop time:                            {}", Stopwatch::format_seconds(dt));
    println!("Bit-matrix size:                      {n} x {n}");
    println!("Number of trials:                     {n_trials}");
    println!("Number of singular matrices:          {singular}");
    println!("Expected number of singular matrices: {expected_singular:.0}");
    println!("All A * A_inv = I tests passed!");
}
