//! [`BitPoly`] is a polynomial over GF(2) --- a _bit-polynomial_.

use crate::{
    BitMat,
    BitStore,
    BitVec,
    Unsigned,
};

use std::{
    fmt::{
        self,
        Write,
    },
    ops::{
        Add,
        AddAssign,
        Index,
        Mul,
        MulAssign,
        Sub,
        SubAssign,
    },
};

#[doc = include_str!("../docs/poly.md")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BitPoly<Word: Unsigned = usize> {
    // The coefficient of `x^i` is stored in the `i`-th position of the vector.
    // The polynomial may not be monic, i.e., it may have high-order, trailing zero coefficients.
    coeffs: BitVec<Word>,
}

/// Constructors.
impl<Word: Unsigned> BitPoly<Word> {
    /// Returns an empty bit-polynomial with *no* coefficients.
    ///
    /// # Note
    /// This is one possible form of the zero polynomial p(x) := 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::new();
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[must_use]
    #[inline]
    pub fn new() -> Self { Self { coeffs: BitVec::new() } }

    /// Returns the zero bit-polynomial p(x) := 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::zero();
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[must_use]
    #[inline]
    pub fn zero() -> Self { Self { coeffs: BitVec::new() } }

    /// Returns the bit-polynomial p(x) := 1.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::one();
    /// assert_eq!(p.to_string(), "1");
    /// ```
    #[must_use]
    #[inline]
    pub fn one() -> Self { Self { coeffs: BitVec::ones(1) } }

    /// Returns the constant bit-polynomial p(x) := val where val is a boolean.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::constant(true);
    /// assert_eq!(p.to_string(), "1");
    /// ```
    #[must_use]
    #[inline]
    pub fn constant(val: bool) -> Self { Self { coeffs: BitVec::constant(val, 1) } }

    /// Returns a bit-polynomial with `n + 1` coefficients, all initialized to zero.
    ///
    /// The polynomial is: 0*x^n + 0*x^(n-1) + ... + 0*x + 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::zeros(3);
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[must_use]
    #[inline]
    pub fn zeros(n: usize) -> Self { Self { coeffs: BitVec::zeros(n + 1) } }

    /// Returns a monic bit-polynomial of degree `n`, with `n + 1` coefficients, all initialized to one.
    ///
    /// This is the polynomial x^n + x^(n-1) + ... + x + 1.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::ones(4);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4");
    /// ```
    #[must_use]
    #[inline]
    pub fn ones(n: usize) -> Self { Self { coeffs: BitVec::ones(n + 1) } }

    /// Returns the bit-polynomial p(x) := x^n.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// ```
    #[must_use]
    #[inline]
    pub fn x_to_the(n: usize) -> Self {
        let mut coeffs = BitVec::zeros(n + 1);
        coeffs.set(n, true);
        Self { coeffs }
    }

    /// Returns a new bit-polynomial by **consuming** a bit-vector of polynomial coefficients.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101011").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.to_string(), "1 + x^2 + x^4 + x^5");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_coefficients(coeffs: BitVec<Word>) -> Self { Self { coeffs } }

    /// Returns a new bit-polynomial of *degree* `n` with coefficients set by calling the function `f` for each index.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::from_fn(10, |i| i % 2 == 0);
    /// assert_eq!(p.to_string(), "1 + x^2 + x^4 + x^6 + x^8 + x^10");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_fn(n: usize, f: impl Fn(usize) -> bool) -> Self { Self { coeffs: BitVec::from_fn(n + 1, f) } }
}

/// Construct bit-polynomials with random coefficients.
impl<Word: Unsigned> BitPoly<Word> {
    /// Returns a new bit-polynomial of *degree* `n` with `n + 1` coefficients picked uniformly at random.
    ///
    /// # Note
    /// If `n > 0` then the returned polynomial is monic.
    #[must_use]
    pub fn random(n: usize) -> Self {
        // BitPoly of degree `n` has `n + 1` coefficients.
        let mut coeffs = BitVec::random(n + 1);

        // If `n > 0` we want the coefficient of `x^n` to be one for sure. If `n == 0` then a random 0/1 is fine.
        if n > 0 {
            coeffs.set(n, true);
        }
        Self { coeffs }
    }

    /// Returns a new bit-polynomial of *degree* `n` with `n + 1` coefficients picked uniformly at random.
    ///
    /// For reproducibility, the underlying random number generator is seeded with the specified `seed`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let seed: u64 = 42;
    /// let p1: BitPoly = BitPoly::random_seeded(3311, seed);
    /// let p2: BitPoly = BitPoly::random_seeded(3311, seed);
    /// assert_eq!(p1, p2, "Polynomials with the same seed should be equal");
    /// ```
    #[must_use]
    pub fn random_seeded(n: usize, seed: u64) -> Self {
        // BitPoly of degree `n` has `n + 1` coefficients.
        let mut coeffs = BitVec::random_seeded(n + 1, seed);

        // If `n > 0` we want the coefficient of `x^n` to be one for sure. If `n == 0` then a random 0/1 is fine.
        if n > 0 {
            coeffs.set(n, true);
        }
        Self { coeffs }
    }
}

