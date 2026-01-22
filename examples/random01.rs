/// Create lots of "random" `BitVector`s and check the number of set bits is roughly as expected.
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use utilities_rs::Pretty;

/// Test a single random bit vector of length `n_size` with probability `p` of a bit being set.
///
/// Returns the relative error between the number of set bits and the expected number of set bits.
fn test_random_bit_vector(n_size: usize) -> f64 {
    let bv: BitVector = BitVector::random(n_size);
    let n_set = bv.count_ones() as f64;
    let expected = n_size as f64 / 2.0;
    let error = (n_set - expected).abs() / expected;
    error
}

fn main() {
    // We will test vectors of lengths up to `max_size` in steps of `n_size_step`.
    let max_size = 100_000;
    let n_sizes = 10;
    let n_size_step = max_size / n_sizes;

    // For each vector length, we will run `n_trials` trials and compute the average error.
    let n_trials = 1_000;

    println!("Running {n_trials} trials, creating vectors with a 50/50 chance of a bit being set.");
    for i in 1..=n_sizes {
        let size = n_size_step * i;
        let mut total_error = 0.0;
        for _ in 0..n_trials {
            total_error += test_random_bit_vector(size);
        }
        let average_pct = 100.0 * total_error / n_trials as f64;
        println!("    vector length: {:10} average error: {average_pct:.3}%", size.pretty());
    }
}
