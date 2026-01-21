#![allow(non_snake_case)]
/// Create lots of "random" bit-matrices with 50% probability of a bit being set and check that the LU
/// decomposition successfully inverts any that are full rank.
///
/// We also check that the number of full rank matrices is roughly as expected.
///
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use std::io::Write;

fn main() {
    type Mat = BitMat<u8>;

    let n = 200;
    let n_trials = 500;
    let n_tick = n_trials / 20;

    let mut singular = 0;
    print!("Running {n_trials} trials decomposing {n} x {n} bit-matrices ");
    for n_trial in 0..n_trials {
        let A = Mat::random(n, n);
        let LU = BitLU::new(&A);
        if let Some(A_inv) = LU.inverse() {
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
    println!(" done!");

    // Print some statistics ...
    let expected_singular = Mat::probability_singular(n) * n_trials as f64;
    println!("Bit-matrix size:                      {n} x {n}");
    println!("Number of trials:                     {n_trials}");
    println!("Number of singular matrices:          {singular}");
    println!("Expected number of singular matrices: {expected_singular:.0}");
    println!("All A * A_inv = I tests passed!");
}