/// Core queries for a bit-polynomial.
impl<Word: Unsigned> BitPoly<Word> {
    /// Returns the degree of the bit-polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.degree(), 4);
    /// ```
    #[must_use]
    #[inline]
    pub fn degree(&self) -> usize { self.coeffs.last_set().unwrap_or(0) }

    /// Returns true if the bit-polynomial is some form of the zero polynomial p(x) := 0
    #[must_use]
    #[inline]
    pub fn is_zero(&self) -> bool { self.coeffs.none() }

    /// Returns true if the bit-polynomial is non-zero.
    #[must_use]
    #[inline]
    pub fn is_non_zero(&self) -> bool { self.coeffs.any() }

    /// Returns true if the bit-polynomial is p(x) := 1
    #[must_use]
    #[inline]
    pub fn is_one(&self) -> bool { self.degree() == 0 && self.coeffs.len() >= 1 && self.coeffs[0] }

    /// Returns true if the bit-polynomial is either p(x) := 0 or 1
    #[must_use]
    #[inline]
    pub fn is_constant(&self) -> bool { self.degree() == 0 }

    /// Returns `true` if the bit-polynomial is *monic*, i.e., no trailing zero coefficients.
    #[must_use]
    #[inline]
    pub fn is_monic(&self) -> bool { self.coeffs.trailing_zeros() == 0 }

    /// Returns the number of coefficients in the bit-polynomial.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize { self.coeffs.len() }

    /// Returns `true` if the bit-polynomial is empty, i.e., has no coefficients.
    ///
    /// A bit-polynomial with no coefficients is treated as a form of p(x) := 0.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool { self.coeffs.is_empty() }
}

/// Coefficient read and write methods for bit=polynomials.
impl<Word: Unsigned> BitPoly<Word> {
    /// Returns a reference to the coefficients of the bit-polynomial as a read-only borrowed bit-vector.
    ///
    /// # Note
    /// You can use this method to get a reference to the polynomial's coefficients as a bit-vector.
    /// Then all the many read-only methods of the [`BitVec`] type are available.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.coefficients().count_ones(), 3);
    /// assert_eq!(p.coefficients().last_set(), Some(4));
    /// ```
    #[must_use]
    #[inline]
    pub fn coefficients(&self) -> &BitVec<Word> { &self.coeffs }

    /// Returns the coefficient of `x^i` for `i`.
    ///
    /// # Note
    /// We also implement the `Index<usize>` trait so you can use `p[i]` to get the coefficient of `x^i`.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.coeff(0), true);
    /// assert_eq!(p[1], false);
    /// ```
    #[must_use]
    #[inline]
    pub fn coeff(&self, i: usize) -> bool { self.coeffs[i] }

    /// Sets the coefficient of `x^i` to the passed `value` and returns `self`.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.set_coeff(2, true);
    /// assert_eq!(p.to_string(), "x^2 + x^3");
    /// p.set_coeff(0, true);
    /// assert_eq!(p.to_string(), "1 + x^2 + x^3");
    /// ```
    #[inline]
    pub fn set_coeff(&mut self, i: usize, value: bool) -> &mut Self {
        self.coeffs.set(i, value);
        self
    }

    /// Clears the polynomial, i.e., sets it to the zero polynomial & returns `self`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.clear();
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.coeffs.clear();
        self
    }

    /// Resizes the polynomial to have the `n` coefficients and returns `self`.
    ///
    /// # Note
    /// If `n` > `self.len()` then the polynomial is padded with zero coefficients and the degree is unchanged.
    /// If `n` < `self.len()` then the polynomial is truncated and the degree is updated.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("111010").unwrap();
    /// let mut p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^4");
    /// p.resize(2);
    /// assert_eq!(p.to_string(), "1 + x");
    /// p.resize(4);
    /// assert_eq!(p.to_full_string(), "1 + x + 0x^2 + 0x^3");
    /// ```
    #[inline]
    pub fn resize(&mut self, n: usize) -> &mut Self {
        self.coeffs.resize(n);
        self
    }

    /// Tries to minimize the storage used by the polynomial --- may do nothing.
    #[inline]
    pub fn shrink_to_fit(&mut self) -> &mut Self {
        self.make_monic();
        self.coeffs.shrink_to_fit();
        self
    }

    /// Kills any trailing zero coefficients, so e.g. p(x) := 0*x^4 + x^2 + x becomes p(x) := x^2 + x.
    ///
    /// # Note
    /// Does nothing to any form of the zero polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let mut p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.is_monic(), false);
    /// p.make_monic();
    /// assert_eq!(p.is_monic(), true);
    /// ```
    #[inline]
    pub fn make_monic(&mut self) {
        if self.is_non_zero() {
            self.coeffs.resize(self.degree() + 1);
        }
    }
}

