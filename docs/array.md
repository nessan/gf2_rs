# The `BitArray` Type

## Introduction

A [`BitArray`] is a fixed sized vector of bit elements stored compactly in an array of unsigned integer words.
The default word type is `usize`.

The type implements the [`BitStore`] trait, which provides a rich API for manipulating the bits in the vector.
In addition to the many methods defined by the [`BitStore`] trait, the `BitArray` type provides several ways to construct bit-arrays from various sources.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

A `BitArray` packs its elements into a standard array of some unsigned integer type defined by the generic parameter of type [`Unsigned`].
The default `Word` is a `usize` which, on modern computer systems, will often be a 64-bit unsigned integer.
Operations on and between bit-arrays and other objects in the `gf2` crate are implemented using bitwise operations on whole underlying words at a time.
These are highly optimised in modern CPUs, allowing for fast computation even on large bit-arrays.
It also means we never have to worry about overflows or carries as we would with normal integer arithmetic.

</div>

In mathematical terms, a bit-array is a vector over [GF(2)], the simplest [Galois-Field] with just two elements, usually denoted 0 & 1, as the booleans true & false, or as the bits set & unset.
Arithmetic over GF(2) is mod 2, so addition/subtraction becomes the `XOR` operation while multiplication/division becomes `AND`.

It is worth noting that by default, a `BitArray` prints in _vector-order_.
For example, a bit-vector of size four will print as `v0v1v2v3` with the elements in increasing index-order with the "least significant" vector element, `v0`, coming **first** on the _left_.
This contrasts to the many bit-set types, which usually print in _bit-order_.
The equivalent object in those libraries with say four elements prints as `b3b2b1b0` with the least significant bit `b0` printed **last** on the _right_.

Of course, for many applications, printing in _bit-order_ makes perfect sense.
A size four bit-array initialized with the hex number `0x1` will print as `1000`.
A bit-ordered version prints the same value as `0001`, which will be more natural in _some_ settings.

However, our main aim is numerical work, where vector order is more natural.
In particular, bit-order is unnatural for _matrices_ over GF(2).
It is too confusing to print a matrix in any order other than the one where the (0,0) element is at the top left, and proceed from there.

## Methods Overview

The `BitArray` type implements the [`BitStore`] trait and also provides several methods for constructing bit-arrays:

