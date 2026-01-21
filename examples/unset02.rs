/// Compare the optimized `*_unset` methods with the naive implementations for speed.
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use utilities_rs::Stopwatch;
mod naive;

fn main() {
    let sw = Stopwatch::default();

    // Create a bit-vector with a single unset bit at the very end...
    let len = 1_000_000;
    let mut bv: BitVec<u8> = BitVec::ones(len);
    bv.set(len - 1, false);

    let mut result: Option<usize> = None;

    // Time the naive first_unset method and check we get the correct result ...
    let mut dt_naive = sw.elapsed();
    let n_trials = 1_000;
    for _ in 0..n_trials {
        result = naive::first_unset(&bv);
    }
    dt_naive = sw.elapsed() - dt_naive;
    assert_eq!(result, Some(len - 1));

    // Time the optimized first_unset method and check we get the correct result ...
    let mut dt_optimized = sw.elapsed();
    for _ in 0..n_trials {
        result = bv.first_unset();
    }
    dt_optimized = sw.elapsed() - dt_optimized;
    assert_eq!(result, Some(len - 1));

    println!("Naive first unset:     {}", Stopwatch::format_seconds(dt_naive));
    println!("Optimized first unset: {}", Stopwatch::format_seconds(dt_optimized));
    println!("Speed-up factor:       {:.0}x", dt_naive / dt_optimized);

    // Work on the last_unset method
    // That works in reverse order so to make it slow we need to unset just the first bit ...
    bv.set(len - 1, true);
    bv.set(0, false);
    result = None;
    dt_naive = sw.elapsed();
    for _ in 0..n_trials {
        result = naive::last_unset(&bv);
    }
    dt_naive = sw.elapsed() - dt_naive;
    assert_eq!(result, Some(0));

    // Time the optimized method and check we get the correct result ...
    result = None;
    dt_optimized = sw.elapsed();
    for _ in 0..n_trials {
        result = bv.last_unset();
    }
    dt_optimized = sw.elapsed() - dt_optimized;
    assert_eq!(result, Some(0));

    println!("Naive last unset:      {}", Stopwatch::format_seconds(dt_naive));
    println!("Optimized last unset:  {}", Stopwatch::format_seconds(dt_optimized));
    println!("Speed-up factor:       {:.0}x", dt_naive / dt_optimized);
}
