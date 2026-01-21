#![allow(unused)]
//! Some alternate/naive implementation of various `BitVec` methods.
//!
//! # Note
//! These are typically simpler bit-by-bit implementations of the methods in `BitVec`.
//! They are used for validity & timing checks.
use gf2::*;

/// Returns the index of the first set bit in the bit-vector.
pub fn first_set<Word: Unsigned>(bv: &BitVec<Word>) -> Option<usize> {
    let len = bv.len();
    if len == 0 {
        return None;
    }
    for i in 0..len {
        if bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the last set bit in the bit-vector.
pub fn last_set<Word: Unsigned>(bv: &BitVec<Word>) -> Option<usize> {
    let len = bv.len();
    if len == 0 {
        return None;
    }
    for i in (0..len).rev() {
        if bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the next set bit after `index`.
pub fn next_set<Word: Unsigned>(bv: &BitVec<Word>, index: usize) -> Option<usize> {
    let len = bv.len();
    if len == 0 {
        return None;
    }
    for i in index + 1..len {
        if bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the previous set bit before `index`.
pub fn previous_set<Word: Unsigned>(bv: &BitVec<Word>, index: usize) -> Option<usize> {
    let len = bv.len();
    if len == 0 || index == 0 {
        return None;
    }
    for i in (0..index).rev() {
        if bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the first set bit in the bit-vector.
pub fn first_unset<Word: Unsigned>(bv: &BitVec<Word>) -> Option<usize> {
    let len = bv.len();
    if len == 0 {
        return None;
    }
    for i in 0..len {
        if !bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the last set bit in the bit-vector.
pub fn last_unset<Word: Unsigned>(bv: &BitVec<Word>) -> Option<usize> {
    let len = bv.len();
    if len == 0 {
        return None;
    }
    for i in (0..len).rev() {
        if !bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the previous unset bit before `index`.
pub fn previous_unset<Word: Unsigned>(bv: &BitVec<Word>, index: usize) -> Option<usize> {
    let len = bv.len();
    if len == 0 || index == 0 {
        return None;
    }
    for i in (0..index).rev() {
        if !bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns the index of the next unset bit after `index`.
pub fn next_unset<Word: Unsigned>(bv: &BitVec<Word>, index: usize) -> Option<usize> {
    let len = bv.len();
    if len == 0 {
        return None;
    }
    for i in index + 1..len {
        if !bv[i] {
            return Some(i);
        }
    }
    None
}

/// Returns a "binary" string representation of a bit-vector.
pub fn to_binary_string<Word: Unsigned>(bv: &BitVec<Word>) -> String {
    let len = bv.len();
    if len == 0 {
        return String::new();
    }
    let mut result = String::with_capacity(len);
    for i in 0..len {
        if bv[i] {
            result.push('1');
        }
        else {
            result.push('0');
        }
    }
    result
}

/// Returns a "hex" string representation of a bit-vector.
pub fn to_hex_string<Word: Unsigned>(bv: &BitVec<Word>) -> String {
    let len = bv.len();
    if len == 0 {
        return String::new();
    }

    // The number of digits in the output string. Generally hexadecimal but the last may be to a lower base.
    let digits = (len + 3) / 4;

    // Preallocate space allowing for a possible two character suffix of the form ".b" where `b` is one of 2, 4 or 8.
    let mut result = String::with_capacity(digits + 2);

    // Iterate through the bit-vector in blocks of 4 elements and convert each block to a hex digit
    let n_hex_digits = len / 4;
    for i in 0..n_hex_digits {
        let index = 4 * i;
        let mut num = 0;
        if bv[index] {
            num += 8;
        }
        if bv[index + 1] {
            num += 4;
        }
        if bv[index + 2] {
            num += 2;
        }
        if bv[index + 3] {
            num += 1;
        }
        let num_str = format!("{:X}", num);
        result.push_str(&num_str);
    }

    let k = len % 4;
    if k != 0 {
        // The bit-vector has a length that is not a multiple of 4 so the last hex digit is encoded to a lower base --
        // 2, 4 or 8.
        let mut num = 0;
        for i in 0..k {
            if bv[len - 1 - i] {
                num |= 1 << i;
            }
        }
        let num_str = format!("{:X}", num);
        result.push_str(&num_str);

        // Append the appropriate base to the output string so that the last digit can be interpreted properly.
        result.push_str(&format!(".{}", 1 << k));
    }
    result
}

/// Returns a new bit-vector that is a right-shifted version of the input bit-vector.
pub fn shift_right<Word: Unsigned>(bv: &BitVec<Word>, shift: usize) -> BitVec<Word> {
    let len = bv.len();

    // Edge case?
    if len == 0 || shift == 0 {
        return bv.clone();
    }

    // Set up the result as a zero bit-vector of the correct length.
    let mut result = BitVec::zeros(len);

    // Perhaps we have shifted the whole bit-vector out so the result is all zeros.
    if shift >= len {
        return result;
    }

    // Shift the bit-vector by the given amount.
    for i in 0..len - shift {
        if bv[i] {
            result.set(i + shift, true);
        }
    }
    result
}

/// Returns a new bit-vector that is a left-shifted version of the input bit-vector.
pub fn shift_left<Word: Unsigned>(bv: &BitVec<Word>, shift: usize) -> BitVec<Word> {
    let len = bv.len();

    // Edge case?
    if len == 0 || shift == 0 {
        return bv.clone();
    }

    // Set up the result as a zero bit-vector of the correct length.
    let mut result = BitVec::zeros(len);

    // Perhaps we have shifted the whole bit-vector out so the result is all zeros.
    if shift >= len {
        return result;
    }

    // Shift the bit-vector by the given amount.
    for i in shift..len {
        if bv[i] {
            result.set(i - shift, true);
        }
    }
    result
}

/// Computes the convolution of two bit-vectors.
pub fn convolution<Word: Unsigned>(lhs: &BitVec<Word>, rhs: &BitVec<Word>) -> BitVec<Word> {
    // Edge case?
    if lhs.is_empty() || rhs.is_empty() {
        return BitVec::new();
    }

    // Set up the result as a zero bit-vector of the correct length.
    let mut result = BitVec::zeros(lhs.len() + rhs.len() - 1);

    // If either bit-vector is all zeros then the convolution is all zeros.
    if lhs.none() || rhs.none() {
        return result;
    }

    // Compute the convolution: result[k] = lhs[i] * rhs[k-i] for all i that make sense.
    for i in 0..lhs.len() {
        if lhs[i] {
            // More efficient: if j = k - i then k = i + j so we can directly flip the result at index i + j.
            for j in 0..rhs.len() {
                if rhs[j] {
                    result.flip(i + j);
                }
            }
        }
    }

    // Done
    result
}

// Computes the polynomial r(x) := x^n mod poly(x) for exponent n, where poly(x) is a bit-polynomial.
// This uses the simplest (and slowest) iterative approach.
pub fn reduce_x_to_power_n<Word: Unsigned>(poly: &BitPoly<Word>, n: usize) -> BitPoly<Word> {
    // Make a mutable copy and drop any high order zero coefficients.
    let mut poly = poly.clone();
    poly.make_monic();

    // Edge case?
    if poly.is_zero() {
        return BitPoly::zero();
    }

    // Edge case: the constant polynomial P(x) := 1. Anything mod 1 is 0.
    if poly.is_one() {
        return BitPoly::zero();
    }

    // Edge case: x^0 = 1 so x^n mod P(x) = 1 for any polynomial P(x) != 1 (we already handled the `P(x) = 1` case).
    if n == 0 {
        return BitPoly::one();
    }

    // The polynomial P(x) is non-zero and can be written as P(x) = x^d + p(x) where degree[p] < d.
    let d = poly.degree();

    // Edge case: P(x) = x + c where c is a constant => x = P(x) + c.
    // Then for any exponent e: x^e = (P(x) + c)^e = terms in powers of P(x) + c^e => x^e mod P(x) = c^e = c.
    if d == 1 {
        return BitPoly::constant(poly.coeff(0));
    }

    // We can write p(x) = p_0 + p_1 x + ... + p_{d-1} x^{d-1}. All that matters are those coefficients.
    let p: BitVec<Word> = poly.coefficients().slice(0..d).into();

    // Small powers n < d:
    if n < d {
        return BitPoly::x_to_the(n);
    }

    // Matching power n == d:
    if n == d {
        return BitPoly::from_coefficients(p);
    }

    // Larger powers n > d: Start with r(x) := x^d mod p(x) where r(x) ~ the coefficient bit-vector `result`.
    let mut result = p.clone();

    // Use an iteration which writes x*r(x) mod p(x) in terms of r(x) mod p(x).
    for _ in d..n {
        let add_p = result[d - 1];
        result >>= 1;
        if add_p {
            result ^= &p;
        }
    }

    // Done
    BitPoly::from_coefficients(result)
}
