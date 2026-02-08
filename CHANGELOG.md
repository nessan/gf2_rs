# Changelog

All notable changes will be documented in this file.

## Feb-2026

- Fixed a bug that occurred when you took a sub-slice of a bit-slice.
- Added methods to maintain feature compatibility with the C++ library:
    - Added some methods to split a bit-polynomial into pieces.
    - Added methods to copy all the bits from an _iteration_ of any unsigneds to an equal-size bit-store.
    - Added methods to construct a bit-vector from an _iteration_ of any unsigned values.
    - Added methods to append all the bits from an _iteration_ of any unsigneds to the end of a bit-vector.

## Jan-2026

- Initial release of the project.
- The crate is named `gf2` and is available on [crates.io](https://crates.io/crates/gf2).
- It is a rewrite in Rust of the [C++ version](https://github.com/nessan/gf2) with some changes to keep it more idiomatic to Rust.
- For the most part, the Rust version is feature-equivalent to the C++ version, with a few exceptions:
    - There is no clean way in Rust to have a reference to a single bit in a bit-vector, so this feature is not implemented.
    - The fixed-size bit vector type (`gf2::BitArray<N>`) is only available in unstable Rust for now, due to `const generic` limitations.
    - Polynomial evaluation for bit-matrices is available but the natural syntax `p(M)` is only available in unstable Rust for now.
- The main provided types and traits are:
    - The `Unsigned` trait that is implemented for all Rust's primitive unsigned integer types.
    - The `BitStore` trait that provides a common interface for the three vector-like types.
        - `BitVector` for dynamically-sized bit vectors.
        - `BitArray<N>` for fixed-size bit vectors (only available in nightly Rust for now).
        - `BitSlice` for non-owning views into any bit-store.
    - Various iterator types for the bits, set bits, unset bits, and store words for all bit-stores.
    - `BitPolynomial` for polynomials over GF(2) along with polynomial operations & algorithms.
    - `BitMatrix` for matrices over GF(2) along with matrix operations & algorithms
    - `BitGauss` to solve linear systems of equations.
    - `BitLU` to provide the `LU` decomposition for bit-matrices.
