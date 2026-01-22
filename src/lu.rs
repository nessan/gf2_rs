//! [`BitLU`] provides the `LU` decomposition for bit-matrices.
#![allow(non_snake_case)]

// Crate types.
use crate::{
    BitMatrix,
    BitStore,
    BitVector,
    Unsigned,
};

#[doc = include_str!("../docs/lu.md")]
pub struct BitLU<Word: Unsigned = usize> {
    // The matrices L & U packed into a single bit-matrix.
    LU: BitMatrix<Word>,

    // The row swap instructions stored LAPACK style.
    swaps: Vec<usize>,

    // The rank of the matrix A.
    rank: usize,
}

impl<Word: Unsigned> BitLU<Word> {
    /// Returns the LU decomposition object for a square matrix `A`.
    ///
    /// On construction, this method computes a unit lower triangular matrix `L`, an upper triangular matrix `U`,
    /// and a permutation matrix `P` such that `P.A = L.U`. The `L` and `U` triangles are efficiently packed into a
    /// single matrix and `P` is stored as a vector of row swap instructions.
    ///
    /// The construction works even if `A` is singular, though the solver methods will not.
    ///
    /// # Note
    /// If `A` is n x n, then the construction takes O(n^3) operations. There are block iterative methods that can
    /// reduce that to a sub-cubic count but they are not implemented here. Of course, the method works on whole words
    /// or bit elements at a time so is very efficient even without those enhancements.
    ///
    /// # Panics
    /// Panics if the `A` matrix is not square. There are generalisations of the LU decomposition for non-square
    /// matrices but those are not considered yet.
    ///
    /// # Examples (checks that `LU = PA` for a random matrix `A`)
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::random(100, 100);
    /// let lu: BitLU = BitLU::new(&A);
    /// let L = lu.L();
    /// let U = lu.U();
    /// let LU = &L * &U;
    /// let mut PA = A.clone();
    /// lu.permute_matrix(&mut PA);
    /// assert_eq!(PA, LU);
    /// ```
    #[must_use]
    pub fn new(A: &BitMatrix<Word>) -> Self {
        assert!(A.is_square(), "Bit-matrix must be square");

        // Set things up
        let mut LU = A.clone();
        let mut swaps = vec![0; A.rows()];
        let mut rank = A.rows();

        // Iterate through the matrix (clippy wants to see a range loop but that obscures the code).
        #[allow(clippy::needless_range_loop)]
        for j in 0..A.rows() {
            // Initialise the row swap instruction.
            swaps[j] = j;

            // Find a non-zero entry in the current column on or below the diagonal (a "pivot").
            let mut p = j;
            while p < A.rows() && !LU[p][j] {
                p += 1;
            }

            // No pivot? The matrix is rank deficient. Record the deficiency and move along.
            if p == A.rows() {
                rank -= 1;
                continue;
            }

            // Found a pivot, so if necessary, swap the current row with the row that has the pivot.
            if p != j {
                LU.swap_rows(p, j);
                swaps[j] = p;
            }

            // Clear out the column below the pivot (at this point LU(j,j) == 1)
            let jp1 = j + 1;
            for i in jp1..A.rows() {
                if LU[i][j] {
                    for k in jp1..A.cols() {
                        let tmp = LU[i][k] ^ LU[j][k];
                        LU.set(i, k, tmp);
                    }
                }
            }
        }

        // Create and return the LU decomposition object.
        Self { LU, swaps, rank }
    }

    /// Returns the rank of the matrix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::left_rotation(100, 1);
    /// let lu: BitLU = BitLU::new(&A);
    /// assert_eq!(lu.rank(), 100);
    /// ```
    #[inline]
    #[must_use]
    pub fn rank(&self) -> usize { self.rank }

    /// Returns `true` if the matrix is singular (i.e. rank deficient).
    #[inline]
    #[must_use]
    pub fn is_singular(&self) -> bool { self.rank < self.LU.rows() }

    /// Returns the value of the determinant of the matrix `A` as `true` or `false` for 1 or 0.
    #[inline]
    #[must_use]
    pub fn determinant(&self) -> bool { !self.is_singular() }

