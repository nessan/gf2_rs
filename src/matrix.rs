//! [`BitMatrix`] is a matrix over GF(2) --- a _bit-matrix_.

// Crate imports.
use crate::{
    BitGauss,
    BitLU,
    BitPolynomial,
    BitSlice,
    BitStore,
    BitVector,
    Unsigned,
    rng,
};

// BitArray requires unstable features.
#[cfg(feature = "unstable")]
use crate::array::BitArray;

// Standard library imports.
use core::f64;
use std::{
    fmt,
    fmt::Write,
    ops::{
        Add,
        AddAssign,
        BitAnd,
        BitAndAssign,
        BitOr,
        BitOrAssign,
        BitXor,
        BitXorAssign,
        Bound,
        Index,
        IndexMut,
        Mul,
        MulAssign,
        Not,
        RangeBounds,
        Sub,
        SubAssign,
    },
};

#[doc = include_str!("../docs/matrix.md")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BitMatrix<Word: Unsigned = usize> {
    /// The rows of the bit-matrix stored as a vector of bit-vectors.
    m_rows: Vec<BitVector<Word>>,
}

/// Constructors for general rectangular `r x c` bit-matrices.
impl<Word: Unsigned> BitMatrix<Word> {
    /// The default constructor creates an empty bit-matrix. <br>
    /// No capacity is reserved until elements are added.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.rows(), 0);
    /// assert_eq!(m.cols(), 0);
    /// ```
    #[must_use]
    #[inline]
    pub fn new() -> Self { Self { m_rows: Vec::new() } }

    /// Constructs a bit-matrix with `r` rows and `c` columns, initializing all elements to zero.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::zeros(3, 2);
    /// assert_eq!(m.to_compact_binary_string(), "00 00 00");
    /// ```
    #[must_use]
    #[inline]
    pub fn zeros(r: usize, c: usize) -> Self { Self { m_rows: vec![BitVector::zeros(c); r] } }

    /// Constructs a square bit-matrix with `n` rows and columns, initializing all elements to zero.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.to_compact_binary_string(), "000 000 000");
    /// ```
    #[must_use]
    #[inline]
    pub fn square(n: usize) -> Self { Self::zeros(n, n) }

    /// Constructs a bit-matrix with `r` rows and `c` columns, initializing all elements to one.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::ones(3, 2);
    /// assert_eq!(m.to_compact_binary_string(), "11 11 11");
    /// ```
    #[must_use]
    #[inline]
    pub fn ones(r: usize, c: usize) -> Self { Self { m_rows: vec![BitVector::ones(c); r] } }

    /// Constructs a bit-matrix with an alternating pattern of `1`s and `0`s.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::alternating(3, 5);
    /// assert_eq!(m.to_compact_binary_string(), "10101 01010 10101");
    /// ```
    #[must_use]
    pub fn alternating(r: usize, c: usize) -> Self {
        let mut result = Self { m_rows: vec![BitVector::alternating(c); r] };
        // Flip every other row.
        for i in (1..r).step_by(2) {
            result.m_rows[i].flip_all();
        }
        result
    }

    /// Constructs an `r` x `c` bit-matrix from the *outer product* of two bit-vectors.
    ///
    /// The outer product of two bit-vectors `a` and `b` is the bit-matrix `M` where `M[i, j] = a[i] & b[j]`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let a: BitVector = BitVector::from_binary_string("101").unwrap();
    /// let b: BitVector = BitVector::from_binary_string("110").unwrap();
    /// let m: BitMatrix = BitMatrix::from_outer_product(&a, &b);
    /// assert_eq!(m.to_compact_binary_string(), "110 000 110");
    /// ```
    #[must_use]
    pub fn from_outer_product(a: &BitVector<Word>, b: &BitVector<Word>) -> Self {
        let r = a.len();
        let c = b.len();
        let mut result = Self::zeros(r, c);
        for i in 0..r {
            if a[i] {
                result.m_rows[i].copy_store(b);
            }
        }
        result
    }

    /// Constructs an `r` x `c` bit-matrix from the *outer sum* of two bit-vectors.
    ///
    /// The outer sum of two bit-vectors `a` and `b` is the bit-matrix `M` where `M[i, j] = a[i] | b[j]`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let a: BitVector = BitVector::from_binary_string("101").unwrap();
    /// let b: BitVector = BitVector::from_binary_string("110").unwrap();
    /// let m: BitMatrix = BitMatrix::from_outer_sum(&a, &b);
    /// assert_eq!(m.to_compact_binary_string(), "001 110 001");
    /// ```
    #[must_use]
    pub fn from_outer_sum(a: &BitVector<Word>, b: &BitVector<Word>) -> Self {
        let r = a.len();
        let c = b.len();
        let mut result = Self::zeros(r, c);
        for i in 0..r {
            result.m_rows[i].copy_store(b);
            if a[i] {
                result.m_rows[i].flip_all();
            }
        }
        result
    }

    /// Constructs a bit-matrix with `r` rows and `c` columns by calling a function `f(i, j)` for each element.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::from_fn(3, 2, |i, _| i % 2 == 0);
    /// assert_eq!(m.to_compact_binary_string(), "11 00 11");
    /// ```
    #[must_use]
    pub fn from_fn(r: usize, c: usize, f: impl Fn(usize, usize) -> bool) -> Self {
        let mut result = Self::zeros(r, c);
        for i in 0..r {
            for j in 0..c {
                if f(i, j) {
                    result.set(i, j, true);
                }
            }
        }
        result
    }
}

/// Constructors for general rectangular `r x c` bit-matrices with random fills.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Constructs a random bit-matrix with `r` rows and `c` columns where each element is set with probability `p`, and
    /// the RNG is seeded to `seed`. A seed of `0` indicates we should randomly seed the RNG.
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-matrix with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let M0: BitMatrix = BitMatrix::random_biased_seeded(50, 50, 1.2, 42); // All bits set
    /// assert_eq!(M0.count_ones(), 2500);
    /// let M1: BitMatrix = BitMatrix::random_biased_seeded(50, 50, 0.75, 42);
    /// let M2: BitMatrix = BitMatrix::random_biased_seeded(50, 50, 0.75, 42);
    /// assert_eq!(M1, M2);
    /// ```
    #[must_use]
    pub fn random_biased_seeded(r: usize, c: usize, p: f64, seed: u64) -> Self {
        // Note: Need `LazyLock` to make `TWO_POWER_64` `static` as `powi` is not `const`.
        static TWO_POWER_64: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| 2.0_f64.powi(64));

        // Edge cases:
        if r == 0 || c == 0 {
            return Self::new();
        }
        if p <= 0.0 {
            return Self::zeros(r, c);
        }
        if p >= 1.0 {
            return Self::ones(r, c);
        }

        // If given a non-zero seed we need to save and restore the old seed.
        let old_seed = rng::seed();
        if seed != 0 {
            rng::set_seed(seed);
        }

        // Scale p by 2^64 to remove floating point arithmetic from the main loop below.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let scaled_p = (*TWO_POWER_64 * p) as u64;
        let mut result = Self::zeros(r, c);
        for i in 0..r {
            for j in 0..c {
                if rng::u64() < scaled_p {
                    result.set(i, j, true);
                }
            }
        }

        // Restore the old RNG seed.
        if seed != 0 {
            rng::set_seed(old_seed);
        }

        result
    }

    /// Constructs a random bit-matrix with `r` rows and `c` columns where each element is set/unset with probability
    /// 50/50.
    ///
    /// The random number generator is seeded on first use with a scrambled version of the current time so you get
    /// different outputs for each run.
    ///
    /// See the `random_seeded` method for a way to get reproducible randomly filled bit-matrices.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::random(3, 5);
    /// assert_eq!(m.rows(), 3);
    /// assert_eq!(m.cols(), 5);
    /// ```
    #[must_use]
    pub fn random(r: usize, c: usize) -> Self { Self::random_biased_seeded(r, c, 0.5, 0) }

    /// Constructs a random bit-matrix with `r` rows and `c` columns where each element is set/unset with probability
    /// `p`/`(1-p)`.
    ///
    /// For reproducibility, the random number generator is seeded with the specified `seed` and then reset to the
    /// previous seed after the bit-matrix is constructed.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::random_seeded(3, 5, 42);
    /// let m2: BitMatrix = BitMatrix::random_seeded(3, 5, 42);
    /// assert_eq!(m1, m2);
    /// ```
    #[must_use]
    pub fn random_seeded(r: usize, c: usize, seed: u64) -> Self { Self::random_biased_seeded(r, c, 0.5, seed) }

    /// Constructs an `r` x `c` bit-matrix where each element is set/unset with probability `p`/`(1-p)`.
    ///
    /// The random number generator is seeded on first use with a scrambled version of the current time so you get
    /// different outputs for each run.
    ///
    /// See the `random_seeded` method for a way to get reproducible randomly filled bit-matrices.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::random_biased(3, 5, 0.3);
    /// assert_eq!(m.rows(), 3);
    /// assert_eq!(m.cols(), 5);
    /// ```
    #[must_use]
    pub fn random_biased(r: usize, c: usize, p: f64) -> Self { Self::random_biased_seeded(r, c, p, 0) }
}

/// Constructors for some "special" square bit-matrices.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Constructs the n x n zero matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::zeros(3, 3);
    /// assert_eq!(m.to_compact_binary_string(), "000 000 000");
    /// ```
    #[must_use]
    #[inline]
    pub fn zero(n: usize) -> Self { Self::zeros(n, n) }

    /// Constructs the n x n identity matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(4);
    /// assert_eq!(m.to_compact_binary_string(), "1000 0100 0010 0001");
    /// ```
    #[must_use]
    #[inline]
    pub fn identity(n: usize) -> Self {
        let mut result = Self::zeros(n, n);
        for i in 0..n {
            result.set(i, i, true);
        }
        result
    }

    /// Constructs the n x n shift-left by `p` places matrix.
    ///
    /// If the returned matrix is multiplied by a bit-vector, the result is the bit-vector shifted left by `p` places.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::left_shift(5, 2);
    /// let v: BitVector = BitVector::ones(5);
    /// assert_eq!((&m * &v).to_string(), "11100");
    /// ```
    #[must_use]
    pub fn left_shift(n: usize, p: usize) -> Self {
        let mut result = Self::zeros(n, n);
        result.set_super_diagonal(p, true);
        result
    }

    /// Constructs the n x n shift-right by `p` places matrix.
    ///
    /// If the returned matrix is multiplied by a bit-vector, the result is the bit-vector shifted right by `p` places.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::right_shift(5, 2);
    /// let v: BitVector = BitVector::ones(5);
    /// assert_eq!((&m * &v).to_string(), "00111");
    /// ```
    #[must_use]
    pub fn right_shift(n: usize, p: usize) -> Self {
        let mut result = Self::zeros(n, n);
        result.set_sub_diagonal(p, true);
        result
    }

    /// Constructs the n x n rotate-left by `p` places matrix.
    ///
    /// If the returned matrix is multiplied by a bit-vector, the result is the bit-vector rotated left by `p` places.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::left_rotation(5, 2);
    /// let v: BitVector = BitVector::from_binary_string("11100").unwrap();
    /// assert_eq!((&m * &v).to_string(), "00111");
    /// ```
    #[must_use]
    pub fn left_rotation(n: usize, p: usize) -> Self {
        let mut result = Self::zeros(n, n);
        for i in 0..n {
            let j = (i + n - p) % n;
            result.set(i, j, true);
        }
        result
    }

    /// Constructs the n x n rotate-right by `p` places matrix.
    ///
    /// If the returned matrix is multiplied by a bit-vector, the result is the bit-vector rotated right by `p` places.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::right_rotation(5, 2);
    /// let v: BitVector = BitVector::from_binary_string("11100").unwrap();
    /// assert_eq!((&m * &v).to_string(), "10011");
    /// ```
    #[must_use]
    pub fn right_rotation(n: usize, p: usize) -> Self {
        let mut result = Self::zeros(n, n);
        for i in 0..n {
            let j = (i + p) % n;
            result.set(i, j, true);
        }
        result
    }

    /// Constructs a square *companion matrix* with a copy of the given top row and a sub-diagonal of `1`s.
    ///
    /// The top row should be passed as a bit-vector or slice and is copied to the first row of the matrix and the
    /// sub-diagonal is set to `1`s.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let top_row: BitVector = BitVector::from_binary_string("10101").unwrap();
    /// let m: BitMatrix = BitMatrix::companion(&top_row);
    /// assert_eq!(m.to_compact_binary_string(), "10101 10000 01000 00100 00010");
    /// ```
    #[must_use]
    pub fn companion<Src: BitStore<Word>>(top_row: &Src) -> Self {
        // Edge case:
        if top_row.len() == 0 {
            return Self::new();
        }
        let mut result = Self::zero(top_row.len());
        result.m_rows[0].copy_store(top_row);
        result.set_sub_diagonal(1, true);
        result
    }
}