/// Arithmetic methods for bit-polynomials.
impl<Word: Unsigned> BitPoly<Word> {
    /// Adds another  bit-polynomial to `self` in-place.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut p: BitPoly = BitPoly::x_to_the(3);
    /// let q: BitPoly = BitPoly::x_to_the(2);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.plus_eq(&q);
    /// assert_eq!(q.to_string(), "x^2");
    /// assert_eq!(p.to_string(), "x^2 + x^3");
    /// ```
    pub fn plus_eq(&mut self, rhs: &BitPoly<Word>) {
        // Edge case: `rhs` is the zero polynomial.
        if rhs.is_zero() {
            return;
        }

        // Edge case: `self` is the zero polynomial.
        if self.is_zero() {
            self.coeffs = rhs.coeffs.clone();
            return;
        }

        // Resize if necessary by adding trailing zero coefficients to `self`.
        if self.len() < rhs.degree() + 1 {
            self.coeffs.resize(rhs.degree() + 1);
        }

        // Only consider coefficient "words" in `rhs` up to the word containing its highest non-zero coefficient.
        let monic_words = if rhs.is_monic() { rhs.degree() / Word::UBITS + 1 } else { 0 };

        // Add the coefficients of the two polynomials word by word.
        for i in 0..monic_words {
            let rhs_word = rhs.coeffs.word(i);
            let self_word = self.coeffs.word(i);
            self.coeffs.set_word(i, rhs_word ^ self_word);
        }
    }

    /// Returns a new bit-polynomial that is the sum of this bit-polynomial with another.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// let q: BitPoly = BitPoly::x_to_the(2);
    /// let r = p.plus(&q);
    /// assert_eq!(r.to_string(), "x^2 + x^3");
    /// ```
    #[must_use]
    pub fn plus(&self, rhs: &BitPoly<Word>) -> BitPoly<Word> {
        let mut result = self.clone();
        result.plus_eq(rhs);
        result
    }

    /// Subtracts another  bit-polynomial from `self` in-place.
    ///
    /// # Note
    /// In GF(2), addition and subtraction are the same operation.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut p: BitPoly = BitPoly::x_to_the(3);
    /// let q: BitPoly = BitPoly::x_to_the(2);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.minus_eq(&q);
    /// assert_eq!(q.to_string(), "x^2");
    /// assert_eq!(p.to_string(), "x^2 + x^3");
    /// ```
    pub fn minus_eq(&mut self, rhs: &BitPoly<Word>) { self.plus_eq(rhs) }

    /// Returns a new bit-polynomial that is this bit-polynomial minus another.
    ///
    /// # Note
    /// In GF(2), addition and subtraction are the same operation.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// let q: BitPoly = BitPoly::x_to_the(2);
    /// let r = p.minus(&q);
    /// assert_eq!(r.to_string(), "x^2 + x^3");
    /// ```
    #[must_use]
    pub fn minus(&self, rhs: &BitPoly<Word>) -> BitPoly<Word> {
        let mut result = self.clone();
        result.plus_eq(rhs);
        result
    }

    /// Fills `dst` with the square of `self`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("111").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.to_string(), "1 + x + x^2");
    /// let mut q: BitPoly = BitPoly::new();
    /// p.square_into(&mut q);
    /// assert_eq!(q.to_string(), "1 + x^2 + x^4");
    /// ```
    pub fn square_into(&self, dst: &mut BitPoly<Word>) {
        // Edge case: any constant polynomial.
        if self.is_constant() {
            *dst = self.clone();
            return;
        }

        // In GF(2) if p(x) = a + bx + cx^2 + ... then p(x)^2 = a^2 + b^2x^2 + c^2x^4 + ...
        // So we can use the `riffle` method to square the polynomial.
        self.coeffs.riffled_into(&mut dst.coeffs);
    }

    /// Returns a new polynomial that is the square of the polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// let q: BitPoly = p.squared();
    /// assert_eq!(q.to_string(), "x^6");
    /// ```
    #[inline]
    #[must_use]
    pub fn squared(&self) -> Self {
        let mut dst = BitPoly::new();
        self.square_into(&mut dst);
        dst
    }

    /// Multiplies the polynomial by `x^n` and returns `self`.
    ///
    /// # Note
    /// This is faster than using the multiply method for this special case.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.times_x_to_the(2);
    /// assert_eq!(p.to_string(), "x^5");
    /// ```
    pub fn times_x_to_the(&mut self, n: usize) -> &mut Self {
        let new_degree = self.degree() + n;
        let new_len = new_degree + 1;
        if self.coeffs.len() < new_len {
            self.coeffs.resize(new_len);
        }
        self.coeffs >>= n;
        self
    }

    /// Multiplies `self` by another bit-polynomial and returns the result as a new bit-polynomial.
    ///
    /// # Note
    /// Multiplication of bit-polynomials is performed by convolving their coefficient vectors over GF(2).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// let q: BitPoly = BitPoly::x_to_the(2);
    /// let r = p.convolved_with(&q);
    /// assert_eq!(p.to_string(), "x^3");
    /// assert_eq!(q.to_string(), "x^2");
    /// assert_eq!(r.to_string(), "x^5");
    /// ```
    #[must_use]
    pub fn convolved_with(&self, rhs: &BitPoly<Word>) -> Self {
        // Edge case: either polynomial is the zero polynomial.
        if self.is_zero() || rhs.is_zero() {
            return BitPoly::zero();
        }

        // Edge case: either polynomial is the constant polynomial p(x) := 1.
        if self.is_one() {
            return rhs.clone();
        }
        if rhs.is_one() {
            return self.clone();
        }

        // Otherwise, multiply the polynomials using the convolution method.
        Self { coeffs: self.coeffs.convolved_with(&rhs.coeffs) }
    }
}