    /// Returns a copy of `L` (unit lower triangular) as a full independent bit-matrix.
    #[inline]
    #[must_use]
    pub fn L(&self) -> BitMatrix<Word> { self.LU.unit_lower() }

    /// Returns a copy of `U` (upper triangular) as a full independent bit-matrix.
    #[inline]
    #[must_use]
    pub fn U(&self) -> BitMatrix<Word> { self.LU.upper() }

    /// Returns a copy of `P` (the permutation matrix) as a full independent bit-matrix.
    #[inline]
    #[must_use]
    pub fn P(&self) -> BitMatrix<Word> {
        let mut P = BitMatrix::identity(self.LU.rows());
        for i in 0..self.LU.rows() {
            P.swap_rows(i, self.swaps[i]);
        }
        P
    }

    /// Returns a reference to the row swap instructions in [`LAPACK`] form.
    ///
    /// A permutation matrix is just some row permutation of the identity matrix, so it has a single non-zero, 1, entry
    /// in each row or column. You don't need to store the entire matrix but instead store the locations of those 1's.
    ///
    /// In the literature, the permutation vector is often given as a permutation of the index vector. For example, the
    /// permutation vector `[0,2,1,4,3]` tells you that elements/rows 1 and 2 are swapped, as are elements/rows 3 and 4.
    /// This form is easy to interpret at a glance. However, it is tedious to use as a guide to actually executing the
    /// permutations in place.
    ///
    /// The [`LAPACK`] style `swaps` vector is an alternate, equally compact, form of the permutation matrix. Our
    /// previous example becomes `[0,2,2,4,4]`. This is interpreted as follows:
    ///
    /// - No swap for row 0.
    /// - Swap row 1 with row 2.
    /// - No swap for row 2.
    /// - Swap row 3 with row 4.
    /// - No swap for row 4.
    ///
    /// [`LAPACK`]: https://en.wikipedia.org/wiki/LAPACK
    #[inline]
    #[must_use]
    pub fn swaps(&self) -> &[usize] { &self.swaps }

    /// Returns the permutation matrix as a vector of showing the index positions of the non-zero entries.
    ///
    /// A permutation matrix is just some row permutation of the identity matrix, so it has a single non-zero, 1, entry
    /// in each row or column. You don't need to store the entire matrix but instead store the locations of those 1's.
    ///
    /// In the literature, the permutation vector is often given as a permutation of the index vector. For example, the
    /// permutation vector `[0,2,1,4,3]` tells you that elements/rows 1 and 2 are swapped, as are elements/rows 3 and 4.
    /// This form is easy to interpret at a glance and is returned by the `P_vector` method.
    ///
    /// See the [`swaps`](BitLU::swaps) method for an alternative form of the permutation matrix that is more
    /// convenient for executing the permutations in place.
    #[inline]
    #[must_use]
    pub fn permutation_vector(&self) -> Vec<usize> {
        let mut P = (0..self.LU.rows()).collect::<Vec<_>>();
        P.sort_by_key(|&i| self.swaps[i]);
        P
    }

    /// Permutes the rows of a bit-matrix `B` in place using the stored row swap instructions.
    ///
    /// # Panics
    /// Panics if the bit-matrix `B` has a different number of rows than the number of row swap instructions.
    pub fn permute_matrix(&self, B: &mut BitMatrix<Word>) {
        assert_eq!(
            B.rows(),
            self.swaps.len(),
            "Bit-matrix has {} rows but there are {} row swap instructions",
            B.rows(),
            self.swaps.len()
        );
        for i in 0..B.rows() {
            B.swap_rows(i, self.swaps[i]);
        }
    }

    /// Permutes the elements of a bit-vector `b` in place using the stored row swap instructions.
    ///
    /// # Panics
    /// Panics if the bit-vector `b` has a different number of elements than the number of row swap instructions.
    pub fn permute_vector(&self, b: &mut BitVector<Word>) {
        assert_eq!(
            b.len(),
            self.swaps.len(),
            "Bit-vector has {} elements but there are {} row swap instructions",
            b.len(),
            self.swaps.len()
        );
        for i in 0..b.len() {
            b.swap(i, self.swaps[i]);
        }
    }