/// Bit-matrix constructors that can fail.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Attempts to construct a bit-matrix by reshaping a bit-vector that is assumed to be a sequence of `r` rows.
    ///
    /// On success, the output bit-matrix will have `r` rows and `c` columns where `c` is the integer `src.len() / r`.
    /// If `r` does not divide `src.len()` evenly, we will return `None`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::ones(15);
    /// let m: BitMatrix = BitMatrix::from_vector_of_rows(&v, 3).unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "11111 11111 11111");
    /// let m: BitMatrix = BitMatrix::from_vector_of_rows(&v, 5).unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "111 111 111 111 111");
    /// let m: BitMatrix = BitMatrix::from_vector_of_rows(&v, 15).unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "1 1 1 1 1 1 1 1 1 1 1 1 1 1 1");
    /// ```
    #[must_use]
    pub fn from_vector_of_rows(src: &BitVector<Word>, r: usize) -> Option<Self> {
        // Edge case:
        if src.len() == 0 {
            return Some(Self::new());
        }

        // Error case:
        if r == 0 || src.len() % r != 0 {
            return None;
        }

        let c = src.len() / r;
        let mut result = Self::zeros(r, c);
        for i in 0..r {
            let start = i * c;
            let end = start + c;
            result.m_rows[i].copy_store(&src.slice(start..end));
        }
        Some(result)
    }

    /// Attempts to construct a bit-matrix by reshaping a bit-vector that is assumed to be a sequence of `c` columns.
    ///
    /// On success, the output bit-matrix will have `r` rows and `c` columns where `r` is the integer `src.len() / c`.
    /// If `c` does not divide `src.len()` evenly, we will return `None`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::ones(15);
    /// let m: BitMatrix = BitMatrix::from_vector_of_cols(&v, 3).unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "111 111 111 111 111");
    /// let m: BitMatrix = BitMatrix::from_vector_of_cols(&v, 5).unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "11111 11111 11111");
    /// let m: BitMatrix = BitMatrix::from_vector_of_cols(&v, 15).unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "111111111111111");
    /// ```
    #[must_use]
    pub fn from_vector_of_cols(src: &BitVector<Word>, c: usize) -> Option<Self> {
        // Edge case:
        if src.len() == 0 {
            return Some(Self::new());
        }

        // Error case:
        if c == 0 || src.len() % c != 0 {
            return None;
        }

        let r = src.len() / c;
        let mut result = Self::zeros(r, c);
        let mut src_index = 0;
        for j in 0..c {
            for i in 0..r {
                if src[src_index] {
                    result.set(i, j, true);
                }
                src_index += 1;
            }
        }
        Some(result)
    }

    /// Attempts to construct a bit-matrix from a string returning `None` on failure.
    ///
    /// We assume the matrix is stored by row where the rows are separated by whitespace or semicolons.
    /// Each row should be a binary or hex string representation of a bit-vector and can contain underscore characters.
    /// The rows can have an optional "0b", "0x", or "0X" prefix.
    ///
    /// A hex string can have a suffix of ".2", ".4", or ".8" to indicate the base of the last digit/character.
    /// This allows for rows of any length as opposed to just a multiple of 4.
    ///
    /// After parsing, the rows must all have the same length.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::from_string("111   111\n111").unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "111 111 111");
    /// let m: BitMatrix = BitMatrix::from_string("0XAA; 0b1111_0000").unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "10101010 11110000");
    /// let m: BitMatrix = BitMatrix::from_string("0x7.8 000").unwrap();
    /// assert_eq!(m.to_compact_binary_string(), "111 000");
    /// ```
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        // Edge case: completely empty string.
        if s.is_empty() {
            return Some(Self::new());
        }

        // Get the rows strings which should be separated by whitespace or semicolons (filtering out empty strings).
        let row_strings: Vec<&str> =
            s.split(|c: char| c.is_whitespace() || c == ';').filter(|s| !s.is_empty()).collect();

        // The number of row strings set the number of rows in the matrix, the first row will set the number of cols.
        let n_rows = row_strings.len();
        let mut n_cols = 0;

        // If all goes well we need a bit-matrix to return at the end.
        let mut result = BitMatrix::new();

        // Proceed through the strings representing the rows of the matrix.
        for (i, row_string) in row_strings.iter().enumerate() {
            if let Some(row) = BitVector::<Word>::from_string(row_string) {
                if i == 0 {
                    // First row sets the number of columns.
                    n_cols = row.len();
                    result.resize(n_rows, n_cols);
                }
                else {
                    // Check the rows are all the same length
                    if row.len() != n_cols {
                        return None;
                    }
                }

                // Copy those bits over to our matrix
                result.m_rows[i].copy_store(&row);
            }
            else {
                // Failed to parse the `row_str` into a bit-vector.
                return None;
            }
        }

        // Success
        Some(result)
    }
}

/// Bit-matrix core queries.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns the number of rows in the bit-matrix.
    #[must_use]
    #[inline]
    pub fn rows(&self) -> usize { self.m_rows.len() }

    /// Returns the number of columns in the bit-matrix.
    #[must_use]
    #[inline]
    pub fn cols(&self) -> usize { if self.m_rows.is_empty() { 0 } else { self.m_rows[0].len() } }

    /// Returns the number of elements in the bit-matrix.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize { self.m_rows.len() * self.cols() }

    /// Returns `true` if the bit-matrix has no elements.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool { self.m_rows.is_empty() }
}

/// Methods for checking the state of a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns `true` if any element of the bit-matrix is set.
    ///
    /// # Note
    /// Empty matrices are considered to have no set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.any(), false);
    /// m.set(0, 0, true);
    /// assert_eq!(m.any(), true);
    /// m.clear();
    /// assert_eq!(m.any(), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn any(&self) -> bool { self.m_rows.iter().any(super::store::BitStore::any) }

    /// Returns `true` if all elements of the bit-matrix are set.
    ///
    /// # Note
    /// Empty matrices are considered to have all set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.all(), false);
    /// m.set_all(true);
    /// assert_eq!(m.all(), true);
    /// m.clear();
    /// assert_eq!(m.all(), true);
    /// ```
    #[must_use]
    #[inline]
    pub fn all(&self) -> bool { self.m_rows.iter().all(super::store::BitStore::all) }

    /// Returns `true` if none of the elements of the bit-matrix are set.
    ///
    /// # Note
    /// Empty matrices are considered to have no set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.none(), true);
    /// m.set_all(true);
    /// assert_eq!(m.none(), false);
    /// m.clear();
    /// assert_eq!(m.none(), true);
    /// ```
    #[must_use]
    #[inline]
    pub fn none(&self) -> bool { self.m_rows.iter().all(super::store::BitStore::none) }
}

/// Is this bit-matrix something special?
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns `true` if the bit-matrix is square.
    ///
    /// # Note
    /// Empty matrices are *not* considered square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.is_square(), false);
    /// m.resize(3, 3);
    /// assert_eq!(m.is_square(), true);
    /// m.resize(3, 2);
    /// assert_eq!(m.is_square(), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_square(&self) -> bool { !self.is_empty() && self.rows() == self.cols() }

    /// Returns `true` if this is a square zero matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.is_zero(), false);
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.is_zero(), false);
    /// let m: BitMatrix = BitMatrix::zeros(3, 3);
    /// assert_eq!(m.is_zero(), true);
    /// let m: BitMatrix = BitMatrix::zeros(3, 2);
    /// assert_eq!(m.is_zero(), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_zero(&self) -> bool { self.is_square() && self.none() }

    /// Returns `true` if the bit-matrix is the identity matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.is_identity(), true);
    /// ```
    #[must_use]
    pub fn is_identity(&self) -> bool {
        if !self.is_square() {
            return false;
        }
        for i in 0..self.rows() {
            let mut row = self.m_rows[i].clone();
            row.flip(i);
            if row.any() {
                return false;
            }
        }
        true
    }

    /// Returns `true` if the square bit-matrix is symmetric.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.is_symmetric(), true);
    /// ```
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        if !self.is_square() {
            return false;
        }
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                if self.get(i, j) != self.get(j, i) {
                    return false;
                }
            }
        }
        true
    }
}

/// Set and unset bit counts for a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns the number of ones in the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.count_ones(), 3);
    /// ```
    #[must_use]
    #[inline]
    pub fn count_ones(&self) -> usize { self.m_rows.iter().map(super::store::BitStore::count_ones).sum() }

    /// Returns the number of zeros in the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.count_zeros(), 6);
    /// ```
    #[must_use]
    #[inline]
    pub fn count_zeros(&self) -> usize { self.len() - self.count_ones() }

    /// Returns the number of ones on the main diagonal of the bit-matrix.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.count_ones_on_diagonal(), 3);
    /// ```
    #[must_use]
    pub fn count_ones_on_diagonal(&self) -> usize {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        let mut count = 0;
        for i in 0..self.rows() {
            if self.get(i, i) {
                count += 1;
            }
        }
        count
    }

    /// Returns the "sum" of the main diagonal elements of the bit-matrix.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.trace(), true);
    /// let m: BitMatrix = BitMatrix::identity(4);
    /// assert_eq!(m.trace(), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn trace(&self) -> bool { self.count_ones_on_diagonal() % 2 == 1 }
}

/// Methods for accessing and setting individual elements of a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns `true` if the element at row `r` and column `c` is set.
    ///
    /// # Panics
    /// In debug mode, panics if `r` or `c` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.get(0, 0), false);
    /// m.set(0, 0, true);
    /// assert_eq!(m.get(0, 0), true);
    /// ```
    #[must_use]
    #[inline]
    pub fn get(&self, r: usize, c: usize) -> bool {
        debug_assert!(r < self.rows(), "Row index {r} out of bounds [0,{})", self.rows());
        debug_assert!(c < self.cols(), "Column index {c} out of bounds [0,{})", self.cols());
        self.m_rows[r].get(c)
    }

    /// Sets the bit at row `r` and column `c` to the bool value `val`.
    ///
    /// # Panics
    /// In debug mode, panics if `r` or `c` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// m.set(0, 0, true);
    /// assert_eq!(m.get(0, 0), true);
    /// ```
    #[inline]
    pub fn set(&mut self, r: usize, c: usize, val: bool) -> &mut Self {
        debug_assert!(r < self.rows(), "Row index {r} out of bounds [0,{})", self.rows());
        debug_assert!(c < self.cols(), "Column index {c} out of bounds [0,{})", self.cols());
        self.m_rows[r].set(c, val);
        self
    }

    /// Flips the bit at row `r` and column `c`.
    ///
    /// # Panics
    /// In debug mode, panics if `r` or `c` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// m.flip(0, 0);
    /// assert_eq!(m.get(0, 0), true);
    /// ```
    #[inline]
    pub fn flip(&mut self, r: usize, c: usize) -> &mut Self {
        debug_assert!(r < self.rows(), "Row index {r} out of bounds [0,{})", self.rows());
        debug_assert!(c < self.cols(), "Column index {c} out of bounds [0,{})", self.cols());
        self.m_rows[r].flip(c);
        self
    }
}

/// Methods for accessing and setting rows of a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns a row `i` of the bit-matrix as a reference to a bit-vector -- this is cheap.
    ///
    /// # Note
    /// You can also just use the indexing operator as in `mat[i]` for the same effect.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.row(0).to_string(), "100");
    /// assert_eq!(m.row(1).to_string(), "010");
    /// assert_eq!(m.row(2).to_string(), "001");
    /// ```
    #[must_use]
    #[inline]
    pub fn row(&self, i: usize) -> &BitVector<Word> {
        debug_assert!(i < self.rows(), "Row index {i} out of bounds [0, {})", self.rows());
        &self.m_rows[i]
    }

    /// Returns a mutable reference to the row `i` of the bit-matrix as a mutable reference to a bit-vector.
    ///
    /// # Note
    /// You can also just use the indexing operator as in `mat[i]` for the same effect.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// m.row_mut(0).set(1, true);
    /// assert_eq!(m.to_compact_binary_string(), "110 010 001");
    /// ```
    #[must_use]
    #[inline]
    pub fn row_mut(&mut self, i: usize) -> &mut BitVector<Word> {
        debug_assert!(i < self.rows(), "Row index {i} out of bounds [0, {})", self.rows());
        &mut self.m_rows[i]
    }

    /// Sets row `i` of the bit-matrix from a `BitStore` source `src`.
    ///
    /// The `src` parameter must have the same number of bits as the number of columns in the bit-matrix.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds or if the number of bits in the `src` is different from the number
    /// of columns in the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// let src: BitVector = BitVector::ones(3);
    /// m.set_row(0, &src);
    /// assert_eq!(m.to_compact_binary_string(), "111 010 001");
    /// ```
    #[inline]
    pub fn set_row<Src: BitStore<Word>>(&mut self, i: usize, src: &Src) -> &mut Self {
        debug_assert!(i < self.rows(), "Row index {i} out of bounds [0, {})", self.rows());
        debug_assert_eq!(src.len(), self.cols(), "Source length mismatch {} != {}", src.len(), self.cols());
        self.m_rows[i].copy_store(src);
        self
    }

    /// Flips all the bits in row `i` of the bit-matrix.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// m.flip_row(0);
    /// assert_eq!(m.to_compact_binary_string(), "011 010 001");
    /// ```
    #[inline]
    pub fn flip_row(&mut self, i: usize) -> &mut Self {
        debug_assert!(i < self.rows(), "Row index {i} out of bounds [0, {})", self.rows());
        self.row_mut(i).flip_all();
        self
    }
}

/// Method to access the columns of a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns a **clone** of the elements in column `c` from the bit-matrix as an independent [`BitVector`].
    ///
    /// # Note
    /// - Matrices are stored by rows and there is no cheap slice style access to the matrix columns.
    /// - In contrast, the `Index` trait implementation `matrix[r]` provides a cheap way to access a row as a reference.
    ///
    /// # Panics
    /// In debug mode, panics if `c` is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// let mut col = m.col(1);
    /// assert_eq!(col.to_string(), "010");
    /// col.set(0, true);
    /// col.set(2, true);
    /// assert_eq!(col.to_string(), "111");
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn col(&self, c: usize) -> BitVector<Word> {
        debug_assert!(c < self.cols(), "Column {c} is not in bounds [0, {})", self.cols());
        let mut result = BitVector::zeros(self.rows());
        for r in 0..self.rows() {
            if self.get(r, c) {
                result.set(r, true);
            }
        }
        result
    }
}

/// Methods to change the state of all the elements of a bit-matrix at once.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Sets all elements of the bit-matrix to the boolean value `v` and returns a reference to the matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// m.set_all(true);
    /// assert_eq!(m.all(), true);
    /// ```
    pub fn set_all(&mut self, v: bool) -> &mut Self {
        for row in &mut self.m_rows {
            row.set_all(v);
        }
        self
    }

    /// Flips all elements of the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// assert_eq!(m.all(), false);
    /// m.flip_all();
    /// assert_eq!(m.all(), true);
    /// m.flip_all();
    /// assert_eq!(m.all(), false);
    /// m.flip_all();
    /// assert_eq!(m.all(), true);
    /// ```
    pub fn flip_all(&mut self) -> &mut Self {
        for row in &mut self.m_rows {
            row.flip_all();
        }
        self
    }

    /// Returns a new bit-matrix that is the result of flipping all the bits in this one.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let M: gf2::BitMatrix = gf2::BitMatrix::ones(3, 3);
    /// let N = M.flipped();
    /// assert_eq!(M.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(N.to_compact_binary_string(), "000 000 000");
    /// ```
    #[must_use]
    pub fn flipped(&self) -> BitMatrix<Word> {
        let mut result: BitMatrix<Word> = self.clone();
        result.flip_all();
        result
    }
}

