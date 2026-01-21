/// Compare a naive `convolution` implementation to the `convolved_with` method implemented for `BitVec`.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
mod naive;

use rand::prelude::*;
use std::io::Write;

fn main() {
    type BV = BitVec<u8>;

    // Number of trials to run & how often to print progress.
    let n_trials = 1_000;
    let n_tick = n_trials / 20;

    // Maximum length of the bit-vectors to test.
    let n_size = 1_000;

    // Set up the random number generator that we use to pick the lengths of the randomly filled bit-vectors.
    let mut rng = rand::rng();

    // Run the trials.
    for n in 0..n_trials {
        // Generate two random bit-vectors of random lengths between 1 and `n_size`.
        let lhs_len = rng.random_range(1..n_size);
        let rhs_len = rng.random_range(1..n_size);
        let lhs: BV = BV::random(lhs_len);
        let rhs: BV = BV::random(rhs_len);

        // Compute the convolution of the two bit-vectors using the naive and optimized implementations.
        let cnv_naive = naive::convolution(&lhs, &rhs);
        let cnv_fast = lhs.convolved_with(&rhs);

        // Check that the two implementations agree.
        if cnv_naive != cnv_fast {
            println!("MISMATCH FOR TRIAL: {n}");
            println!("lhs bit-vector (length {}):    {}", lhs_len, lhs.to_string());
            println!("rhs bit-vector (length {}):    {}", rhs_len, rhs.to_string());
            println!("Naive convolution (length {}): {}", cnv_naive.len(), cnv_naive.to_string());
            println!("Fast convolution (length {}):  {}", cnv_fast.len(), cnv_fast.to_string());
            println!("EXITING ...");
            std::process::exit(1);
        }

        // Occasionally add a tick to the progress "bar".
        if n % n_tick == 0 {
            print!(".");
            std::io::stdout().flush().unwrap();
        }
    }

    println!();
    println!("All {n_trials} trials matched!");
}
