/// Check that what we get back from `from_string` is what we expect.
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
    let n_size = 10_000;

    // Set up the random number generator that we use to pick the lengths of the randomly filled bit-vectors.
    let mut rng = rand::rng();

    // Run the trials.
    for n in 0..n_trials {
        // Generate a random bit-vector of random length between 1 and `n_size`.
        let len = rng.random_range(1..n_size);
        let bv1: BV = BV::random(len);

        // Convert the bit-vector to a binary string.
        let bin_str = bv1.to_binary_string();

        // Convert the binary string back to a bit-vector.
        let bv2 = BV::from_binary_string(&bin_str).unwrap_or_else(|| {
            println!("BINARY CONVERSION ERROR FOR TRIAL: {n}");
            println!("bv1 (length {}): {}", len, bv1.to_string());
            println!("EXITING ...");
            std::process::exit(1)
        });

        // Check that the two bit-vectors agree.
        if bv1 != bv2 {
            println!("BINARY MISMATCH FOR TRIAL: {n}");
            println!("bv1 (length {}):    {}", len, bv1.to_string());
            println!("bv2 (length {}):    {}", len, bv2.to_string());
            println!("EXITING ...");
            std::process::exit(1);
        }

        // Convert the bit-vector to a hex string.
        let hex_str = bv1.to_hex_string();

        // Convert the hex string back to a bit-vector.
        let bv3 = BV::from_hex_string(&hex_str).unwrap_or_else(|| {
            println!("HEX CONVERSION ERROR FOR TRIAL: {n}");
            println!("bv1 (length {}): {}", len, bv1.to_string());
            println!("EXITING ...");
            std::process::exit(1);
        });

        // Check that the two bit-vectors agree.
        if bv1 != bv3 {
            println!("HEX MISMATCH FOR TRIAL: {n}");
            println!("bv1 (length {}): {}", len, bv1.to_string());
            println!("bv3 (length {}): {}", len, bv3.to_string());
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
    println!("All {n_trials} binary and hex conversion trials matched!");
}