/// Methods to change the state of the elements on the diagonals of a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Sets the main diagonal of a square bit-matrix to the boolean value `val`.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// m.set_diagonal(true);
    /// for i in 0..3 {
    ///     assert_eq!(m.get(i, i), true);
    /// }
    /// ```
    pub fn set_diagonal(&mut self, val: bool) -> &mut Self {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        for i in 0..self.rows() {
            self.set(i, i, val);
        }
        self
    }

    /// Flips the elements on the main diagonal of a square bit-matrix.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(3);
    /// m.set_diagonal(true);
    /// for i in 0..3 {
    ///     assert_eq!(m.get(i, i), true);
    /// }
    /// m.flip_diagonal();
    /// for i in 0..3 {
    ///     assert_eq!(m.get(i, i), false);
    /// }
    /// ```
    pub fn flip_diagonal(&mut self) -> &mut Self {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        for i in 0..self.rows() {
            self.flip(i, i);
        }
        self
    }

    /// Sets the elements on super-diagonal `d` of a square bit-matrix to the boolean value `val`.
    ///
    /// Here `d = 0` is the main diagonal and `d = 1` is the first super-diagonal etc.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(5);
    /// m.set_super_diagonal(1, true);
    /// for i in 0..4 {
    ///     assert_eq!(m.get(i, i + 1), true);
    /// }
    /// ```
    pub fn set_super_diagonal(&mut self, d: usize, val: bool) -> &mut Self {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        for i in 0..(self.rows() - d) {
            self.set(i, i + d, val);
        }
        self
    }

    /// Flips the elements on super-diagonal `d` of a square bit-matrix.
    ///
    /// Note that `d = 0` is the main diagonal.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(5);
    /// m.set_super_diagonal(1, true);
    /// for i in 0..4 {
    ///     assert_eq!(m.get(i, i + 1), true);
    /// }
    /// m.flip_super_diagonal(1);
    /// for i in 0..4 {
    ///     assert_eq!(m.get(i, i + 1), false);
    /// }
    /// ```
    pub fn flip_super_diagonal(&mut self, d: usize) -> &mut Self {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        for i in 0..(self.rows() - d) {
            self.flip(i, i + d);
        }
        self
    }

    /// Sets the elements on sub-diagonal `d` of a square bit-matrix to the boolean value `val`.
    ///
    /// Here `d = 0` is the main diagonal and `d = 1` is the first sub-diagonal etc.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(5);
    /// m.set_sub_diagonal(1, true);
    /// for i in 0..4 {
    ///     assert_eq!(m.get(i + 1, i), true);
    /// }
    /// ```
    pub fn set_sub_diagonal(&mut self, d: usize, val: bool) -> &mut Self {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        for i in 0..(self.rows() - d) {
            self.set(i + d, i, val);
        }
        self
    }

    /// Flips the elements on sub-diagonal `d` of a square bit-matrix.
    ///
    /// Note that `d = 0` is the main diagonal.
    ///
    /// # Panics
    /// In debug mode, panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::square(5);
    /// m.set_sub_diagonal(1, true);
    /// for i in 0..4 {
    ///     assert_eq!(m.get(i + 1, i), true);
    /// }
    /// m.flip_sub_diagonal(1);
    /// for i in 0..4 {
    ///     assert_eq!(m.get(i + 1, i), false);
    /// }
    /// ```
    pub fn flip_sub_diagonal(&mut self, d: usize) -> &mut Self {
        debug_assert!(self.is_square(), "Bit-matrix is not square");
        for i in 0..(self.rows() - d) {
            self.flip(i + d, i);
        }
        self
    }
}

/// Bit-matrix resizing methods.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Resizes the bit-matrix, to have `r` rows and `c` columns, initializing any added elements to zero.
    ///
    /// Note:
    /// If *either* `r` or `c` is zero, the bit-matrix is cleared to be 0x0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::new();
    /// m.resize(10, 10);
    /// assert_eq!(m.rows(), 10);
    /// assert_eq!(m.cols(), 10);
    /// assert_eq!(m.len(), 100);
    /// m.resize(3, 7);
    /// assert_eq!(m.rows(), 3);
    /// assert_eq!(m.cols(), 7);
    /// assert_eq!(m.len(), 21);
    /// m.resize(0, 10);
    /// assert_eq!(m.rows(), 0);
    /// assert_eq!(m.cols(), 0);
    /// assert_eq!(m.len(), 0);
    /// ```
    pub fn resize(&mut self, r: usize, c: usize) -> &mut Self {
        // Edge case: no change.
        if r == self.rows() && c == self.cols() {
            return self;
        }

        // Resizes to zero in either dimension is taken to mean clear the matrix completely.
        if r == 0 || c == 0 {
            for row in &mut self.m_rows {
                row.resize(0);
            }
            self.m_rows.resize(0, BitVector::default());
            return self;
        }
        let old_cols = self.cols();

        // Resize the vector of rows adding new, correct-length, all-zero rows if needed.
        self.m_rows.resize(r, BitVector::zeros(c));

        // If necessary, resize each row to the new column count.
        // Any added rows will be no-ops, otherwise any added elements will be initialized to zero.
        if c != old_cols {
            for row in &mut self.m_rows {
                row.resize(c);
            }
        }
        self
    }

    /// Clears the bit-matrix back to an empty matrix.
    #[inline]
    pub fn clear(&mut self) -> &mut Self { self.resize(0, 0) }

    /// Shrinks the bit-matrix to the smallest possible size.
    pub fn shrink_to_fit(&mut self) -> &mut Self {
        for row in &mut self.m_rows {
            row.shrink_to_fit();
        }
        self.m_rows.shrink_to_fit();
        self
    }

    /// Makes an arbitrary rectangular bit-matrix into a square matrix.
    ///
    /// Existing elements are preserved. Any added elements are initialized to zero.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::from_string("111 111 111 111").unwrap();
    /// m.make_square(3);
    /// assert_eq!(m.to_compact_binary_string(), "111 111 111");
    /// ```
    pub fn make_square(&mut self, n: usize) -> &mut Self {
        self.resize(n, n);
        self
    }
}

/// Bit-matrix methods to append/remove rows and columns.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Consumes the input row and appends it to the end of the bit-matrix.
    ///
    /// # Panics
    /// Panics if the row has a different number of elements than the matrix has columns.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// m.append_row(BitVector::from_string("111").unwrap());
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001 111");
    /// ```
    pub fn append_row(&mut self, row: BitVector<Word>) -> &mut Self {
        assert_eq!(row.len(), self.cols(), "Row must have same number of elements as the matrix has columns");
        self.m_rows.push(row);
        self
    }

    /// Pops a row from the end of the bit-matrix and returns it or `None` if the matrix is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.remove_row(), Some(BitVector::from_string("001").unwrap()));
    /// assert_eq!(m.to_compact_binary_string(), "100 010");
    /// ```
    pub fn remove_row(&mut self) -> Option<BitVector<Word>> { self.m_rows.pop() }

    /// Appends the bits from the input column to the right of the bit-matrix.
    ///
    /// # Panics
    /// Panics if the column has a different number of elements than the matrix has rows.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// m.append_col(&BitVector::from_string("111").unwrap());
    /// assert_eq!(m.to_compact_binary_string(), "1001 0101 0011");
    /// ```
    pub fn append_col(&mut self, col: &BitVector<Word>) -> &mut Self {
        assert_eq!(col.len(), self.rows(), "Column must have same number of elements as the matrix has rows");
        for (i, row) in self.m_rows.iter_mut().enumerate() {
            row.push(col[i]);
        }
        self
    }

    /// Pops a column from the right of the bit-matrix and returns it or `None` if the matrix is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.remove_col(), Some(BitVector::from_string("001").unwrap()));
    /// assert_eq!(m.to_compact_binary_string(), "10 01 00");
    /// ```
    pub fn remove_col(&mut self) -> Option<BitVector<Word>> {
        if self.is_empty() {
            return None;
        }
        let result = self.col(self.cols() - 1);
        for row in &mut self.m_rows {
            row.pop();
        }
        Some(result)
    }

    /// Appends the columns of the `src` bit-matrix to the right of `self`.
    ///
    /// # Panics
    /// Panics if the source matrix does not have the same number of rows as the matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::ones(3, 3);
    /// m.append_cols(&BitMatrix::ones(3, 3));
    /// assert_eq!(m.to_compact_binary_string(), "111111 111111 111111");
    /// ```
    pub fn append_cols(&mut self, src: &BitMatrix<Word>) -> &mut Self {
        assert_eq!(src.rows(), self.rows(), "Input matrix must have same number of rows as the matrix");
        for (i, row) in self.m_rows.iter_mut().enumerate() {
            row.append_store(&src.m_rows[i]);
        }
        self
    }

    /// Pops `k` columns from the right of `self` and returns them as a new bit-matrix or `None` if `k` is too large.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::ones(3, 3);
    /// assert_eq!(m.remove_cols(2), Some(BitMatrix::ones(3, 2)));
    /// assert_eq!(m.to_compact_binary_string(), "1 1 1");
    /// ```
    pub fn remove_cols(&mut self, k: usize) -> Option<BitMatrix<Word>> {
        if k > self.cols() {
            return None;
        }
        let result = self.sub_matrix(0..self.rows(), self.cols() - k..self.cols());
        self.resize(self.rows(), self.cols() - k);
        Some(result)
    }

    /// Consumes the `src` bit-matrix and appends it to the bottom of `self`.
    ///
    /// # Panics
    /// Panics if the source matrix does not have the same number of columns as the matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::ones(3, 3);
    /// m.append_rows(BitMatrix::ones(3, 3));
    /// assert_eq!(m.to_compact_binary_string(), "111 111 111 111 111 111");
    /// ```
    pub fn append_rows(&mut self, src: BitMatrix<Word>) -> &mut Self {
        assert_eq!(src.cols(), self.cols(), "Input matrix must have same number of columns as the matrix");
        self.m_rows.extend(src.m_rows);
        self
    }

    /// Pops `k` rows from the bottom of `self` and returns them as a new bit-matrix or `None` if `k` is too large.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::ones(3, 3);
    /// assert_eq!(m.remove_rows(2), Some(BitMatrix::ones(2, 3)));
    /// assert_eq!(m.to_compact_binary_string(), "111");
    /// ```
    pub fn remove_rows(&mut self, k: usize) -> Option<BitMatrix<Word>> {
        if k > self.rows() {
            return None;
        }
        let result = self.sub_matrix(self.rows() - k..self.rows(), 0..self.cols());
        self.resize(self.rows() - k, self.cols());
        Some(result)
    }
}

/// Bit-matrix "elementary operations" (used in various linear algebra algorithms).
impl<Word: Unsigned> BitMatrix<Word> {
    /// Swaps two rows of a bit-matrix in place.
    ///
    /// # Panics
    /// This method panics if either of the row indices is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::zeros(3, 3);
    /// m[0].set_all(true);
    /// assert_eq!(m.to_compact_binary_string(), "111 000 000");
    /// m.swap_rows(0, 1);
    /// assert_eq!(m.to_compact_binary_string(), "000 111 000");
    /// m.swap_rows(1, 2);
    /// assert_eq!(m.to_compact_binary_string(), "000 000 111");
    /// ```
    #[inline]
    pub fn swap_rows(&mut self, i0: usize, i1: usize) -> &mut Self {
        self.m_rows.swap(i0, i1);
        self
    }

    /// Swaps two columns of a bit-matrix in place.
    ///
    /// # Panics
    /// This method panics if either of the column indices is out of bounds.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001");
    /// m.swap_cols(0, 1);
    /// assert_eq!(m.to_compact_binary_string(), "010 100 001");
    /// m.swap_cols(1, 2);
    /// assert_eq!(m.to_compact_binary_string(), "001 100 010");
    /// ```
    #[inline]
    pub fn swap_cols(&mut self, j0: usize, j1: usize) -> &mut Self {
        for i in 0..self.rows() {
            self.m_rows[i].swap(j0, j1);
        }
        self
    }

    /// Adds the identity matrix to this bit-matrix.
    ///
    /// If the matrix is M, then self becomes M + I.
    ///
    /// # Panics
    /// This method panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::zeros(3, 3);
    /// m.add_identity();
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001");
    /// m.add_identity();
    /// assert_eq!(m.to_compact_binary_string(), "000 000 000");
    /// ```
    pub fn add_identity(&mut self) -> &mut Self {
        assert!(self.is_square(), "`add_identity` requires a square matrix");
        for i in 0..self.rows() {
            self.flip(i, i);
        }
        self
    }
}

/// Bit-matrix transposition methods.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Transposes a square bit-matrix in place.
    ///
    /// # Panics
    /// This method panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::zero(3);
    /// m[0].set_all(true);
    /// assert_eq!(m.to_compact_binary_string(), "111 000 000");
    /// m.transpose();
    /// assert_eq!(m.to_compact_binary_string(), "100 100 100");
    /// ```
    pub fn transpose(&mut self) -> &mut Self {
        assert!(self.is_square(), "`transpose_in_place` requires a square matrix");
        for i in 0..self.rows() {
            for j in 0..i {
                if self.get(i, j) != self.get(j, i) {
                    self.flip(i, j);
                    self.flip(j, i);
                }
            }
        }
        self
    }

    /// Returns a new bit-matrix that is the transpose of an arbitrary bit-matrix.
    ///
    /// # Note
    /// - This method does not require the bit-matrix to be square, and it does not modify the original bit-matrix.
    /// - It isn't particularly efficient as it works by iterating over all elements of the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::zeros(3, 2);
    /// m[0].set_all(true);
    /// assert_eq!(m.to_compact_binary_string(), "11 00 00");
    /// let n = m.transposed();
    /// assert_eq!(n.to_compact_binary_string(), "100 100");
    /// ```
    #[must_use]
    pub fn transposed(&self) -> Self {
        let r = self.rows();
        let c = self.cols();
        let mut result = BitMatrix::zeros(c, r);
        for i in 0..r {
            for j in 0..c {
                if self.get(i, j) {
                    result.set(j, i, true);
                }
            }
        }
        result
    }
}

