# Introduction

`gf2` is a Rust crate for _numerical_ work in _bit space_, where mathematical entities such as vectors, matrices, and polynomial coefficients are represented as zeros and ones.

All arithmetic is carried out modulo 2, so what starts in bit-space stays in bit-space. Addition/subtraction becomes the `XOR` operation, and multiplication/division becomes the `AND` operation. The `gf2` library uses those equivalences to efficiently perform most operations by simultaneously operating on entire blocks of elements at a time.

Mathematicians refer to bit-space as [GF(2)][]. It is the simplest [Galois Field] with just two elements, 0 and 1.

While computer hardware famously operates on bits, computers don't really provide _direct_ access to _single_ bits.

Instead, computers have memory registers for _words_ where the smallest addressable unit is an eight-bit word, a _byte_.
Other "native" word lengths vary by computer architecture, but 8, 16, 32, 64, and even 128-bit words are widely supported.
Computers perform operations on and between those short word types optimally.

Computers have many primitive data types --- bytes, characters, various-sized integers (positive or negative), and floating-point numbers with various degrees of precision.
Blocks of zeros and ones are best modelled by the simplest _unsigned integer_ primitive types.

In this library, we pack contiguous bit elements into arrays of one of those unsigned word types

For example, if we have a bit-vector of size 200, and the underlying word is a `u64`, the bit elements will be packed into four words (a total of 256 bits), and there will be 56 bits of unused capacity.
The library will efficiently perform almost all operations on that vector 64 bits at a time in an inherently parallel manner.

## Traits

The crate has two principal traits:

| Trait        | Description                                                                   |
| ------------ | ----------------------------------------------------------------------------- |
| [`Unsigned`] | This is a trait that is implemented for all Rust's primitive unsigned types.  |
| [`BitStore`] | This is the core trait implemented by bit-vectors, bit-array, and bit-slices. |

The types in this crate pack the individual bit elements into natural-word blocks --- this is a user's choice of a particular primitive unsigned integer type, captured in the crate by the [`Unsigned`] trait.
The default generic `Word` is `usize,` which is appropriate for most applications.
You might choose to use, say, a `u8` instead if your application involved creating a huge number of smaller bit-vectors and bit-matrices.

Because arithmetic operations in [GF(2)] are mod 2, addition/subtraction becomes the `XOR` operation, and multiplication/division becomes the `AND` operation. The `gf2` library uses those equivalences to efficiently perform most interactions on and between bit-vectors and bit-matrices by simultaneously working on whole blocks of elements.

This means we never have to worry about overflows or carries as we would with normal integer arithmetic.
Moreover, these operations are highly optimised in modern CPUs, enabling fast computation even on large bit matrices and bit vectors.

## Vector-Like Types

The crate provides the following vector-like types --- _bit-stores_:

| Type          | Description                                                                       |
| ------------- | --------------------------------------------------------------------------------- |
| [`BitArray`]  | A _bit-set_ --- fixed-size vector of bits (this requires the `unstable` feature). |
| [`BitVector`] | A _bit-vector_ --- dynamically-sized vector of bits.                              |
| [`BitSlice`]  | A _bit-slice_ --- non-owning view into contiguous ranges of bits.                 |

The [`BitArray`], [`BitVector`], and [`BitSlice`] types have _many_ methods in common, and they all satisfy the [`BitStore`] trait, which provides a rich set of associated methods for manipulating collections of bits. The methods include bit accessors, mutators, fills, queries, stringification methods, bit-wise operators, arithmetic operators, and more.

There are also various iterator types for iterating over all bits, set bits, unset bits, and underlying words in any bit-store.

| Iterator      | Associated Type | Description                                         |
| ------------- | --------------- | --------------------------------------------------- |
| [`Bits`]      | `bool`          | An iterator over all the _bits_.                    |
| [`SetBits`]   | `usize`         | An iterator over the _indices_ of the _set_ bits.   |
| [`UnsetBits`] | `usize`         | An iterator over the _indices_ of the _unset_ bits. |
| [`Words`]     | `Unsigned`      | An iterator over the _words_ that hold the bits.    |

## Bit-Polynomials

The [`BitPolynomial`] type represents polynomials over GF(2) --- _bit-polynomials_.
The coefficients are stored as a [`BitVector`].

The type has methods for polynomial arithmetic (addition, multiplication, etc.)

The type can be used to compute `x^N` modulo any bit-polynomial, where `N` can be a huge integer.
This is useful for computing large jumps and parallelising simulations for some pseudo-random number generators.

## Bit-Matrices

The [`BitMatrix`] type is a dynamically-sized matrix of bits— _bit-matrices_.

| Type          | Description                                                                |
| ------------- | -------------------------------------------------------------------------- |
| [`BitMatrix`] | A _bit-matrix_ --- dynamically-sized matrix of bits.                       |
| [`BitGauss`]  | A Gaussian elimination solver for systems of linear equations over [GF(2)] |
| [`BitLU`]     | Provides the `LU` decomposition for bit-matrices.                          |

There are methods for all the usual interactions between bit-matrices and bit-vectors.

There are also methods for computing characteristic polynomials, solving linear systems, and more.

## A Simple Example

Here is a simple example of a program that uses `gf2`:

```rs
use gf2::*;
let m: BitMatrix = BitMatrix::random(6, 6);
let c = m.characteristic_polynomial();
println!("The matrix m:\n{}", m);
println!("has the characteristic polynomial: c(x) = {}.", c);
println!("The polynomial sum c(m):\n{}", c(&m));
```

This program creates a random 6 x 6 bit-matrix `m` where 0 & 1 are equally likely to occur and then extracts its characteristic polynomial c(x) = c0 + c1 x + c2 x^2 + ... + c6 x^6. Finally, the program verifies that `m` satisfies its characteristic equation as expected from the [Cayley-Hamilton] theorem.

