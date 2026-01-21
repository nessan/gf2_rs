/// Compare the optimized `*_set` methods with the naive implementations for accuracy.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
/// Runs some trials of random bit-vectors and checks that methods agree on the location of the set bits.
use gf2::*;
use utilities_rs::Pretty;
mod naive;

fn main() {
    let len = 1_000_000;
    let prob = 0.5;
    let n_trials = 10;

    println!("Testing forward search ... ");
    for trial in 0..n_trials {
        print!("Trial {}: ", trial.pretty());

        // Create a random bit-vector ...
        let bv: BitVec = BitVec::random_biased(len, prob);

        // We will count the number of set bits one-by-one naively ...
        let mut n_set = 0;

        // Grab the index of the first set bit by both methods and check that they agree ...
        let mut n = naive::first_set(&bv);
        let mut o = bv.first_set();
        assert_eq!(n, o, "naive first set = {n:?}, optimized first set = {o:?}",);

        // Iterate through the set bits and check that the methods agree ...
        while let Some(i) = n {
            n_set += 1;

            // Grab the index of the next set bit by both methods and check that they agree ...
            n = naive::next_set(&bv, i);
            o = bv.next_set(i);
            assert_eq!(n, o, "naive next set = {n:?}, optimized next set = {o:?}");
        }

        // Check that the number of set bits is correct ...
        let o_set = bv.count_ones();
        assert_eq!(n_set, o_set, "naive count = {n_set}, optimized count = {o_set}");
        println!(
            "PASS - both methods counted the same {} set bits (for a bit-vector of length = {}).",
            n_set.pretty(),
            len.pretty()
        );
    }
    println!();

    println!("Testing backward search ... ");
    for trial in 0..n_trials {
        print!("Trial {}: ", trial.pretty());

        // Create a random bit-vector ...
        let bv: BitVec = BitVec::random_biased(len, prob);

        // We will count the number of set bits one-by-one naively ...
        let mut n_set = 0;

        // Grab the index of the last set bit by both methods and check that they agree ...
        let mut n = naive::last_set(&bv);
        let mut o = bv.last_set();
        assert_eq!(n, o, "naive last set = {n:?}, optimized last set = {o:?}",);

        // Iterate through the set bits and check that the methods agree ...
        while let Some(i) = n {
            n_set += 1;

            // Grab the index of the previous set bit by both methods and check that they agree ...
            n = naive::previous_set(&bv, i);
            o = bv.previous_set(i);
            assert_eq!(n, o, "naive previous set = {n:?}, optimized previous set = {o:?}");
        }

        // Check that the number of set bits is correct ...
        let o_set = bv.count_ones();
        assert_eq!(n_set, o_set, "naive count = {n_set}, optimized count = {o_set}");
        println!(
            "PASS - both methods counted the same {} set bits (for a bit-vector of length = {}).",
            n_set.pretty(),
            len.pretty()
        );
    }
    println!("All tests passed!");
}
