/// Compare the optimized `*_set` methods with the naive implementations for accuracy.
/// Runs some trials of random bit-vectors and checks that methods agree on the location of the set bits.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
use utilities_rs::Pretty;
mod naive;

fn main() {
    let len = 1_000_000;
    let prob = 0.5;
    let n_trials = 10;

    type BV = BitVec<u8>;

    println!("Testing forward search ... ");
    for trial in 0..n_trials {
        print!("Trial {}: ", trial.pretty());

        // Create a random bit-vector ...
        let bv = BV::random_biased(len, prob);

        // We will count the number of unset bits one-by-one naively ...
        let mut n_unset = 0;

        // Grab the index of the first unset bit by both methods and check that they agree ...
        let mut n = naive::first_unset(&bv);
        let mut o = bv.first_unset();
        assert_eq!(n, o, "naive first unset = {n:?}, optimized first unset = {o:?}",);

        // Iterate through the unset bits and check that the methods agree ...
        while let Some(i) = n {
            n_unset += 1;

            // Grab the index of the next unset bit by both methods and check that they agree ...
            n = naive::next_unset(&bv, i);
            o = bv.next_unset(i);
            assert_eq!(n, o, "naive next unset = {n:?}, optimized next unset = {o:?}");
        }

        // Check that the number of unset bits is correct ...
        let o_unset = bv.count_zeros();
        assert_eq!(n_unset, o_unset, "naive count = {n_unset}, optimized count = {o_unset}");
        println!(
            "PASS - both methods counted the same {} unset bits (for a bit-vector of length = {}).",
            n_unset.pretty(),
            len.pretty()
        );
    }
    println!();

    println!("Testing backward search ... ");
    for trial in 0..n_trials {
        print!("Trial {}: ", trial.pretty());

        // Create a random bit-vector ...
        let bv = BV::random_biased(len, prob);

        // We will count the number of unset bits one-by-one naively ...
        let mut n_unset = 0;

        // Grab the index of the first unset bit by both methods and check that they agree ...
        let mut n = naive::last_unset(&bv);
        let mut o = bv.last_unset();
        assert_eq!(n, o, "naive last unset = {n:?}, optimized last unset = {o:?}",);

        // Iterate through the unset bits and check that the methods agree ...
        while let Some(i) = n {
            n_unset += 1;

            // Grab the index of the previous unset bit by both methods and check that they agree ...
            n = naive::previous_unset(&bv, i);
            o = bv.previous_unset(i);
            assert_eq!(n, o, "naive previous unset = {n:?}, optimized previous unset = {o:?}");
        }

        // Check that the number of unset bits is correct ...
        let o_unset = bv.count_zeros();
        assert_eq!(n_unset, o_unset, "naive count = {n_unset}, optimized count = {o_unset}");
        println!(
            "PASS - both methods counted the same {} unset bits (for a bit-vector of length = {}).",
            n_unset.pretty(),
            len.pretty()
        );
    }
    println!("All tests passed!");
}
