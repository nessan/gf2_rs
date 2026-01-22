# Foreign Trait Implementations

## Introduction

There are three vector-like types in this library:

| Type             | Description                           |
| ---------------- | ------------------------------------- |
| `gf2::BitArray`  | Fixed-size bit-vectors.               |
| `gf2::BitVector` | Dynamically sized bit-vectors.        |
| `gf2::BitSlice`  | A non-owning view of contiguous bits. |

These types have a large number of methods in common, which they inherit from the [`BitStore`] trait that they all implement.

There are also many foreign traits in the Rust standard library that we would like to implement for all bit-store types. However, Rust's orphan rules prevent us from doing this in a blanket way. If you try, you will get compiler errors and somewhat opaque complaints about "coherence and overlap" issues.

You simply cannot implement foreign traits for any `BitStore` type in a blanket manner. The compiler worries that someone might later define a new type that implements `BitStore` and also implements its own version of the foreign trait, leading to ambiguity. There is no way to close a trait in Rust to prevent this from happening.

Instead, you have to implement the foreign traits for each concrete type separately. This leads to a lot of code duplication, particularly as many of the foreign traits of interest work on _pairs_ of bit-stores (for example, the `Add` trait). We have three bit-store types, so there are nine possible pairs to consider, and that is before you consider whether each argument was passed by value or by reference.

To avoid this code duplication, we define a set of macros that implement the foreign traits for all the bit-store types in a uniform manner.

## Foreign Traits for Individual Bit-Stores

The simplest case is where a foreign trait acts on a single bit-store type:

- `Index`
- `Display`
- `Binary`
- `UpperHex`
- `LowerHex`
- `ShlAssign`
- `ShrAssign`
- `Shl`
- `Shr`

Our `impl_unary_traits!` macro implements these foreign traits for any concrete _individual_ bit-store type.
For example, we can invoke `impl_unary_traits!(BitVector)` to implement all these traits for the `BitVector` type.

## Foreign Traits for Pairs of Bit-Stores

Other foreign traits act on _pairs_ of bit-store types:

- `BitXorAssign`
- `BitAndAssign`
- `BitOrAssign`
- `AddAssign`
- `SubAssign`
- `MulAssign`
- `BitXor`
- `BitAnd`
- `BitOr`
- `Add`
- `Sub`
- `Mul`

Our `impl_binary_traits!` macro implements these foreign traits for any concrete _pair_ of bit-store types.
For example, we can invoke `impl_binary_traits!(BitVector, BitSlice)` to implement all these traits for the `BitVector` type getting operations with a `BitSlice` type.

## The Macros

The macros are lengthy but straightforward, with a single meaningful match arm.

The one twist is that while all of our bit-store types have a generic `Word: Unsigned` parameter, some types have an extra generic parameter (a lifetime for `BitSlice`, and a `const N: usize` for `BitArray`).

- `BitVector<Word>` has a single generic parameter.
- `BitSlice<'a, Word>` has two generic parameters, the first of which is a lifetime.
- `BitArray<const N, Word>` has two generic parameters, the first of which is `const usize`.

Handling the existence/non-existence of these extra generic parameters is the main complexity in the macros.