/// Sub-matrix cloning/replacing methods.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns an independent *clone* of the sub-matrix from the given row and column ranges.
    ///
    /// # Panics
    /// Panics if the bit-matrix has incompatible dimensions with the requested sub-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(5);
    /// let sub_m = m.sub_matrix(1..4, 1..4);
    /// assert_eq!(sub_m.to_compact_binary_string(), "100 010 001");
    /// let sub_m = m.sub_matrix(1..1, 1..1);
    /// assert_eq!(sub_m.to_compact_binary_string(), "");
    /// ```
    #[must_use]
    pub fn sub_matrix<R: RangeBounds<usize>>(&self, rows: R, cols: R) -> Self {
        // Get the start and end of the row range.
        let r_start = match rows.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let r_end = match rows.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.rows(),
        };

        // Check that the row range is valid.
        assert!(r_start <= r_end, "Invalid row range");
        assert!(r_end <= self.rows(), "Row range extends beyond the end of the bit-matrix");

        // Get the start and end of the column range.
        let c_start = match cols.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let c_end = match cols.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.cols(),
        };

        // Check that the column range is valid.
        assert!(c_start <= c_end, "Invalid column range");
        assert!(c_end <= self.cols(), "Column range extends beyond the right edge of the bit-matrix");

        // Get the number of rows and columns in the sub-matrix.
        let r = r_end - r_start;
        let c = c_end - c_start;

        // Create the sub-matrix.
        let mut result = BitMatrix::zeros(r, c);
        for i in 0..r {
            result.m_rows[i].copy_store(&self.m_rows[i + r_start].slice(c_start..c_end));
        }
        result
    }

    /// Replaces the sub-matrix starting at row `top` and column `left` with a copy of the sub-matrix `src`.
    ///
    /// The sub-matrix `src` must fit within this bit-matrix starting at row `top` and column `left`.
    ///
    /// # Panics
    /// Panics if `src` does not fit within this bit-matrix starting at row `top` and column `left`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(5);
    /// m.replace_sub_matrix(1, 1, &BitMatrix::ones(3, 3));
    /// assert_eq!(m.to_compact_binary_string(), "10000 01110 01110 01110 00001");
    /// ```
    pub fn replace_sub_matrix(&mut self, top: usize, left: usize, src: &BitMatrix<Word>) -> &mut Self {
        let r = src.rows();
        let c = src.cols();
        assert!(top + r <= self.rows(), "Too many rows for the replacement sub-matrix to fit");
        assert!(left + c <= self.cols(), "Too many columns for the replacement sub-matrix to fit");
        for i in 0..r {
            self.m_rows[top + i].slice_mut(left..left + c).copy_store(&src.m_rows[i]);
        }
        self
    }
}

/// Triangular sub-matrix methods.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns an independent *clone* of the lower triangular part of the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::ones(3, 3);
    /// let sub_m = m.lower();
    /// assert_eq!(sub_m.to_compact_binary_string(), "100 110 111");
    /// ```
    #[must_use]
    pub fn lower(&self) -> Self {
        // Edge case:
        if self.is_empty() {
            return BitMatrix::new();
        }

        // Start with a copy of the bit-matrix.
        let mut result = self.clone();

        // Set the upper triangular part to zero.
        let c = self.cols();
        for i in 0..self.rows() {
            let first = i + 1;
            if first < c {
                result.m_rows[i].slice_mut(first..c).set_all(false);
            }
        }
        result
    }

    /// Returns an independent *clone* of the upper triangular part of the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::ones(3, 3);
    /// let sub_m = m.upper();
    /// assert_eq!(sub_m.to_compact_binary_string(), "111 011 001");
    /// ```
    #[must_use]
    pub fn upper(&self) -> Self {
        // Edge case:
        if self.is_empty() {
            return BitMatrix::new();
        }

        // Start with a copy of the bit-matrix.
        let mut result = self.clone();

        // Set the lower triangular part to zero.
        let c = self.cols();
        for i in 0..self.rows() {
            let len = std::cmp::min(i, c);
            if len > 0 {
                result.m_rows[i].slice_mut(0..len).set_all(false);
            }
        }
        result
    }

    /// Returns an independent *clone* of the strictly lower triangular part of the bit-matrix.
    ///
    /// This is the same as `lower()` but with the diagonal reset to zero.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::ones(3, 3);
    /// let sub_m = m.strictly_lower();
    /// assert_eq!(sub_m.to_compact_binary_string(), "000 100 110");
    /// ```
    #[must_use]
    pub fn strictly_lower(&self) -> Self {
        let mut result = self.lower();
        result.set_diagonal(false);
        result
    }

    /// Returns an independent *clone* of the strictly upper triangular part of the bit-matrix.
    ///
    /// This is the same as `upper()` but with the diagonal reset to zero.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::ones(3, 3);
    /// let sub_m = m.strictly_upper();
    /// assert_eq!(sub_m.to_compact_binary_string(), "011 001 000");
    /// ```
    #[must_use]
    pub fn strictly_upper(&self) -> Self {
        let mut result = self.upper();
        result.set_diagonal(false);
        result
    }

    /// Returns an independent *clone* of the unit lower triangular part of the bit-matrix.
    ///
    /// This is the same as `lower()` but with the diagonal set to one.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::zeros(3, 3);
    /// let sub_m = m.unit_lower();
    /// assert_eq!(sub_m.to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn unit_lower(&self) -> Self {
        let mut result = self.lower();
        result.set_diagonal(true);
        result
    }

    /// Returns an independent *clone* of the unit upper triangular part of the bit-matrix.
    ///
    /// This is the same as `upper()` but with the diagonal set to one.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::zeros(3, 3);
    /// let sub_m = m.unit_upper();
    /// assert_eq!(sub_m.to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn unit_upper(&self) -> Self {
        let mut result = self.upper();
        result.set_diagonal(true);
        result
    }
}

/// Dot product methods for a bit-matrix with any bit-store type or with another bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Matrix-vector multiplication returning `M * v` as a new [`BitVector`].
    ///
    /// Both operands are passed by reference and the `v` can be any bit-store type.
    ///
    /// # Note
    /// We also use the `Mul` trait to overload the `*` operator to denote the same operation.
    ///
    /// # Panics
    /// Panics if the operands have incompatible dimensions.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// let v: BitVector = BitVector::ones(3);
    /// assert_eq!(m.dot(&v), BitVector::ones(3));
    /// assert_eq!(&m * &v, BitVector::ones(3));
    /// ```
    pub fn dot<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        assert_eq!(self.cols(), rhs.len(), "Incompatible dimensions: {} != {}", self.cols(), rhs.len());
        let mut result = BitVector::zeros(self.rows());
        for i in 0..self.rows() {
            if self.row(i).dot(rhs) {
                result.set(i, true);
            }
        }
        result
    }

    /// Vector-matrix multiplication returning `v * M` as a new [`BitVector`].
    ///
    /// Both operands are passed by reference and the `v` can be any bit-store type.
    ///
    /// # Note
    /// We store matrices in row-major order, so this operation is significantly less efficient than `M * v`.
    ///
    /// # Panics
    /// Panics if the operands have incompatible dimensions.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// let v: BitVector = BitVector::ones(3);
    /// assert_eq!(m.left_dot(&v), BitVector::ones(3));
    /// assert_eq!(&v * &m, BitVector::ones(3));
    /// ```
    pub fn left_dot<Lhs: BitStore<Word>>(&self, lhs: &Lhs) -> BitVector<Word> {
        assert_eq!(self.rows(), lhs.len(), "Incompatible dimensions: {} != {}", self.rows(), lhs.len());
        let mut result = BitVector::zeros(self.cols());
        for i in 0..self.cols() {
            if lhs.dot(&self.col(i)) {
                result.set(i, true);
            }
        }
        result
    }

    /// Matrix-matrix multiplication returning `M * N` as a new [`BitMatrix`].
    ///
    /// # Panics
    /// Panics if the operands have incompatible dimensions.
    ///
    /// # Note
    /// We also use the `Mul` trait to overload the `*` operator to denote the same operation.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::ones(3, 3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// assert_eq!(m1.dot_matrix(&m2).to_compact_binary_string(), "111 111 111");
    /// let m1: BitMatrix = BitMatrix::ones(4, 4);
    /// let m2: BitMatrix = BitMatrix::ones(4, 4);
    /// assert_eq!(m1.dot_matrix(&m2).to_compact_binary_string(), "0000 0000 0000 0000");
    /// ```
    #[must_use]
    pub fn dot_matrix(&self, rhs: &BitMatrix<Word>) -> Self {
        assert_eq!(self.cols(), rhs.rows(), "Incompatible dimensions: {} != {}", self.cols(), rhs.rows());

        let r = self.rows();
        let c = rhs.cols();
        let mut result = BitMatrix::zeros(r, c);

        // Row access is cheap, columns expensive, so arrange things to pull out columns as few times as possible.
        for j in 0..c {
            let rhs_col = rhs.col(j);
            for i in 0..r {
                if self.row(i).dot(&rhs_col) {
                    result.set(i, j, true);
                }
            }
        }
        result
    }
}

/// Methods to raise a bit-matrix to a power.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns a new bit-matrix that is the result of raising this bit-matrix to the power `n`.
    ///
    /// # Note
    /// We use an efficient square and square-and-multiply algorithm to compute the power.
    ///
    /// # Panics
    /// Panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::random(100, 100);
    /// let m2 = m1.to_the(3);
    /// let mut m3 = &m1 * &m1;
    /// m3 = &m3 * &m1;
    /// assert_eq!(m3, m2);
    /// ```
    #[must_use]
    pub fn to_the(&self, n: usize) -> Self {
        assert!(self.is_square(), "Bit-matrix must be square");

        // Edge case
        if n == 0 {
            return Self::identity(self.rows());
        }

        // If BitMatrix.g. n = 0b10101, then n_bit = 0b10000.
        let mut n_bit = n.prev_power_of_two();

        // Square and square-and-multiply algorithm starts with a copy of the bit-matrix.
        let mut result = self.clone();

        // That handled the most significant bit in `n`.
        n_bit >>= 1;

        // Square & multiply as needed ...
        while n_bit > 0 {
            // Always do a square step.
            result = &result * &result;

            // If the current bit in `n` is set, do a multiply step.
            if (n & n_bit) != 0 {
                result = &result * self;
            }

            // Move to the next bit in `n`.
            n_bit >>= 1;
        }
        result
    }

    /// Returns a new bit-matrix that is the result of raising this bit-matrix to the power `2^n`.
    ///
    /// # Panics
    /// Panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::random(100, 100);
    /// let m2 = m1.to_the_2_to_the(2);
    /// let mut m3 = &m1 * &m1;
    /// m3 *= &m1;
    /// m3 *= &m1;
    /// assert_eq!(m3, m2);
    /// ```
    #[must_use]
    pub fn to_the_2_to_the(&self, n: usize) -> Self {
        assert!(self.is_square(), "Bit-matrix must be square");

        // Note that 2^0 = 1 so M^(2^0) = M.
        let mut result = self.clone();
        for _ in 0..n {
            result = &result * &result;
        }
        result
    }
}

/// Methods that convert bit-matrices to bit-vectors.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns a bit-vector that is the concatenation of the rows of the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.to_vector().to_string(), "100010001");
    /// ```
    #[must_use]
    pub fn to_vector(&self) -> BitVector<Word> {
        let mut result = BitVector::with_capacity(self.len());
        for row in &self.m_rows {
            result.append_store(row);
        }
        result
    }

    /// Returns a bit-vector that is the concatenation of the columns of the bit-matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.to_vector_of_cols().to_string(), "100010001");
    /// ```
    #[must_use]
    pub fn to_vector_of_cols(&self) -> BitVector<Word> {
        let mut result = BitVector::with_capacity(self.len());
        for col in 0..self.cols() {
            result.append_store(&self.col(col));
        }
        result
    }
}

/// Methods to compute echelon forms for a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Transforms an arbitrary shaped, non-empty, bit-matrix to row-echelon form (in-place).
    ///
    /// The method returns a bit-vector that shows which columns have a "pivot" (a non-zero on or below the diagonal).
    /// The matrix *rank* is the number of set bits in that bit-vector.
    ///
    /// A bit-matrix is in echelon form if the first 1 in any row is to the right of the first 1 in the preceding row.
    /// It is a generalization of an upper triangular form --- the result is a matrix with a "staircase" shape.
    ///
    /// The transformation is Gaussian elimination. Any all zero rows are moved to the bottom of the matrix.
    ///
    /// The echelon form is not unique.
    ///
    /// # Panics
    /// Panics if the bit-matrix is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// m.set(2, 1, false);
    /// let has_pivot = m.to_echelon_form();
    /// assert_eq!(has_pivot.to_string(), "111");
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn to_echelon_form(&mut self) -> BitVector<Word> {
        assert!(!self.is_empty(), "Bit-matrix must not be empty");

        // We return a bit-vector that shows which columns have a pivot -- start by assuming none.
        let mut has_pivot: BitVector<Word> = BitVector::zeros(self.cols());

        // The current row of the echelon form we are working on.
        let mut r = 0;
        let num_rows = self.rows();

        // Iterate over each column.
        for j in 0..self.cols() {
            // Find a non-zero entry in this column below the diagonal (a "pivot").
            let mut p = r;
            while p < num_rows && !self[p][j] {
                p += 1;
            }

            // Did we find a pivot in this column?
            if p < num_rows {
                // Mark this column as having a pivot.
                has_pivot.set(j, true);

                // If necessary, swap the current row with the row that has the pivot.
                if p != r {
                    self.swap_rows(p, r);
                }

                // Below the working row make sure column j is zero by elimination if necessary.
                let row_r = self[r].clone();
                for i in r + 1..num_rows {
                    if self[i][j] {
                        self[i] ^= &row_r;
                    }
                }

                // Move to the next row. If we've reached the end of the matrix, we're done.
                r += 1;
                if r == num_rows {
                    break;
                }
            }
        }

        // Return the bit-vector that shows which columns have a pivot.
        has_pivot
    }

    /// Transforms the bit-matrix to reduced row-echelon form (in-place).
    ///
    /// The method returns a bit-vector that shows which columns have a "pivot" (a non-zero on or below the diagonal).
    /// The matrix *rank* is the number of set bits in that bit-vector.
    ///
    /// A bit-matrix is in reduced echelon form if it is in echelon form with at most one 1 in each column.
    /// The reduced echelon form is unique.
    ///
    /// # Panics
    /// Panics if the bit-matrix is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m: BitMatrix = BitMatrix::identity(3);
    /// m.set(2, 1, false);
    /// let pivots = m.to_reduced_echelon_form();
    /// assert_eq!(pivots.to_string(), "111");
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn to_reduced_echelon_form(&mut self) -> BitVector<Word> {
        // Start with the echelon form.
        let has_pivot = self.to_echelon_form();

        // Iterate over each row from the bottom up - Gauss Jordan elimination.
        for r in (0..self.rows()).rev() {
            // Find the first set bit in the current row if there is one.
            if let Some(p) = self[r].first_set() {
                // Clear out everything in column p *above* row r (already cleared out below the pivot).
                let row_r = self[r].clone();
                for i in 0..r {
                    if self[i][p] {
                        self[i] ^= &row_r;
                    }
                }
            }
        }

        // Return the bit-vector that shows which columns have a pivot.
        has_pivot
    }
}

