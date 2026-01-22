//! [`BitGauss`] is a Gaussian elimination solver for systems of linear equations over GF(2).

#![allow(non_snake_case)]

use crate::{
    BitMatrix,
    BitStore,
    BitVector,
    Unsigned,
};

#[doc = include_str!("../docs/gauss.md")]
pub struct BitGauss<Word: Unsigned = usize> {
    // The *row echelon form* of the matrix `A` where we are solving `A = b`.
    A_ref: BitMatrix<Word>,

    // The equivalent transformed version of the vector `b` where we are solving `A.x = b`.
    b_ref: BitVector<Word>,

    // The rank of the matrix `A`. This is also the number of non-zero rows in `A_ref`
    rank: usize,

    // The index locations of any "free" variables if the system is underdetermined.
    free: Vec<usize>,

    // The number of solutions we can index into. This is either 0 or 2^f where `f` is the number of free variables.
    // However, for the `xi(i: usize)` function we must limit that to the largest power of 2 that fits in `usize`.
    solution_count: usize,
}

impl<Word: Unsigned> BitGauss<Word> {
    /// Constructs a new `BitGauss` struct where we are solving the system of linear equations `A.x = b`.
    ///
    /// # Panics
    /// Panics if the `A` matrix is not square or if the `A` matrix and `b` vector have a different number of rows.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.rank(), 1);
    /// assert_eq!(solver.is_underdetermined(), true);
    /// assert_eq!(solver.is_consistent(), true);
    /// assert_eq!(solver.free_count(), 2);
    /// assert_eq!(solver.solution_count(), 4);
    /// ```
    #[must_use]
    pub fn new(A: &BitMatrix<Word>, b: &BitVector<Word>) -> Self {
        assert!(A.is_square(), "The matrix must be square not {}x{}", A.rows(), A.cols());
        assert!(A.rows() == b.len(), "The matrix and vector must have the same number of rows");

        // Create a working copy of A, and augment it with b as an extra column on the right.
        let mut A_ref = A.clone();
        A_ref.append_col(b);

        // Get the reduced row echelon form of A|b and the vector that marks the pivot columns.
        let mut has_pivot = A_ref.to_reduced_echelon_form();

        // Grab the last column of the reduced A|b as a standalone vector, removing it from the matrix.
        let b_ref = A_ref.remove_col().unwrap();

        // Can also remove the last element from `has_pivot` as it corresponded to the extra column we added to `A_ref`.
        let _ = has_pivot.pop();

        // The rank of the matrix `A` is the number of columns with a pivot.
        // This is also the number of non-zero rows in `A_ref`
        let rank = has_pivot.count_ones();

        // Any column *without* a pivot corresponds to a free variable in the system. Collect those indices.
        has_pivot.flip_all();
        let free: Vec<usize> = has_pivot.set_bits().collect();

        // Check that the zero rows in `A_ref` are matched with zero entries in `b_ref`. This is consistency.
        let mut consistent = true;
        // Any zero rows in `A_ref` are at the bottom from `rank` to the end.
        for i in rank..A_ref.rows() {
            if b_ref[i] {
                consistent = false;
                break;
            }
        }

        // The number of solutions we can index into. This is either 0 or 2^f where `f` is the number of free variables.
        // However, for the `xi(i: usize)` function we must limit that to the largest power of 2 that fits in
        // `usize`. If `usize` has 64 bits then`solution_count = min(2^f, 2^63)`.
        let mut solution_count = 0;
        if consistent {
            let act_pow = free.len();
            let max_pow = (usize::BITS - 1) as usize;
            solution_count = 1 << std::cmp::min(act_pow, max_pow);
        }

        // Return the struct.
        Self { A_ref, b_ref, rank, free, solution_count }
    }

    /// Returns the rank of the matrix `A`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.rank(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn rank(&self) -> usize { self.rank }

    /// Returns the number of free variables in the system.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.free_count(), 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn free_count(&self) -> usize { self.free.len() }