/// Bit-polynomial evaluation.
impl<Word: Unsigned> BitPoly<Word> {
    /// Evaluates the polynomial for a scalar `bool` argument.
    ///
    /// # Note
    /// - Ideally this should be implemented via the `Fn` trait but the necessary machinery is not yet available in
    ///   stable Rust. If unstable features are enabled then we have a `Fn` implementation that forwards to this method.
    /// - The `BitMat` module has a `eval_matrix` method to compute `p(M)` where `M` is a bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.eval_bool(true), true);
    /// assert_eq!(p.eval_bool(false), false);
    /// ```
    #[must_use]
    pub fn eval_bool(&self, x: bool) -> bool {
        // Edge case: the zero polynomial.
        if self.is_zero() {
            return false;
        }

        // Edge case: `x = false` which is the same as `x = 0`: p(0) = p_0.
        if !x {
            return self.coeff(0);
        }

        // Only consider coefficient "words" in `self` up to the word containing its highest non-zero coefficient.
        // We are evaluating the polynomial at `x = true` which is the same as `x = 1`: p(1) = p_0 + p_1 + p_2 + ...
        let monic_words = if self.is_monic() { self.degree() / Word::UBITS + 1 } else { 0 };
        let mut sum = Word::ZERO;
        for i in 0..monic_words {
            sum ^= self.coeffs.word(i);
        }
        sum.count_ones() % 2 == 1
    }

    /// Evaluates the bit-polynomial for a square [`BitMat`] argument.
    ///
    /// Uses Horner's method to evaluate `p(M)` where `M` is a square matrix and returns the result as a bit-matrix.
    ///
    /// # Panics
    /// Panics if the matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::from_coefficients(BitVec::alternating(12));
    /// let m: BitMat = BitMat::identity(6);
    /// assert_eq!(p.eval_matrix(&m), BitMat::zeros(6, 6));
    /// let p = BitPoly::from_coefficients(!BitVec::alternating(6));
    /// assert_eq!(p.eval_matrix(&m), BitMat::identity(6));
    /// ```
    #[must_use]
    pub fn eval_matrix(&self, mat: &BitMat<Word>) -> BitMat<Word> {
        // Error case: the matrix is not square.
        assert!(mat.is_square(), "BitMat must be square not {}x{}", mat.rows(), mat.cols());

        // Edge case: the zero polynomial.
        if self.is_zero() {
            return BitMat::zeros(mat.rows(), mat.cols());
        }

        // Otherwise we start with the identity matrix.
        let mut result = BitMat::identity(mat.rows());

        // Work backwards a la Horner's method from the highest non-zero power in the polynomial.
        let mut d = self.degree();
        while d > 0 {
            // Always do a multiply step.
            result = result.dot_matrix(mat);

            // Add the identity to the sum if the polynomial has a non-zero coefficient for `x^(d-1)`
            if self.coeffs[d - 1] {
                result.add_identity();
            }
            // And count down.
            d -= 1;
        }
        result
    }
}

/// String representation methods.
impl<Word: Unsigned> BitPoly<Word> {
    /// Returns a readable "full" string for the bit-polynomial in terms of the default "variable" name `x`.
    ///
    /// # Note
    /// We show all terms including those with zero coefficients.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.to_full_string(), "1 + 0x + x^2 + 0x^3 + x^4 + 0x^5");
    /// ```
    #[must_use]
    pub fn to_full_string(&self) -> String { self.to_full_string_with_var("x") }

    /// Returns a readable string representation of the bit-polynomial in terms of user supplied "variable" name.
    ///
    /// # Note
    /// We do not show any terms with zero coefficients.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.to_string_with_var("M"), "1 + M^2 + M^4");
    /// ```
    #[must_use]
    pub fn to_string_with_var(&self, var: &str) -> String {
        // Edge case: the zero polynomial.
        if self.is_zero() {
            return "0".to_string();
        }

        // Otherwise, build the string representation term by term.
        // All terms other than first are preceded by a plus sign.
        let mut result = String::new();
        let mut first_term = true;
        for i in 0..=self.degree() {
            if self.coeffs[i] {
                if i == 0 {
                    result.push('1');
                }
                else {
                    if !first_term {
                        result.push_str(" + ");
                    }
                    if i == 1 {
                        result.push_str(var);
                    }
                    else {
                        write!(result, "{var}^{i}").unwrap();
                    }
                }
                first_term = false;
            }
        }
        result
    }

    /// Returns a full string representation of the bit-polynomial.
    ///
    /// # Note
    /// The string representation is always in terms of a variable `x` and we show all terms with zero coefficients.
    ///
    /// # Examples
    /// Returns a full string representation of the bit-polynomial.
    ///
    /// # Note
    /// The string representation is always in terms of a variable `x` and we show all terms with zero coefficients.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
    /// let p: BitPoly = BitPoly::from_coefficients(coeffs);
    /// assert_eq!(p.to_full_string_with_var("M"), "1 + 0M + M^2 + 0M^3 + M^4 + 0M^5");
    /// ```
    #[must_use]
    pub fn to_full_string_with_var(&self, var: &str) -> String {
        // Edge case: no coefficients.
        if self.coeffs.is_empty() {
            return "0".to_string();
        }

        // Otherwise, build the string representation term by term.
        // All terms other than first are preceded by a plus sign.
        let mut result = String::new();

        // Start with the first term.
        if self.coeffs[0] {
            result.push('1');
        }
        else {
            result.push('0');
        }

        // Then the rest of the terms.
        for i in 1..self.len() {
            result.push_str(" + ");
            let c = if self.coeffs[i] { "" } else { "0" };
            if i == 1 {
                write!(result, "{c}{var}").unwrap();
            }
            else {
                write!(result, "{c}{var}^{i}").unwrap();
            }
        }
        result
    }
}