| Category                                  | Description                                                      |
| ----------------------------------------- | ---------------------------------------------------------------- |
| [Trait Requirements](#trait-requirements) | Methods needed to implement the [`BitStore`] trait.              |
| [Constructors](#constructors)             | Methods to create bit-arrays with specific properties and fills. |

The type also inherits dozens of associated methods from the [`BitStore`] trait.
These methods fall into categories:

| Inherited Category                                    | Description                                                                     |
| ----------------------------------------------------- | ------------------------------------------------------------------------------- |
| [Bit Access](BitStore#bit-access)                     | Methods to access individual bit elements in a bit-array.                       |
| [Queries](BitStore#queries)                           | Methods to query the overall state of a bit-array.                              |
| [Mutators](BitStore#mutators)                         | Methods to mutate the overall state of a bit-array.                             |
| [Copies & Fills](BitStore#copies-and-fills)           | Methods to fill a bit-array from various sources.                               |
| [Slices](BitStore#slices)                             | Methods to create non-owning views over a part of a bit-array --- _bit-slices_. |
| [Sub-vectors](BitStore#sub-vectors)                   | Methods to clone a piece of a bit-array as a new bit-vector.                    |
| [Riffling](BitStore#riffling)                         | Methods to create vectors that copy a bit-array with interleaved zeros.         |
| [Set/Unset Indices](BitStore#indices)                 | Methods to find the indices of set & unset bits in a bit-array.                 |
| [Iterators](BitStore#iterators)                       | Methods to create various iterators over a bit-array.                           |
| [Stringification](#stringification)                   | Methods to create string representations of a bit-array.                        |
| [Bit Shifts](BitStore#shifts)                         | Methods to shift the bits in a bit-array left or right.                         |
| [Bitwise Operations](BitStore#bit-wise-operations)    | Methods to combine any bit-store and a bit-array using logical operations.      |
| [Arithmetic Operators](BitStore#arithmetic-operators) | Methods to add or subtract any bit-store and a bit-array.                       |
| [Other Functions](BitStore#other-functions)           | Dot products, convolutions, etc. for bit-stores with bit-arrays.                |

## Trait Requirements

To implement the [`BitStore`] trait, the type defines the following seven methods:

| Method                  | Description                                                                          |
| ----------------------- | ------------------------------------------------------------------------------------ |
| [`BitArray::len`]       | Returns the number of bits in the bit-array --- the generic parameter `N`.           |
| [`BitArray::store`]     | Provides read-only access to the first _word_ holding bits in the bit-array.         |
| [`BitArray::store_mut`] | Provides read-write access to the first _word_ holding bits in the bit-array.        |
| [`BitArray::offset`]    | Returns the offset in bits from start of `word(0)` to the bit-array's first element. |
| [`BitArray::words`]     | This is always `Word::words_needed(self.len())` but cached for efficiency.           |
| [`BitArray::word`]      | Returns a "word" from the bit-array.                                                 |
| [`BitArray::set_word`]  | Sets the value of a "word" in the bit-array to a passed value.                       |

These methods are trivial to implement for bit-arrays.

The one place where care is needed is in the [`BitArray::set_word`] method, which must ensure that any bits beyond the size of the bit-array remain set to zero.

## Constructors

The `BitArray` type provides several constructors to create bit-arrays with specific properties and fills:

| Method Name                        | Description                                                                                    |
| ---------------------------------- | ---------------------------------------------------------------------------------------------- |
| [`BitArray::new`]                  | Returns a bit-array that has `N` zero elements.                                                |
| [`BitArray::zeros`]                | Returns a bit-array where all the elements are 0.                                              |
| [`BitArray::ones`]                 | Returns a bit-array where all the elements are 1.                                              |
| [`BitArray::constant`]             | Returns a bit-array where all the elements are whatever is passed as a `value`.                |
| [`BitArray::unit`]                 | Returns a bit-array where all the elements are zero except for a single 1.                     |
| [`BitArray::alternating`]          | Returns a bit-array where all the elements follow the pattern `101010...`                      |
| [`BitArray::from_word`]            | Returns a bit-array filled with bits copied from a `Word` value.                               |
| [`BitArray::from_fn`]              | Returns a bit-array filled with bits set by calling a function for each index.                 |
| [`BitArray::random`]               | Returns a bit-array filled by flipping a fair coin seeded from entropy.                        |
| [`BitArray::random_seeded`]        | Returns a bit-array with a reproducible fair random fill.                                      |
| [`BitArray::random_biased` ]       | Returns a random bit-array where you set the probability of bits being 1.                      |
| [`BitArray::random_biased_seeded`] | Returns a random bit-array where you set the probability of bits being 1 _and_ the RNG's seed. |

**Note:** We have implemented the [`Default`] trait for `BitArray` to return a zero-sized bit-vector.

## Bit Access (Inherited)

The following methods provide access to individual bit elements in the bit-array.

| Method              | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| [`BitStore::get`]   | Returns the value of a single bit element as a read-only boolean. |
| [`BitStore::first`] | Returns the value of the first element in the bit-array.          |
| [`BitStore::last`]  | Returns the value of the last element in the bit-array.           |
| [`BitStore::set`]   | Sets a bit to the given boolean value.                            |
| [`BitStore::flip`]  | Flips the value of the bit element at a given index.              |
| [`BitStore::swap`]  | Swaps the values of bit elements at locations `i` and `j`.        |

## Queries (Inherited)

The following methods let you query the overall state of a bit-array.

| Method                       | Description                                                 |
| ---------------------------- | ----------------------------------------------------------- |
| [`BitStore::is_empty`]       | Returns true if the bit-array is empty                      |
| [`BitStore::any`]            | Returns true if _any_ bit in the bit-array is set.          |
| [`BitStore::all`]            | Returns true if _every_ bit in the bit-array is set.        |
| [`BitStore::none`]           | Returns true if _no_ bit in the bit-array is set.           |
| [`BitStore::count_ones`]     | Returns the number of set bits in the bit-array.            |
| [`BitStore::count_zeros`]    | Returns the number of unset bits in the bit-array.          |
| [`BitStore::leading_zeros`]  | Returns the number of leading unset bits in the bit-array.  |
| [`BitStore::trailing_zeros`] | Returns the number of trailing unset bits in the bit-array. |

## Mutators (Inherited)

The following methods let you mutate the entire bit-array in a single call.

| Method                 | Description                                                                         |
| ---------------------- | ----------------------------------------------------------------------------------- |
| [`BitStore::set_all`]  | Sets all the bits in the bit-array to the passed value.                             |
| [`BitStore::flip_all`] | Flips the values of all the bits in the bit-array.                                  |
| [`BitStore::flipped`]  | Returns a new bit-array that is a copy of the bit-vector with all the bits flipped. |

## Copies and Fills (Inherited)

The following methods let you populate the entire bit-array from multiple sources in a single call.

| Method                                  | Description                                                                          |
| --------------------------------------- | ------------------------------------------------------------------------------------ |
| [`BitStore::copy_unsigned`]             | Copies bit values from any unsigned value to this bit-array.                         |
| [`BitStore::copy_store`]                | Copies bit values from any source bit-store to this bit-array.                       |
| [`BitStore::copy_fn`]                   | Copies bit values from a function that returns a boolean for an index.               |
| [`BitStore::fill_random_biased_seeded`] | Very general method to fill the bit-array with random 0's and 1's.                   |
| [`BitStore::fill_random_biased`]        | Fill the bit-array with random 0's and 1's, where the RNG itself is randomly seeded. |
| [`BitStore::fill_random`]               | Fill the bit-array with random 0's and 1's from flips of a _fair_ coin.              |

## Slices (Inherited)

The following methods let you create a [`BitSlice`], which is a non-owning view of some contiguous subset of bits in the bit-array.

| Method                  | Description                                                                  |
| ----------------------- | ---------------------------------------------------------------------------- |
| [`BitStore::slice`]     | Returns a [`BitSlice`] encompassing the bits in a half-open range.           |
| [`BitStore::slice_mut`] | Returns a _mutable_ [`BitSlice`] encompassing the bits in a half-open range. |

## Sub-vectors (Inherited)

The following methods create or fill _independent_ bit-vectors with copies of some contiguous subset of the bits in the bit-array.

| Method                      | Description                                                                              |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| [`BitStore::sub`]           | Returns a new [`BitVector`] encompassing the bits in a half-open range.                  |
| [`BitStore::split_at_into`] | Fills two bit-vectors with the bits in the ranges `[0, at)` and `[at, len())`.           |
| [`BitStore::split_at`]      | Returns two new two bit-vectors with the bits in the ranges `[0, at)` and `[at, len())`. |

## Riffling (Inherited)

We have methods that can interleave (_riffle_) the bits in a bit-array with zeros.

| Method                     | Description                                                                           |
| -------------------------- | ------------------------------------------------------------------------------------- |
| [`BitStore::riffled_into`] | Fills a pre-existing bit-vector with the result of riffling this bit-array.           |
| [`BitStore::riffled`]      | Returns a new bit-vector that is this bit-array with its bits interleaved with zeros. |

## Set and Unset Bit Indices (Inherited)

The following methods find the indices of set or unset bits in the bit-array.

| Method                       | Description                                                                             |
| ---------------------------- | --------------------------------------------------------------------------------------- |
| [`BitStore::first_set`]      | Returns the index of the first set bit in the bit-array.                                |
| [`BitStore::last_set`]       | Returns the index of the last set bit in the bit-array.                                 |
| [`BitStore::next_set`]       | Returns the index of the next set bit in the bit-array _after_ the passed index.        |
| [`BitStore::previous_set`]   | Returns the index of the previous set bit in the bit-array _before_ the passed index.   |
| [`BitStore::first_unset`]    | Returns the index of the first unset bit in the bit-array.                              |
| [`BitStore::last_unset`]     | Returns the index of the last unset bit in the bit-array.                               |
| [`BitStore::next_unset`]     | Returns the index of the next unset bit in the bit-array _after_ the passed index.      |
| [`BitStore::previous_unset`] | Returns the index of the previous unset bit in the bit-array _before_ the passed index. |

## Iterators (Inherited)

The following methods create iterators for traversing the bits or underlying words in the bit-array:

| Method                    | Description                                                                 |
| ------------------------- | --------------------------------------------------------------------------- |
| [`BitStore::bits`]        | Returns a [`Bits`] iterator over the bits in the bit-array.                 |
| [`BitStore::set_bits`]    | Returns a [`SetBits`] iterator to view the indices of all the set bits.     |
| [`BitStore::unset_bits`]  | Returns a [`UnsetBits`] iterator to view the indices of all the unset bits. |
| [`BitStore::store_words`] | Returns a [`Words`] iterator to view the "words" underlying the bit-array.  |
| [`BitStore::to_words`]    | Returns a copy of the "words" underlying the bit-array.                     |

## Stringification (Inherited)

The following functions returns a string representation of a bit-array.
The string can be in the obvious binary format or a more compact hex format.

| Method                                | Description                                                                                   |
| ------------------------------------- | --------------------------------------------------------------------------------------------- |
| [`BitStore::to_custom_binary_string`] | Returns a binary string representation for a bit-array with various customisation parameters. |
| [`BitStore::to_binary_string`]        | Returns the simplest binary string representation for a bit-array.                            |
| [`BitStore::to_pretty_string`]        | Returns a "pretty" binary string representation for a bit-array.                              |
| [`BitStore::to_hex_string`]           | Returns a compact hex string representation for a bit-array.                                  |
| [`std::string::ToString::to_string`]  | Delegates to [`BitStore::to_binary_string`].                                                  |
| [`BitStore::describe`]                | Returns a multi-line string describing the bit-array in some detail.                          |

## Bit Shifts (Inherited)

We have methods to shift the bits in a bit-vector left or right.

| Methods                     | Description                                                            |
| --------------------------- | ---------------------------------------------------------------------- |
| [`BitStore::left_shift`]    | Left shifts in-place.                                                  |
| [`BitStore::right_shift`]   | Right shifts in-place.                                                 |
| [`BitStore::left_shifted`]  | Copies the bit-array to a new bit-vector and left shifts that vector.  |
| [`BitStore::right_shifted`] | Copies the bit-array to a new bit-vector and right shifts that vector. |

**Note:** We have also implemented the [`std::ops::ShlAssign`], [`std::ops::ShrAssign`], [`std::ops::Shl`], and [`std::ops::Shr`] foreign traits to provide operator overloads for the shift operations. Those implementations forward to the associated methods above.

## Bitwise Operations (Inherited)

We have methods that combine a bit-array with any other bit-store using the logical operations `XOR`, `AND`, and `OR`.

| Method               | Description                                                                         |
| -------------------- | ----------------------------------------------------------------------------------- |
| [`BitStore::xor_eq`] | In-place `XOR` operation of equal-sized bit-stores: `lhs = lhs ^ rhs`.              |
| [`BitStore::and_eq`] | In-place `AND` operation of equal-sized bit-stores: `lhs = lhs & rhs`.              |
| [`BitStore::or_eq`]  | In-place `OR` operation of equal-sized bit-stores: `lhs = lhs \| rhs`.              |
| [`BitStore::xor`]    | Returns the `XOR` of this store with another equal-sized store as a new bit-vector. |
| [`BitStore::and`]    | Returns the `AND` of this store with another equal-sized store as a new bit-vector. |
| [`BitStore::or`]     | Returns the `OR` of this store with another equal-sized store as a new bit-vector.  |

**Note:** We have also implemented the [`std::ops::BitXorAssign`], [`std::ops::BitAndAssign`], [`std::ops::BitOrAssign`], [`std::ops::BitXor`], [`std::ops::BitAnd`], and [`std::ops::BitOr`] foreign traits to provide operator overloads for the bit-wise operations. Those implementations forward to the associated methods above.

## Arithmetic Operations (Inherited)

In GF(2), the arithmetic operators `+` and `-` are both the `XOR` operator.

| Method                 | Description                                                                  |
| ---------------------- | ---------------------------------------------------------------------------- |
| [`BitStore::plus_eq`]  | Adds the passed (equal-sized) `rhs` bit-store to this bit-vector.            |
| [`BitStore::minus_eq`] | Subtracts the passed (equal-sized) `rhs` bit-store from this bit-vector.     |
| [`BitStore::plus`]     | Adds two equal-sized bit-stores and returns the result as a bit-vector.      |
| [`BitStore::minus`]    | Subtracts two equal-sized bit-stores and returns the result as a bit-vector. |

**Note:** We have also implemented the [`std::ops::AddAssign`], [`std::ops::SubAssign`], [`std::ops::Add`], and [`std::ops::Sub`] foreign traits to provide operator overloads for the arithmetic operations. Those implementations forward to the associated methods above.

## Other Inherited Functions

| Method                       | Description                                                         |
| ---------------------------- | ------------------------------------------------------------------- |
| [`BitStore::dot`]            | Returns the dot product of two equal-sized bit-stores as a boolean. |
| [`BitStore::convolved_with`] | Returns the convolution of two bit-stores as a new bit-vector.      |

## Foreign Traits for Individual Bit-Arrays

We have implemented several foreign traits from the standard library for bit-vectors.

| Trait Name              | Description                                  |
| ----------------------- | -------------------------------------------- |
| [`Default`]             | Forwarded to [`BitArray::new`].              |
| [`std::ops::Index`]     | Forwarded to [`BitStore::get`].              |
| [`std::ops::Not`]       | Forwarded to [`BitStore::flipped`].          |
| [`std::fmt::Display`]   | Forwarded to [`BitStore::to_binary_string`]. |
| [`std::fmt::Binary`]    | Forwarded to [`BitStore::to_binary_string`]. |
| [`std::fmt::UpperHex`]  | Forwarded to [`BitStore::to_hex_string`].    |
| [`std::fmt::LowerHex`]  | Forwarded to [`BitStore::to_hex_string`].    |
| [`std::ops::ShlAssign`] | Forwarded to [`BitStore::left_shift`].       |
| [`std::ops::ShrAssign`] | Forwarded to [`BitStore::right_shift`].      |
| [`std::ops::Shl`]       | Forwarded to [`BitStore::left_shifted`].     |
| [`std::ops::Shr`]       | Forwarded to [`BitStore::right_shifted`].    |

The [`std::ops::Not`] trait is implemented for bit-arrays by value and by reference.

## Foreign Traits for Pairwise Bit-Array Operations with Other Bit-Stores

We have implemented several foreign traits from the standard library for bit-vectors interacting with other bit-stores.

| Trait Name                 | Description                         |
| -------------------------- | ----------------------------------- |
| [`std::ops::BitXorAssign`] | Forwarded to [`BitStore::xor_eq`]   |
| [`std::ops::BitAndAssign`] | Forwarded to [`BitStore::and_eq`]   |
| [`std::ops::BitOrAssign`]  | Forwarded to [`BitStore::or_eq`]    |
| [`std::ops::AddAssign`]    | Forwarded to [`BitStore::plus_eq`]  |
| [`std::ops::SubAssign`]    | Forwarded to [`BitStore::minus_eq`] |
| [`std::ops::BitXor`]       | Forwarded to [`BitStore::xor`]      |
| [`std::ops::BitAnd`]       | Forwarded to [`BitStore::and`]      |
| [`std::ops::BitOr`]        | Forwarded to [`BitStore::or`]       |
| [`std::ops::Add`]          | Forwarded to [`BitStore::plus`]     |
| [`std::ops::Sub`]          | Forwarded to [`BitStore::minus`]    |
| [`std::ops::Mul`]          | Forwarded to [`BitStore::dot`]      |

<!-- Internal Reference Links -->

[`BitVector`]: crate::BitVector
[`BitSlice`]: crate::BitSlice
[`Unsigned`]: crate::Unsigned
[`Bits`]: crate::Bits
[`SetBits`]: crate::SetBits
[`UnsetBits`]: crate::UnsetBits
[`Words`]: crate::Words

<!-- External Reference Links -->

[GF(2)]: https://en.wikipedia.org/wiki/Finite_field_arithmetic
[Galois-Field]: https://en.wikipedia.org/wiki/Finite_field
