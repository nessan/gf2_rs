/// Compare the naive `reduce` method with the optimized one for `BitVector`
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
mod naive;

use rand::prelude::*;
use std::io::Write;

fn main() {
    type BP = BitPolynomial<u8>;

    // Number of trials to run & how often to print progress.
    let n_trials = 1_000;
    let n_tick = n_trials / 20;

    // Maximum degree of the bit-polynomial to test.
    let n_degree = 100;

    // Reducing x^power where the power is in the range power_low..power_high.
    let power_low = 42;
    let power_high = power_low + 1_000_000;

    // Set up the random number generator that we use to pick the lengths of the randomly filled bit-vectors.
    let mut rng = rand::rng();

    // Run the trials.
    for n in 0..n_trials {
        // Pick a random power in the range power_low..power_high.
        let power = rng.random_range(power_low..power_high);

        // Generate an all-ones bit-polynomial of random degree up to `n_degree`.
        let degree = rng.random_range(0..=n_degree);
        let bp: BP = BP::ones(degree);

        // Compute the reduction of the bit-vector using the naive and optimized implementations.
        let reduce_naive = naive::reduce_x_to_power_n(&bp, power);
        let reduce_fast = bp.reduce_x_to_power(power, false);

        // Check that the two implementations agree.
        if reduce_naive != reduce_fast {
            println!("MISMATCH FOR TRIAL: {n}");
            println!("BitPolynomial (degree {}):      {}", bp.degree(), bp.to_string());
            println!("Naive reduction (degree {}): {}", reduce_naive.degree(), reduce_naive.to_string());
            println!("Fast reduction (degree {}):  {}", reduce_fast.degree(), reduce_fast.to_string());
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
