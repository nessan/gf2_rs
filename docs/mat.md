# The `BitMat` Type

## Introduction

A [`BitMat`] is a dynamically sized matrix of bit elements stored compactly _by row_ in a [`Vec`] of [`BitVec`]s.
The default word type for the bit-vector rows is `usize`.

In mathematical terms, a bit-matrix is a matrix over [GF(2)], the simplest [Galois-Field] with just two elements, usually denoted 0 & 1, as the booleans true & false, or as the bits set & unset.
Arithmetic over GF(2) is mod 2, so addition/subtraction becomes the `XOR` operation while multiplication/division becomes `AND`.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

Operations on and between bit-matrices and other objects in the `gf2` library are implemented using bitwise operations on whole underlying words at a time.These operations are highly optimised in modern CPUs, allowing for fast computation even on large bit-matrices. It also means we never have to worry about overflows or carries as we would with normal integer arithmetic.

</div>

A bit-matrix is stored in _row-major mode_ where each row is a single `gf2::BitVec`.
This means that arranging computations to work row by row instead of column by column is typically much more efficient.
The methods and functions in the library take this into consideration.

This bit-matrix class is a [`Vec`] of rows where each row is a single bit-vector.
If the primary aim was to minimize storage, we would store the bit-matrix as a single long bit-vector with appropriate index operations.
However, in that case, matrix operations would often need to be done across word boundaries, which is much slower than doing things word-by-word.

**Note:** Arbitrary `m x n` bit-matrices are supported, but some functions only make sense for square matrices where `m = n`

The `BitMat` type has many of the same methods defined for [`BitVec`], such as bitwise operations and stringification.
We also have methods for matrix-vector, vector-matrix, and matrix-matrix multiplication.

There are methods to solve linear systems of equations `A.x = b`.

Danilevsky's method to compute characteristic polynomials (and the determinant) for a bit-matrix is available and works for quite large matrices (ones with millions of entries) that would choke a naive implementation that didn't take into account the nature of arithmetic over [GF(2)]

## Constructors

The `BitMat` type provides a rich set of methods for constructing bit-matrices:

| Method Name                      | Description                                                  |
| -------------------------------- | ------------------------------------------------------------ |
| [`BitMat::new`]                  | Creates the empty matrix with no elements.                   |
| [`BitMat::zeros`]                | Creates a matrix with all elements set to 0.                 |
| [`BitMat::square`]               | Creates a square matrix with all elements set to 0.          |
| [`BitMat::ones`]                 | Creates a matrix with all elements set to 1.                 |
| [`BitMat::alternating`]          | Creates the matrix with elements in a checker-board pattern. |
| [`BitMat::from_outer_product`]   | Creates a matrix as the outer product of two bit-vectors.    |
| [`BitMat::from_outer_sum`]       | Creates a matrix as the outer sum of two bit-vectors.        |
| [`BitMat::from_fn`]              | Creates a matrix by repeatedly invoking a function `f(i,j)`. |
| [`BitMat::random`]               | Creates a matrix with a _fair_ random fill.                  |
| [`BitMat::random_seeded`]        | Creates a matrix with a repeatable _fair_ random fill.       |
| [`BitMat::random_biased`]        | Creates a matrix with a biased random fill.                  |
| [`BitMat::random_biased_seeded`] | Creates a matrix with a biased & seeded random fill.         |

The random fill methods use an RNG that, by default, is seeded randomly, but can be seeded with a specific value for repeatable results. A seed of `0` is special and reverts to using a random seed.
The biased random fill methods take a single parameter p in the range `[0.0, 1.0]` that specifies the probability that each element is set to `1`.

### Special matrices

We have methods to create some special matrices:

| Method Name                | Description                                                                     |
| -------------------------- | ------------------------------------------------------------------------------- |
| [`BitMat::zero`]           | The square zero matrix.                                                         |
| [`BitMat::identity`]       | The square identity matrix.                                                     |
| [`BitMat::left_shift`]     | The `n x n` shift-left by `p` places matrix.                                    |
| [`BitMat::right_shift`]    | The `n x n` shift-right by `p` places matrix.                                   |
| [`BitMat::left_rotation`]  | The `n x n` rotate-left by `p` places matrix.                                   |
| [`BitMat::right_rotation`] | The `n x n` rotate-right by `p` places matrix.                                  |
| [`BitMat::companion`]      | A square [companion matrix] with a top-row and a sub-diagonal that is all ones. |

