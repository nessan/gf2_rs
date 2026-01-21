# The [`BitGauss`] Type

## Introduction

`BitGauss` is a Gaussian elimination solver for systems of linear equations over [GF(2)]:

```txt
A.x = b
```

where `A` is an `n x n` square bit-matrix, `b` is the known right-hand side vector or length `n`, and `x` is the vector of unknowns of length `n` to be solved for.

On construction, a `BitGauss` object captures copies of `A` and `b`.
Then, it uses [elementary row operations] to transform the left-hand side matrix to [reduced row echelon form] while simultaneously performing identical operations to the right-hand side vector.
With those in place, the solver can quickly produce solutions `x` by simple back-substitution.

As well as getting solutions for the system `A.x = b`, the `BitGauss` object can be queried for other helpful information, such as the [rank] of `A`, whether the system is consistent (i.e., whether any solutions exist), and so on.
See the complete list below.

Over the reals, systems of linear equations can have `0`, `1`, or, if the system is underdetermined, an infinite number
of solutions. By contrast, over [GF(2)], in an underdetermined system, the number of solutions is `2^f` where `f` is
the number of "free" variables. This is because underdetermined variables can be set to only two values.
Therefore, over [GF(2)] the number of solutions is `0`, `1`, or `2^f,` where `f` is the number of free variables.

If the system is consistent, then we can index the solutions by an integer `i` such that `0 <= i < 2^f`.

- The [`BitGauss::x`] method returns a random solution.
- The [`BitGauss::xi`] method returns "the" solution indexed `i`.

For underdetermined systems, the "indexing" is something convenient and consistent across runs but not unique.

### Example

```rust
use gf2::*;
let mat: BitMat = BitMat::ones(4, 4);
let b: BitVec = BitVec::ones(4);

let solver: BitGauss = BitGauss::new(&mat, &b);
println!("The matrix:\n{}", mat);
println!("Matrix rank:              {}", solver.rank());

println!("The `b` vector:           {}", b.to_pretty_string());
println!("Solution count:           {}", solver.solution_count());
println!("Number of free variables: {}", solver.free_count());
println!("Under-determined system?  {}", solver.is_underdetermined());
println!("Consistent system?        {}", solver.is_consistent());
println!("Solutions:");
for i in 0..solver.solution_count() {
    println!("x({}): {}", i, solver.xi(i).unwrap().to_pretty_string());
}
```

### Output

```txt
The matrix:
│1 1 1 1│
│1 1 1 1│
│1 1 1 1│
│1 1 1 1│
Matrix rank:              1
The `b` vector:           [1 1 1 1]
Solution count:           8
Number of free variables: 3
Under-determined system?  true
Consistent system?        true
Solutions:
x(0): [1 0 0 0]
x(1): [0 1 0 0]
x(2): [0 0 1 0]
x(3): [1 1 1 0]
x(4): [0 0 0 1]
x(5): [1 1 0 1]
x(6): [1 0 1 1]
x(7): [0 1 1 1]
```

## Method Summary

We have the following methods available on `BitGauss` objects:

| Method                           | Description                                                                                            |
| -------------------------------- | ------------------------------------------------------------------------------------------------------ |
| [`BitGauss::new`]                | Constructs a new `BitGauss` solver from a given bit-matrix `A` and bit-vector `b`.                     |
| [`BitGauss::rank`]               | Returns the [rank] of the matrix `A`.                                                                  |
| [`BitGauss::is_consistent`]      | Returns `true` if the system is consistent (i.e., has at least one solution).                          |
| [`BitGauss::is_underdetermined`] | Returns `true` if the system is underdetermined (i.e., has more variables than independent equations). |
| [`BitGauss::free_count`]         | Returns the number of free variables in the system.                                                    |
| [`BitGauss::solution_count`]     | Returns the total number of solutions to the system.                                                   |
| [`BitGauss::x`]                  | Returns a random solution vector `x` if the system is consistent, or `None` if it is not.              |
| [`BitGauss::xi`]                 | Returns the solution vector `x` indexed by `i` if the system is consistent.                            |

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

Over the reals, systems of linear equations can have `0`, `1`, or, if the system is underdetermined, an infinite number
of solutions. By contrast, over [GF(2)], in an underdetermined system, the number of solutions is `2^f` where `f` is
the number of "free" variables. This is because underdetermined variables can be set to only two values.
Therefore, over [GF(2)] the number of solutions is `0`, `1`, or `2^f,` where `f` is the number of free variables.
If more than one solution exists, the [`BitGauss::x`] method returns one of them at random, while the [`BitGauss::xi`] method returns a specific solution indexed by a parameter `i`.

</div>

## See Also

- [`BitStore`](BitStore.md) for the concept API shared by all bit-stores.
- [`BitArray`](BitArray.md) for fixed-size vectors of bits.
- [`BitVec`](BitVec.md) for dynamically-sized vectors of bits.
- [`BitSlice`](BitSlice.md) for non-owning views into any bit-store.
- [`BitPoly`](BitPoly.md) for polynomials over GF(2).
- [`BitLU`](BitLU.md) for LU decomposition of bit-matrices.

<!-- Internal Reference Links -->

[`BitLU`]: crate::BitLU

<!-- External Reference Links -->

[GF(2)]: https://en.wikipedia.org/wiki/Finite_field_arithmetic
[Gaussian elimination]: https://en.wikipedia.org/wiki/Gaussian_elimination
[elementary row operations]: https://en.wikipedia.org/wiki/Elementary_matrix
[reduced row echelon form]: https://en.wikipedia.org/wiki/Row_echelon_form
[rank]: https://en.wikipedia.org/wiki/Rank_(linear_algebra)
