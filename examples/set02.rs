/// Compare the optimized `*_set` methods with the naive implementations for speed.
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use utilities_rs::Stopwatch;
mod naive;

fn main() {
    let sw = Stopwatch::default();

    // Create a bit-vector with a single set bit at the very end...
    let len = 1_000_000;
    let mut bv: BitVec = BitVec::zeros(len);
    bv.set(len - 1, true);

    let mut result: Option<usize> = None;

    // Time the naive first_set method and check we get the correct result ...
    let mut dt_naive = sw.elapsed();
    let n_trials = 1_000;
    for _ in 0..n_trials {
        result = naive::first_set(&bv);
    }
    dt_naive = sw.elapsed() - dt_naive;
    assert_eq!(result, Some(len - 1));

    // Time the optimized first_set method and check we get the correct result ...
    let mut dt_optimized = sw.elapsed();
    for _ in 0..n_trials {
        result = bv.first_set();
    }
    dt_optimized = sw.elapsed() - dt_optimized;
    assert_eq!(result, Some(len - 1));

    println!("Naive first set:     {}", Stopwatch::format_seconds(dt_naive));
    println!("Optimized first set: {}", Stopwatch::format_seconds(dt_optimized));
    println!("Speed-up factor:     {:.0}x", dt_naive / dt_optimized);

    // Work on the last_set method
    // That works in reverse order so to make it slow we need to set just the first bit ...
    bv.set(len - 1, false);
    bv.set(0, true);
    result = None;
    dt_naive = sw.elapsed();
    for _ in 0..n_trials {
        result = naive::last_set(&bv);
    }
    dt_naive = sw.elapsed() - dt_naive;
    assert_eq!(result, Some(0));

    // Time the optimized method and check we get the correct result ...
    result = None;
    dt_optimized = sw.elapsed();
    for _ in 0..n_trials {
        result = bv.last_set();
    }
    dt_optimized = sw.elapsed() - dt_optimized;
    assert_eq!(result, Some(0));

    println!("Naive last set:      {}", Stopwatch::format_seconds(dt_naive));
    println!("Optimized last set:  {}", Stopwatch::format_seconds(dt_optimized));
    println!("Speed-up factor:     {:.0}x", dt_naive / dt_optimized);
}
