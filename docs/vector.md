# The `BitVector` Type

## Introduction

A [`BitVector`] is a dynamically sized vector of bit elements stored compactly in a [`Vec`] of unsigned integer words.
The default word type is `usize`.

The type implements the [`BitStore`] trait, which provides a rich API for manipulating the bits in the vector.
In addition to the many methods defined by the [`BitStore`] trait, the `BitVector` type provides ways to construct bit-vectors from various sources, methods to resize bit-vectors, and methods to append or remove elements from the end of bit-vectors.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

A `BitVector` packs its elements into an [`Vec`] of some unsigned integer type defined by the generic parameter of type [`Unsigned`].
The default `Word` is a `usize` which, on modern computer systems, will often be a 64-bit unsigned integer.
Operations on and between bit-vectors and other objects in the `gf2` crate are implemented using bitwise operations on whole underlying words at a time.
These are highly optimised in modern CPUs, allowing for fast computation even on large bit-vectors.
It also means we never have to worry about overflows or carries as we would with normal integer arithmetic.

</div>

In mathematical terms, a bit-vector is a vector over [GF(2)], the simplest [Galois-Field] with just two elements, usually denoted 0 & 1, as the booleans true & false, or as the bits set & unset.
Arithmetic over GF(2) is mod 2, so addition/subtraction becomes the `XOR` operation while multiplication/division becomes `AND`.

It is worth noting that by default, a `BitVector` prints in _vector-order_.
For example, a bit-vector of size four will print as `v0v1v2v3` with the elements in increasing index-order with the "least significant" vector element, `v0`, coming **first** on the _left_.
This contrasts to the many bit-array types, which usually print in _bit-order_.
The equivalent object in those libraries with say four elements prints as `b3b2b1b0` with the least significant bit `b0` printed **last** on the _right_.

Of course, for many applications, printing in _bit-order_ makes perfect sense.
A size four bit-vector initialized with the hex number `0x1` will print as `1000`.
A bit-ordered version prints the same value as `0001`, which will be more natural in _some_ settings.

However, our main aim is numerical work, where vector order is more natural.
In particular, bit-order is unnatural for _matrices_ over GF(2).
It is too confusing to print a matrix in any order other than the one where the (0,0) element is at the top left, and proceed from there.

## Methods Overview

The `BitVector` type provides a rich set of methods for constructing and resizing bit-vectors:

