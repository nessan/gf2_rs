//! [`BitPolynomial`] is a polynomial over GF(2) --- a _bit-polynomial_.

use crate::{
    BitMatrix,
    BitStore,
    BitVector,
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
pub struct BitPolynomial<Word: Unsigned = usize> {
    // The coefficient of `x^i` is stored in the `i`-th position of the vector.
    // The polynomial may not be monic, i.e., it may have high-order, trailing zero coefficients.
    coeffs: BitVector<Word>,
}

/// Constructors.
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Returns an empty bit-polynomial with *no* coefficients.
    ///
    /// # Note
    /// This is one possible form of the zero polynomial p(x) := 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::new();
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[must_use]
    #[inline]
    pub fn new() -> Self { Self { coeffs: BitVector::new() } }

    /// Returns the zero bit-polynomial p(x) := 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::zero();
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[must_use]
    #[inline]
    pub fn zero() -> Self { Self { coeffs: BitVector::new() } }

    /// Returns the bit-polynomial p(x) := 1.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::one();
    /// assert_eq!(p.to_string(), "1");
    /// ```
    #[must_use]
    #[inline]
    pub fn one() -> Self { Self { coeffs: BitVector::ones(1) } }

    /// Returns the constant bit-polynomial p(x) := val where val is a boolean.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::constant(true);
    /// assert_eq!(p.to_string(), "1");
    /// ```
    #[must_use]
    #[inline]
    pub fn constant(val: bool) -> Self { Self { coeffs: BitVector::constant(val, 1) } }

    /// Returns a bit-polynomial with `n + 1` coefficients, all initialized to zero.
    ///
    /// The polynomial is: 0*x^n + 0*x^(n-1) + ... + 0*x + 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::zeros(3);
    /// assert_eq!(p.to_string(), "0");
    /// ```
    #[must_use]
    #[inline]
    pub fn zeros(n: usize) -> Self { Self { coeffs: BitVector::zeros(n + 1) } }

    /// Returns a monic bit-polynomial of degree `n`, with `n + 1` coefficients, all initialized to one.
    ///
    /// This is the polynomial x^n + x^(n-1) + ... + x + 1.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::ones(4);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4");
    /// ```
    #[must_use]
    #[inline]
    pub fn ones(n: usize) -> Self { Self { coeffs: BitVector::ones(n + 1) } }

    /// Returns the bit-polynomial p(x) := x^n.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// ```
    #[must_use]
    #[inline]
    pub fn x_to_the(n: usize) -> Self {
        let mut coeffs = BitVector::zeros(n + 1);
        coeffs.set(n, true);
        Self { coeffs }
    }

    /// Returns a new bit-polynomial by **consuming** a bit-vector of polynomial coefficients.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVector = BitVector::from_string("101011").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
    /// assert_eq!(p.to_string(), "1 + x^2 + x^4 + x^5");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_coefficients(coeffs: BitVector<Word>) -> Self { Self { coeffs } }

    /// Returns a new bit-polynomial of *degree* `n` with coefficients set by calling the function `f` for each index.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::from_fn(10, |i| i % 2 == 0);
    /// assert_eq!(p.to_string(), "1 + x^2 + x^4 + x^6 + x^8 + x^10");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_fn(n: usize, f: impl Fn(usize) -> bool) -> Self { Self { coeffs: BitVector::from_fn(n + 1, f) } }
}

/// Construct bit-polynomials with random coefficients.
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Returns a new bit-polynomial of *degree* `n` with `n + 1` coefficients picked uniformly at random.
    ///
    /// # Note
    /// If `n > 0` then the returned polynomial is monic.
    #[must_use]
    pub fn random(n: usize) -> Self {
        // BitPolynomial of degree `n` has `n + 1` coefficients.
        let mut coeffs = BitVector::random(n + 1);

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
    /// let p1: BitPolynomial = BitPolynomial::random_seeded(3311, seed);
    /// let p2: BitPolynomial = BitPolynomial::random_seeded(3311, seed);
    /// assert_eq!(p1, p2, "Polynomials with the same seed should be equal");
    /// ```
    #[must_use]
    pub fn random_seeded(n: usize, seed: u64) -> Self {
        // BitPolynomial of degree `n` has `n + 1` coefficients.
        let mut coeffs = BitVector::random_seeded(n + 1, seed);

        // If `n > 0` we want the coefficient of `x^n` to be one for sure. If `n == 0` then a random 0/1 is fine.
        if n > 0 {
            coeffs.set(n, true);
        }
        Self { coeffs }
    }
}