/// Reduction methods to compute x^exponent mod P(x) where P is a bit-polynomial and exponent might be huge.
impl<Word: Unsigned> BitPoly<Word> {
    /// If `self` is P(x) then this returns the polynomial r(x) := x^n mod P(x).
    ///
    /// # Note
    /// In general, we can write any polynomial h(x) as h(x) = q(x) * P(x) + r(x) where degree(r) < degree(P).
    /// Here q(x) is called the quotient polynomial and r(x) is the remainder polynomial that we compute and return.
    /// In our case, h(x) = x^n.
    ///
    /// # Panics
    /// Panics if `self` is the zero polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.reduce_x_to_the(2).to_string(), "x^2");
    /// ```
    #[must_use]
    pub fn reduce_x_to_the(&self, n: usize) -> Self { self.reduce_x_to_power(n, false) }

    /// If `self` is P(x) then this returns the polynomial r(x) := x^(2^n) mod P(x).
    ///
    /// # Note
    /// In general, we can write any polynomial h(x) as h(x) = q(x) * P(x) + r(x) where degree(r) < degree(P).
    /// Here q(x) is called the quotient polynomial and r(x) is the remainder polynomial that we compute and return.
    /// In our case, h(x) = x^(2^n) which allows us to consider enormous powers of `x`.
    ///
    /// # Panics
    /// Panics if `self` is the zero polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(6);
    /// assert_eq!(p.reduce_x_to_the_2_to_the(2).to_string(), "x^4");
    /// ```
    #[must_use]
    pub fn reduce_x_to_the_2_to_the(&self, n: usize) -> Self { self.reduce_x_to_power(n, true) }