### Reshaping Vectors

We have methods to reshape a bit-vector into a matrix, either by rows or by columns:

| Method Name                     | Description                                     |
| ------------------------------- | ----------------------------------------------- |
| [`BitMat::from_vector_of_rows`] | Reshapes a bit-vector of rows into a matrix.    |
| [`BitMat::from_vector_of_cols`] | Reshapes a bit-vector of columns into a matrix. |

These methods can fail, so we return a `BitMat` wrapped in an [`Option`] and [`None`] if the size of the vector is not compatible with the requested matrix shape.

### From Strings

Finally, we have a method to construct a bit-matrix from one of its string representations:

| Method Name             | Description                             |
| ----------------------- | --------------------------------------- |
| [`BitMat::from_string`] | Tries to create a matrix from a string. |

This method can fail, so it returns a `BitMat` wrapped in an [Option] and [None] if we cannot parse the string.

The rows in the matrix string can be separated by newlines, white space, commas, single quotes, or semicolons.
Each row should be a binary or hex string representation of a bit-vector.
See [constructing bit-vectors from strings](BitVec.md#construction-from-strings) for details of the accepted formats for each row.

## Resizing and Reshaping

We have methods to resize the matrix where current elements are preserved as much as possible, and any added elements are initialised to zero.

| Method Name             | Description                                  |
| ----------------------- | -------------------------------------------- |
| [`BitMat::resize`]      | Resizes the matrix to the passed dimensions. |
| [`BitMat::clear`]       | This is equivalent to `resize(0,0)`.         |
| [`BitMat::make_square`] | This is equivalent to `resize(m,m)`.         |

## Appending and Removing Rows/Columns

We have methods to append and remove rows and columns from the matrix:

| Method Name             | Description                                                                                         |
| ----------------------- | --------------------------------------------------------------------------------------------------- |
| [`BitMat::append_row`]  | Appends a row to the end of the matrix either by copying or moving a bit-vector.                    |
| [`BitMat::append_rows`] | Appends rows to the end of the matrix either by copying or moving a standard vector of bit-vectors. |
| [`BitMat::append_col`]  | Appends a column to the right of the matrix by copying a bit-vector.                                |
| [`BitMat::append_cols`] | Appends columns to the right of the matrix by copying a standard vector of bit-vectors.             |
| [`BitMat::remove_row`]  | Removes a row from the end of the matrix and returns it as a bit-vector.                            |
| [`BitMat::remove_rows`] | Removes multiple rows from the end of the matrix and returns them as a bit-matrix.                  |
| [`BitMat::remove_col`]  | Removes a column from the right of the matrix and returns it as a bit-vector.                       |

The `remove` methods return an [`Option`]- wrapped result because the request can fail.

## Size and Type Queries

We have methods to query the matrix dimensions and check if it is "special" in some way:

| Method Name              | Description                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| [`BitMat::rows`]         | Returns the number of rows in the matrix.                                |
| [`BitMat::cols`]         | Returns the number of columns in the matrix.                             |
| [`BitMat::len`]          | Returns the number of elements in the matrix.                            |
| [`BitMat::is_empty`]     | Returns `true` if the matrix has no elements.                            |
| [`BitMat::is_square`]    | Returns `true` if the number of rows equals the number of columns.       |
| [`BitMat::is_zero`]      | Returns `true` if this is a _square_ zero matrix.                        |
| [`BitMat::is_identity`]  | Returns `true` if this is a _square_ identity matrix.                    |
| [`BitMat::is_symmetric`] | Returns `true` if this is a _square_ symmetric matrix $M(i,j) = M(j,i)$. |

## Bit Counts

We have methods to count the number of set and unset elements in the matrix, as well as on the main diagonal.

| Method Name                        | Description                                                      |
| ---------------------------------- | ---------------------------------------------------------------- |
| [`BitMat::count_ones`]             | Returns the overall number of set elements in the matrix.        |
| [`BitMat::count_zeros`]            | Returns the overall number of unset elements in the matrix.      |
| [`BitMat::count_ones_on_diagonal`] | Returns the overall number of set elements on the main diagonal. |
| [`BitMat::trace`]                  | Returns the "sum" of the main diagonal elements.                 |
| [`BitMat::any`]                    | Returns `true` if the matrix has any set elements.               |
| [`BitMat::all`]                    | Returns `true` if the matrix is all ones.                        |
| [`BitMat::none`]                   | Returns `true` if the matrix is all zeros.                       |

## General Access

We have methods to access individual elements, rows and columns, and the entire matrix.

| Method Name          | Description                                                                    |
| -------------------- | ------------------------------------------------------------------------------ |
| [`BitMat::get`]      | Access an individual matrix element as a bool.                                 |
| [`BitMat::set`]      | Set an individual matrix element to a value.                                   |
| [`BitMat::flip`]     | Flips the value of an individual matrix element.                               |
| [`BitMat::row`]      | Returns a read-only reference to a matrix row.                                 |
| [`BitMat::row_mut`]  | Returns a mutable reference to a matrix row.                                   |
| [`BitMat::set_row`]  | Sets all the elements of a matrix row to a value.                              |
| [`BitMat::flip_row`] | Flips the value of all the elements of a matrix row.                           |
| [`BitMat::col`]      | Returns a _copy_ of a matrix column as a `gf2::BitVec`.                        |
| [`BitMat::set_all`]  | Sets all matrix elements to a value that defaults to `true`                    |
| [`BitMat::flip_all`] | Flips the values of all matrix elements.                                       |
| [`BitMat::flipped`]  | Returns a new bit-matrix that is a copy of this one with all elements flipped. |

## Diagonal Access

We have methods to access and modify the main diagonal, super-diagonals, and sub-diagonals of a square matrix:

| Method Name                     | Description                                                                       |
| ------------------------------- | --------------------------------------------------------------------------------- |
| [`BitMat::set_diagonal`]        | Sets all the main diagonal elements to a value that defaults to `true`            |
| [`BitMat::flip_diagonal`]       | Flips the values of all the main diagonal elements.                               |
| [`BitMat::set_super_diagonal`]  | Sets all elements of a super-diagonal elements to a value that defaults to `true` |
| [`BitMat::flip_super_diagonal`] | Flips the values of all elements on a super-diagonal.                             |
| [`BitMat::set_sub_diagonal`]    | Sets all elements of a sub-diagonal to a value that defaults to `true`            |
| [`BitMat::flip_sub_diagonal`]   | Flips the values of all elements on a sub-diagonal.                               |

## Sub-Matrices

We have methods to extract and replace sub-matrices:

| Method Name                    | Description                                |
| ------------------------------ | ------------------------------------------ |
| [`BitMat::sub_matrix`]         | Extracts a sub-matrix as a new bit-matrix. |
| [`BitMat::replace_sub_matrix`] | Replaces a sub-matrix.                     |

These methods panic if the requested sub-matrix is out of bounds.

## Triangular Sub-Matrices

We have methods to extract triangular sub-matrices:

| Method Name                | Description                                                             |
| -------------------------- | ----------------------------------------------------------------------- |
| [`BitMat::lower`]          | Returns the lower triangular sub-matrix including the main diagonal.    |
| [`BitMat::upper`]          | Returns the upper triangular sub-matrix including the main diagonal.    |
| [`BitMat::strictly_lower`] | Returns the lower triangular sub-matrix excluding the main diagonal.    |
| [`BitMat::strictly_upper`] | Returns the upper triangular sub-matrix excluding the main diagonal.    |
| [`BitMat::unit_lower`]     | Returns the lower triangular sub-matrix with ones on the main diagonal. |
| [`BitMat::unit_upper`]     | Returns the upper triangular sub-matrix with ones on the main diagonal. |

Triangular extraction methods do not require the matrix to be square.
The returned sub-matrix will have the same number of rows and columns as the original matrix, with the appropriate elements set to zero.

## Elementary Operations

We have methods to perform elementary row and column operations, as well as adding the identity matrix in-place:

| Method Name              | Description                                           |
| ------------------------ | ----------------------------------------------------- |
| [`BitMat::swap_rows`]    | Swaps two rows in a matrix.                           |
| [`BitMat::swap_cols`]    | Swap two columns in a matrix.                         |
| [`BitMat::add_identity`] | Adds the identity matrix to a square matrix in-place. |

These operations are fundamental to many matrix algorithms.

## Transposition

We have methods to transpose a matrix either in-place (for square matrices) or to return a new transposed matrix:

| Method Name            | Description                                                                |
| ---------------------- | -------------------------------------------------------------------------- |
| [`BitMat::transpose`]  | Transposes a square matrix in place.                                       |
| [`BitMat::transposed`] | Returns a new matrix that is the transpose of this arbitrarily shaped one. |

The [`BitMat::transposed`] method works for non-square matrices by creating a new matrix with the appropriate dimensions and filling it in.

## Exponentiation

We have methods to efficiently compute `M^e` for square bit-matrices. $2^n$ for some $n$.

| Method Name                 | Description                                                           |
| --------------------------- | --------------------------------------------------------------------- |
| [`BitMat::to_the`]          | Returns a new matrix that is this one raised to the passed power `n`. |
| [`BitMat::to_the_2_to_the`] | Returns a new matrix that is this one raised to `2^n`.                |

These methods use a square and multiply algorithm, where `e = n` or `e = 2^n` for some `n`.

## Matrix Inversion

We have methods to reduce a matrix to echelon form, reduced echelon form, and to compute the inverse of a square matrix:

| Method Name                         | Description                                                                   |
| ----------------------------------- | ----------------------------------------------------------------------------- |
| [`BitMat::to_echelon_form`]         | Reduces a matrix to echelon form in-place.                                    |
| [`BitMat::to_reduced_echelon_form`] | Reduces a matrix to reduced echelon form in-place.                            |
| [`BitMat::inverse`]                 | Returns the inverse of a matrix or `std::nullopt` on failure.                 |
| [`BitMat::probability_invertible`]  | Returns the probability of a fair random `n x n` matrix being invertible.     |
| [`BitMat::probability_singular`]    | Returns the probability of a fair random `n x n` matrix not being invertible. |

The inversion method can fail so we return an [`Option`] wrapped result.

## Linear System Solvers

| Method Name                  | Description                                                             |
| ---------------------------- | ----------------------------------------------------------------------- |
| [`BitMat::lu_decomposition`] | Returns a [`BitLU`] decomposition object for the matrix.                |
| [`BitMat::solver_for`]       | Returns a [`BitGauss`] object for the matrix and given right-hand side. |
| [`BitMat::x_for`]            | Tries to find one solution to the system of linear equations.           |

**Note:** Over the reals, systems of linear equations can have `0`, `1`, or, if the system is underdetermined, an infinite number
of solutions. By contrast, over [GF(2)], in an underdetermined system, the number of solutions is `2^f,` where `f` is
the number of "free" variables. This is because underdetermined variables can be set to only two values.
Therefore, over [GF(2)] the number of solutions is `0`, `1`, or `2^f,` where `f` is the number of free variables.
The [`BitMat::x_for`] method returns just one solution if there are any; you can use the [`BitGauss`] type to explore the full solution space.

## Characteristic Polynomials

| Method Name                           | Description                                                 |
| ------------------------------------- | ----------------------------------------------------------- |
| [`BitMat::characteristic_polynomial`] | Returns the [characteristic polynomial] of a square matrix. |
| [`BitMat::frobenius_form`]            | Returns the [Frobenius normal] form of a square matrix.     |

The [characteristic polynomial] is computed using Danilevsky's method, which is not well known but is efficient for bit-matrices.
It works by reducing the matrix to [Frobenius normal] form using a series of [similarity transformations] implemented using row and column operations.
See [Danilevsky's method] for all the details.

## Stringification

The following methods return a string representation for a bit-matrix.
The string can be in the obvious binary format or a more compact hex format.

| Method                               | Description                                                               |
| ------------------------------------ | ------------------------------------------------------------------------- |
| [`BitMat::to_custom_binary_string`]  | Returns a configurable binary string representation for a bit-matrix.     |
| [`BitMat::to_binary_string`]         | Returns a multi-line binary string representation for a bit-matrix.       |
| [`BitMat::to_pretty_binary_string`]  | Returns a "pretty" binary string representation for a bit-matrix.         |
| [`std::string::ToString::to_string`] | Delegates to [`BitMat::to_pretty_binary_string`].                         |
| [`BitMat::to_compact_binary_string`] | Returns a one-line minimal binary string representation for a bit-matrix. |
| [`BitMat::to_hex_string`]            | Returns a hex string representation for a bit-matrix.                     |
| [`BitMat::to_compact_hex_string`]    | Returns a one-line minimal hex string representation for a bit-matrix.    |

See the [stringification](BitStore#stringification) section of the [`BitStore`] documentation for details on the available options for formatting the matrix's rows.

### Example

```rust
use gf2::*;
let mat: BitMat = BitMat::random(5, 15);
println!("mat.to_binary_string():\n{}\n", mat.to_binary_string());
println!("mat.to_pretty_binary__string():\n{}\n", mat.to_pretty_binary_string());
println!("mat.to_string():\n{}\n", mat.to_string());
println!("mat.to_compact_binary_string():\n{}\n", mat.to_compact_binary_string());
println!("mat.to_hex_string():\n{}\n", mat.to_hex_string());
println!("mat.to_compact_hex_string():\n{}\n", mat.to_compact_hex_string());
```

That might produce output like this:

```txt
mat.to_binary_string():
1 1 0 0 1 0 1 0 0 0 1 1 0 0 1
0 0 0 0 0 0 0 0 1 0 0 0 1 0 1
0 0 1 0 0 0 0 1 0 1 1 1 1 1 1
1 0 0 1 0 1 0 0 0 1 1 1 1 1 0
1 1 1 1 0 1 0 1 0 1 1 1 0 1 1

mat.to_pretty_binary__string():
│1 1 0 0 1 0 1 0 0 0 1 1 0 0 1│
│0 0 0 0 0 0 0 0 1 0 0 0 1 0 1│
│0 0 1 0 0 0 0 1 0 1 1 1 1 1 1│
│1 0 0 1 0 1 0 0 0 1 1 1 1 1 0│
│1 1 1 1 0 1 0 1 0 1 1 1 0 1 1│

mat.to_string():
│1 1 0 0 1 0 1 0 0 0 1 1 0 0 1│
│0 0 0 0 0 0 0 0 1 0 0 0 1 0 1│
│0 0 1 0 0 0 0 1 0 1 1 1 1 1 1│
│1 0 0 1 0 1 0 0 0 1 1 1 1 1 0│
│1 1 1 1 0 1 0 1 0 1 1 1 0 1 1│

mat.to_compact_binary_string():
110010100011001 000000001000101 001000010111111 100101000111110 111101010111011

mat.to_hex_string():
CA31.8
0085.8
2177.8
9476.8
F573.8

mat.to_compact_hex_string():
CA31.8 0085.8 2177.8 9476.8 F573.8
```

## Utility String Functions

We have some utility functions that return string representations for matrices & vectors side-by-side:

| Method              | Description                                           |
| ------------------- | ----------------------------------------------------- |
| [`string_for_Au`]   | A string for a matrix and vector side-by-side.        |
| [`string_for_Auv`]  | A string for a matrix and two vectors side-by-side.   |
| [`string_for_Auvw`] | A string for a matrix and three vectors side-by-side. |
| [`string_for_AB`]   | A string for two matrices side-by-side.               |
| [`string_for_ABC`]  | A string for two matrices side-by-side.               |

These functions are not re-exported by default so you need to import the `gf2::mat` module to use them or prefix them with `gf2::mat::` as shown in the following example, where we show a matrix alongside its lower and upper triangular parts:

```rust
use gf2::*;
let mat: BitMat = BitMat::ones(7, 7);
println!("    M      L       U");
println!("{}", mat::string_for_ABC(&mat,&mat.lower(), &mat.strictly_upper()));
```

Output:

```txt
    M      L       U
1111111 1000000 0111111
1111111 1100000 0011111
1111111 1110000 0001111
1111111 1111000 0000111
1111111 1111100 0000011
1111111 1111110 0000001
1111111 1111111 0000000
```

## Bitwise Operations

We have methods that combine two bit-matrices using the logical operations `XOR`, `AND`, and `OR`.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">❗</div>

These methods require that the two bit-matrices use the same underlying word type.
They also require that the left-hand-side and right-hand-side bit-store operands are the same size.
That precondition is always checked.
Interactions between bit-matrices with different word types are only possible at the cost of increased code complexity, and are not a common use case.

</div>

The methods can act in place, mutating the left-hand side caller: `lhs.xor_eq(rhs)`.
There are also non-mutating versions like `result = lhs.xor(rhs)`, which returns a new `result` bit-matrix in each case.

| Method             | Description                                                                           |
| ------------------ | ------------------------------------------------------------------------------------- |
| [`BitMat::xor_eq`] | In-place `XOR` operation of equal-sized bit-matrices: `lhs = lhs ^ rhs`.              |
| [`BitMat::and_eq`] | In-place `AND` operation of equal-sized bit-matrices: `lhs = lhs & rhs`.              |
| [`BitMat::or_eq`]  | In-place `OR` operation of equal-sized bit-matrices: `lhs = lhs \| rhs`.              |
| [`BitMat::xor`]    | Returns the `XOR` of this matrix with another equal-sized matrix as a new bit-matrix. |
| [`BitMat::and`]    | Returns the `AND` of this matrix with another equal-sized matrix as a new bit-matrix. |
| [`BitMat::or`]     | Returns the `OR` of this matrix with another equal-sized matrix as a new bit-matrix.  |

**Note:** We have also implemented the [`std::ops::BitXorAssign`], [`std::ops::BitAndAssign`], [`std::ops::BitOrAssign`], [`std::ops::BitXor`], [`std::ops::BitAnd`], and [`std::ops::BitOr`] foreign traits to provide operator overloads for the bit-wise operations. Those implementations forward to the associated methods above.

## Arithmetic Operations

In GF(2), the arithmetic operators `+` and `-` are both the `XOR` operator.

| Method               | Description                                                                        |
| -------------------- | ---------------------------------------------------------------------------------- |
| [`BitMat::plus_eq`]  | Adds the passed (equal-sized) `rhs` bit-matrix to this one.                        |
| [`BitMat::minus_eq`] | Subtracts the passed (equal-sized) `rhs` bit-matrix from this one.                 |
| [`BitMat::plus`]     | Adds two equal-sized bit-matrices and returns the result as a new bit-matrix.      |
| [`BitMat::minus`]    | Subtracts two equal-sized bit-matrices and returns the result as a new bit-matrix. |

**Note:** We have also implemented the [`std::ops::AddAssign`], [`std::ops::SubAssign`], [`std::ops::Add`], and [`std::ops::Sub`] foreign traits to provide operator overloads for the arithmetic operations. Those implementations forward to the associated methods above.

## Multiplication Operations

We have methods to perform vector-matrix, matrix-vector, and matrix-matrix multiplication:

| Method Name            | Description                                                         |
| ---------------------- | ------------------------------------------------------------------- |
| [`BitMat::dot`]        | Matrix-vector multiplication returning `M * v` as a new bit-vector. |
| [`BitMat::left_dot`]   | Vector-matrix multiplication returning `v * M` as a new bit-vector. |
| [`BitMat::dot_matrix`] | Matrix-matrix multiplication returning `M * N` as a new bit-matrix. |

These methods panic if the dimensions of the operands are not compatible for multiplication.

### Notes

- We have also implemented the [`std::ops::Mul`] and [`std::ops::MulAssign`] foreign traits to provide operator overloads for matrix-matrix multiplication. That implementation forwards to the associated `dot_matrix` method above.
- We have also implemented the [`std::ops::Mul`] foreign trait to provide operator overloads for matrix-vector and vector-matrix multiplication for each of the concrete bit-store types. Those implementations forward to the associated `dot` and `left_dot` method above. The implementations were generated using macros to avoid code duplication and work on any combination of bit-matrix and bit-vector concrete types passed by value or by reference.

## Foreign Traits for Individual Bit-Matrices

We have implemented several foreign traits that acts on a single bit-matrix:

| Trait Name             | Description                                        |
| ---------------------- | -------------------------------------------------- |
| [`Default`]            | Forwarded to [`BitMat::new`].                      |
| [`std::ops::Index`]    | Forwarded to [`BitMat::row`].                      |
| [`std::ops::IndexMut`] | Forwarded to [`BitMat::row_mut`].                  |
| [`std::ops::Not`]      | Forwarded to [`BitMat::flipped`].                  |
| [`std::fmt::Display`]  | Forwarded to [`BitMat::to_pretty_binary_string`].  |
| [`std::fmt::Debug`]    | Forwarded to [`BitMat::to_compact_binary_string`]. |
| [`std::fmt::Binary`]   | Forwarded to [`BitMat::to_binary_string`].         |
| [`std::fmt::UpperHex`] | Forwarded to [`BitMat::to_hex_string`].            |
| [`std::fmt::LowerHex`] | Forwarded to [`BitMat::to_hex_string`].            |

The [`std::ops::Not`] trait is implemented both for bit-matrices by value and by reference.

## Foreign Traits for Pairs of Bit-Matrices

Other foreign traits act on _pairs_ of bit-matrices:

| Trait Name                 | Description                       |
| -------------------------- | --------------------------------- |
| [`std::ops::BitXorAssign`] | Forwarded to [`BitMat::xor_eq`]   |
| [`std::ops::BitAndAssign`] | Forwarded to [`BitMat::and_eq`]   |
| [`std::ops::BitOrAssign`]  | Forwarded to [`BitMat::or_eq`]    |
| [`std::ops::AddAssign`]    | Forwarded to [`BitMat::plus_eq`]  |
| [`std::ops::SubAssign`]    | Forwarded to [`BitMat::minus_eq`] |
| [`std::ops::BitXor`]       | Forwarded to [`BitMat::xor`]      |
| [`std::ops::BitAnd`]       | Forwarded to [`BitMat::and`]      |
| [`std::ops::BitOr`]        | Forwarded to [`BitMat::or`]       |
| [`std::ops::Add`]          | Forwarded to [`BitMat::plus`]     |
| [`std::ops::Sub`]          | Forwarded to [`BitMat::minus`]    |
| [`std::ops::Mul`]          | Forwarded to [`BitMat::dot`]      |

These traits are implemented for bit-matrices passed either by value or by reference, so the following combinations are supported:
For example:

```rust
use gf2::*;
let u: BitMat = BitMat::random(10, 10);
let v: BitMat = BitMat::random(10, 10);
let a = &u + &v;    // `a` is a new `BitMat`; `u` and `v` are both preserved.
let b = &u + v;     // `b` is a new `BitMat`; we cannot use `v` again.
let c = u + &b;     // `c` is a new `BitMat`; we cannot use `u` again.
let d = b + c;      // `d` is a new `BitMat`; we cannot use either `b` or `c` again.
```

This is very different from C++, where operator overloads are typically defined to preserve both arguments.

```cpp
auto u = gf2::BitMat<>::random(10, 10);
auto v = gf2::BitMat<>::random(10, 10);
auto a = u + v;     // `a` is a new `BitMat`; `u` and `v` are both preserved.
```

In C++, you don't have to write `a = &u + &v` to preserve both operands, instead, you just write `a = u + v` with no ampersands.
The syntax is cleaner for the most common use case.

## Foreign Traits for Bit-Matrix and Bit-Store Pairs

We have also implemented foreign traits for matrix-sore products, for each concrete bit-store type:

| Trait Name        | Description                                                  |
| ----------------- | ------------------------------------------------------------ |
| [`std::ops::Mul`] | Forwarded to [`BitMat::dot`] for matrix-store products.      |
| [`std::ops::Mul`] | Forwarded to [`BitMat::left_dot`] for store-matrix products. |

These traits are implemented for all combinations of bit-matrix and concrete bit-store passed either by value or by reference.
The implementations are generated using macros to avoid code duplication.

## See Also

- [`BitStore`](BitStore.md) for the concept API shared by all bit-stores.
- [`BitArray`](BitArray.md) for fixed-size vectors of bits.
- [`BitVec`](BitVec.md) for dynamically-sized vectors of bits.
- [`BitSlice`](BitSlice.md) for non-owning views into any bit-store.
- [`BitPoly`](BitPoly.md) for polynomials over GF(2).
- [`BitLU`](BitLU.md) for LU decomposition of bit-matrices.
- [`BitGauss`](BitGauss.md) for solving linear systems of equations over GF(2).
- [Danilevsky's method] for computing characteristic polynomials.

<!-- Reference Links -->

[GF(2)]: https://en.wikipedia.org/wiki/GF(2)
[Galois-Field]: https://en.wikipedia.org/wiki/Galois_field
[Danilevsky's method]: https://nessan.github.io/gf2/Danilevsky.html
[companion matrix]: https://en.wikipedia.org/wiki/Companion_matrix
[Frobenius form]: https://encyclopediaofmath.org/wiki/Frobenius_matrix
[characteristic polynomial]: https://en.wikipedia.org/wiki/Characteristic_polynomial
[similarity transformations]: https://en.wikipedia.org/wiki/Matrix_similarity
