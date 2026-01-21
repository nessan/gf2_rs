/// Compare the speed of the naive `to_*_string` implementation to the ones we implemented for `BitVec`
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use utilities_rs::Stopwatch;
mod naive;

fn compare_bin(n_size: usize, n_trials: usize) {
    let sw = Stopwatch::default();

    type BV = BitVec<usize>;
    let bv: BV = BV::random(n_size);

    let mut s1: String = String::new();
    let mut dt_naive = sw.elapsed();
    for _ in 0..n_trials {
        s1 = naive::to_binary_string(&bv);
    }
    dt_naive = sw.elapsed() - dt_naive;

    let mut s2: String = String::new();
    let mut dt_optimized = sw.elapsed();
    for _ in 0..n_trials {
        s2 = bv.to_binary_string();
    }
    dt_optimized = sw.elapsed() - dt_optimized;

    // Check that we are looking at the same final strings.
    assert_eq!(s1, s2);

    println!("Naive binary algorithm took:     {}", Stopwatch::format_seconds(dt_naive));
    println!("Optimized binary algorithm took: {}", Stopwatch::format_seconds(dt_optimized));
    println!("Ratio:                           {:.2}", dt_naive / dt_optimized);
}

fn compare_hex(n_size: usize, n_trials: usize) {
    let sw = Stopwatch::default();

    type BV = BitVec<usize>;
    let bv: BV = BV::random(n_size);

    let mut s1: String = String::new();
    let mut dt_naive = sw.elapsed();
    for _ in 0..n_trials {
        s1 = naive::to_hex_string(&bv);
    }
    dt_naive = sw.elapsed() - dt_naive;

    let mut s2: String = String::new();
    let mut dt_optimized = sw.elapsed();
    for _ in 0..n_trials {
        s2 = bv.to_hex_string();
    }
    dt_optimized = sw.elapsed() - dt_optimized;

    // Check that we are looking at the same final strings.
    assert_eq!(s1, s2);

    println!("Naive hex algorithm took:        {}", Stopwatch::format_seconds(dt_naive));
    println!("Optimized hex algorithm took:    {}", Stopwatch::format_seconds(dt_optimized));
    println!("Ratio:                           {:.2}", dt_naive / dt_optimized);
}

fn main() {
    let n_size = 10_000;
    let n_trials = 10_000;

    compare_bin(n_size, n_trials);
    compare_hex(n_size, n_trials);

    return;
}
