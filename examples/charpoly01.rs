/// Check our char-poly computation vs. pre-canned results.
/// Run in release mode for realistic timings.
///
/// SPDX-FileCopyrightText:  2025 Nessan Fitzmaurice <nzznfitz+gh@icloud.com>
/// SPDX-License-Identifier: MIT
use gf2::*;

use std::{
    io::{
        self,
        Write,
    },
    path::Path,
};
use utilities_rs::Stopwatch;

fn main() {
    // Prompt until we can open a file (or the user exits)
    let data_file_path = loop {
        print!("Data file name (x to exit ...): ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read input. Try again ...");
            continue;
        }
        let trimmed = input.trim();

        // Exit if the user types 'x' or 'X'
        if trimmed.eq_ignore_ascii_case("x") {
            return;
        }

        // Check if the file exists
        if Path::new(trimmed).exists() {
            break trimmed.to_string();
        }
        else {
            eprintln!("Failed to open '{}'. Please try again ...", trimmed);
        }
    };

    // Read the file content.
    let file_content =
        std::fs::read_to_string(&data_file_path).unwrap_or_else(|_| panic!("Failed to open '{}'", data_file_path));

    // Filter out blank and comment lines.
    let filtered_lines: Vec<String> = file_content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect();

    // Count the number of tests.
    let mut num_tests: usize = 0;

    // Iterate through the pairs of lines (the bit-matrix and the coefficients or its characteristic polynomial).
    for pair in filtered_lines.chunks(2) {
        if pair.len() < 2 {
            break; // ignore a trailing unmatched line
        }

        let matrix_string = &pair[0];
        let coeffs_string = &pair[1];

        // Parse matrix
        let m = BitMatrix::<usize>::from_string(matrix_string).unwrap_or_else(|| {
            eprintln!("Failed to parse a bit-matrix from file: '{}'", data_file_path);
            std::process::exit(1);
        });

        // Parse coefficients -> polynomial
        let coeffs = BitVector::<usize>::from_string(coeffs_string).unwrap_or_else(|| {
            eprintln!("Failed to parse a characteristic polynomial from file: '{}'", data_file_path);
            std::process::exit(2);
        });
        let canned = BitPolynomial::<usize>::from_coefficients(coeffs);

        // Progress message.
        num_tests += 1;
        println!("Test {} of {}: Matrix is {} x {} ... ", num_tests, num_tests, m.rows(), m.cols());

        // Compute characteristic polynomial and time it
        let sw = Stopwatch::default();
        let computed = m.characteristic_polynomial();
        let elapsed = sw.elapsed();
        println!("done in {}.", Stopwatch::format_seconds(elapsed));

        // Compare
        if computed != canned {
            println!("TEST {} FAILED! Matrix:\n{}", num_tests, m);
            println!("Computed characteristic:   {}", computed);
            println!("Pre-canned characteristic: {}", canned);
            std::process::exit(1);
        }
    }

    println!("\n Congratulations: All {} tests passed!", num_tests);
}