    /// Returns `true` if the system is underdetermined.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.is_underdetermined(), true);
    /// ```
    #[inline]
    #[must_use]
    pub fn is_underdetermined(&self) -> bool { !self.free.is_empty() }

    /// Returns `true` if the system of linear equations `A.x = b` is consistent.
    ///
    /// A system is consistent if there is at least one solution.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert!(solver.is_consistent());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_consistent(&self) -> bool { self.solution_count > 0 }

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
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.x().unwrap().to_string(), "111");
    /// ```
    #[must_use]
    pub fn x(&self) -> Option<BitVector<Word>> {
        if !self.is_consistent() {
            return None;
        }

        // Create a random starting point.
        let mut result = BitVector::random(self.b_ref.len());

        // All non-free variables will be overwritten by back substitution.
        self.back_substitute_into(&mut result);
        Some(result)
    }

    /// Returns the maximum number of solutions we can index into.
    ///
    /// This may be 0, 1, or 2^f for some `f` where `f` is the number of free variables in an underdetermined system.
    /// For the `xi(i: usize)` function we limit that to the largest power of 2 that fits in `usize`.
    ///
    /// If `usize` is 64 bits then `solution_count = min(2^f, 2^63)`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.solution_count(), 4);
    /// ```
    #[inline]
    #[must_use]
    pub fn solution_count(&self) -> usize { self.solution_count }

    /// Returns the `i`th solution to the system of linear equations `A.x = b` or `None` if the system is
    /// inconsistent or if `i` is out of bounds.
    ///
    /// If the system is consistent and determined, then there is a unique solution and `xi(0)` is the same as
    /// `x()`.
    ///
    /// If the system is underdetermined with `f` free variables, it has `2^f` possible solutions.
    /// If `f` is large, `2^f` may not fit in `usize` but here we limit the number of *indexable* solutions to the
    /// largest power of 2 that fits in `usize`. The indexing scheme is certainly not unique but it is consistent across
    /// runs.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::from_string("111 111 111").unwrap();
    /// let b: BitVector = BitVector::from_string("111").unwrap();
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.solution_count(), 4);
    /// assert_eq!(solver.xi(0).unwrap().to_string(), "100", "xi(0) = 100");
    /// assert_eq!(solver.xi(1).unwrap().to_string(), "010", "xi(1) = 010");
    /// assert_eq!(solver.xi(2).unwrap().to_string(), "001", "xi(2) = 001");
    /// assert_eq!(solver.xi(3).unwrap().to_string(), "111", "xi(3) = 111");
    /// let A: BitMatrix = BitMatrix::identity(3);
    /// let solver: BitGauss = BitGauss::new(&A, &b);
    /// assert_eq!(solver.solution_count(), 1);
    /// assert_eq!(solver.xi(0).unwrap().to_string(), "111", "xi(0) = 111");
    /// ```
    #[must_use]
    pub fn xi(&self, i: usize) -> Option<BitVector<Word>> {
        if !self.is_consistent() {
            return None;
        }
        if i > self.solution_count() {
            return None;
        }

        // We start with a zero vector and then set the free variable slots to the fixed bit pattern for `i`.
        let mut x = BitVector::zeros(self.b_ref.len());
        let mut i = i;
        for f in 0..self.free.len() {
            x.set(self.free[f], i & 1 != 0);
            i >>= 1;
        }

        // Back substitution will now overwrite any non-free variables in `x` with their correct values.
        self.back_substitute_into(&mut x);
        Some(x)
    }

    /// Helper function that performs back substitution to solve for the non-free variables in `x`.
    fn back_substitute_into(&self, x: &mut BitVector<Word>) {
        // Iterate from the bottom up, starting at the first non-zero row, solving for the non-free variables in `x`.
        for i in (0..self.rank).rev() {
            let j = self.A_ref[i].first_set().unwrap();
            x.set(j, self.b_ref[i]);
            for k in j + 1..x.len() {
                if self.A_ref[i][k] {
                    x.set(j, x[j] ^ x[k]);
                }
            }
        }
    }
}
