/// Compare the naive `shift_left` and `shift_right` implementations to the ones we implemented for `BitVec`
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
    let n_size = 100_000;

    // Set up the random number generator that we use to pick the lengths of the randomly filled bit-vectors.
    let mut rng = rand::rng();

    // Run the trials.
    for n in 0..n_trials {
        // Generate an all-ones bit-vector of random length between 1 and `n_size`.
        let len = rng.random_range(1..n_size);
        let bv: BV = BV::ones(len);

        // Pick a random shift amount.
        let shift = rng.random_range(0..len);

        // Compute the right shift of the bit-vector using the naive and optimized implementations.
        let shr_naive = naive::shift_right(&bv, shift);
        let shr_fast = &bv >> shift;

        // Check that the two implementations agree.
        if shr_naive != shr_fast {
            println!("MISMATCH FOR TRIAL: {n}");
            println!("bv bit-vector (length {}):    {}", len, bv.to_string());
            println!("Naive right shift (length {}): {}", shr_naive.len(), shr_naive.to_string());
            println!("Fast right shift (length {}):  {}", shr_fast.len(), shr_fast.to_string());
            println!("EXITING ...");
            std::process::exit(1);
        }

        // Compute the left shift of the bit-vector using the naive and optimized implementations.
        let bv: BV = BV::ones(len);
        let shl_naive = naive::shift_left(&bv, shift);
        let shl_fast = &bv << shift;

        // Check that the two implementations agree.
        if shl_naive != shl_fast {
            println!("MISMATCH FOR TRIAL: {n}");
            println!("bv bit-vector (length {}):    {}", len, bv.to_string());
            println!("Naive left shift (length {}): {}", shl_naive.len(), shl_naive.to_string());
            println!("Fast left shift (length {}):  {}", shl_fast.len(), shl_fast.to_string());
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