/// Method to compute the inverse of a bit-matrix if it exists.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns the inverse of a square bit-matrix or `None` if the matrix is singular.
    ///
    /// # Panics
    /// Panics if the bit-matrix is empty or not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.inverse().unwrap().to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn inverse(&self) -> Option<BitMatrix<Word>> {
        // The bit-matrix must be square.
        if !self.is_square() {
            return None;
        }

        // Create a copy of the bit-matrix & augment it with the identity matrix on the right.
        let mut matrix = self.clone();
        matrix.append_cols(&BitMatrix::identity(self.rows()));

        // Transform the augmented matrix to reduced row-echelon form (we don't need the pivot info).
        let _ = matrix.to_reduced_echelon_form();

        // If all went well the left half is the identity matrix and the right half is the inverse.
        if matrix.sub_matrix(0..self.rows(), 0..self.cols()).is_identity() {
            let inverse = matrix.sub_matrix(0..self.rows(), self.cols()..self.cols() * 2);
            Some(inverse)
        }
        else {
            None
        }
    }
}

/// Associated functions that determine the probability of a "fair coin" bit-matrix being invertible or singular.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns the probability that a square `n x n` bit-matrix is invertible if each element is chosen independently
    /// and uniformly at random by flips of a fair coin.
    ///
    /// # Note
    /// For large `n`, the value is roughly 29% and that holds for n as low as 10.
    ///
    /// # Panics
    /// Panics if `n` is 0. Based on the assumption that querying the probability of a 0 x 0 bit-matrix being
    /// invertible is an upstream error somewhere.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// assert!((<BitMatrix>::probability_invertible(3) - 0.289).abs() < 1e-3);
    /// ```
    #[must_use]
    pub fn probability_invertible(n: usize) -> f64 {
        // Edge case: 0 x 0 matrix is likely a mistake!
        assert!(n > 0, "Querying the probability of a 0 x 0 bit-matrix being invertible. Upstream error???");

        // Formula is p(n) = \prod_{k = 1}^{n} (1 - 2^{-k}) which runs out of juice once n hits any size at all!
        let mut n_prod = f64::MANTISSA_DIGITS;

        // Probability is the product of the probabilities of each row being linearly independent.
        let mut result = 1.0;
        let mut pow2 = 1.0;
        while n_prod > 0 {
            pow2 *= 0.5;
            result *= 1.0 - pow2;
            n_prod -= 1;
        }
        result
    }

    /// Returns the probability that a square `n x n` bit-matrix is singular if each element is chosen independently
    /// and uniformly at random by flips of a fair coin.
    ///
    /// # Note
    /// For large `n`, the value is 71% and that holds for n as low as 10.
    ///
    /// # Panics
    /// Panics if `n` is 0. Based on the assumption that querying the probability of a 0 x 0 bit-matrix being
    /// invertible is an upstream error somewhere.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// assert!((BitMatrix::<u8>::probability_singular(3) - 0.711).abs() < 1e-3);
    /// ```
    #[must_use]
    pub fn probability_singular(n: usize) -> f64 { 1.0 - Self::probability_invertible(n) }
}

/// Linear system solvers and decompositions ...
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns the Gaussian elimination solver for this bit-matrix and the passed r.h.s. vector `b`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::ones(3, 3);
    /// let b: BitVector = BitVector::ones(3);
    /// let solver = A.solver_for(&b);
    /// assert_eq!(solver.rank(), 1);
    /// assert_eq!(solver.free_count(), 2);
    /// assert_eq!(solver.solution_count(), 4);
    /// assert_eq!(solver.is_underdetermined(), true);
    /// assert_eq!(solver.is_consistent(), true);
    /// ```
    #[must_use]
    pub fn solver_for(&self, b: &BitVector<Word>) -> BitGauss<Word> { BitGauss::new(self, b) }

    /// Returns a solution to the system of linear equations `A.x = b` or `None` if the system is inconsistent.
    ///
    /// If the system is underdetermined with `f` free variables the returned solution will have `f` random 0/1 entries
    /// for those indices.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::identity(3);
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// assert_eq!(A.x_for(&b).unwrap().to_string(), "111");
    /// ```
    #[must_use]
    pub fn x_for(&self, b: &BitVector<Word>) -> Option<BitVector<Word>> { self.solver_for(b).x() }

    /// Returns the LU decomposition of this bit-matrix which must be square.
    ///
    /// On construction, this method computes a unit lower triangular matrix `L`, an upper triangular matrix `U`,
    /// and a permutation matrix `P` such that `P.A = L.U`. The `L` and `U` triangles are efficiently packed into a
    /// single matrix and `P` is stored as a vector of row swap instructions.
    ///
    /// The construction works even if `A` is singular, though the solver methods will not.
    ///
    /// # Note
    /// If the matrix is `n x n`, then the construction takes O(n^3) operations. There are block iterative methods that
    /// can reduce that to a sub-cubic count but they are not implemented here. Of course, the method works on whole
    /// words or bit elements at a time so is very efficient even without those enhancements.
    ///
    /// # Panics
    /// Panics if the matrix is not square. There are generalisations of the LU decomposition for non-square matrices
    /// but those are not considered yet.
    ///
    /// # Examples (checks that `LU = PA` for a random matrix `A`)
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::random(40, 40);
    /// let lu = A.lu_decomposition();
    /// let LU = lu.L() * lu.U();
    /// let mut PA = A.clone();
    /// lu.permute_matrix(&mut PA);
    /// assert_eq!(PA, LU);
    /// ```
    #[must_use]
    pub fn lu_decomposition(&self) -> BitLU<Word> { BitLU::new(self) }
}

/// Methods to compute the characteristic polynomial of a bit-matrix.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns the characteristic polynomial of any square bit-matrix as a [`BitPolynomial`].
    ///
    /// # Note
    /// The method uses similarity transformations to convert the bit-matrix to *Frobenius form* which has a readily
    /// computable characteristic polynomial. Similarity transformations preserve eigen-structure, and in particular
    /// the characteristic polynomial.
    ///
    /// # Panics
    /// Panics if the bit-matrix is not square.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(2);
    /// assert_eq!(m.characteristic_polynomial().to_string(), "1 + x^2");
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.characteristic_polynomial().to_string(), "1 + x + x^2 + x^3");
    /// let m: BitMatrix = BitMatrix::random(100, 100);
    /// let p = m.characteristic_polynomial();
    /// assert_eq!(p.eval_matrix(&m).is_zero(), true);
    /// ```
    #[must_use]
    pub fn characteristic_polynomial(&self) -> BitPolynomial<Word> {
        assert!(self.is_square(), "Bit-matrix must be square not {}x{}", self.rows(), self.cols());
        Self::characteristic_polynomial_frobenius_matrix(&self.frobenius_form())
    }

    /// Associated function that returns the characteristic polynomial of a *Frobenius matrix* as a [`BitPolynomial`].
    ///
    /// A Frobenius matrix is a square matrix that consists of blocks of *companion matrices* along the diagonal.
    /// Each companion matrix is a square matrix that is all zeros except for an arbitrary top row and a principal
    /// sub-diagonal of all ones. Companion matrices can be compactly represented by their top rows only.
    ///
    /// This associated function expects to be passed the top rows of the companion matrices as an array of bit-vectors.
    /// The characteristic polynomial of a Frobenius matrix is the product of the characteristic polynomials of its
    /// block companion matrices which are readily computed.
    #[must_use]
    pub fn characteristic_polynomial_frobenius_matrix(top_rows: &[BitVector<Word>]) -> BitPolynomial<Word> {
        let n_companions = top_rows.len();
        if n_companions == 0 {
            return BitPolynomial::zero();
        }

        // Compute the product of the characteristic polynomials of the companion matrices.
        let mut result = Self::characteristic_polynomial_companion_matrix(&top_rows[0]);

        // clippy wants to see a range loop but in this case an imperative loop is easier to understand.
        #[allow(clippy::needless_range_loop)]
        for i in 1..n_companions {
            result *= Self::characteristic_polynomial_companion_matrix(&top_rows[i]);
        }
        result
    }

    /// Associated methods to return the characteristic polynomial of a *companion matrix* as a [`BitPolynomial`].
    ///
    /// The function expects to be passed the top row of the companion matrix as a bit-vector.
    ///
    /// A companion matrix is a square matrix that is all zeros except for an arbitrary top row and a principal
    /// sub-diagonal of all ones. Companion matrices can be compactly represented by their top rows only.
    ///
    /// The characteristic polynomial of a companion matrix can be computed from its top row.
    ///
    /// # Note
    /// Some references define a companion matrix as having a single arbitrary final column instead of our top-row
    /// version.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let top_row: BitVector = BitVector::from_binary_string("101").unwrap();
    /// assert_eq!(BitMatrix::characteristic_polynomial_companion_matrix(&top_row).to_string(), "1 + x^2 + x^3");
    /// ```
    #[must_use]
    pub fn characteristic_polynomial_companion_matrix(top_row: &BitVector<Word>) -> BitPolynomial<Word> {
        let n = top_row.len();

        // The characteristic polynomial is degree n with n + 1 coefficients (leading coefficient is 1).
        let mut coeffs = BitVector::ones(n + 1);

        // The lower order coefficients are the top row of the companion matrix in reverse order.
        for j in 0..n {
            coeffs.set(n - j - 1, top_row[j]);
        }
        BitPolynomial::from_coefficients(coeffs)
    }

    /// Returns the *Frobenius form* of this bit-matrix in compact top-row only form.
    ///
    /// A Frobenius matrix is a square matrix that consists of one or more blocks of *companion matrices* along the
    /// diagonal. The companion matrices are square matrices that are all zeros except for an arbitrary top row and
    /// a principal sub-diagonal of all ones. Companion matrices can be compactly represented by their top rows
    /// only.
    ///
    /// We can convert any bit-matrix to Frobenius form via a sequence of similarity transformations that preserve the
    /// eigen-structure of the original matrix.
    ///
    /// We return the Frobenius companion matrices in a compact form as a `Vec` of their top rows as bit-vectors.
    ///
    /// # Panics
    /// Panics if the bit-matrix is not square.
    #[must_use]
    pub fn frobenius_form(&self) -> Vec<BitVector<Word>> {
        // The bit-matrix must be square.
        assert!(self.is_square(), "Bit-matrix must be square not {}x{}", self.rows(), self.cols());

        // Space for the top rows of the companion matrices which we will return.
        let mut top_rows = Vec::new();

        // Make a working copy of the bit-matrix to work through using Danilevsky's algorithm.
        let mut copy = self.clone();
        let mut n = copy.rows();
        while n > 0 {
            let companion = copy.danilevsky_step(n);
            n -= companion.len();
            top_rows.push(companion);
        }
        top_rows
    }

    /// Performs a single step of Danilevsky's algorithm to reduce a bit-matrix to Frobenius form.
    ///
    /// Frobenius form is block diagonal with one or more companion matrices along the diagonal. All matrices can be
    /// reduced to this form via a sequence of similarity transformations. This methods performs a single one of those
    /// transformations.
    ///
    /// This `frobenius_form` function calls here with an N x N bit-matrix. In each call the method concentrates on just
    /// the top-left n x n sub-matrix. On the first call, n should be set to N. The method returns the top row of the
    /// companion matrix that is the transformation of the bottom-right (n-k) x (n-k) sub-matrix. The caller can store
    /// that result, decrement n, and call again on the smaller top-left sub-matrix. It may be that the whole matrix
    /// gets reduced to a single companion matrix in one step and then there will be no need to call again.
    ///
    /// The method tries to transform the n x n top-left sub-matrix to a companion matrix working from its bottom-right
    /// corner up. It stops when it gets to a point where the bottom-right (n-k) x (n-k) sub-matrix is in companion form
    /// and returns the top row of that sub-matrix. The caller can store that result, decrement n, and call again on
    /// the smaller top-left sub-matrix.
    ///
    /// # Panics
    /// Panics if the bit-matrix is not square.
    ///
    /// # Examples
    #[must_use]
    fn danilevsky_step(&mut self, n: usize) -> BitVector<Word> {
        assert!(
            n <= self.rows(),
            "Asked to look at the top-left {n} x {n} sub-matrix but the matrix has only {} rows",
            self.rows()
        );

        // Edge case: A 1 x 1 matrix is already in companion form.
        if n == 1 {
            return BitVector::constant(self[0][0], 1);
        }

        // Step k of algorithm attempts to reduce row k to companion form.
        // By construction, rows k+1 or later are already in companion form.
        let mut k = n - 1;
        while k > 0 {
            // If row k's sub-diagonal is all zeros we look for an earlier column with a 1.
            // If found, we swap that column here & then swap the equivalent rows to preserve similarity.
            if !self[k][k - 1] {
                for j in 0..k - 1 {
                    if self[k][j] {
                        self.swap_rows(j, k - 1);
                        self.swap_cols(j, k - 1);
                        break;
                    }
                }
            }

            // No joy? Perhaps we have a companion matrix in the lower left corner and can return its top row?
            if !self[k][k - 1] {
                break;
            }

            // No joy? The sub-diagonal is not all zeros so apply transform to make it so: self <- M^-1 * self * M,
            // where M is the identity matrix with the (k-1)'st row replaced by the k'th row of `self`.
            // We can sparsely represent M as just a clone of that k'th row of `self`.
            let m = self[k].clone();

            // Note the M^-1 is the same as M and self <- M^-1 * self just alters a few of our elements.
            for j in 0..n {
                self.set(k - 1, j, &m * &self.col(j));
            }

            // We also use the sparsity of M when computing self <- self * M.
            for i in 0..k {
                for j in 0..n {
                    let tmp = self[i][k - 1] & m[j];
                    if j == k - 1 {
                        self.set(i, j, tmp);
                    }
                    else {
                        self.set(i, j, self[i][j] ^ tmp);
                    }
                }
            }

            // Now put row k into companion form of all zeros with one on the sub-diagonal.
            // All the rows below k are already in companion form.
            self.m_rows[k].set_all(false);
            self.set(k, k - 1, true);

            // Done with row k
            k -= 1;
        }

        // At this point, k == 0 OR the bit-matrix has non-removable zero on the sub-diagonal of row k.
        // Either way, the bottom-right (n-k) x (n-k) sub-matrix, starting at self[k][k], is in companion form.
        // We return the top row of that companion sub-matrix.
        let mut top_row = BitVector::zeros(n - k);
        for j in 0..n - k {
            top_row.set(j, self[k][k + j]);
        }
        top_row
    }
}