Here is the output from one run of the program:

```txt
The matrix m:
│1 0 0 0 1 0│
│0 0 1 1 1 0│
│0 1 1 0 1 0│
│1 0 0 0 1 1│
│0 0 1 1 1 1│
│1 0 0 1 0 1│
has the characteristic polynomial: c(x) = x^3 + x^6.
The polynomial sum c(m):
│0 0 0 0 0 0│
│0 0 0 0 0 0│
│0 0 0 0 0 0│
│0 0 0 0 0 0│
│0 0 0 0 0 0│
│0 0 0 0 0 0│
```

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; background-color: #f9f9f9; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>
<div style="flex: 1;">

The `gf2` crate makes it possible to quickly extract the characteristic polynomial for a bit-matrix with millions of elements. This problem chokes a naive implementation that fails to account for the unique nature of arithmetic in GF(2).

</div>
</div>

## Why Use `gf2`?

There are other bit-vector and even some bit-matrix crates available from the [crate registry](crates.io).

However, the types in those crates are typically aimed at _bitsets_ instead of _bit-vectors_. So, for example, they print in _bit-order_ with the least significant element/bit on the right. More importantly, the types don't have any numerical methods, and neither does the standard library's vector class [`Vec`].

On the other hand, several well-known linear algebra libraries, such as [`Eigen`] in C++ and [`nalgebra`] in Rust, exist. Those packages efficiently manage all the standard _numeric_ types (floats, doubles, integers, etc.) but do not correctly handle GF(2). You can create matrices of integers whose elements are 0 or 1, but there is no built-in knowledge in those libraries that arithmetic is mod 2.

For example, you might use `Eigen` to create an integer matrix of all 0's and 1's and then use a built-in function from that library to extract the characteristic polynomial. Modding the coefficients of that polynomial by 2 gets the appropriate version for GF(2). Technically, this works, but you will have overflow problems for even relatively modest-sized matrices with just a few hundred rows and columns. Of course, you might use an underlying `BitInt` type that never overflows, but the calculations become dog slow for larger bit-matrices, which doesn't help much.

This specialised `gf2` library is a much better option for numerical problems over GF(2). Consider it if, for example, your interest is in cryptography or random number generation.

## Installation

This crate is stand-alone with no dependencies beyond the standard library. <br>
Drop the `gf2` crate somewhere convenient, and you're good to go.

## Unstable Features

Some APIs (notably the [`BitArray`] type, and [`BitPolynomial`]'s function-call syntax `p(x)`/`p(M)`) require a few features that Rust considers "unstable" at the time of writing (though the features have been available for a very long time).

To use those APIs you can use either use stable Rust with the environment variable `RUSTC_BOOTSTRAP` set to `1` or use the nightly Rust compiler.

In either case, you need to enable the features by adding this to your `Cargo.toml`:

```toml
[dependencies.gf2]
features = ["unstable"]
```

or, on the command line, by adding the `--features unstable` flag to `cargo` builds.

If the `unstable` feature is not enabled, the `BitArray` type will not be available, and the `BitPolynomial` type will not support the function-call syntax `p(x)`/`p(M)`. Instead, you can use the methods `eval_bool`/`eval_matrix` to evaluate polynomials at a bit-vector or bit-matrix argument.

## C++ Version

This Rust crate started life as a _port_ of an equivalent header-only [C++ library][], which has its own [documentation site][].

The port was done _manually_ --- at least for now, LLM's cannot handle this sort of translation task and produce anything that is at all readable or verifiable.

As you might expect with a rewrite, the new version considerably improved on the original. There were two beneficial factors at play:

- We approached the problem anew, and fresh eyes quickly saw several areas for improvement that had nothing to do with the implementation language per se.
- Other improvements came about _because_ we were using a different language with its own idioms, strengths, and weaknesses that forced some new thinking.

The C++ version has been completely rewritten to incorporate those improvements and to backport some of the new ideas from using Rust.

Writing solutions to the same problem in multiple languages has significant benefits, but of course, it is rather expensive and unlikely to find favour in commercial settings.

Perhaps we should repeat the exercise for a third language someday!

For the most part, the two versions are feature equivalent (a few things are not possible in Rust). There are some name changes to accommodate language idioms, for example, the `BitSpan` C++ class is the `BitSlice` type in Rust (C++ uses spans, Rust uses slices), C++ vectors have a `size()` method, Rust vectors have a `len()` method, and so on.

The two versions have very similar performance characteristics, with neither being significantly faster than the other in most scenarios.

## Useful Links

Here are links to the project's [repository][] and [documentation site][]

You can contact me by [email][].

### Copyright and License

Copyright (c) 2026--present Nessan Fitzmaurice. <br>
You can use this software under the [MIT license][].

<!-- Reference Links -->

[repository]: https://github.com/nessan/gf2
[documentation site]: https://nessan.github.io/gf2
[email]: mailto:nzznfitz+gh@icloud.com
[MIT License]: https://opensource.org/license/mit
[GF(2)]: https://en.wikipedia.org/wiki/Finite_field
[`nalgebra`]: https://www.nalgebra.rs/docs/
[`Eigen`]: https://eigen.tuxfamily.org/overview.php?title=Main_Page
[Cayley-Hamilton]: https://en.wikipedia.org/wiki/Cayley–Hamilton_theorem
[Galois field]: https://en.wikipedia.org/wiki/Finite_field
[LU decomposition]: https://en.wikipedia.org/wiki/LU_decomposition
[C++ library]: https://github.com/nessan/gf2
[documentation site]: https://nessan.github.io/gf2