/// Core queries for a bit-polynomial.
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Returns the degree of the bit-polynomial.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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

/// Coefficient read and write methods for bit-polynomials.
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Returns a reference to the coefficients of the bit-polynomial as a read-only borrowed bit-vector.
    ///
    /// # Note
    /// You can use this method to get a reference to the polynomial's coefficients as a bit-vector.
    /// Then all the many read-only methods of the [`BitVector`] type are available.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
    /// assert_eq!(p.coefficients().count_ones(), 3);
    /// assert_eq!(p.coefficients().last_set(), Some(4));
    /// ```
    #[must_use]
    #[inline]
    pub fn coefficients(&self) -> &BitVector<Word> { &self.coeffs }

    /// Returns a reference to the coefficients of the bit-polynomial as a read-write borrowed bit-vector.
    ///
    /// # Note
    /// You can use this method to get a reference to the polynomial's coefficients as a bit-vector.
    /// Then all the many read-only methods of the [`BitVector`] type are available.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let mut p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
    /// p.coefficients_mut().set_all(true);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5");
    /// ```
    #[must_use]
    #[inline]
    pub fn coefficients_mut(&mut self) -> &mut BitVector<Word> { &mut self.coeffs }

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
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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
    /// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
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
    /// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
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
    /// let coeffs: BitVector = BitVector::from_string("111010").unwrap();
    /// let mut p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let mut p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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

/// Methods to extract pieces of a bit-polynomial.
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Makes the destination bit-polynomial a copy of the low `d + 1` coefficients of this bit-polynomial.
    /// The destination bit-polynomial will have degree at most `d`.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::ones(6);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// let mut q: BitPolynomial = BitPolynomial::zero();
    /// p.sub_polynomial_into(4, &mut q);
    /// assert_eq!(q.to_string(), "1 + x + x^2 + x^3 + x^4");
    /// p.sub_polynomial_into(6, &mut q);
    /// assert_eq!(q.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// p.sub_polynomial_into(16, &mut q);
    /// assert_eq!(q.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// p.sub_polynomial_into(0, &mut q);
    /// assert_eq!(q.to_string(), "1");
    /// ```
    pub fn sub_polynomial_into(&self, d: usize, dst: &mut BitPolynomial<Word>) {
        dst.resize((d + 1).min(self.coeffs.len()));
        if d == 0 {
            *dst = BitPolynomial::<Word>::constant(self.coeffs.get(0));
        }
        else if d + 1 >= self.coeffs.len() {
            *dst = self.clone();
        }
        else {
            dst.coeffs.copy_store(&self.coeffs.slice(0..d + 1));
        }
    }

    /// Returns a new bit-polynomial that is a copy of the low `d + 1` coefficients of this bit-polynomial.
    /// The destination bit-polynomial will have degree at most `d`.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::ones(6);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// let q = p.sub_polynomial(4);
    /// assert_eq!(q.to_string(), "1 + x + x^2 + x^3 + x^4");
    /// let q = p.sub_polynomial(6);
    /// assert_eq!(q.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// let q = p.sub_polynomial(16);
    /// assert_eq!(q.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// let q = p.sub_polynomial(0);
    /// assert_eq!(q.to_string(), "1");
    /// ```
    pub fn sub_polynomial(&self, d: usize) -> BitPolynomial<Word> {
        let mut dst = BitPolynomial::<Word>::zero();
        self.sub_polynomial_into(d, &mut dst);
        dst
    }

    /// Splits the bit-polynomial into a low and high part where the low part has degree at most `d`.
    /// On return `self(x) = low(x) + x^(d+1) * high(x)`.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::ones(6);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// let mut low: BitPolynomial = BitPolynomial::zero();
    /// let mut high: BitPolynomial = BitPolynomial::zero();
    /// p.split_into(4, &mut low, &mut high);
    /// assert_eq!(low.to_string(), "1 + x + x^2 + x^3 + x^4");
    /// assert_eq!(high.to_string(), "1 + x");
    /// p.split_into(6, &mut low, &mut high);
    /// assert_eq!(low.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// assert_eq!(high.to_string(), "0");
    /// p.split_into(0, &mut low, &mut high);
    /// assert_eq!(low.to_string(), "1");
    /// assert_eq!(high.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5");
    /// ```
    pub fn split_into(&self, d: usize, lo: &mut BitPolynomial<Word>, hi: &mut BitPolynomial<Word>) {
        let len = self.coeffs.len();
        let split = (d + 1).min(len);
        let hi_len = len.saturating_sub(d + 1);

        lo.resize(split);
        hi.resize(hi_len);

        if split > 0 {
            lo.coeffs.copy_store(&self.coeffs.slice(0..split));
        }
        if hi_len > 0 {
            hi.coeffs.copy_store(&self.coeffs.slice(d + 1..len));
        }
    }

    /// Splits the bit-polynomial into a low and high part where the low part has degree at most `d`.
    /// Returns `(low, high)` such that `self(x) = low(x) + x^(d+1) * high(x)`.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::ones(6);
    /// assert_eq!(p.to_string(), "1 + x + x^2 + x^3 + x^4 + x^5 + x^6");
    /// let (low, high) = p.split(4);
    /// assert_eq!(low.to_string(), "1 + x + x^2 + x^3 + x^4");
    /// assert_eq!(high.to_string(), "1 + x");
    /// ```
    pub fn split(&self, d: usize) -> (BitPolynomial<Word>, BitPolynomial<Word>) {
        let mut lo = BitPolynomial::<Word>::zero();
        let mut hi = BitPolynomial::<Word>::zero();
        self.split_into(d, &mut lo, &mut hi);
        (lo, hi)
    }
}