    /// If `self` is P(x) then this returns the polynomial r(x) := x^e mod P(x) for some exponent `e`.
    ///
    /// If `n_is_exponent` is `true` then the exponent of `x` is `e = 2^n` otherwise `e = n`.
    ///
    /// # Note
    /// In general, we can write any polynomial h(x) as h(x) = q(x) * P(x) + r(x) where degree(r) < degree(P).
    /// Here q(x) is called the quotient polynomial and r(x) is the remainder polynomial that we compute and return.
    ///
    /// In our case, h(x) = x^e where the exponent `e` is either `n` or `2^n` depending on the final argument.
    /// Setting `n_is_exponent = true` allows us to consider enormous powers of `x` which is useful for some
    /// applications.
    ///
    /// # Panics
    /// Panics if `self` is the zero polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPoly = BitPoly::x_to_the(3);
    /// assert_eq!(p.reduce_x_to_power(2, false).to_string(), "x^2");
    /// ```
    #[allow(clippy::many_single_char_names)]
    #[must_use]
    pub fn reduce_x_to_power(&self, n: usize, n_is_exponent: bool) -> Self {
        // Error case: the zero polynomial P(x) := 0.
        assert!(!self.is_zero(), " ... mod P(x) is undefined if P(x) := 0");

        // Edge case: the constant polynomial P(x) := 1. Anything mod 1 is 0.
        if self.is_one() {
            return Self::zero();
        }

        // Edge case: x^0 = 1 so x^n mod P(x) = 1 for any polynomial P(x) != 1 (we already handled the `P(x) = 1` case).
        if n == 0 && !n_is_exponent {
            return Self::one();
        }

        // The polynomial P(x) is non-zero and can be written as P(x) = x^d + p(x) where degree[p] < d.
        let d = self.degree();

        // Edge case: P(x) = x + c where c is a constant => x = P(x) + c.
        // Then for any exponent e: x^e = (P(x) + c)^e = terms in powers of P(x) + c^e => x^e mod P(x) = c^e = c.
        if d == 1 {
            return Self::constant(self.coeff(0));
        }

        // We can write p(x) = p_0 + p_1 x + ... + p_{d-1} x^{d-1}. All that matters are those coefficients.
        let p: BitVec<Word> = self.coeffs.slice(0..d).into();

        // Closure that computes q(x) -> x * q(x) mod P(x) where degree[q] < d.
        // The closure works on the coefficients of q(x) passed as the bit-vector `q`.
        let times_x_step = |q: &mut BitVec<Word>| {
            let add_p = q[d - 1];
            *q >>= 1;
            if add_p {
                *q ^= &p;
            }
        };

        // Iteratively precompute x^{d+i} mod P(x) for i = 0, 1, ..., d-1 starting with x^d mod P(x) ~ p.
        // We store all the bit-vectors in a standard `Vec` of length d.
        let mut power_mod = Vec::<BitVec<Word>>::with_capacity(d);
        power_mod.push(p.clone());
        for i in 1..d {
            let mut q = power_mod[i - 1].clone();
            times_x_step(&mut q);
            power_mod.push(q);
        }

        // Create some workspace for the reduction.
        let mut s = BitVec::zeros(2 * d);
        let mut h = BitVec::zeros(d);

        // Closure that computes q(x) -> q(x)^2 mod P(x) where degree[q] < d.
        // The closure works on the coefficients of q(x) passed as the bit-vector `q`.
        let mut square_step = |q: &mut BitVec<Word>| {
            // Compute q(x)^2, storing the resulting coefficients in the bit-vector `s`.
            q.riffled_into(&mut s);

            // Split s(x) as s(x) = l(x) + x^d h(h) where l(x) & h(x) are both of degree < d.
            // We reuse q to store l(x).
            s.split_at_into(d, q, &mut h);

            // Now s(x) = q(x) + x^d h(x) so s(x) mod P(x) = q(x) + x^n h(x) mod P(x) which we handle term by term.
            // If h(x) is non-zero then at most every second term in h is 1 (by the nature of squaring in GF(2)).
            if let Some(h_first) = h.first_set() {
                let h_last = h.last_set().unwrap();
                for i in (h_first..=h_last).step_by(2) {
                    if h[i] {
                        *q ^= &power_mod[i];
                    }
                }
            }
        };

        // If `n_is_exponent` is `true`, we are reducing x^(2^n) mod P(x).
        // Note that we already handled edge case where P(x) = x + c above.
        if n_is_exponent {
            // Start with r(x) = x mod P(x)
            let mut r = BitVec::zeros(d);
            r.set(1, true);

            // Squaring, r(x) = x mod P(x) -> x^2 mod P(x) -> x^4 mod P(x) ... we get to x^(2^n) mod P(x).
            for _ in 0..n {
                square_step(&mut r);
            }
            return Self::from_coefficients(r);
        }

        // Normal small exponent case: n < d => x^n mod P(x) = x^n.
        if n < d {
            return BitPoly::x_to_the(n);
        }

        // Matching power case: n == d => x^n mod P(x) = p(x)
        if n == d {
            return Self::from_coefficients(p);
        }

        // Larger power case: n > d: Multiply & square until we get to x^n mod P(x).
        // Note that if e.g. n = 0b00010111 then n.prev_power_of_two() = 0b00010000.
        let mut n_bit = n.prev_power_of_two();

        // Returning r(x) where degree[r] < d so r(x) = r_0 + r_1 x + ... + r_{d-1} x^{d-1} has d coefficients.
        let mut r = BitVec::zeros(d);

        // We start with r(x) = x mod P(x) which handles `n`'s most significant binary digit.
        r.set(1, true);
        n_bit >>= 1;

        // And off we go from there squaring & multiplying as needed ...
        while n_bit > 0 {
            // Always do a square step ...
            square_step(&mut r);

            // Do  a times_x step if the current bit in `n` is set.
            if (n & n_bit) != 0 {
                times_x_step(&mut r);
            }

            // Move to the next bit position in n.
            n_bit >>= 1;
        }

        // Done
        Self::from_coefficients(r)
    }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Default` trait implementation for the `BitPoly` type.
// --------------------------------------------------------------------------------------------------------------------

/// Implement the `Default` constructor trait for the `BitPoly` type.
///
/// The default constructor creates an empty polynomial which is one form of the zero polynomial.
/// No capacity is reserved until coefficients are added.
///
/// # Examples
/// ```
/// use gf2::*;
/// let p: BitPoly = Default::default();
/// assert_eq!(p.to_string(), "0");
/// ```
impl<Word: Unsigned> Default for BitPoly<Word> {
    #[inline]
    fn default() -> Self { Self::new() }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Index` trait implementation for the `BitPoly` type.
// --------------------------------------------------------------------------------------------------------------------

/// The `Index` trait implementation for the `BitPoly` type.
///
/// Returns the coefficient of `x^i`.
///
/// # Panics
/// In debug mode, panics if `i` is out of bounds.
///
/// # Examples
/// ```
/// use gf2::*;
/// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
/// let p: BitPoly = BitPoly::from_coefficients(coeffs);
/// assert_eq!(p[2], true);
/// assert_eq!(p[3], false);
/// ```
impl<Word: Unsigned> Index<usize> for BitPoly<Word> {
    type Output = bool;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output { if self.coeffs[i] { &true } else { &false } }
}

// --------------------------------------------------------------------------------------------------------------------
// Various `Display`-like trait implementations for the `BitPoly` type.
// --------------------------------------------------------------------------------------------------------------------

/// The `fmt::Display` trait implementation for the `BitPoly` type.
///
/// Returns a readable string representation of the bit-polynomial.
///
/// # Note
/// The string representation is always in terms of a variable `x` and we do not show any terms with zero
/// coefficients unless the `alternate` flag is set.
///
/// # Examples
/// ```
/// use gf2::*;
/// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
/// let p: BitPoly = BitPoly::from_coefficients(coeffs);
/// assert_eq!(format!("{p}"), "1 + x^2 + x^4");
/// assert_eq!(format!("{p:#}"), "1 + 0x + x^2 + 0x^3 + x^4 + 0x^5");
/// ```
impl<Word: Unsigned> fmt::Display for BitPoly<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.to_full_string_with_var("x"))
        }
        else {
            write!(f, "{}", self.to_string_with_var("x"))
        }
    }
}

