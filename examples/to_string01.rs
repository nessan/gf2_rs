/// Compare the naive `to_*_string` implementation to the ones we implemented for `BitVector`
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;
mod naive;

fn test_to_binary_string(len: usize, n_trials: usize) {
    type BV = BitVector<u8>;
    // Create lots of random bit-vectors and check that the naive and optimized versions agree.
    for _ in 0..n_trials {
        let bv: BV = BV::random(len);
        assert_eq!(naive::to_binary_string(&bv), bv.to_binary_string(), "Mismatch for len = {len}");
    }
}

fn test_to_hex_string(len: usize, n_trials: usize) {
    type BV = BitVector<u8>;
    // Create lots of random bit-vectors and check that the naive and optimized versions agree.
    for _ in 0..n_trials {
        let bv: BV = BV::random(len);
        assert_eq!(naive::to_hex_string(&bv), bv.to_hex_string(), "Mismatch for len = {len}");
    }
}

fn main() {
    let n_trials = 10_000;
    let len = 8127;
    test_to_binary_string(len, n_trials);
    println!("Binary string test complete -- all good!");
    test_to_hex_string(len, n_trials);
    println!("Hex string test complete -- all good!");
}