/// Arithmetic methods for bit-polynomials.
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Adds another  bit-polynomial to `self` in-place.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// let q: BitPolynomial = BitPolynomial::x_to_the(2);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.plus_eq(&q);
    /// assert_eq!(q.to_string(), "x^2");
    /// assert_eq!(p.to_string(), "x^2 + x^3");
    /// ```
    pub fn plus_eq(&mut self, rhs: &BitPolynomial<Word>) {
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
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// let q: BitPolynomial = BitPolynomial::x_to_the(2);
    /// let r = p.plus(&q);
    /// assert_eq!(r.to_string(), "x^2 + x^3");
    /// ```
    #[must_use]
    pub fn plus(&self, rhs: &BitPolynomial<Word>) -> BitPolynomial<Word> {
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
    /// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// let q: BitPolynomial = BitPolynomial::x_to_the(2);
    /// assert_eq!(p.to_string(), "x^3");
    /// p.minus_eq(&q);
    /// assert_eq!(q.to_string(), "x^2");
    /// assert_eq!(p.to_string(), "x^2 + x^3");
    /// ```
    pub fn minus_eq(&mut self, rhs: &BitPolynomial<Word>) { self.plus_eq(rhs) }

    /// Returns a new bit-polynomial that is this bit-polynomial minus another.
    ///
    /// # Note
    /// In GF(2), addition and subtraction are the same operation.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// let q: BitPolynomial = BitPolynomial::x_to_the(2);
    /// let r = p.minus(&q);
    /// assert_eq!(r.to_string(), "x^2 + x^3");
    /// ```
    #[must_use]
    pub fn minus(&self, rhs: &BitPolynomial<Word>) -> BitPolynomial<Word> {
        let mut result = self.clone();
        result.plus_eq(rhs);
        result
    }

    /// Fills `dst` with the square of `self`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVector = BitVector::from_string("111").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
    /// assert_eq!(p.to_string(), "1 + x + x^2");
    /// let mut q: BitPolynomial = BitPolynomial::new();
    /// p.square_into(&mut q);
    /// assert_eq!(q.to_string(), "1 + x^2 + x^4");
    /// ```
    pub fn square_into(&self, dst: &mut BitPolynomial<Word>) {
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
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// assert_eq!(p.to_string(), "x^3");
    /// let q: BitPolynomial = p.squared();
    /// assert_eq!(q.to_string(), "x^6");
    /// ```
    #[inline]
    #[must_use]
    pub fn squared(&self) -> Self {
        let mut dst = BitPolynomial::new();
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
    /// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
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
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
    /// let q: BitPolynomial = BitPolynomial::x_to_the(2);
    /// let r = p.convolved_with(&q);
    /// assert_eq!(p.to_string(), "x^3");
    /// assert_eq!(q.to_string(), "x^2");
    /// assert_eq!(r.to_string(), "x^5");
    /// ```
    #[must_use]
    pub fn convolved_with(&self, rhs: &BitPolynomial<Word>) -> Self {
        // Edge case: either polynomial is the zero polynomial.
        if self.is_zero() || rhs.is_zero() {
            return BitPolynomial::zero();
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
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Evaluates the polynomial for a scalar `bool` argument.
    ///
    /// # Note
    /// - Ideally this should be implemented via the `Fn` trait but the necessary machinery is not yet available in
    ///   stable Rust. If unstable features are enabled then we have a `Fn` implementation that forwards to this method.
    /// - The `BitMatrix` module has a `eval_matrix` method to compute `p(M)` where `M` is a bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
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

    /// Evaluates the bit-polynomial for a square [`BitMatrix`] argument.
    ///
    /// Uses Horner's method to evaluate `p(M)` where `M` is a square matrix and returns the result as a bit-matrix.
    ///
    /// # Panics
    /// Panics if the matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(BitVector::alternating(12));
    /// let m: BitMatrix = BitMatrix::identity(6);
    /// assert_eq!(p.eval_matrix(&m), BitMatrix::zeros(6, 6));
    /// let p = BitPolynomial::from_coefficients(!BitVector::alternating(6));
    /// assert_eq!(p.eval_matrix(&m), BitMatrix::identity(6));
    /// ```
    #[must_use]
    pub fn eval_matrix(&self, mat: &BitMatrix<Word>) -> BitMatrix<Word> {
        // Error case: the matrix is not square.
        assert!(mat.is_square(), "BitMatrix must be square not {}x{}", mat.rows(), mat.cols());

        // Edge case: the zero polynomial.
        if self.is_zero() {
            return BitMatrix::zeros(mat.rows(), mat.cols());
        }

        // Otherwise we start with the identity matrix.
        let mut result = BitMatrix::identity(mat.rows());

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
impl<Word: Unsigned> BitPolynomial<Word> {
    /// Returns a readable "full" string for the bit-polynomial in terms of the default "variable" name `x`.
    ///
    /// # Note
    /// We show all terms including those with zero coefficients.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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
    /// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
    /// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
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
impl<Word: Unsigned> BitPolynomial<Word> {
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
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
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
    /// let p: BitPolynomial = BitPolynomial::x_to_the(6);
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
    /// let p: BitPolynomial = BitPolynomial::x_to_the(3);
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
        let p: BitVector<Word> = self.coeffs.slice(0..d).into();

        // Closure that computes q(x) -> x * q(x) mod P(x) where degree[q] < d.
        // The closure works on the coefficients of q(x) passed as the bit-vector `q`.
        let times_x_step = |q: &mut BitVector<Word>| {
            let add_p = q[d - 1];
            *q >>= 1;
            if add_p {
                *q ^= &p;
            }
        };

        // Iteratively precompute x^{d+i} mod P(x) for i = 0, 1, ..., d-1 starting with x^d mod P(x) ~ p.
        // We store all the bit-vectors in a standard `Vec` of length d.
        let mut power_mod = Vec::<BitVector<Word>>::with_capacity(d);
        power_mod.push(p.clone());
        for i in 1..d {
            let mut q = power_mod[i - 1].clone();
            times_x_step(&mut q);
            power_mod.push(q);
        }

        // Create some workspace for the reduction.
        let mut s = BitVector::zeros(2 * d);
        let mut h = BitVector::zeros(d);

        // Closure that computes q(x) -> q(x)^2 mod P(x) where degree[q] < d.
        // The closure works on the coefficients of q(x) passed as the bit-vector `q`.
        let mut square_step = |q: &mut BitVector<Word>| {
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
            let mut r = BitVector::zeros(d);
            r.set(1, true);

            // Squaring, r(x) = x mod P(x) -> x^2 mod P(x) -> x^4 mod P(x) ... we get to x^(2^n) mod P(x).
            for _ in 0..n {
                square_step(&mut r);
            }
            return Self::from_coefficients(r);
        }

        // Normal small exponent case: n < d => x^n mod P(x) = x^n.
        if n < d {
            return BitPolynomial::x_to_the(n);
        }

        // Matching power case: n == d => x^n mod P(x) = p(x)
        if n == d {
            return Self::from_coefficients(p);
        }

        // Larger power case: n > d: Multiply & square until we get to x^n mod P(x).
        // Note that if e.g. n = 0b00010111 then n.prev_power_of_two() = 0b00010000.
        let mut n_bit = n.prev_power_of_two();

        // Returning r(x) where degree[r] < d so r(x) = r_0 + r_1 x + ... + r_{d-1} x^{d-1} has d coefficients.
        let mut r = BitVector::zeros(d);

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
// The `Default` trait implementation for the `BitPolynomial` type.
// --------------------------------------------------------------------------------------------------------------------

/// Implement the `Default` constructor trait for the `BitPolynomial` type.
///
/// The default constructor creates an empty polynomial which is one form of the zero polynomial.
/// No capacity is reserved until coefficients are added.
///
/// # Examples
/// ```
/// use gf2::*;
/// let p: BitPolynomial = Default::default();
/// assert_eq!(p.to_string(), "0");
/// ```
impl<Word: Unsigned> Default for BitPolynomial<Word> {
    #[inline]
    fn default() -> Self { Self::new() }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Index` trait implementation for the `BitPolynomial` type.
// --------------------------------------------------------------------------------------------------------------------

/// The `Index` trait implementation for the `BitPolynomial` type.
///
/// Returns the coefficient of `x^i`.
///
/// # Panics
/// In debug mode, panics if `i` is out of bounds.
///
/// # Examples
/// ```
/// use gf2::*;
/// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
/// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
/// assert_eq!(p[2], true);
/// assert_eq!(p[3], false);
/// ```
impl<Word: Unsigned> Index<usize> for BitPolynomial<Word> {
    type Output = bool;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output { if self.coeffs[i] { &true } else { &false } }
}

// --------------------------------------------------------------------------------------------------------------------
// Various `Display`-like trait implementations for the `BitPolynomial` type.
// --------------------------------------------------------------------------------------------------------------------

/// The `fmt::Display` trait implementation for the `BitPolynomial` type.
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
/// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
/// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
/// assert_eq!(format!("{p}"), "1 + x^2 + x^4");
/// assert_eq!(format!("{p:#}"), "1 + 0x + x^2 + 0x^3 + x^4 + 0x^5");
/// ```
impl<Word: Unsigned> fmt::Display for BitPolynomial<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.to_full_string_with_var("x"))
        }
        else {
            write!(f, "{}", self.to_string_with_var("x"))
        }
    }
}

/// The `fmt::Debug` trait implementation for the `BitPolynomial` type.
///
/// Returns a debug string representation of the bit-polynomial.
///
/// # Note
/// The string representation is always in terms of a variable `x` and we show all terms with zero coefficients.
///
/// # Examples
/// ```
/// use gf2::*;
/// let coeffs: BitVector = BitVector::from_string("101010").unwrap();
/// let p: BitPolynomial = BitPolynomial::from_coefficients(coeffs);
/// assert_eq!(format!("{p:?}"), "1 + 0x + x^2 + 0x^3 + x^4 + 0x^5");
/// ```
impl<Word: Unsigned> fmt::Debug for BitPolynomial<Word> {
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

/// The `AddAssign` trait implementation for a `BitPolynomial` value and a `BitPolynomial` reference.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// let q: BitPolynomial = BitPolynomial::x_to_the(2);
/// assert_eq!(p.to_string(), "x^3");
/// p += &q;
/// assert_eq!(q.to_string(), "x^2");
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> AddAssign<&BitPolynomial<Word>> for BitPolynomial<Word> {
    #[inline]
    fn add_assign(&mut self, rhs: &BitPolynomial<Word>) { self.plus_eq(rhs); }
}

/// The `AddAssign` trait implementation for a `BitPolynomial` value and a `BitPolynomial` value.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// assert_eq!(p.to_string(), "x^3");
/// p += BitPolynomial::x_to_the(2);
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> AddAssign<BitPolynomial<Word>> for BitPolynomial<Word> {
    #[inline]
    fn add_assign(&mut self, rhs: BitPolynomial<Word>) { self.plus_eq(&rhs); }
}

/// The `SubAssign` trait implementation for a `BitPolynomial` value and a `BitPolynomial` reference.
///
/// # Note
/// In GF(2), addition and subtraction are the same operation.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// let q: BitPolynomial = BitPolynomial::x_to_the(2);
/// assert_eq!(p.to_string(), "x^3");
/// p -= &q;
/// assert_eq!(q.to_string(), "x^2");
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> SubAssign<&BitPolynomial<Word>> for BitPolynomial<Word> {
    #[inline]
    fn sub_assign(&mut self, rhs: &BitPolynomial<Word>) { self.plus_eq(rhs); }
}

/// The `SubAssign` trait implementation for a `BitPolynomial` with another `BitPolynomial` value.
///
/// # Note
/// - In GF(2) subtraction is the same as addition.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// assert_eq!(p.to_string(), "x^3");
/// p -= BitPolynomial::x_to_the(2);
/// assert_eq!(p.to_string(), "x^2 + x^3");
/// ```
impl<Word: Unsigned> SubAssign<BitPolynomial<Word>> for BitPolynomial<Word> {
    #[inline]
    fn sub_assign(&mut self, rhs: BitPolynomial<Word>) { self.plus_eq(&rhs); }
}

/// The `MulAssign` trait implementation for a `BitPolynomial` value and a `BitPolynomial` reference.
///
/// Multiplying polynomials is achieved by convolving their coefficient vectors.
///
/// # Note
/// This does not consume the right-hand side but it must be called using the `&` operator.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// let q: BitPolynomial = BitPolynomial::x_to_the(2);
/// assert_eq!(p.to_string(), "x^3");
/// assert_eq!(q.to_string(), "x^2");
/// p *= &q;
/// assert_eq!(p.to_string(), "x^5");
/// ```
impl<Word: Unsigned> MulAssign<&BitPolynomial<Word>> for BitPolynomial<Word> {
    #[inline]
    fn mul_assign(&mut self, rhs: &BitPolynomial<Word>) {
        let result = self.convolved_with(rhs);
        *self = result;
    }
}

/// The `MulAssign` trait implementation for two `BitPolynomial` values.
///
/// # Note
/// This consumes the right-hand side.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// assert_eq!(p.to_string(), "x^3");
/// p *= BitPolynomial::x_to_the(2);
/// assert_eq!(p.to_string(), "x^5");
/// ```
impl<Word: Unsigned> MulAssign<BitPolynomial<Word>> for BitPolynomial<Word> {
    #[inline]
    fn mul_assign(&mut self, rhs: BitPolynomial<Word>) {
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
impl<Word: Unsigned> Add<&BitPolynomial<Word>> for &BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn add(self, rhs: &BitPolynomial<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs + rhs` as new bit-polynomial consuming `rhs`.
impl<Word: Unsigned> Add<&BitPolynomial<Word>> for BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn add(self, rhs: &BitPolynomial<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs + &rhs` as new bit-polynomial consuming `lhs`.
impl<Word: Unsigned> Add<BitPolynomial<Word>> for &BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn add(self, rhs: BitPolynomial<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs + rhs` as new bit-polynomial consuming both operands.
impl<Word: Unsigned> Add<BitPolynomial<Word>> for BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn add(self, rhs: BitPolynomial<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs - &rhs` as new bit-polynomial
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<&BitPolynomial<Word>> for &BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn sub(self, rhs: &BitPolynomial<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs - rhs` as new bit-polynomial consuming `rhs`.
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<&BitPolynomial<Word>> for BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn sub(self, rhs: &BitPolynomial<Word>) -> Self::Output { self.plus(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs - &rhs` as new bit-polynomial consuming `lhs`.
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<BitPolynomial<Word>> for &BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn sub(self, rhs: BitPolynomial<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs - rhs` as new bit-polynomial consuming both operands.
/// Note that subtraction in GF(2) is equivalent to addition.
impl<Word: Unsigned> Sub<BitPolynomial<Word>> for BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn sub(self, rhs: BitPolynomial<Word>) -> Self::Output { self.plus(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs * &rhs` as new bit-polynomial
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<&BitPolynomial<Word>> for &BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn mul(self, rhs: &BitPolynomial<Word>) -> Self::Output { self.convolved_with(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `&lhs * rhs` as new bit-polynomial consuming `rhs`.
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<&BitPolynomial<Word>> for BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn mul(self, rhs: &BitPolynomial<Word>) -> Self::Output { self.convolved_with(rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs * &rhs` as new bit-polynomial consuming `lhs`.
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<BitPolynomial<Word>> for &BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn mul(self, rhs: BitPolynomial<Word>) -> Self::Output { self.convolved_with(&rhs) }
}

/// If `lhs` and `rhs` are bit-polynomials, this returns `lhs * rhs` as new bit-polynomial consuming both operands.
/// Polynomial multiplication is performed using convolutions.
impl<Word: Unsigned> Mul<BitPolynomial<Word>> for BitPolynomial<Word> {
    type Output = BitPolynomial<Word>;

    #[inline]
    fn mul(self, rhs: BitPolynomial<Word>) -> Self::Output { self.convolved_with(&rhs) }
}

// --------------------------------------------------------------------------------------------------------------------
// If the compiler supports the `unboxed_closures` & `fn_traits` features, we can use the `BitPolynomial` type as a
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

/// The `Fn` trait implementation for the `BitPolynomial` type with a `bool` argument.
///
/// # Note
/// Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let p: BitPolynomial = BitPolynomial::x_to_the(3);
/// assert_eq!(p(true), true);
/// assert_eq!(p(false), false);
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> Fn<(bool,)> for BitPolynomial<Word> {
    extern "rust-call" fn call(&self, args: (bool,)) -> Self::Output { self.eval_bool(args.0) }
}

/// The `FnMut` trait implementation for the `BitPolynomial` type with a `bool` argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnMut` as a super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// assert_eq!(p(true), true);
/// assert_eq!(p(false), false);
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnMut<(bool,)> for BitPolynomial<Word> {
    extern "rust-call" fn call_mut(&mut self, args: (bool,)) -> Self::Output { self.eval_bool(args.0) }
}

/// The `FnOnce` trait implementation for the `BitPolynomial` type with a `bool` argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnOnce` as a super-super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// assert_eq!(p(true), true);
/// assert_eq!(p(false), false);
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnOnce<(bool,)> for BitPolynomial<Word> {
    type Output = bool;

    extern "rust-call" fn call_once(self, args: (bool,)) -> Self::Output { self.eval_bool(args.0) }
}

/// The `Fn` trait implementation for the `BitPolynomial` type with a `BitMatrix` reference argument.
///
/// # Note
/// Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let p: BitPolynomial = BitPolynomial::x_to_the(3);
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(p(&m), BitMatrix::identity(3));
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> Fn<(&BitMatrix<Word>,)> for BitPolynomial<Word> {
    extern "rust-call" fn call(&self, args: (&BitMatrix<Word>,)) -> Self::Output { self.eval_matrix(args.0) }
}

/// The `FnMut` trait implementation for the `BitPolynomial` type with a `BitMatrix` reference argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnMut` as a super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(p(&m), BitMatrix::identity(3));
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnMut<(&BitMatrix<Word>,)> for BitPolynomial<Word> {
    extern "rust-call" fn call_mut(&mut self, args: (&BitMatrix<Word>,)) -> Self::Output { self.eval_matrix(args.0) }
}

/// The `FnOnce` trait implementation for the `BitPolynomial` type with a `BitMatrix` reference argument.
///
/// # Note
/// - We really only care about the `Fn` trait, but it has `FnOnce` as a super-super-trait.
/// - Currently (rust 1.87.0) this requires unstable features (nightly toolchain).
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut p: BitPolynomial = BitPolynomial::x_to_the(3);
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(p(&m), BitMatrix::identity(3));
/// ```
#[cfg(feature = "unstable")]
impl<Word: Unsigned> FnOnce<(&BitMatrix<Word>,)> for BitPolynomial<Word> {
    type Output = BitMatrix<Word>;

    extern "rust-call" fn call_once(self, args: (&BitMatrix<Word>,)) -> Self::Output { self.eval_matrix(args.0) }
}