/// Methods to convert bit-matrices to strings.
impl<Word: Unsigned> BitMatrix<Word> {
    /// Returns a multi-line binary string representation of the bit-matrix.
    ///
    /// The matrix rows are separated by *newlines*.
    /// Each row is a string of 0's and 1's with a space separator between the elements.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.to_binary_string(), format!("1 0 0\n0 1 0\n0 0 1"));
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.to_binary_string(), "");
    /// ```
    #[must_use]
    pub fn to_binary_string(&self) -> String { self.to_custom_binary_string("\n", " ", "", "") }

    /// Returns a "pretty" binary string representation of the bit-matrix.
    ///
    /// The matrix rows are separated by *newlines*.
    /// Each row is a string of 0's and 1's with a space separator between the elements.
    /// The rows are delimited by a light vertical bar on the left and right.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// let bar: char = '\u{2502}';
    /// assert_eq!(m.to_pretty_binary_string(), format!("{bar}1 0 0{bar}\n{bar}0 1 0{bar}\n{bar}0 0 1{bar}"));
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.to_pretty_binary_string(), "");
    /// ```
    #[must_use]
    pub fn to_pretty_binary_string(&self) -> String {
        const BAR: &str = "\u{2502}";
        self.to_custom_binary_string("\n", " ", BAR, BAR)
    }

    /// Returns a compact "binary" string representation of the bit-matrix.
    ///
    /// The matrix rows are separated by a single *space* character.
    /// Each row is a string of 0's and 1's with no separator between the elements.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.to_compact_binary_string(), "100 010 001");
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.to_compact_binary_string(), "");
    /// ```
    #[must_use]
    pub fn to_compact_binary_string(&self) -> String { self.to_custom_binary_string(" ", "", "", "") }

    /// Returns a customised binary string representation of the bit-matrix.
    ///
    /// The matrix rows are separated by the `row_separator` parameter.
    /// Each row is a string of 0's and 1's with a custom `separator` between the elements.
    /// You can also provide custom `left` and `right` delimiters for each row.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!(m.to_custom_binary_string("\n", "", "[", "]"), "[100]\n[010]\n[001]");
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.to_custom_binary_string(" ", " ", "[", "]"), "");
    /// ```
    #[must_use]
    pub fn to_custom_binary_string(&self, row_separator: &str, separator: &str, left: &str, right: &str) -> String {
        self.m_rows
            .iter()
            .map(|row| row.to_custom_binary_string(separator, left, right))
            .collect::<Vec<_>>()
            .join(row_separator)
    }

    /// Returns a hex string representation of the bit-matrix.
    ///
    /// The matrix rows are separated by *newlines*.
    /// Each row is a hex string representation of a bit-vector (see e.g. [`BitVector::to_hex_string`]).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix<u8> = BitMatrix::ones(4, 4);
    /// assert_eq!(m.to_hex_string(), "F\nF\nF\nF");
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.to_hex_string(), "");
    /// ```
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        self.m_rows.iter().map(super::store::BitStore::to_hex_string).collect::<Vec<_>>().join("\n")
    }

    /// Returns a compact hex string representation of the bit-matrix.
    ///
    /// The matrix rows are separated by a single *space* character.
    /// Each row is a hex string representation of a bit-vector (see e.g. [`BitVector::to_hex_string`]).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix<u8> = BitMatrix::ones(4, 4);
    /// assert_eq!(m.to_compact_hex_string(), "F F F F");
    /// let m: BitMatrix = BitMatrix::new();
    /// assert_eq!(m.to_compact_hex_string(), "");
    /// ```
    #[must_use]
    pub fn to_compact_hex_string(&self) -> String {
        self.m_rows.iter().map(super::store::BitStore::to_hex_string).collect::<Vec<_>>().join(" ")
    }
}

/// Methods to perform bitwise operations between bit-matrices (these are also available via operator overloading).
impl<Word: Unsigned> BitMatrix<Word> {
    /// Performs an in-place bitwise XOR of this bit-matrix with another.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// m1.xor_eq(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "011 101 110");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// ```
    pub fn xor_eq(&mut self, rhs: &BitMatrix<Word>) {
        assert_eq!(self.rows(), rhs.rows(), "Length mismatch {} != {}", self.rows(), rhs.rows());
        assert_eq!(self.cols(), rhs.cols(), "Length mismatch {} != {}", self.cols(), rhs.cols());
        for i in 0..self.rows() {
            self.m_rows[i].xor_eq(&rhs.m_rows[i]);
        }
    }

    /// Returns a new bit-matrix that is the bitwise XOR of this bit-matrix with another.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// let m3 = m1.xor(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(m3.to_compact_binary_string(), "011 101 110");
    /// ```
    #[must_use]
    pub fn xor(&self, rhs: &BitMatrix<Word>) -> BitMatrix<Word> {
        let mut result = self.clone();
        result.xor_eq(rhs);
        result
    }

    /// Performs an in-place bitwise AND of this bit-matrix with another.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// m1.and_eq(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// ```
    pub fn and_eq(&mut self, rhs: &BitMatrix<Word>) {
        assert_eq!(self.rows(), rhs.rows(), "Length mismatch {} != {}", self.rows(), rhs.rows());
        assert_eq!(self.cols(), rhs.cols(), "Length mismatch {} != {}", self.cols(), rhs.cols());
        for i in 0..self.rows() {
            self.m_rows[i].and_eq(&rhs.m_rows[i]);
        }
    }

    /// Returns a new bit-matrix that is the bitwise AND of this bit-matrix with another.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// let m3 = m1.and(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(m3.to_compact_binary_string(), "100 010 001");
    /// ```
    #[must_use]
    pub fn and(&self, rhs: &BitMatrix<Word>) -> BitMatrix<Word> {
        let mut result = self.clone();
        result.and_eq(rhs);
        result
    }

    /// Performs an in-place bitwise OR of this bit-matrix with another.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// m1.or_eq(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// ```
    pub fn or_eq(&mut self, rhs: &BitMatrix<Word>) {
        assert_eq!(self.rows(), rhs.rows(), "Length mismatch {} != {}", self.rows(), rhs.rows());
        assert_eq!(self.cols(), rhs.cols(), "Length mismatch {} != {}", self.cols(), rhs.cols());
        for i in 0..self.rows() {
            self.m_rows[i].or_eq(&rhs.m_rows[i]);
        }
    }

    /// Returns a new bit-matrix that is the bitwise OR of this bit-matrix with another.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// let m3 = m1.or(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(m3.to_compact_binary_string(), "111 111 111");
    /// ```
    #[must_use]
    pub fn or(&self, rhs: &BitMatrix<Word>) -> BitMatrix<Word> {
        let mut result = self.clone();
        result.or_eq(rhs);
        result
    }
}

/// Methods to perform arithmetic between bit-matrices (these are also available via operator overloading).
impl<Word: Unsigned> BitMatrix<Word> {
    /// Adds another bit-matrix to this one in-place.
    ///
    /// In GF(2) addition is the same as the XOR operation.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// m1.plus_eq(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "011 101 110");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// ```
    pub fn plus_eq(&mut self, rhs: &BitMatrix<Word>) { self.xor_eq(rhs); }

    /// Returns a new bit-matrix that is the sum of this bit-matrix with another.
    ///
    /// In GF(2) addition is the same as the XOR operation.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// let m3 = m1.plus(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(m3.to_compact_binary_string(), "011 101 110");
    /// ```
    #[must_use]
    pub fn plus(&self, rhs: &BitMatrix<Word>) -> BitMatrix<Word> {
        let mut result = self.clone();
        result.xor_eq(rhs);
        result
    }

    /// Subtracts another bit-matrix from this one in-place.
    ///
    /// In GF(2) subtraction is the same as the XOR operation.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// m1.minus_eq(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "011 101 110");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// ```
    pub fn minus_eq(&mut self, rhs: &BitMatrix<Word>) { self.xor_eq(rhs); }

    /// Returns a new bit-matrix that is the difference of this bit-matrix with another.
    ///
    /// In GF(2) subtraction is the same as the XOR operation.
    ///
    /// # Panics
    /// This method panics if the dimensions of the input bit-matrices don't match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m1: BitMatrix = BitMatrix::identity(3);
    /// let m2: BitMatrix = BitMatrix::ones(3, 3);
    /// let m3 = m1.minus(&m2);
    /// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
    /// assert_eq!(m2.to_compact_binary_string(), "111 111 111");
    /// assert_eq!(m3.to_compact_binary_string(), "011 101 110");
    /// ```
    #[must_use]
    pub fn minus(&self, rhs: &BitMatrix<Word>) -> BitMatrix<Word> {
        let mut result = self.clone();
        result.xor_eq(rhs);
        result
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `Default` trait implementation for a bit-matrix.
// ---------------------------------------------------------------------------------------------------------------------

/// The `Default` trait implementation for a bit-matrix.
impl<Word: Unsigned> Default for BitMatrix<Word> {
    /// The default constructor creates an empty bit-matrix. <br>
    /// No capacity is reserved until elements are added.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let m: BitMatrix = Default::default();
    /// assert_eq!(m.to_compact_binary_string(), "");
    /// ```
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `Index` & `IndexMut` trait implementations for the `BitMatrix` type.
// ---------------------------------------------------------------------------------------------------------------------

/// The `Index` trait implementation for the `BitMatrix` type.
///
/// Returns a reference to *row* `i` of the matrix.
///
/// # Panics
/// In debug mode, panics if the row or column is out of bounds.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(m[0].to_string(), "100");
/// assert_eq!(m[1].to_string(), "010");
/// assert_eq!(m[2].to_string(), "001");
/// ```
impl<Word: Unsigned> Index<usize> for BitMatrix<Word> {
    type Output = BitVector<Word>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.rows(), "Row {} is not in bounds [0, {})", index, self.rows());
        &self.m_rows[index]
    }
}

/// The `IndexMut` trait implementation for the `BitMatrix` type.
///
/// Returns a mutable reference to *row* `i` of the matrix.
///
/// # Panics
/// In debug mode, panics if the row or column is out of bounds.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut m: BitMatrix = BitMatrix::zeros(3, 3);
/// m[0].set(0, true);
/// assert_eq!(m[0].to_string(), "100");
/// assert_eq!(m[1].to_string(), "000");
/// assert_eq!(m[2].to_string(), "000");
/// ```
impl<Word: Unsigned> IndexMut<usize> for BitMatrix<Word> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.rows(), "Row {} is not in bounds [0, {})", index, self.rows());
        &mut self.m_rows[index]
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `Display`-like trait implementations for the `BitMatrix` type.
// ---------------------------------------------------------------------------------------------------------------------

/// The `Debug` trait implementation for a `BitMatrix`.
///
/// The output is a one-line "binary" string representation of the bit-matrix.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(format!("{m:?}"), "100 010 001");
/// let m: BitMatrix = BitMatrix::new();
/// assert_eq!(format!("{m:?}"), "");
/// ```
impl<Word: Unsigned> fmt::Debug for BitMatrix<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.to_compact_binary_string()) }
}

/// The `Display` trait implementation for a `BitMatrix`.
///
/// The rows of the matrix are output as a 0's and 1's formatted as follows:
///
/// # Default Output
/// The default output puts each matrix row on a separate line.
/// The rows are delimited by a '|' on the left and right.
/// The row elements are separated by a single space.
///
/// # Alternate Compact Output
/// The alternate output is on a single line with the rows separated by a single space.
/// The row elements are output as a 0s and 1s without any separators between them.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m: BitMatrix = BitMatrix::identity(3);
/// let bar: char = '\u{2502}';
/// assert_eq!(format!("{m}"), format!("{bar}1 0 0{bar}\n{bar}0 1 0{bar}\n{bar}0 0 1{bar}"));
/// assert_eq!(format!("{m:#}"), "100 010 001");
/// let m: BitMatrix = BitMatrix::new();
/// assert_eq!(format!("{m}"), "");
/// assert_eq!(format!("{m:#}"), "");
/// ```
impl<Word: Unsigned> fmt::Display for BitMatrix<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.to_compact_binary_string())
        }
        else {
            write!(f, "{}", self.to_pretty_binary_string())
        }
    }
}

/// The `Binary` trait implementation for a `BitMatrix`.
///
/// Each row of the matrix is output as a binary number string with a "0b" prefix.
///
/// # Default Output
/// The default output puts each matrix row on a separate line.
///
/// # Alternate Compact Output
/// The alternate output is on a single line with the rows separated by a single space.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(format!("{m:b}"), "0b100\n0b010\n0b001");
/// assert_eq!(format!("{m:#b}"), "0b100 0b010 0b001");
/// let m: BitMatrix = BitMatrix::new();
/// assert_eq!(format!("{m:b}"), "");
/// ```
impl<Word: Unsigned> fmt::Binary for BitMatrix<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let row_strings: Vec<String> = self.m_rows.iter().map(|row| format!("{row:#b}")).collect();
        if f.alternate() { write!(f, "{}", row_strings.join(" ")) } else { write!(f, "{}", row_strings.join("\n")) }
    }
}

