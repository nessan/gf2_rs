/// Compare using an iterator over the words in a bit-vector with using a simple loop over the words.
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use utilities_rs::Stopwatch;
mod naive;

fn main() {
    let sw = Stopwatch::default();

    // Create a large random bit-vector.
    let len = 1_000_000;
    let bv: BitVector = BitVector::random(len);

    // We will run for lots of trials to get a good average.
    let n_trials = 100_000;

    // Going to compute the XOR of all the words in the bit-vector using three different methods.
    // In each case we will store the result of the XOR operation and make sure those match at the end.
    let mut result_simple: usize = 0;
    let mut result_iter = result_simple;
    let mut result_functional = result_simple;

    // 1. A simple loop over the words.
    let mut dt_simple = sw.elapsed();
    for _ in 0..n_trials {
        for i in 0..bv.words() {
            result_simple = result_simple ^ bv.word(i);
        }
    }
    dt_simple = sw.elapsed() - dt_simple;

    // 2. A standard iterator over the words.
    let mut dt_iter = sw.elapsed();
    for _ in 0..n_trials {
        for word in bv.store_words() {
            result_iter = result_iter ^ word;
        }
    }
    dt_iter = sw.elapsed() - dt_iter;

    // 3. A functional approach using an iterator over the words.
    let mut dt_functional = sw.elapsed();
    for _ in 0..n_trials {
        result_functional = result_functional ^ bv.store_words().fold(0, |acc, word| acc ^ word);
    }
    dt_functional = sw.elapsed() - dt_functional;

    // Check we get the same result.
    assert_eq!(result_simple, result_iter, "Simple loop and simple iterator give different results");
    assert_eq!(result_simple, result_functional, "Simple loop and functional approach give different results");

    // Print the timing results ...
    println!("Simple loop over words: {}", Stopwatch::format_seconds(dt_simple));
    println!("Iterator over words:    {}", Stopwatch::format_seconds(dt_iter));
    println!("Functional approach:    {}", Stopwatch::format_seconds(dt_functional));
}
