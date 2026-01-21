# Changelog

All notable changes will be documented in this file.

## Jan-2026

-   Initial release of the project.
-   The crate is named `gf2` and is available on [crates.io](https://crates.io/crates/gf2).
-   It is a rewrite in Rust of the [C++ version](https://github.com/nessan/gf2) with some changes to keep it more idiomatic to Rust.
-   For the most part, the Rust version is feature-equivalent to the C++ version, with a few exceptions:
    -   There is no clean way in Rust to have a reference to a single bit in a bit-vector, so this feature is not implemented.
    -   The fixed-size bit vector type (`gf2::BitArray<N>`) is only available in unstable Rust for now, due to `const generic` limitations.
    -   Polynomial evaluation for bit-matrices is available but the natural syntax `p(M)` is only available in unstable Rust for now.
-   The main provided types and traits are:
    -   The `Unsigned` trait that is implemented for all Rust's primitive unsigned integer types.
    -   The `BitStore` trait that provides a common interface for the three vector-like types.
        -   `BitVec` for dynamically-sized bit vectors.
        -   `BitArray<N>` for fixed-size bit vectors (only available in nightly Rust for now).
        -   `BitSlice` for non-owning views into any bit-store.
    -   Various iterator types for the bits, set bits, unset bits, and store words for all bit-stores.
    -   `BitPoly` for polynomials over GF(2) along with polynomial operations & algorithms.
    -   `BitMat` for matrices over GF(2) along with matrix operations & algorithms
    -   `BitGauss` to solve linear systems of equations.
    -   `BitLU` to provide the `LU` decomposition for bit-matrices.
