#![doc = include_str!("../docs/gf2.md")]
// Enable unstable Rust features when the `unstable` feature is enabled.
// - The `BitArray` type requires simple arithmetic on const generic parameters.
// - Function traits lets us have the extra evaluation: `p(M)` for a bit-polynomial `p` and a bit-matrix `M`.
#![cfg_attr(feature = "unstable", feature(fn_traits))]
#![cfg_attr(feature = "unstable", feature(unboxed_closures))]
#![cfg_attr(feature = "unstable", feature(generic_const_exprs))]
#![cfg_attr(feature = "unstable", allow(incomplete_features))]

// Most foreign trait implementations for the concrete `BitStore` types are defined using macros to avoid code
// duplication. Some of the macros are also used by `BitMat` so need to be seen first.
// The macros themselves are not part of the public API.
#[macro_use]
mod store_traits;

// `Unsigned` is a trait for the primitive unsigned integer types that can back a bit-store.
pub mod unsigned;
pub use unsigned::Unsigned;

// `BitStore` is the core trait for `BitArray`, `BitVec`, and `BitSlice`.
pub mod store;
pub use store::BitStore;

// `BitArray` is a _statically sized_ array of bits --- a _bit-array_.
// We need  arithmetic on const generic parameters to implement `BitArray`, so gate it behind the `unstable` feature.
#[cfg(feature = "unstable")]
pub mod array;
#[cfg(feature = "unstable")]
pub use array::BitArray;

// `BitVec` is a _dynamically sized_ vector of bits --- a _bit-vector_.
pub mod vec;
pub use vec::BitVec;

// `BitSlice` is a non-owning view of a range of bits within any bit-store --- a _bit-slice_.
pub mod slice;
pub use slice::BitSlice;

// `Bits`, `SetBits`, `UnsetBits`, and `Words` iterators over any bit-store.
pub mod iterators;
pub use iterators::{
    Bits,
    SetBits,
    UnsetBits,
    Words,
};

// `BitPoly` is a polynomial over GF(2) --- a _bit-polynomial_.
pub mod poly;
pub use poly::BitPoly;

// `BitMat` is a _dynamically sized_ matrix of bits --- a _bit-matrix_.
pub mod mat;
pub use mat::BitMat;

// `BitGauss` is a Gaussian elimination solver for systems of linear equations over GF(2).
pub mod gauss;
pub use gauss::BitGauss;

// `BitLU` provides the LU decomposition for bit-matrices.
pub mod lu;
pub use lu::BitLU;

// `rng` is a helper module that needs to be visible but which exports nothing outside the crate.
// It provides a simple shared PRNG that is used to fill bit-stores and bit-matrices with random values.
mod rng;