/// The `UpperHex` trait implementation for a `BitMatrix`.
///
/// The output is the "upper hex" string representation of the bit-matrix.
///
/// # Note
/// - If the `UpperHex` trait is used with the `alternate` flag, the output is on a single line.
/// - The default output is on multiple lines -- one line per matrix row.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(format!("{m:X}"), "0X4.8\n0X2.8\n0X1.8");
/// let m: BitMatrix = BitMatrix::new();
/// assert_eq!(format!("{m:X}"), "");
/// ```
impl<Word: Unsigned> fmt::UpperHex for BitMatrix<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let row_strings: Vec<String> = self.m_rows.iter().map(|row| format!("{row:#X}")).collect();
        if f.alternate() { write!(f, "{}", row_strings.join(" ")) } else { write!(f, "{}", row_strings.join("\n")) }
    }
}

/// The `LowerHex` trait implementation for a `BitMatrix`.
///
/// The output is the "lower hex" string representation of the bit-matrix.
///
/// # Note
/// - If the `LowerHex` trait is used with the `alternate` flag, the output is on a single line.
/// - The default output is on multiple lines -- one line per matrix row.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m: BitMatrix = BitMatrix::identity(3);
/// assert_eq!(format!("{m:x}"), "0x4.8\n0x2.8\n0x1.8");
/// let m: BitMatrix = BitMatrix::new();
/// assert_eq!(format!("{m:x}"), "");
/// ```
impl<Word: Unsigned> fmt::LowerHex for BitMatrix<Word> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let row_strings: Vec<String> = self.m_rows.iter().map(|row| format!("{row:#x}")).collect();
        if f.alternate() { write!(f, "{}", row_strings.join(" ")) } else { write!(f, "{}", row_strings.join("\n")) }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `Not` bitwise trait implementations for the `BitMatrix` type.
// ---------------------------------------------------------------------------------------------------------------------

/// The `Not` trait implementation for a `BitMatrix` reference.
///
/// Returns a new bit-matrix that has the same bits but all flipped.
///
/// # Note
/// This version does *not* consume `self` but must be called by reference.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m1: BitMatrix = BitMatrix::identity(3);
/// let m2 = !&m1;
/// assert_eq!(m1.to_compact_binary_string(), "100 010 001");
/// assert_eq!(m2.to_compact_binary_string(), "011 101 110");
/// ```
impl<Word: Unsigned> Not for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn not(self) -> Self::Output { self.flipped() }
}

/// The `Not` trait implementation for a `BitMatrix`.
///
/// Returns a new bit-matrix that has the same bits but all flipped.
///
/// # Note
/// This version *consumes* `self`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let m1: BitMatrix = BitMatrix::identity(3);
/// let m2 = !m1;
/// assert_eq!(m2.to_compact_binary_string(), "011 101 110");
/// ```
impl<Word: Unsigned> Not for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn not(self) -> Self::Output { self.flipped() }
}

// --------------------------------------------------------------------------------------------------------------------
// Implementations of in-place bitwise operation traits for pairs of bit-matrices & references to bit-matrices.
//
// The traits are:
//
// - BitXorAssign  for ^=
// - BitAndAssign  for &=
// - BitOrAssign   for |=
// - AddAssign     for +=
// - SubAssign     for -=
//
// We have implemented the traits where right-hand side may or may not be consumed by the call.
// For example if A and B are bit-matrices, then for the pairwise XOR operator we have implemented:
//
// - A ^= &B leaves B untouched.
// - A ^= B  consumes B.
// --------------------------------------------------------------------------------------------------------------------

/// Performs `&lhs ^= &rhs` where `lhs` and `rhs` are bit-matrices. Does not consume `rhs`.
impl<Word: Unsigned> BitXorAssign<&BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &BitMatrix<Word>) { self.xor_eq(rhs); }
}

/// Performs `&lhs ^= rhs` where `lhs` and `rhs` are bit-matrices. Consumes `rhs`.
impl<Word: Unsigned> BitXorAssign<BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: BitMatrix<Word>) { self.xor_eq(&rhs); }
}

/// Performs `&lhs &= &rhs` where `lhs` and `rhs` are bit-matrices. Does not consume `rhs`.
impl<Word: Unsigned> BitAndAssign<&BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn bitand_assign(&mut self, rhs: &BitMatrix<Word>) { self.and_eq(rhs); }
}

/// Performs `&lhs &= rhs` where `lhs` and `rhs` are bit-matrices. Consumes `rhs`.
impl<Word: Unsigned> BitAndAssign<BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn bitand_assign(&mut self, rhs: BitMatrix<Word>) { self.and_eq(&rhs); }
}

/// Performs `&lhs |= &rhs` where `lhs` and `rhs` are bit-matrices. Does not consume `rhs`.
impl<Word: Unsigned> BitOrAssign<&BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn bitor_assign(&mut self, rhs: &BitMatrix<Word>) { self.or_eq(rhs); }
}

/// Performs `&lhs |= rhs` where `lhs` and `rhs` are bit-matrices. Consumes `rhs`.
impl<Word: Unsigned> BitOrAssign<BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn bitor_assign(&mut self, rhs: BitMatrix<Word>) { self.or_eq(&rhs); }
}

/// Performs `&lhs += &rhs` where `lhs` and `rhs` are bit-matrices. Does not consume `rhs`.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> AddAssign<&BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn add_assign(&mut self, rhs: &BitMatrix<Word>) { self.xor_eq(rhs); }
}

/// Performs `&lhs += rhs` where `lhs` and `rhs` are bit-matrices. Consumes `rhs`.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> AddAssign<BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn add_assign(&mut self, rhs: BitMatrix<Word>) { self.xor_eq(&rhs); }
}

/// Performs `&lhs -= &rhs` where `lhs` and `rhs` are bit-matrices. Does not consume `rhs`.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> SubAssign<&BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn sub_assign(&mut self, rhs: &BitMatrix<Word>) { self.xor_eq(rhs); }
}

/// Performs `&lhs -= rhs` where `lhs` and `rhs` are bit-matrices. Consumes `rhs`.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> SubAssign<BitMatrix<Word>> for BitMatrix<Word> {
    #[inline]
    fn sub_assign(&mut self, rhs: BitMatrix<Word>) { self.xor_eq(&rhs); }
}

