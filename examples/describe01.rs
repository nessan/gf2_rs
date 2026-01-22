/// Run the `describe` method on a variety of bit-vectors.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;

pub fn main() {
    let mut bv: BitVector<u8> = BitVector::alternating(20);
    println!("Initial bit-vector:");
    println!("{}", bv.describe());

    println!("Shrinking to fit:");
    bv.shrink_to_fit();
    println!("{}", bv.describe());

    println!("Clearing bit-vector:");
    bv.clear();
    println!("{}", bv.describe());

    println!("Shrinking to fit:");
    bv.shrink_to_fit();
    println!("{}", bv.describe());
}