    /// Solves the linear system `A.x = b` for any `b` where `A` is the matrix used to construct the
    /// `BitLU` object. Returns `None` if the matrix is singular.
    ///
    /// # Panics
    /// Panics if the bit-matrix `b` has a different number of rows than the number of row swap instructions.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let n = 100;
    /// let mut A: BitMatrix = BitMatrix::left_rotation(n, 1);
    /// let mut lu: BitLU = BitLU::new(&A);
    /// let b: gf2::BitVector = gf2::BitVector::random(n);
    /// let x = lu.x(&b).unwrap();
    /// assert_eq!(&A * &x, b);
    /// ```
    #[must_use]
    pub fn x(&self, b: &BitVector<Word>) -> Option<BitVector<Word>> {
        let n = self.LU.rows();
        assert_eq!(b.len(), n, "Bit-vector has {} elements but the matrix has {} rows", b.len(), n);
        if self.is_singular() {
            return None;
        }

        // Start with a copy of `b` and permute it.
        let mut x = b.clone();
        self.permute_vector(&mut x);

        // Forward substitution.
        for i in 0..n {
            for j in 0..i {
                if self.LU[i][j] {
                    x.set(i, x[i] ^ x[j]);
                }
            }
        }
        // Backward substitution.
        for i in (0..n).rev() {
            for j in i + 1..n {
                if self.LU[i][j] {
                    x.set(i, x[i] ^ x[j]);
                }
            }
        }
        Some(x)
    }

    /// Solves the linear system `A.X_for = B` for any `B` where `A` is the matrix used to construct the
    /// `BitLU` object. Returns `None` if the matrix is singular.
    ///
    /// Each column of `X` is a solution to the linear system `A.x = b` where `b` is the corresponding column of
    /// `B`.
    ///
    /// # Panics
    /// Panics if the bit-matrix `B` has a different number of rows than the number of row swap instructions.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::left_rotation(100, 5);
    /// let B: BitMatrix = BitMatrix::random(100, 12);
    /// let lu: BitLU = BitLU::new(&A);
    /// let X = lu.X(&B).unwrap();
    /// assert_eq!(&A * &X, B);
    /// ```
    #[must_use]
    pub fn X(&self, B: &BitMatrix<Word>) -> Option<BitMatrix<Word>> {
        let n = self.LU.rows();
        assert_eq!(B.rows(), n, "Right-hand side has {} rows but the matrix has {} rows", B.rows(), n);

        // Perhaps there is no solution.
        if self.is_singular() {
            return None;
        }

        // Start with a copy of `B` and permute it.
        let mut X = B.clone();
        self.permute_matrix(&mut X);

        // Solve for each column.
        for c in 0..B.cols() {
            // Forward substitution.
            for i in 0..n {
                for j in 0..i {
                    if self.LU[i][j] {
                        X.set(i, c, X[i][c] ^ X[j][c]);
                    }
                }
            }
            // Backward substitution.
            for i in (0..n).rev() {
                for j in i + 1..n {
                    if self.LU[i][j] {
                        X.set(i, c, X[i][c] ^ X[j][c]);
                    }
                }
            }
        }
        // Return the solution.
        Some(X)
    }

    /// Returns the inverse of the matrix `A` as a full independent bit-matrix. Returns `None` if the matrix is
    /// singular.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let A: BitMatrix = BitMatrix::left_rotation(100, 1);
    /// let A_inv: BitMatrix = BitLU::new(&A).inverse().unwrap();
    /// assert_eq!(A_inv, BitMatrix::right_rotation(100, 1));
    /// ```
    #[must_use]
    pub fn inverse(&self) -> Option<BitMatrix<Word>> {
        if self.is_singular() {
            return None;
        }
        let B: BitMatrix<Word> = BitMatrix::identity(self.LU.rows());
        self.X(&B)
    }
}