// --------------------------------------------------------------------------------------------------------------------
// Implementations of out-of-place bitwise operation traits for pairs of bit-matrices & references to bit-matrices.
//
// The traits are:
//
// - BitXor  for ^
// - BitAnd  for =
// - BitOr   for |
// - Add     for +
// - Sub     for -=
//
// We have implemented the traits for all four combinations of bit-matrices and *reference*s to bit-matrices.
// For example if A and B are bit-matrices type, then for the pairwise XOR operator we have implemented:
//
// - &A ^ &B leaves A and B untouched
// - &A ^ B  consumes B
// - A ^ &B  consumes A
// - A ^ B   consumes both A and B
// --------------------------------------------------------------------------------------------------------------------

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs ^ &rhs` as new bit-matrix without consuming either operand.
impl<Word: Unsigned> BitXor<&BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitxor(self, rhs: &BitMatrix<Word>) -> Self::Output { self.xor(rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs ^ rhs` as new bit-matrix, consuming the `rhs` operand.
impl<Word: Unsigned> BitXor<&BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitxor(self, rhs: &BitMatrix<Word>) -> Self::Output { self.xor(rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `lhs ^ &rhs` as new bit-matrix, consuming the `lhs` operand.
impl<Word: Unsigned> BitXor<BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitxor(self, rhs: BitMatrix<Word>) -> Self::Output { self.xor(&rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `lhs ^ rhs` as new bit-matrix, consuming both operands.
impl<Word: Unsigned> BitXor<BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitxor(self, rhs: BitMatrix<Word>) -> Self::Output { self.xor(&rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs & &rhs` as new bit-matrix without consuming either operand.
impl<Word: Unsigned> BitAnd<&BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitand(self, rhs: &BitMatrix<Word>) -> Self::Output { self.and(rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs & rhs` as new bit-matrix, consuming the `rhs` operand.
impl<Word: Unsigned> BitAnd<&BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitand(self, rhs: &BitMatrix<Word>) -> Self::Output { self.and(rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `lhs & &rhs` as new bit-matrix, consuming the `lhs` operand.
impl<Word: Unsigned> BitAnd<BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitand(self, rhs: BitMatrix<Word>) -> Self::Output { self.and(&rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `lhs & rhs` as new bit-matrix, consuming both operands.
impl<Word: Unsigned> BitAnd<BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitand(self, rhs: BitMatrix<Word>) -> Self::Output { self.and(&rhs) }
}

/// If `lhs` or `rhs` are bit-matrices this returns `&lhs | &rhs` as new bit-matrix without consuming either operand.
impl<Word: Unsigned> BitOr<&BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitor(self, rhs: &BitMatrix<Word>) -> Self::Output { self.or(rhs) }
}

/// If `lhs` or `rhs` are bit-matrices this returns `&lhs | rhs` as new bit-matrix, consuming the `rhs` operand.
impl<Word: Unsigned> BitOr<&BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitor(self, rhs: &BitMatrix<Word>) -> Self::Output { self.or(rhs) }
}

/// If `lhs` or `rhs` are bit-matrices this returns `lhs | &rhs` as new bit-matrix, consuming the `lhs` operand.
impl<Word: Unsigned> BitOr<BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitor(self, rhs: BitMatrix<Word>) -> Self::Output { self.or(&rhs) }
}

/// If `lhs` or `rhs` are bit-matrices this returns `lhs | rhs` as new bit-matrix, consuming both operands.
impl<Word: Unsigned> BitOr<BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn bitor(self, rhs: BitMatrix<Word>) -> Self::Output { self.or(&rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `&lhs + &rhs` as new bit-matrix without consuming either operand.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Add<&BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn add(self, rhs: &BitMatrix<Word>) -> Self::Output { self.xor(rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `&lhs + rhs` as new bit-matrix, consuming the `rhs` operand.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Add<&BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn add(self, rhs: &BitMatrix<Word>) -> Self::Output { self.xor(rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `lhs + &rhs` as new bit-matrix, consuming the `lhs` operand.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Add<BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn add(self, rhs: BitMatrix<Word>) -> Self::Output { self.xor(&rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `lhs + rhs` as new bit-matrix, consuming both operands.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Add<BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn add(self, rhs: BitMatrix<Word>) -> Self::Output { self.xor(&rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `&lhs - &rhs` as new bit-matrix without consuming either operand.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Sub<&BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn sub(self, rhs: &BitMatrix<Word>) -> Self::Output { self.xor(rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `&lhs - rhs` as new bit-matrix, consuming the `rhs` operand.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Sub<&BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn sub(self, rhs: &BitMatrix<Word>) -> Self::Output { self.xor(rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `lhs - &rhs` as new bit-matrix, consuming the `lhs` operand.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Sub<BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn sub(self, rhs: BitMatrix<Word>) -> Self::Output { self.xor(&rhs) }
}

/// If `lhs` xor `rhs` are bit-matrices this returns `lhs - rhs` as new bit-matrix, consuming both operands.
/// In GF(2) addition & subtraction are the same as the XOR operation.
impl<Word: Unsigned> Sub<BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn sub(self, rhs: BitMatrix<Word>) -> Self::Output { self.xor(&rhs) }
}

// ---------------------------------------------------------------------------------------------------------------------
// Matrix-matrix multiplication where the operands may or may not be consumed by the operation.
// ---------------------------------------------------------------------------------------------------------------------

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs * &rhs` as new bit-matrix without consuming either operand.
impl<Word: Unsigned> Mul<&BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn mul(self, rhs: &BitMatrix<Word>) -> Self::Output { self.dot_matrix(rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs * &rhs` as new bit-matrix consuming the `rhs` operand.
impl<Word: Unsigned> Mul<BitMatrix<Word>> for &BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn mul(self, rhs: BitMatrix<Word>) -> Self::Output { self.dot_matrix(&rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs * &rhs` as new bit-matrix consuming the `lhs` operand.
impl<Word: Unsigned> Mul<&BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn mul(self, rhs: &BitMatrix<Word>) -> Self::Output { self.dot_matrix(rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this returns `&lhs * &rhs` as new bit-matrix consuming both operands.
impl<Word: Unsigned> Mul<BitMatrix<Word>> for BitMatrix<Word> {
    type Output = BitMatrix<Word>;

    #[inline]
    fn mul(self, rhs: BitMatrix<Word>) -> Self::Output { self.dot_matrix(&rhs) }
}

/// If `lhs` and `rhs` are bit-matrices this performs `lhs = &lhs * &rhs` without consuming `rhs`.
///
/// # Note
/// Rust will be unhappy if you try to use `M *= &M` (cannot borrow `M` as mutable and immutable).
///
/// # Panics
/// This method panics if the dimensions of the input bit-matrices don't match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut lhs: BitMatrix = BitMatrix::identity(3);
/// let rhs: BitMatrix = BitMatrix::ones(3, 3);
/// lhs *= &rhs;
/// assert_eq!(lhs.to_compact_binary_string(), "111 111 111");
/// assert_eq!(rhs.to_compact_binary_string(), "111 111 111");
/// ```
impl<Word: Unsigned> MulAssign<&BitMatrix<Word>> for BitMatrix<Word> {
    fn mul_assign(&mut self, rhs: &BitMatrix<Word>) { *self = &*self * rhs; }
}

/// If `lhs` and `rhs` are bit-matrices this performs `lhs = &lhs * rhs` consuming `rhs`.
///
/// # Note
/// Rust will be unhappy if you try to use `M *= M` (cannot move out of `M` because it is borrowed).
///
/// # Panics
/// This method panics if the dimensions of the input bit-matrices don't match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut lhs: BitMatrix = BitMatrix::identity(3);
/// lhs *= BitMatrix::ones(3, 3);
/// assert_eq!(lhs.to_compact_binary_string(), "111 111 111");
/// ```
impl<Word: Unsigned> MulAssign<BitMatrix<Word>> for BitMatrix<Word> {
    fn mul_assign(&mut self, rhs: BitMatrix<Word>) { *self = &*self * rhs; }
}

// ---------------------------------------------------------------------------------------------------------------------
// Matrix-vector multiplication: M * v for a bit-matrix M and any bit-store type v is implemented using a macro.
//
// Note:
// Ideally we would implement the `Mul` trait for a `BitMatrix` with anything `BitStore` and be done.
// Unfortunately, Rust worries that someone downstream might also make `BitMatrix` to be `BitStore` and then we'd have a
// conflict (e.g. matrix-vector multiplication is not the same as matrix-matrix multiplication implemented above).
//
// So we use a macro to implement the `Mul` trait for a `BitMatrix` with any of our _concrete_ bit-stores.
// The two operands may or may not be consumed by the operation.
// ---------------------------------------------------------------------------------------------------------------------
macro_rules! M_dot_v {

    // The `BitVector` case which has just the one generic parameter: `Word: Unsigned`.
    (BitVector) => {
        M_dot_v!(@impl BitVector[Word]; [Word: Unsigned]);
    };

    // The `BitSlice` case with an `'a` lifetime parameter as well as the `Word: Unsigned` parameter.
    (BitSlice) => {
        M_dot_v!(@impl BitSlice['a, Word]; ['a, Word: Unsigned]);
    };

    // The `BitArray` case with a `const N: usize` parameter as well as the `Word: Unsigned` parameter.
    // Until Rust gets better const generics, there is also an extra fudge `const WORDS: usize` automatic parameter.
    (BitArray) => {
        M_dot_v!(@impl BitArray[N, Word, WORDS]; [const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The other arms funnel to this one which does the actual work of implementing the various foreign traits:
    // This matches on the pattern `$Rhs[$RhsParams]; [$ImplParams]` where in our case:
    //
    // $Rhs:        one of `BitVector`, `BitSlice`, or `BitArray`
    // $RhsParams:  some combo of `Word`, `'a, Word`, or `N, Word`.
    // $ImplParams: some combo of `Word: Unsigned`, `'a, Word:Unsigned` or `const N: usize, Word: Unsigned, const WORDS: usize.
    //
    // The trait implementations follow and are all straightforward …
    (@impl $Rhs:ident[$($RhsParams:tt)*]; [$($ImplParams:tt)*]) => {

#[doc = concat!("`BitMatrix`, `", stringify!($Rhs), "` multiplication where neither operand is consumed by the operation.")]
///
/// Matrix-vector multiplication returning `&M * &v` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<&$Rhs<$($RhsParams)*>> for &BitMatrix<Word> {
    type Output = BitVector<Word>;
    #[inline] fn mul(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.dot(rhs) }
}

#[doc = concat!("`BitMatrix`, `", stringify!($Rhs), "` multiplication where the vector is consumed by the operation.")]
///
/// Matrix-vector multiplication returning `&M * v` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<$Rhs<$($RhsParams)*>> for &BitMatrix<Word> {
    type Output = BitVector<Word>;
    #[inline] fn mul(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.dot(&rhs) }
}

#[doc = concat!("`BitMatrix`, `", stringify!($Rhs), "` multiplication where the matrix is consumed by the operation.")]
///
/// Matrix-vector multiplication returning `M * &v` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<&$Rhs<$($RhsParams)*>> for BitMatrix<Word> {
    type Output = BitVector<Word>;
    #[inline] fn mul(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.dot(rhs) }
}

#[doc = concat!("`BitMatrix`, `", stringify!($Rhs), "` multiplication where both operands are consumed by the operation.")]
///
/// Matrix-vector multiplication returning `M * v` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<$Rhs<$($RhsParams)*>> for BitMatrix<Word> {
    type Output = BitVector<Word>;
    #[inline] fn mul(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.dot(&rhs) }
}

};} // End of M_dot_v macro.

// Invoke the macro to implement the dot product of a `BitMatrix` with all bit-store types.
M_dot_v!(BitVector);
M_dot_v!(BitSlice);
#[cfg(feature = "unstable")]
M_dot_v!(BitArray);

// ---------------------------------------------------------------------------------------------------------------------
// Vector-matrix multiplication: u * M for a bit-matrix M and any bit-store u is implemented using a macro.
//
// Note:
// Ideally we would implement the `Mul` trait for any bit-store type (anything `BitStore`) with a bit-matrix.
// Unfortunately, Rust worries that someone downstream might also make `BitMatrix` to be `BitStore` and then we'd have a
// conflict (vector-matrix multiplication is not the same as matrix-matrix multiplication implemented above).
//
// So we use a macro to implement the `Mul` trait for any of our concrete bit-store types with a bit-matrix.
// The macro also creates a `dot_matrix` method `u.dot_matrix(M)` works for any bit-store type `u`.
// ---------------------------------------------------------------------------------------------------------------------
macro_rules! u_dot_M {

    // The `BitVector` case which has just the one generic parameter: `Word: Unsigned`.
    (BitVector) => {
        u_dot_M!(@impl BitVector[Word]; [Word: Unsigned]);
    };

    // The `BitSlice` case with an `'a` lifetime parameter as well as the `Word: Unsigned` parameter.
    (BitSlice) => {
        u_dot_M!(@impl BitSlice['a, Word]; ['a, Word: Unsigned]);
    };

    // The `BitArray` case with a `const N: usize` parameter as well as the `Word: Unsigned` parameter.
    // Until Rust gets better const generics, there is also an extra fudge `const WORDS: usize` automatic parameter.
    (BitArray) => {
        u_dot_M!(@impl BitArray[N, Word, WORDS]; [const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The other arms funnel to this one which does the actual work of implementing the various foreign traits:
    // This matches on the pattern `$Lhs[$LhsParams]; [$ImplParams]` where in our case:
    //
    // $Lhs:        one of `BitVector`, `BitSlice`, or `BitArray`
    // $LhsParams:  some combo of `Word`, `'a, Word`, or `N, Word`.
    // $ImplParams: some combo of `Word: Unsigned`, `'a, Word:Unsigned` or `const N: usize, Word: Unsigned, const WORDS: usize.
    //
    // The trait implementations follow and are all straightforward …
    (@impl $Lhs:ident[$($LhsParams:tt)*]; [$($ImplParams:tt)*]) => {

#[doc = concat!("Vector-matrix multiplication for a `", stringify!($Lhs), "`")]
impl<$($ImplParams)*> $Lhs<$($LhsParams)*> {
    #[doc = concat!(stringify!($Lhs), " matrix multiplication` as a new [`BitVector`].")]
    ///
    /// # Panics
    /// Panics if the operands have incompatible dimensions.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::ones(3);
    /// let m: BitMatrix = BitMatrix::identity(3);
    /// assert_eq!((&v * &m).to_string(), "111");
    /// ```
    #[inline] #[must_use]
    pub fn dot_matrix(&self, rhs: &BitMatrix<Word>) -> BitVector<Word> { rhs.left_dot(self) }
}

#[doc = concat!("`", stringify!($Rhs), "`, `BitMatrix` multiplication where neither operand is consumed by the operation.")]
///
/// Vector-matrix multiplication returning `&v * &M` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<&BitMatrix<Word>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVector<Word>;

    fn mul(self, rhs: &BitMatrix<Word>) -> Self::Output {
        self.dot_matrix(rhs)
    }
}

#[doc = concat!("`", stringify!($Rhs), "`, `BitMatrix` multiplication where the vector is consumed by the operation.")]
///
/// Vector-matrix multiplication returning `v * &M` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<&BitMatrix<Word>> for $Lhs<$($LhsParams)*> {
    type Output = BitVector<Word>;

    fn mul(self, rhs: &BitMatrix<Word>) -> Self::Output {
        self.dot_matrix(rhs)
    }
}

#[doc = concat!("`", stringify!($Rhs), "`, `BitMatrix` multiplication where the matrix is consumed by the operation.")]
///
/// Vector-matrix multiplication returning `&v * M` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<BitMatrix<Word>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVector<Word>;

    fn mul(self, rhs: BitMatrix<Word>) -> Self::Output {
        self.dot_matrix(&rhs)
    }
}

#[doc = concat!("`", stringify!($Rhs), "`, `BitMatrix` multiplication where both operands are consumed by the operation.")]
///
/// Vector-matrix multiplication returning `&v * M` as a new [`BitVector`].
///
/// # Panics
/// Panics if the operands have incompatible dimensions.
impl<$($ImplParams)*> Mul<BitMatrix<Word>> for $Lhs<$($LhsParams)*> {
    type Output = BitVector<Word>;

    fn mul(self, rhs: BitMatrix<Word>) -> Self::Output {
        self.dot_matrix(&rhs)
    }
}

};} // End of u_dot_M macro.

// Invoke the macro to implement the dot product of the various bit-store types with a `BitMatrix`.
// Dot product of all bit-store types with a BitMatrix.
u_dot_M!(BitVector);
u_dot_M!(BitSlice);
#[cfg(feature = "unstable")]
u_dot_M!(BitArray);

// ---------------------------------------------------------------------------------------------------------------------
// String representations of matrices side by side with vectors or other matrices.
// These are useful for debugging and testing.
// ---------------------------------------------------------------------------------------------------------------------

/// Return a string representation of two matrices side by side.
#[allow(non_snake_case)]
#[must_use]
pub fn string_for_AB<Word: Unsigned>(A: &BitMatrix<Word>, B: &BitMatrix<Word>) -> String {
    // What is the maximum number of rows in the three matrices?
    let num_rows = A.rows().max(B.rows());

    // Create the spacer strings for the matrices to use if they have less than `num_rows` rows.
    let A_fill = " ".repeat(A.cols() + 1);
    let B_fill = " ".repeat(B.cols() + 1);

    // Build the string representation of the matrices side by side with a vertical line between them.
    // Pad the columns with the fill strings to make them the same width.
    let mut result = String::new();
    for i in 0..num_rows {
        let A_str = if i < A.rows() { A[i].to_string() } else { A_fill.clone() };
        let B_str = if i < B.rows() { B[i].to_string() } else { B_fill.clone() };
        let _ = write!(result, "{}", &format!("| {A_str} | {B_str} |\n"));
    }
    result
}

/// Return a string representation of three matrices side by side.
#[allow(non_snake_case)]
#[must_use]
pub fn string_for_ABC<Word: Unsigned>(A: &BitMatrix<Word>, B: &BitMatrix<Word>, C: &BitMatrix<Word>) -> String {
    // What is the maximum number of rows in the three matrices?
    let num_rows = A.rows().max(B.rows()).max(C.rows());

    // Create the spacer strings for the matrices to use if they have less than `num_rows` rows.
    let A_fill = " ".repeat(A.cols() + 1);
    let B_fill = " ".repeat(B.cols() + 1);
    let C_fill = " ".repeat(C.cols() + 1);

    // Build the string representation of the matrices side by side with a vertical line between them.
    // Pad the columns with the fill strings to make them the same width.
    let mut result = String::new();
    for i in 0..num_rows {
        let A_str = if i < A.rows() { A[i].to_string() } else { A_fill.clone() };
        let B_str = if i < B.rows() { B[i].to_string() } else { B_fill.clone() };
        let C_str = if i < C.rows() { C[i].to_string() } else { C_fill.clone() };
        let _ = write!(result, "{}", &format!("| {A_str} | {B_str} | {C_str} |\n"));
    }
    result
}

/// Return a string representation of a matrix and vector side by side.
#[allow(non_snake_case)]
#[must_use]
pub fn string_for_Au<Word: Unsigned>(A: &BitMatrix<Word>, u: &BitVector<Word>) -> String {
    // What is the maximum number of rows between the matrix and vector?
    let num_rows = A.rows().max(u.len());

    // Create the spacer strings for the matrix and vector to use if they have less than `num_rows` rows.
    let A_fill = " ".repeat(A.cols() + 1);
    let u_fill = " ";

    // Build the string representation of the matrix and vector side by side with a vertical line between them.
    // Pad the rows with the fill strings to make them the same height.
    let mut result = String::new();
    for i in 0..num_rows {
        let A_str = if i < A.rows() { A[i].to_string() } else { A_fill.clone() };
        let u_str = if i < u.len() { i32::from(u[i]).to_string() } else { u_fill.to_string() };
        let _ = write!(result, "{}", &format!("| {A_str} | {u_str} |\n"));
    }
    result
}

/// Return a string representation of a matrix and two vectors side by side.
#[allow(non_snake_case)]
#[must_use]
pub fn string_for_Auv<Word: Unsigned>(A: &BitMatrix<Word>, u: &BitVector<Word>, v: &BitVector<Word>) -> String {
    // What is the maximum number of rows between the matrix and vectors?
    let num_rows = A.rows().max(u.len()).max(v.len());

    // Create the spacer strings for the matrix and vectors to use if they have less than `num_rows` rows.
    let A_fill = " ".repeat(A.cols() + 1);
    let u_fill = " ";
    let v_fill = " ";

    // Build the string representation of the matrix and vectors side by side with vertical lines between them.
    // Pad the rows with the fill strings to make them the same height.
    let mut result = String::new();
    for i in 0..num_rows {
        let A_str = if i < A.rows() { A[i].to_string() } else { A_fill.clone() };
        let u_str = if i < u.len() { i32::from(u[i]).to_string() } else { u_fill.to_string() };
        let v_str = if i < v.len() { i32::from(v[i]).to_string() } else { v_fill.to_string() };
        let _ = write!(result, "{}", &format!("| {A_str} | {u_str} | {v_str} |\n"));
    }
    result
}

/// Return a string representation of a matrix and three vectors side by side.
#[allow(non_snake_case)]
#[must_use]
pub fn string_for_Auvw<Word: Unsigned>(
    A: &BitMatrix<Word>, u: &BitVector<Word>, v: &BitVector<Word>, w: &BitVector<Word>,
) -> String {
    // What is the maximum number of rows between the matrix and vectors?
    let num_rows = A.rows().max(u.len()).max(v.len()).max(w.len());

    // Create the spacer strings for the matrix and vectors to use if they have less than `num_rows` rows.
    let A_fill = " ".repeat(A.cols() + 1);
    let u_fill = " ";
    let v_fill = " ";
    let w_fill = " ";

    // Build the string representation of the matrix and vectors side by side with vertical lines between them.
    // Pad the rows with the fill strings to make them the same height.
    let mut result = String::new();
    for i in 0..num_rows {
        let A_str = if i < A.rows() { A[i].to_string() } else { A_fill.clone() };
        let u_str = if i < u.len() { i32::from(u[i]).to_string() } else { u_fill.to_string() };
        let v_str = if i < v.len() { i32::from(v[i]).to_string() } else { v_fill.to_string() };
        let w_str = if i < w.len() { i32::from(w[i]).to_string() } else { w_fill.to_string() };
        let _ = write!(result, "{}", &format!("| {A_str} | {u_str} | {v_str} | {w_str} |\n"));
    }
    result
}