| Category                                                | Description                                                               |
| ------------------------------------------------------- | ------------------------------------------------------------------------- |
| [Constructors](#constructors)                           | Methods to create bit-vectors with specific properties and fills.         |
| [Construction from Strings](#construction-from-strings) | Construction of bit-vectors from string representations (these can fail). |
| [Resizing](#resizing)                                   | Methods to query and manipulate the size and capacity of a bit-vector.    |
| [Appending Elements](#appending-elements)               | Methods to append bits from various sources to the end of a bit-vector.   |
| [Removing Elements](#removing-elements)                 | Methods to remove bits from the end of a bit-vector.                      |

The type also defines the methods needed to implement the [`BitStore`] trait:

| Category                                  | Description                                         |
| ----------------------------------------- | --------------------------------------------------- |
| [Trait Requirements](#trait-requirements) | Methods needed to implement the [`BitStore`] trait. |

The type then inherits dozens of associated methods from the [`BitStore`] trait.
These methods fall into categories:

| Inherited Category                                    | Description                                                                      |
| ----------------------------------------------------- | -------------------------------------------------------------------------------- |
| [Bit Access](BitStore#bit-access)                     | Methods to access individual bit elements in a bit-vector.                       |
| [Queries](BitStore#queries)                           | Methods to query the overall state of a bit-vector.                              |
| [Mutators](BitStore#mutators)                         | Methods to mutate the overall state of a bit-vector.                             |
| [Copies & Fills](BitStore#copies-and-fills)           | Methods to fill a bit-vector from various sources.                               |
| [Slices](BitStore#slices)                             | Methods to create non-owning views over a part of a bit-vector --- _bit-slices_. |
| [Sub-vectors](BitStore#sub-vectors)                   | Methods to clone a piece of a bit-vector as a new bit-vector.                    |
| [Riffling](BitStore#riffling)                         | Methods to create vectors that copy a bit-vector with interleaved zeros.         |
| [Set/Unset Indices](BitStore#indices)                 | Methods to find the indices of set & unset bits in a bit-vector.                 |
| [Iterators](BitStore#iterators)                       | Methods to create various iterators over a bit-vector.                           |
| [Stringification](#stringification)                   | Methods to create string representations of a bit-vector.                        |
| [Bit Shifts](BitStore#shifts)                         | Methods to shift the bits in a bit-vector left or right.                         |
| [Bitwise Operations](BitStore#bit-wise-operations)    | Methods to combine any bit-store and a bit-vector using logical operations.      |
| [Arithmetic Operators](BitStore#arithmetic-operators) | Methods to add or subtract any bit-store and a bit-vector.                       |
| [Other Functions](BitStore#other-functions)           | Dot products, convolutions, etc. for bit-stores with bit-vectors.                |

## Trait Requirements

To implement the [`BitStore`] trait, the type defines the following seven methods:

| Method                   | Description                                                                           |
| ------------------------ | ------------------------------------------------------------------------------------- |
| [`BitVector::len`]       | Returns the number of bits in the bit-vector.                                         |
| [`BitVector::store`]     | Provides read-only access to the first _word_ holding bits in the bit-vector.         |
| [`BitVector::store_mut`] | Provides read-write access to the first _word_ holding bits in the bit-vector.        |
| [`BitVector::offset`]    | Returns the offset in bits from start of `word(0)` to the bit-vector's first element. |
| [`BitVector::words`]     | This is always `Word::words_needed(self.len())` but cached for efficiency.            |
| [`BitVector::word`]      | Returns a "word" from the bit-vector.                                                 |
| [`BitVector::set_word`]  | Sets the value of a "word" in the bit-vector to a passed value.                       |

These methods are trivial to implement for bit-vectors.

The one place where care is needed is in the [`BitVector::set_word`] method, which must ensure that any bits beyond the size of the bit-vector remain set to zero.

## Constructors

The `BitVector` type provides several constructors to create bit-vectors with specific properties and fills:

| Method Name                         | Description                                                                                     |
| ----------------------------------- | ----------------------------------------------------------------------------------------------- |
| [`BitVector::new`]                  | Returns a zero-sized bit-vector.                                                                |
| [`BitVector::with_capacity`]        | Returns a zero-sized bit-vector that can add some elements without any extra allocations.       |
| [`BitVector::zeros`]                | Returns a bit-vector where all the elements are 0.                                              |
| [`BitVector::ones`]                 | Returns a bit-vector where all the elements are 1.                                              |
| [`BitVector::constant`]             | Returns a bit-vector where all the elements are whatever is passed as a `value`.                |
| [`BitVector::unit`]                 | Returns a bit-vector where all the elements are zero except for a single 1.                     |
| [`BitVector::alternating`]          | Returns a bit-vector where all the elements follow the pattern `101010...`                      |
| [`BitVector::from_unsigned`]        | Returns a bit-vector filled with bits from any [`Unsigned`] value.                              |
| [`BitVector::from_store`]           | Returns a bit-vector filled with bits from any bit-store.                                       |
| [`BitVector::from_fn`]              | Returns a bit-vector filled with bits set by calling a function for each index.                 |
| [`BitVector::random`]               | Returns a bit-vector filled by flipping a fair coin seeded from entropy.                        |
| [`BitVector::random_seeded`]        | Returns a bit-vector with a reproducible fair random fill.                                      |
| [`BitVector::random_biased` ]       | Returns a random bit-vector where you set the probability of bits being 1.                      |
| [`BitVector::random_biased_seeded`] | Returns a random bit-vector where you set the probability of bits being 1 _and_ the RNG's seed. |

### Notes

- We have implemented the [`Default`] trait for `BitVector` to return a zero-sized bit-vector.
- The [`BitVector::from_store`] constructor is one of the few methods in the library that _doesn't_ require the two stores to have the same underlying `Unsigned` word type for their storage -- i.e., the `Word` type for `self` may differ from the `SrcWord` type for the `src` bit-store.
- We have implemented the [`From`] trait for [`BitVector`] from any bit-store type by forwarding to [`BitVector::from_store`] method.

## Construction from Strings

We can construct a `BitVector` from strings --- these methods can fail, so they return an `Option<BitVector>` and `None` on failure.

| Method Name                       | Description                                                  |
| --------------------------------- | ------------------------------------------------------------ |
| [`BitVector::from_string`]        | Tries to construct a bit-vector from an arbitrary string.    |
| [`BitVector::from_binary_string`] | Tries to construct a bit-vector from a _binary_ string.      |
| [`BitVector::from_hex_string`]    | Tries to construct a bit-vector from a _hexadecimal_ string. |

Space, comma, single quote, and underscore characters are removed from the string.

If the string has an optional `"0b"` prefix, it is assumed to be binary.
If it has an optional `"0x"` prefix, it is assumed to be hex.
If there is no prefix and the string consists entirely of 0s and 1s, we assume it is binary; otherwise, we think it is hex.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">⚡</div>

This means the string `"0x11` is interpreted as the bit-vector of size 8 `"11110001"`, whereas the same string without a prefix, `"11"` is interpreted as the bit-vector of size 2 `"11"`. To avoid any ambiguity, it is best to use a prefix.

</div>

See the [string-encodings](BitStore#stringification) documentation for more details on the accepted string formats.

## Resizing

We have methods to query and manipulate the size and capacity of a bit-vector:

| Method Name                       | Description                                                                                      |
| --------------------------------- | ------------------------------------------------------------------------------------------------ |
| [`BitVector::len`]                | Returns the number of bit elements in the bit-vector.                                            |
| [`BitVector::capacity`]           | Returns the total number of bits the vector can hold without allocating more memory.             |
| [`BitVector::remaining_capacity`] | Returns the number of _additional_ elements we can store in the bit-vector without reallocating. |
| [`BitVector::shrink_to_fit`]      | Tries to shrink the vector's capacity as much as possible.                                       |
| [`BitVector::clear`]              | Sets the `len()` to zero. Leaves the capacity unaltered.                                         |
| [`BitVector::resize`]             | Resizes the bit-vector, either adding zeros, or truncating existing elements.                    |

## Appending Elements

We have methods to append elements from various sources to the end of a bit-vector:

| Method Name                     | Description                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------ |
| [`BitVector::push`]             | Pushes a single bit (0 or 1) onto the end of the bit-vector.                   |
| [`BitVector::append_unsigned`]  | Appends the bits from any unsigned integer value to the end of the bit-vector. |
| [`BitVector::append_store`]     | Appends bits from another bit-store to the end of the bit-vector.              |
| [`BitVector::append_digit`]     | Appends a "character's" worth of bits to the end of the bit-vector.            |
| [`BitVector::append_hex_digit`] | Appends four bits from a "hex-character" to the end of the bit-vector.         |

The [`BitVector::append_store`] is one of the few methods in the library that _doesn't_ require the two stores to have the same underlying `Unsigned` word type for their storage -- i.e., the `Word` type for `self` may differ from the `SrcWord` type for the `src` bit-store.

The [`BitVector::append_digit`] method appends bits from a character representing a digit in one of the bases 2, 4, 8, or 16.
It does nothing if it fails to parse the character.

## Removing Elements

We have methods to remove elements from the end of a bit-vector:

| Method Name                       | Description                                                                                      |
| --------------------------------- | ------------------------------------------------------------------------------------------------ |
| [`BitVector::pop`]                | Removes the last element off the end of the bit-vector and returns it.                           |
| [`BitVector::split_off_word`]     | Removes a single `Word` off the end of the bit-vector and returns it.                            |
| [`BitVector::split_off_unsigned`] | Removes a single arbitrary-sized unsigned integer off the end of the bit-vector and returns it.  |
| [`BitVector::split_off_into`]     | Splits a bit-vector into two at a given index and fills a passed vector with the second "half".  |
| [`BitVector::split_off`]          | Splits a bit-vector into two at a given index and returns the second "half" as a new bit-vector. |

The first three methods return the removed elements as an [`Option`], and as a [`None`] if the vector is empty.

The two [`BitVector::split_off_into`] and [`BitVector::split_off`] methods complement the [`BitStore::split_at_into`] and [`BitStore::split_at`] methods.
The two `BitVector` only versions change the size of the bit-vector _in place_.

## Bit Access (Inherited)

The following methods provide access to individual bit elements in the bit-vector.

| Method              | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| [`BitStore::get`]   | Returns the value of a single bit element as a read-only boolean. |
| [`BitStore::first`] | Returns the value of the first element in the bit-vector.         |
| [`BitStore::last`]  | Returns the value of the last element in the bit-vector.          |
| [`BitStore::set`]   | Sets a bit to the given boolean value.                            |
| [`BitStore::flip`]  | Flips the value of the bit element at a given index.              |
| [`BitStore::swap`]  | Swaps the values of bit elements at locations `i` and `j`.        |

## Queries (Inherited)

The following methods let you query the overall state of a bit-store.

| Method                       | Description                                                  |
| ---------------------------- | ------------------------------------------------------------ |
| [`BitStore::is_empty`]       | Returns true if the bit-vector is empty                      |
| [`BitStore::any`]            | Returns true if _any_ bit in the bit-vector is set.          |
| [`BitStore::all`]            | Returns true if _every_ bit in the bit-vector is set.        |
| [`BitStore::none`]           | Returns true if _no_ bit in the bit-vector is set.           |
| [`BitStore::count_ones`]     | Returns the number of set bits in the bit-vector.            |
| [`BitStore::count_zeros`]    | Returns the number of unset bits in the bit-vector.          |
| [`BitStore::leading_zeros`]  | Returns the number of leading unset bits in the bit-vector.  |
| [`BitStore::trailing_zeros`] | Returns the number of trailing unset bits in the bit-vector. |

## Mutators (Inherited)

The following methods let you mutate the entire bit-vector in a single call.

| Method                 | Description                                                                          |
| ---------------------- | ------------------------------------------------------------------------------------ |
| [`BitStore::set_all`]  | Sets all the bits in the store to the passed value.                                  |
| [`BitStore::flip_all`] | Flips the values of all the bits in the bit-vector.                                  |
| [`BitStore::flipped`]  | Returns a new bit-vector that is a copy of the bit-vector with all the bits flipped. |

## Copies and Fills (Inherited)

The following methods let you populate the entire bit-vector from multiple sources in a single call.

| Method                                  | Description                                                                      |
| --------------------------------------- | -------------------------------------------------------------------------------- |
| [`BitStore::copy_unsigned`]             | Copies bit values from any unsigned value to this bit-vector.                    |
| [`BitStore::copy_store`]                | Copies bit values from any source bit-vector to this bit-vector.                 |
| [`BitStore::copy_fn`]                   | Copies bit values from a function that returns a boolean for an index.           |
| [`BitStore::fill_random_biased_seeded`] | Very general method to fill the bit-vector with random 0's and 1's.              |
| [`BitStore::fill_random_biased`]        | Fill the store with random 0's and 1's, where the RNG it itself randomly seeded. |
| [`BitStore::fill_random`]               | Fill the store with random 0's and 1's from flips of a _fair_ coin.              |

## Slices (Inherited)

The following methods let you create a [`BitSlice`], which is a non-owning view of some contiguous subset of bits in the bit-vector.

| Method                  | Description                                                                  |
| ----------------------- | ---------------------------------------------------------------------------- |
| [`BitStore::slice`]     | Returns a [`BitSlice`] encompassing the bits in a half-open range.           |
| [`BitStore::slice_mut`] | Returns a _mutable_ [`BitSlice`] encompassing the bits in a half-open range. |

## Sub-vectors (Inherited)

The following methods create or fill _independent_ bit-vectors with copies of some contiguous subset of the bits in the bit-vector.

| Method                      | Description                                                                              |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| [`BitStore::sub`]           | Returns a new [`BitVector`] encompassing the bits in a half-open range.                  |
| [`BitStore::split_at_into`] | Fills two bit-vectors with the bits in the ranges `[0, at)` and `[at, len())`.           |
| [`BitStore::split_at`]      | Returns two new two bit-vectors with the bits in the ranges `[0, at)` and `[at, len())`. |

## Riffling (Inherited)

We have methods that can interleave (_riffle_) the bits in a bit-vector with zeros.

| Method                     | Description                                                                       |
| -------------------------- | --------------------------------------------------------------------------------- |
| [`BitStore::riffled_into`] | Fills a pre-existing bit-vector with the result of riffling this store.           |
| [`BitStore::riffled`]      | Returns a new bit-vector that is this store with its bits interleaved with zeros. |

## Set and Unset Bit Indices (Inherited)

The following methods find the indices of set or unset bits in the bit-vector.

| Method                       | Description                                                                              |
| ---------------------------- | ---------------------------------------------------------------------------------------- |
| [`BitStore::first_set`]      | Returns the index of the first set bit in the bit-vector.                                |
| [`BitStore::last_set`]       | Returns the index of the last set bit in the bit-vector.                                 |
| [`BitStore::next_set`]       | Returns the index of the next set bit in the bit-vector _after_ the passed index.        |
| [`BitStore::previous_set`]   | Returns the index of the previous set bit in the bit-vector _before_ the passed index.   |
| [`BitStore::first_unset`]    | Returns the index of the first unset bit in the bit-vector.                              |
| [`BitStore::last_unset`]     | Returns the index of the last unset bit in the bit-vector.                               |
| [`BitStore::next_unset`]     | Returns the index of the next unset bit in the bit-vector _after_ the passed index.      |
| [`BitStore::previous_unset`] | Returns the index of the previous unset bit in the bit-vector _before_ the passed index. |

## Iterators (Inherited)

The following methods create iterators for traversing the bits or underlying words in the bit-vector:

| Method                    | Description                                                                 |
| ------------------------- | --------------------------------------------------------------------------- |
| [`BitStore::bits`]        | Returns a [`Bits`] iterator over the bits in the bit-vector.                |
| [`BitStore::set_bits`]    | Returns a [`SetBits`] iterator to view the indices of all the set bits.     |
| [`BitStore::unset_bits`]  | Returns a [`UnsetBits`] iterator to view the indices of all the unset bits. |
| [`BitStore::store_words`] | Returns a [`Words`] iterator to view the "words" underlying the bit-vector. |
| [`BitStore::to_words`]    | Returns a copy of the "words" underlying the bit-vector.                    |

## Stringification (Inherited)

The following functions returns a string representation of a bit-vector.
The string can be in the obvious binary format or a more compact hex format.

| Method                                | Description                                                                                    |
| ------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [`BitStore::to_custom_binary_string`] | Returns a binary string representation for a bit-vector with various customisation parameters. |
| [`BitStore::to_binary_string`]        | Returns the simplest binary string representation for a bit-vector.                            |
| [`BitStore::to_pretty_string`]        | Returns a "pretty" binary string representation for a bit-vector.                              |
| [`BitStore::to_hex_string`]           | Returns a compact hex string representation for a bit-vector.                                  |
| [`std::string::ToString::to_string`]  | Delegates to [`BitStore::to_binary_string`].                                                   |
| [`BitStore::describe`]                | Returns a multi-line string describing the bit-vector in some detail.                          |

## Bit Shifts (Inherited)

We have methods to shift the bits in a bit-vector left or right.

| Methods                     | Description                                                             |
| --------------------------- | ----------------------------------------------------------------------- |
| [`BitStore::left_shift`]    | Left shifts in-place.                                                   |
| [`BitStore::right_shift`]   | Right shifts in-place.                                                  |
| [`BitStore::left_shifted`]  | Copies the bit-vector to a new bit-vector and left shifts that vector.  |
| [`BitStore::right_shifted`] | Copies the bit-vector to a new bit-vector and right shifts that vector. |

**Note:** We have also implemented the [`std::ops::ShlAssign`], [`std::ops::ShrAssign`], [`std::ops::Shl`], and [`std::ops::Shr`] foreign traits to provide operator overloads for the shift operations. Those implementations forward to the associated methods above.

## Bitwise Operations (Inherited)

We have methods that combine a bit-vector with any other bit-store using the logical operations `XOR`, `AND`, and `OR`.

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
| [`BitStore::plus_eq`]  | Adds the passed (equal-sized) `rhs` bit-store to this one.                   |
| [`BitStore::minus_eq`] | Subtracts the passed (equal-sized) `rhs` bit-store from this one.            |
| [`BitStore::plus`]     | Adds two equal-sized bit-stores and returns the result as a bit-vector.      |
| [`BitStore::minus`]    | Subtracts two equal-sized bit-stores and returns the result as a bit-vector. |

**Note:** We have also implemented the [`std::ops::AddAssign`], [`std::ops::SubAssign`], [`std::ops::Add`], and [`std::ops::Sub`] foreign traits to provide operator overloads for the arithmetic operations. Those implementations forward to the associated methods above.

## Other Inherited Functions

| Method                       | Description                                                         |
| ---------------------------- | ------------------------------------------------------------------- |
| [`BitStore::dot`]            | Returns the dot product of two equal-sized bit-stores as a boolean. |
| [`BitStore::convolved_with`] | Returns the convolution of two bit-stores as a new bit-vector.      |

## Foreign Traits for Individual Bit-Vectors

We have implemented several foreign traits from the standard library for bit-vectors.

| Trait Name              | Description                                  |
| ----------------------- | -------------------------------------------- |
| [`Default`]             | Forwarded to [`BitVector::new`].             |
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

The [`std::ops::Not`] trait is implemented for bit-vectors by value and by reference.

## Foreign Traits for Pairwise Bit-Vector Operations with Other Bit-Stores

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

<!-- Reference Links -->

[`Bits`]: crate::Bits
[`SetBits`]: crate::SetBits
[`UnsetBits`]: crate::UnsetBits
[`Words`]: crate::Words
[GF(2)]: https://en.wikipedia.org/wiki/Finite_field_arithmetic
[Galois-Field]: https://en.wikipedia.org/wiki/Finite_field
