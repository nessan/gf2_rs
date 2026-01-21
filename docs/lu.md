# The `BitLU` Type

## Introduction

`BitLU` computes and stores the [LU decomposition] of a square `n x n` bit-matrix `A` as:

```txt
P.A = L.U
```

where `P` is a _permutation matrix_ so `P.A` simply swaps rows in `A`, `U` is some upper triangular matrix and
`L` is a unit lower triangular matrix. The `L` and `U` triangles can be efficiently packed into a single bit-matrix
of the same size as `A`

`P` is a permutation of the identity matrix with one non-zero entry per row. In practice, we just need to store the
locations of those entries as some form of vector.

## Construction

You can construct a `BitLU` object from any square bit-matrix `A` either by:

- calling the [`BitLU::new`] method or
- using the [`BitMat::lu_decomposition`] directly on the bit-matrix.

The decomposition always works even if `A` is singular, but some of the other `BitLU` methods will not.

If `A` is `n x n`, then construction is an `O(n^3)` operation (though due to the nature of [GF(2)], things are done in words at a time).
There are sub-cubic ways of doing this work using various block-iterative methods, but those methods have not been implemented here yet.

**Note:** There are generalisations of the LU decomposition for non-square matrices but those are not considered here yet.

## Queries

Most of the work is done during construction.
After that, the following query methods are available:

| Method                 | Description                                                           |
| ---------------------- | --------------------------------------------------------------------- |
| [`BitLU::rank`]        | Returns the [rank] of the matrix `A`.                                 |
| [`BitLU::is_singular`] | Return `true` if the matrix `A` is not invertible.                    |
| [`BitLU::determinant`] | Returns the scalar boolean that is the determinant of `A`.            |
| [`BitLU::L`]           | Returns the unit lower triangular matrix `L` in the LU decomposition. |
| [`BitLU::U`]           | Returns the upper triangular matrix `U` in the LU decomposition.      |
| [`BitLU::P`]           | Returns the full permutation matrix `P` in the LU decomposition.      |

The [`BitLU::P`] method constructs the full permutation matrix `P` on demand but it is for exposition only.
In practice, you should use the permute methods described below to work with `P` in

## Permutations

You can access permutation information using the following methods:

| Method                        | Description                                                                                        |
| ----------------------------- | -------------------------------------------------------------------------------------------------- |
| [`BitLU::swaps`]              | Returns a reference to the row swap instructions in [`LAPACK`] form.                               |
| [`BitLU::permutation_vector`] | Returns the permutation matrix as a vector of showing the index positions of the non-zero entries. |
| [`BitLU::permute_matrix`]     | Permutes the rows of the input matrix in-place using our row-swap instruction vector.              |
| [`BitLU::permute_vector`]     | Permute the rows of the input vector in-place using our row-swap instruction vector.               |

A permutation matrix is just some row permutation of the identity matrix, so it has a single non-zero, 1, entry in each row or column.
You don't need to store the entire matrix; instead, store the locations of the 1s.

In the literature, the permutation vector is often given as a permutation of the index vector.
For example, the permutation vector `[0,2,1,4,3]` tells you that elements/rows 1 and 2 are swapped, as are elements/rows 3 and 4.
This form is easy to interpret at a glance. However, it is tedious to use as a guide to actually executing the permutations in place.

The [`LAPACK`] style `swaps` vector is an alternate, equally compact, form of the permutation matrix.
Our previous example becomes `[0,2,2,4,4]`.
This is interpreted as follows:

- No swap for row 0.
- Swap row 1 with row 2.
- No swap for row 2.
- Swap row 3 with row 4.
- No swap for row 4.

## Solving Linear Systems

| Method     | Description                                                                                     |
| ---------- | ----------------------------------------------------------------------------------------------- |
| `BitLU::x` | Returns a solution to the equation `A.x = b` where `x` and `b` are bit-vectors.                 |
| `BitLU::X` | Returns a solution to the collection of equations `A.X = B` where `x` and `b` are bit-matrices. |

In the second case, each column of the bit-matrix `B` is considered a separate right-hand side, and the corresponding column of $X$ is the solution vector.

Once you have the [LU decomposition] of `A`, it is easy to solve systems like these.
If `A` is `n x n,` each system solution takes just `O(n^2)` operations.

These methods return an [`Option`] wrapping a [`BitVec`] or a [`BitMat`] or [`None`] if `A` is singular.
You can check for singularity by first calling the [`BitLU::is_singular`] method.

### Panics

The methods panic if the number of elements in `b` or the number of rows in `B` does not match the number of rows in `A`.
They could instead return a [`None`], but a dimension mismatch is likely an indication of a coding error somewhere.

## Matrix Inversion

| Method             | Description                                                           |
| ------------------ | --------------------------------------------------------------------- |
| [`BitLU::inverse`] | Returns the inverse of the matrix `A` or [`None`] if `A` is singular. |

The inverse is returned as a new `BitMat` object wrapped in an [`Option`].

## Example

```rs
use gf2::*;
let mat: BitMat = BitMat::right_rotation(100, 5);
let inv: BitMat = BitLU::new(&mat).inverse().unwrap();
assert_eq!(inv, BitMat::left_rotation(100, 5));
println!("Confirmed that the inverse of a right rotation is a left rotation.");
```

[GF(2)]: https://en.wikipedia.org/wiki/Finite_field_arithmetic
[LU decomposition]: https://en.wikipedia.org/wiki/LU_decomposition
[rank]: https://en.wikipedia.org/wiki/Rank_(linear_algebra)
[`LAPACK`]: https://www.netlib.org/lapack/
