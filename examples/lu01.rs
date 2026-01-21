#![allow(non_snake_case)]
/// Create lots of "random" bit-matrices with 50% probability of a bit being set and check that the LU
/// decomposition works (i.e. `L * U = P * A`).
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

    let n = 400;
    let n_trials = 1_000;
    let n_tick = n_trials / 20;

    let mut full_rank = 0;
    print!("Running {n_trials} trials decomposing {n} x {n} bit-matrices ");
    for n_trial in 0..n_trials {
        let A = Mat::random(n, n);
        let lu = BitLU::new(&A);
        let L = lu.L();
        let U = lu.U();
        let mut PA = A.clone();
        lu.permute_matrix(&mut PA);
        assert_eq!(&L * &U, PA, "Oops! L * U != PA for trial number {n_trial}");
        if lu.rank() == n {
            full_rank += 1;
        }
        if n_trial % n_tick == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
    println!(" done!");
    println!("All {n_trials} P*A = L*U decomposition tests passed.");

    let expected_full_rank = Mat::probability_invertible(n) * n_trials as f64;

    println!("Number of trials:                      {n_trials}");
    println!("Expected number of full-rank matrices: {expected_full_rank:.0}");
    println!("Actual number of full-rank matrices:   {full_rank}");
}
