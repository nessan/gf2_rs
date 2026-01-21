#![allow(non_snake_case)]
/// Prints an all ones matrix and its lower and upper triangular parts side by side.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;

fn main() {
    let n = 20;
    let A = BitMat::<u8>::ones(n, n);

    let line = "-".repeat(n * 3 + 10);
    println!("{line}");
    print!("{}", mat::string_for_ABC(&A, &A.lower(), &A.strictly_lower()));
    println!("{line}");
    print!("{}", mat::string_for_ABC(&A, &A.upper(), &A.strictly_upper()));
}