/// The `fmt::Debug` trait implementation for the `BitPoly` type.
///
/// Returns a debug string representation of the bit-polynomial.
///
/// # Note
/// The string representation is always in terms of a variable `x` and we show all terms with zero coefficients.
///
/// # Examples
/// ```
/// use gf2::*;
/// let coeffs: BitVec = BitVec::from_string("101010").unwrap();
/// let p: BitPoly = BitPoly::from_coefficients(coeffs);
/// assert_eq!(format!("{p:?}"), "1 + 0x + x^2 + 0x^3 + x^4 + 0x^5");
/// ```
impl<Word: Unsigned> fmt::Debug for BitPoly<Word> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.to_full_string_with_var("x")) }
}

// --------------------------------------------------------------------------------------------------------------------
// The `AddAssign`, `SubAssign` and `MulAssign` trait implementations for two bit-polynomials
//
// We have implemented the traits where right-hand side may or may not be consumed by the call.
// For example if p and q are bit-polynomials, then for the pairwise `+=` operator we have implemented:
//
// - p += &q leaves q untouched.
// - p += q  consumes q.
// --------------------------------------------------------------------------------------------------------------------

/// The `AddAssign` trait implementation for a `BitPoly` value and a `BitPoly` reference.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// let q: BitPoly = BitPoly::x_to_the(2);
/// assert_eq!(p.to_string(), "x^3");
/// p += &q;
/// assert_eq!(q.to_string(), "x^2");
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> AddAssign<&BitPoly<Word>> for BitPoly<Word> {
    #[inline]
    fn add_assign(&mut self, rhs: &BitPoly<Word>) { self.plus_eq(rhs); }
}

/// The `AddAssign` trait implementation for a `BitPoly` value and a `BitPoly` value.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// assert_eq!(p.to_string(), "x^3");
/// p += BitPoly::x_to_the(2);
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> AddAssign<BitPoly<Word>> for BitPoly<Word> {
    #[inline]
    fn add_assign(&mut self, rhs: BitPoly<Word>) { self.plus_eq(&rhs); }
}

/// The `SubAssign` trait implementation for a `BitPoly` value and a `BitPoly` reference.
///
/// # Note
/// In GF(2), addition and subtraction are the same operation.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// let q: BitPoly = BitPoly::x_to_the(2);
/// assert_eq!(p.to_string(), "x^3");
/// p -= &q;
/// assert_eq!(q.to_string(), "x^2");
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> SubAssign<&BitPoly<Word>> for BitPoly<Word> {
    #[inline]
    fn sub_assign(&mut self, rhs: &BitPoly<Word>) { self.plus_eq(rhs); }
}

/// The `SubAssign` trait implementation for a `BitPoly` with another `BitPoly` value.
///
/// # Note
/// - In GF(2) subtraction is the same as addition.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// assert_eq!(p.to_string(), "x^3");
/// p -= BitPoly::x_to_the(2);
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> SubAssign<BitPoly<Word>> for BitPoly<Word> {
    #[inline]
    fn sub_assign(&mut self, rhs: BitPoly<Word>) { self.plus_eq(&rhs); }
}

/// The `MulAssign` trait implementation for a `BitPoly` value and a `BitPoly` reference.
///
/// Multiplying polynomials is achieved by convolving their coefficient vectors.
///
/// # Note
/// This does not consume the right-hand side but it must be called using the `&` operator.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// let q: BitPoly = BitPoly::x_to_the(2);
/// assert_eq!(p.to_string(), "x^3");
/// assert_eq!(q.to_string(), "x^2");
/// p *= &q;
/// assert_eq!(p.to_string(), "x^5");
/// ```
impl<Word: Unsigned> MulAssign<&BitPoly<Word>> for BitPoly<Word> {
    #[inline]
    fn mul_assign(&mut self, rhs: &BitPoly<Word>) {
        let result = self.convolved_with(rhs);
        *self = result;
    }
}

/// The `MulAssign` trait implementation for two `BitPoly` values.
///
/// # Note
/// This consumes the right-hand side.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// assert_eq!(p.to_string(), "x^3");
/// p *= BitPoly::x_to_the(2);
/// assert_eq!(p.to_string(), "x^5");
/// ```
impl<Word: Unsigned> MulAssign<BitPoly<Word>> for BitPoly<Word> {
    #[inline]
    fn mul_assign(&mut self, rhs: BitPoly<Word>) {
        let result = self.convolved_with(&rhs);
        *self = result;
    }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Add`, `Sub` and `Mul` trait implementations for two bit-polynomials
//
// We have implemented the traits where right-hand side may or may not be consumed by the call.
// For example if p and q are bit-polynomials, then for the pairwise `+` operator we have implemented:
//
// - &p + &q leaves both p and q untouched.
// - &p + q  consumes q.
// - p + &q  consumes p.
// - p + q   consumes both p and q.
// --------------------------------------------------------------------------------------------------------------------

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs + &rhs` as new bit-polynomial
impl<Word: Unsigned> Add<&BitPoly<Word>> for &BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn add(self, rhs: &BitPoly<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs + rhs` as new bit-polynomial consuming `rhs`.
impl<Word: Unsigned> Add<&BitPoly<Word>> for BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn add(self, rhs: &BitPoly<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs + &rhs` as new bit-polynomial consuming `lhs`.
impl<Word: Unsigned> Add<BitPoly<Word>> for &BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn add(self, rhs: BitPoly<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs + rhs` as new bit-polynomial consuming both operands.
impl<Word: Unsigned> Add<BitPoly<Word>> for BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn add(self, rhs: BitPoly<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs - &rhs` as new bit-polynomial
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<&BitPoly<Word>> for &BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn sub(self, rhs: &BitPoly<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs - rhs` as new bit-polynomial consuming `rhs`.
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<&BitPoly<Word>> for BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn sub(self, rhs: &BitPoly<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs - &rhs` as new bit-polynomial consuming `lhs`.
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<BitPoly<Word>> for &BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn sub(self, rhs: BitPoly<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs - rhs` as new bit-polynomial consuming both operands.
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<BitPoly<Word>> for BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn sub(self, rhs: BitPoly<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs * &rhs` as new bit-polynomial
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<&BitPoly<Word>> for &BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn mul(self, rhs: &BitPoly<Word>) -> Self::Output { self.convolved_with(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs * rhs` as new bit-polynomial consuming `rhs`.
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<&BitPoly<Word>> for BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn mul(self, rhs: &BitPoly<Word>) -> Self::Output { self.convolved_with(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs * &rhs` as new bit-polynomial consuming `lhs`.
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<BitPoly<Word>> for &BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn mul(self, rhs: BitPoly<Word>) -> Self::Output { self.convolved_with(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs * rhs` as new bit-polynomial consuming both operands.
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<BitPoly<Word>> for BitPoly<Word> {
    type Output = BitPoly<Word>;

    #[inline]
    fn mul(self, rhs: BitPoly<Word>) -> Self::Output { self.convolved_with(&rhs) }
}

// --------------------------------------------------------------------------------------------------------------------
// If the compiler supports the `unboxed_closures` & `fn_traits` features, we can use the `BitPoly` type as a
// function over the field GF(2). So you can use the natural call `p(x)` instead of the long hand `p.eval_bool(x)`.
// You can also call `p(M)` where `M` is a bit-matrix instead of the long hand `p.eval_matrix(M)`.
//
// Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
//
// We really only care about the `Fn` trait, but it has `FnMut` as a super-trait, and that has `FnOnce` as a
// super-trait so we must implement all three. This is much messier than the equivalent C++ code.
//
// We implement the three traits when unstable features are enabled:
//
// - `Fn`
// - `FnMut`
// - `FnOnce`
// --------------------------------------------------------------------------------------------------------------------

/// The `Fn` trait implementation for the `BitPoly` type with a `bool` argument.
///
/// # Note
/// Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let p: BitPoly = BitPoly::x_to_the(3);
/// assert_eq!(p(true), true);
/// assert_eq!(p(false), false);
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> Fn<(bool,)> for BitPoly<Word> {
    extern "rust-call" fn call(&self, args: (bool,)) -> Self::Output { self.eval_bool(args.0) }
}

/// The `FnMut` trait implementation for the `BitPoly` type with a `bool` argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnMut` as a super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// assert_eq!(p(true), true);
/// assert_eq!(p(false), false);
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnMut<(bool,)> for BitPoly<Word> {
    extern "rust-call" fn call_mut(&mut self, args: (bool,)) -> Self::Output { self.eval_bool(args.0) }
}

/// The `FnOnce` trait implementation for the `BitPoly` type with a `bool` argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnOnce` as a super-super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// assert_eq!(p(true), true);
/// assert_eq!(p(false), false);
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnOnce<(bool,)> for BitPoly<Word> {
    type Output = bool;

    extern "rust-call" fn call_once(self, args: (bool,)) -> Self::Output { self.eval_bool(args.0) }
}

/// The `Fn` trait implementation for the `BitPoly` type with a `BitMat` reference argument.
///
/// # Note
/// Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let p: BitPoly = BitPoly::x_to_the(3);
/// let m: BitMat = BitMat::identity(3);
/// assert_eq!(p(&m), BitMat::identity(3));
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> Fn<(&BitMat<Word>,)> for BitPoly<Word> {
    extern "rust-call" fn call(&self, args: (&BitMat<Word>,)) -> Self::Output { self.eval_matrix(args.0) }
}

/// The `FnMut` trait implementation for the `BitPoly` type with a `BitMat` reference argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnMut` as a super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// let m: BitMat = BitMat::identity(3);
/// assert_eq!(p(&m), BitMat::identity(3));
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnMut<(&BitMat<Word>,)> for BitPoly<Word> {
    extern "rust-call" fn call_mut(&mut self, args: (&BitMat<Word>,)) -> Self::Output { self.eval_matrix(args.0) }
}

/// The `FnOnce` trait implementation for the `BitPoly` type with a `BitMat` reference argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnOnce` as a super-super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPoly = BitPoly::x_to_the(3);
/// let m: BitMat = BitMat::identity(3);
/// assert_eq!(p(&m), BitMat::identity(3));
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnOnce<(&BitMat<Word>,)> for BitPoly<Word> {
    type Output = BitMat<Word>;

    extern "rust-call" fn call_once(self, args: (&BitMat<Word>,)) -> Self::Output { self.eval_matrix(args.0) }
}
