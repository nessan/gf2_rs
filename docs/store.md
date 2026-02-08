# The `BitStore` Trait

## Introduction

The library's vector-like types implement the `BitStore` trait:

| Type          | Description                                                                             |
| ------------- | --------------------------------------------------------------------------------------- |
| [`BitArray`]  | A fixed-size vector of bits (requires compilation with the `unstable` feature enabled). |
| [`BitVector`] | A dynamically-sized vector of bits.                                                     |
| [`BitSlice`]  | A non-owning view into contiguous ranges of bits.                                       |

These types own or view individual bit elements packed into some underlying "store" of [`Unsigned`] words.
The particular choice of `Word` is generic and user selectable from one of the primitive unsigned integer types.
We refer to any type that implements the `BitStore` trait as a _bit-store_.

Bit-stores have _dozens_ of methods in common.
That `BitStore` trait defines the requirements for implementing the shared functionality _once_ as associated methods of the trait.
Each concrete bit-store type inherits those methods.

The functions include bit accessors, mutators, fills, queries, iterators, stringification methods, bit-wise operators, arithmetic operators, and more.
Operations on and between bit-stores work on a whole-word basis, so are inherently parallel.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

Users typically will not use this trait directly -- it's an implementation detail to avoid code duplication.
Instead, they will create and use [`BitArray`] and [`BitVector`] objects, and _slices_ from those bit-vectors.
Those concrete types _inherit_ the dozens of methods provided by this trait.

</div>

## Trait Requirements

To implement the [`BitStore`] trait, a type must define the following seven methods:

| Method                  | Expected Return Value/Method Functionality                                       |
| ----------------------- | -------------------------------------------------------------------------------- |
| [`BitStore::len`]       | Returns the number of bits in the store.                                         |
| [`BitStore::store`]     | Read-only access to the first _word_ holding bits in the store.                  |
| [`BitStore::store_mut`] | Read-write access to the first _word_ holding bits in the store.                 |
| [`BitStore::offset`]    | Returns the offset in bits from start of `word(0)` to the store's first element. |
| [`BitStore::words`]     | This is always `Word::words_needed(self.len())` but cached for efficiency.       |
| [`BitStore::word`]      | Returns a "word" from the store; possibly synthesised from two real words.       |
| [`BitStore::set_word`]  | Sets the value of a "word" in the store, possibly altering two real words.       |

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

We could implement the last three methods above using the first four.
For example, the `words` method is a trivial computation based on `len` and the number of bits per underlying word.
However, all the concrete bit-store types already cache the required value, so we use that instead.
Every hot loop in the library calls `words`, and benchmarking shows that precomputing the value _significantly_ improves performance.
Having optimised versions of the `word` and `set_word` methods has an even larger impact on performance.

</div>

### Other Notes

- The underlying store must contain enough words of storage to accommodate `len` bits.
- The `words` method always returns the same number as `Word::words_needed(len()),` but this value is cached and used in constant use.
- The store's final word can have extra unused bits, but the `word` method should always set those unused bits to zero.
- The `set_word` method sets a "word" to a passed value, affecting only the _accessible_ bits in the store.

### Example

The methods are trivial to implement for [`BitArray`] and [`BitVector`].

Here is a _sketch_ of how they might work for the `BitVector` type, which stores `m_len` bits in a `Vec<Word>` called `m_store`:

```c++
impl<Word: Unsigned> BitStore<Word> for BitVector<Word> {
    fn len(&self) -> usize                 { self.m_len }
    fn store(&self) -> &[Word]             { self.m_store.as_slice() }
    fn store_mut(&mut self) -> &mut [Word] { self.m_store.as_mut_slice() }
    fn offset(&self) -> u32                { 0 }
    fn words(&self) -> usize               { self.m_store.len() }
    fn word(&self, i: usize) -> Word       { self.m_store[i] }                  // <1>
    fn set_word(&mut self, i: usize, word: Word) { self.m_store[i] = word; }    // <2>
};
```

1. The required `BitStore` methods are all trivially implemented, though the real implementation allows for range checks for debug builds.
2. In this simple sketch, the `set_word` method directly sets the underlying word. The real implementation is careful to avoid touching unoccupied bits in the final word.

A sketch of the `BitArray` type is similar, except that the underlying store is a standard fixed array of words rather than a `Vec`.

### Bit-slices

The [`BitSlice`] type is a bit different because it is a non-owning view into some contiguous subset of bits held by another bit-store, and that subset may not align with the underlying words.

However, all the `BitStore`-associated methods operate as if bit element 0 is the **lowest-order** bit of "word" **0**.
This constraint means that the implementation of the `word(i)` and `set_word(i,v)` methods for bit-slice is more complex than for the other two bit-stores because they often have to synthesise words from two underlying words in the real store.

### Sample Layout

Consider a bit-store `store` with 20 elements, where the `Word` type used to store those bits is an unsigned 8-bit integer.

The `BitStore` methods all naturally expect that `store.len()` returns 20.
Less obviously, they all expect `store.words()` to return `3`, as it takes three 8-bit words to hold 20 bits with four bits to spare.

The methods expect that `store.word(0)` holds the first 8 bits in the bit-store, `store.word(1)` has the following 8 bits, and `store.word(2)` holds the final four elements in its four lowest-order bits.
It also expects that the four highest "unoccupied" bits in `store.word(2)` are set to 0.

If the store is a bit-array or a bit-vector, the implementation of these `BitStore` expectations is easy.
Those types just have to be careful to ensure that any unoccupied high-order bits in the final word remain zeros.

It is a different matter for a bit-slice, which isn't usually zero-aligned with the real underlying array of unsigned words, `w[0]`, `w[1]`, ...
The various bit-store functions still expect that `store.words()` returns three even though the span may touch bits in _four_ underlying words!

For a bit-slice, the return value for `store.word(i)` will often be synthesised from two contiguous "real" words
`w[j]` and `w[j+1]` for some `j`.
`store.word[i]` will use some high-order bits from `w[j]` and low-order bits from `w[j+1]`.

The following diagram shows how bits in a bit-slice lie within the underlying words, which are `u8`s in this example:

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; background-color: #f9f9f9; display: flex; align-items: center; justify-content: center;">

![Embedded Figure][bit-slice-example]

</div>

The `BitSlice` must always behave _as if_ bits from the real underlying store were copied and shuffled down so that element zero is bit 0 of word 0 in the bit-slice. However, it never actually copies anything; instead, it synthesises "words" as needed.

The same principle applies to the `store.set_word(i, value)` method.
The implementation of `set_word` for bit-vectors and bit-arrays is trivial, with the one caveat that we have to be careful not to inadvertently touch any unoccupied bits in the final underlying word, or at least be sure to leave them as zeros.

In the case of a bit-slice, calls to `set_word(i, value)` will generally copy low-order bits from `value` into the high-order bits of some real underlying word `w[j]` and copy the rest of the high-order bits from `value` into the low-order bits of `w[j+1]`. The other bits in `w[j]` and `w[j+1]` will not be touched.

## Overview of Provided Methods

With the trait requirements in place, we can implement dozens of associated methods that all concrete bit-store types inherit.

The provided methods fall into categories:

| Category                                        | Description                                                                     |
| ----------------------------------------------- | ------------------------------------------------------------------------------- |
| [Bit Access](#bit-access)                       | Methods to access individual bit elements in a bit-store.                       |
| [Queries](#queries)                             | Methods to query the overall state of a bit-store.                              |
| [Mutators](#mutators)                           | Methods to mutate the overall state of a bit-store.                             |
| [Copies & Fills](#copies-and-fills)             | Methods to fill a bit-store from various sources.                               |
| [Slices](#slices)                               | Methods to create non-owning views over a part of a bit-store --- _bit-slices_. |
| [Sub-vectors](#sub-vectors)                     | Methods to clone a piece of a bit-store as a new bit-vector.                    |
| [Riffling](#riffling)                           | Methods to create vectors that copy a bit-store with interleaved zeros.         |
| [Set/Unset Indices](#indices)                   | Methods to find the indices of set & unset bits in a bit-store.                 |
| [Iterators](#iterators)                         | Methods to create various iterators over a bit-store.                           |
| [Stringification](#stringification)             | Methods to create string representations of a bit-store.                        |
| [Bit Shifts](#shifts)                           | Methods to shift the bits in a bit-store left or right.                         |
| [Bitwise Operations](#bit-wise-operations)      | Methods to combine two bit-stores using logical operations.                     |
| [Arithmetic Operations](#arithmetic-operations) | Methods to add or subtract two bit-stores.                                      |
| [Other Functions](#other-functions)             | Dot products, convolutions, etc. for bit-stores.                                |

## Bit Access

The following methods provide access to individual bit elements in the bit-store.

| Method              | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| [`BitStore::get`]   | Returns the value of a single bit element as a read-only boolean. |
| [`BitStore::first`] | Returns the value of the first element in the store.              |
| [`BitStore::last`]  | Returns the value of the last element in the store.               |
| [`BitStore::set`]   | Sets a bit to the given boolean value.                            |
| [`BitStore::flip`]  | Flips the value of the bit element at a given index.              |
| [`BitStore::swap`]  | Swaps the values of bit elements at locations `i` and `j`.        |

We have implemented the [`std::ops::Index`] foreign trait to provide array-like indexing for bit-stores.
That implementation forwards to the `get` method above.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

The C++ version of this library also overloads the index operator to provide array-like indexing for bit-stores.
In the C++ case, the operator returns a proxy object that can be used as an l-value or an r-value which allows for natural syntax like `v[i] = true;`.
Rust has no equivalent mechanism, so we must rely on the `set` method.

</div>

## Queries

The following methods let you query the overall state of a bit-store.

| Method                       | Description                                             |
| ---------------------------- | ------------------------------------------------------- |
| [`BitStore::is_empty`]       | Returns true if the store is empty                      |
| [`BitStore::any`]            | Returns true if _any_ bit in the store is set.          |
| [`BitStore::all`]            | Returns true if _every_ bit in the store is set.        |
| [`BitStore::none`]           | Returns true if _no_ bit in the store is set.           |
| [`BitStore::count_ones`]     | Returns the number of set bits in the store.            |
| [`BitStore::count_zeros`]    | Returns the number of unset bits in the store.          |
| [`BitStore::leading_zeros`]  | Returns the number of leading unset bits in the store.  |
| [`BitStore::trailing_zeros`] | Returns the number of trailing unset bits in the store. |

These methods efficiently operate on words at a time, so they are inherently parallel.

## Mutators

The following methods let you mutate the entire store in a single call.

| Method                 | Description                                                                     |
| ---------------------- | ------------------------------------------------------------------------------- |
| [`BitStore::set_all`]  | Sets all the bits in the store to the passed value.                             |
| [`BitStore::flip_all`] | Flips the values of all the bits in the store.                                  |
| [`BitStore::flipped`]  | Returns a new bit-vector that is a copy of the store with all the bits flipped. |

They efficiently operate on words at a time, so they are inherently parallel.

We have implemented the [`std::ops::Not`] foreign trait to provide the unary `!` operator for bit-stores. That implementation forwards to the `flipped` method above.

## Copies and Fills

The following methods let you populate the entire store from multiple sources in a single call.

| Method                                  | Description                                                                      |
| --------------------------------------- | -------------------------------------------------------------------------------- |
| [`BitStore::copy_unsigned`]             | Copies bit values from any unsigned value to this store.                         |
| [`BitStore::copy_store`]                | Copies bit values from any source store to this store.                           |
| [`BitStore::copy_fn`]                   | Copies bit values from a function that returns a boolean for an index.           |
| [`BitStore::fill_random_biased_seeded`] | Very general method to fill the store with random 0's and 1's.                   |
| [`BitStore::fill_random_biased`]        | Fill the store with random 0's and 1's, where the RNG it itself randomly seeded. |
| [`BitStore::fill_random`]               | Fill the store with random 0's and 1's from flips of a _fair_ coin.              |

### Copies

- In each case, the _size_ of the source and destinations must match exactly! You can always use a [`BitSlice`] to change/copy a subset of bits if needed.
- However, the underlying _word types_ need **not** match, so you can copy between bit-stores that use different underlying word types. You can use the [`BitStore::copy_store`] method to convert between different `Word` type stores (e.g., from `BitVector<u32>` to `BitVector<u8>`) as long as the size of the source and destinations match.

### Random Fills

The general random fill method uses a random number generator seeded with system entropy, so the results change from run to run.
You can set a specific seed to get reproducible fills (a seed of `0` is reserved to means "use system entropy").

The simplest [`BitStore::fill_random`] method fills the store with random bits from a fair coin so the probability of a 0 or 1 is equal (i.e., 50/50).

## Slices

The following methods let you create a [`BitSlice`], which is a non-owning view of some contiguous subset of bits in the store.

| Method                  | Description                                                                  |
| ----------------------- | ---------------------------------------------------------------------------- |
| [`BitStore::slice`]     | Returns a [`BitSlice`] encompassing the bits in a half-open range.           |
| [`BitStore::slice_mut`] | Returns a _mutable_ [`BitSlice`] encompassing the bits in a half-open range. |

A [`BitSlice`] also implements the [`BitStore`] trait, so you can take a slice of a slice.

## Sub-vectors

The following methods create or fill _independent_ bit-vectors with copies of some contiguous subset of the bits in the store.

| Method                      | Description                                                                              |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| [`BitStore::sub`]           | Returns a new [`BitVector`] encompassing the bits in a half-open range.                  |
| [`BitStore::split_at_into`] | Fills two bit-vectors with the bits in the ranges `[0, at)` and `[at, len())`.           |
| [`BitStore::split_at`]      | Returns two new two bit-vectors with the bits in the ranges `[0, at)` and `[at, len())`. |

The `split_at_into` method takes two pre-existing bit-vectors to fill, thereby avoiding unnecessary allocations in some iterative algorithms that repeatedly use this method.

**Note:** These methods do not alter the underlying store.

## Riffling

We have methods that can interleave (_riffle_) the bits in a store with zeros.

| Method                     | Description                                                                       |
| -------------------------- | --------------------------------------------------------------------------------- |
| [`BitStore::riffled_into`] | Fills a pre-existing bit-vector with the result of riffling this store.           |
| [`BitStore::riffled`]      | Returns a new bit-vector that is this store with its bits interleaved with zeros. |

If the store looks like `v0 v1 ... vn`, then the riffling operation produces the vector `v0 0 v1 0 ... 0 vn` where a zero is interleaved _between_ every bit in the original store (there is no trailing zero at the end).

If you think of a bit-store as representing the coefficients of a polynomial over GF(2), then riffling corresponds to squaring that polynomial.

## Set and Unset Bit Indices

The following methods find the indices of set or unset bits in the store.

| Method                       | Description                                                                         |
| ---------------------------- | ----------------------------------------------------------------------------------- |
| [`BitStore::first_set`]      | Returns the index of the first set bit in the store.                                |
| [`BitStore::last_set`]       | Returns the index of the last set bit in the store.                                 |
| [`BitStore::next_set`]       | Returns the index of the next set bit in the store _after_ the passed index.        |
| [`BitStore::previous_set`]   | Returns the index of the previous set bit in the store _before_ the passed index.   |
| [`BitStore::first_unset`]    | Returns the index of the first unset bit in the store.                              |
| [`BitStore::last_unset`]     | Returns the index of the last unset bit in the store.                               |
| [`BitStore::next_unset`]     | Returns the index of the next unset bit in the store _after_ the passed index.      |
| [`BitStore::previous_unset`] | Returns the index of the previous unset bit in the store _before_ the passed index. |

## Iterators

The following methods create iterators for traversing the bits or underlying words in the store:

- Read-only iteration through the individual bits.
- Read-write iteration through the individual bits.
- Read-only iteration through the indices of the set bits.
- Read-only iteration through the indices of the unset bits.
- Read-write iteration through the underlying store words.

| Method                      | Description                                                                 |
| --------------------------- | --------------------------------------------------------------------------- |
| [`BitStore::bits`]          | Returns a [`Bits`] iterator over the bits in the store.                     |
| [`BitStore::set_bits`]      | Returns a [`SetBits`] iterator to view the indices of all the set bits.     |
| [`BitStore::unset_bits`]    | Returns a [`UnsetBits`] iterator to view the indices of all the unset bits. |
| [`BitStore::store_words`]   | Returns a [`Words`] iterator to view the "words" underlying the store.      |
| [`BitStore::to_words`]      | Returns a copy of the "words" underlying the bit-store.                     |
| [`BitStore::to_words_into`] | Fills a destination vector with the "words" underlying the bit-store.       |

## Stringification

The following functions returns a string representation of a bit store.
The string can be in the obvious binary format or a more compact hex format.

| Method                                | Description                                                                                   |
| ------------------------------------- | --------------------------------------------------------------------------------------------- |
| [`BitStore::to_custom_binary_string`] | Returns a binary string representation for a bit-store with various customisation parameters. |
| [`BitStore::to_binary_string`]        | Returns the simplest binary binary string representation for a bit-store.                     |
| [`BitStore::to_pretty_string`]        | Returns a "pretty" binary string representation for a bit-store.                              |
| [`BitStore::to_hex_string`]           | Returns a compact hex string representation for a bit-store.                                  |
| [`std::string::ToString::to_string`]  | Delegates to [`BitStore::to_binary_string`].                                                  |
| [`BitStore::describe`]                | Returns a multi-line string describing the bit-store in some detail.                          |

A bit-store has two different string representations: as a binary string or as a compact hex string.
The two encodings are:

### Binary String Encoding

The straightforward character encoding for a bit-store is a _binary_ string containing just 0's and 1's, for example, `"10101"`.
Each character in a binary string represents a single element in the store.
The `to_binary_string` method produces this string.
The method allows for an optional prefix, suffix, and separator between each bit.

The `to_string` calls `to_binary_string` to produce the most compact output, e.g. `"10101"`.
The `to_pretty_string` method produces a more human-friendly version, e.g., `"[1 0 1 0 1]"`.

The format used by the output stream operator is the same as that used by `to_string`.
That is also the default format used by the `std::formatter` specialisation.
However, you can use a `:p` format specifier to get the "pretty" version instead.
For example, if `std::format("{}, v)` is `"1010101010"`, then `std::format("{:p}", v)` is `"[1 0 1 0 1 0 1 0 1 0]"`.

**Note:** The output is in _vector order_ `v[0] v[1] v[2] ...` with the first element in the vector on the left.

### Hex String Encoding

The other supported encoding for bit-stores is a compact hex-type string containing just the 16 hex characters `0123456789ABCDEF`.
For example, the string `"3ED02"`.
We allow for hex strings with an optional prefix `"0x"` or `"0X"`, for example `"0x3ED02"`.

Each hex character translates to _four_ elements in a `BitStore`.
The hex string `0x0` is equivalent to the binary string `0000`, and so on, up to the string `0xF`, which is equivalent to the binary string `1111`.

The hex pair `0x0F` will be interpreted in the store as the eight-bit value `00001111`.
Of course, this is the advantage of hex.
It is a more compact format that occupies a quarter of the space needed to write out the equivalent binary string.

However, what happens if you want to encode a vector whose size is _not_ a multiple of 4?
We handle that by allowing the final character in the string to have a base that is _not_ 16.
To accomplish that, we allow for an optional _suffix_, which must be one of `.2`, `.4`, or `.8`.
If present, the suffix gives the base for just the _preceding_ character in the otherwise hex-based string.
If there is no suffix, the final character is assumed to be hex-encoded, as with all the others.

Therefore, the string `0x1` (without a suffix, so the last character is the default hexadecimal base 16) is equivalent to `0001`.
On the other hand, the string `0x1.8` (the last character is base 8) is equivalent to `001`.
Similarly, the string `0x1.4` (the last character is base 4) is equivalent to `01,` and finally, the string `0x1.2` (the previous character is base 2) is comparable to `1`

In the string `0x3ED01.8`, the first four characters, `3`, `E`, `D`, and `0`, are interpreted as hex values, and each will translate to four slots in the store.
However, the final 1.8 is parsed as an octal 1, which takes up three slots (001).
Therefore, this store has a size of 19 (i.e., 4 × 4 + 3).

The `std::formatter` specialisation recognises the `:x` format specifier as a request to produce a hex string representation of a bit-store.
For example, if `std::format("{}, v)` is `"1010101010"`, then `std::format("{:x}", v)` is `"AA2.4"`.

## Bit Shifts

We have methods to shift the bits in a store left or right.

These methods act in vector space, so if the vector is `v_0, v_1, ..., v_n-1, v_n` then a right shift produces the vector `0, v_0, v_1, ..., v_n-1` where we have shifted out the last element and shifted in a zero at the start.
Similarly, a left shift produces the vector `v_1, ..., v_n-1, v_n, 0` where we have shifted out the first element and shifted in a zero at the end.

Contrast this to shifts in bit space, where if a bit container is `b_n, b_n-1, ..., b_1, b_0`, then a right shift produces `0, b_n, b_n-1, ..., b_1` and a left shift produces `b_n-1, ..., b_1, b_0, 0`.

Essentially, right shifts in vector space correspond to left shifts in bit space, and vice versa.

| Methods                     | Description                                                        |
| --------------------------- | ------------------------------------------------------------------ |
| [`BitStore::left_shift`]    | Left shifts in-place.                                              |
| [`BitStore::right_shift`]   | Right shifts in-place.                                             |
| [`BitStore::left_shifted`]  | Copies the store to a new bit-vector and left shifts that vector.  |
| [`BitStore::right_shifted`] | Copies the store to a new bit-vector and right shifts that vector. |

**Note:** We have also implemented the [`std::ops::ShlAssign`], [`std::ops::ShrAssign`], [`std::ops::Shl`], and [`std::ops::Shr`] foreign traits to provide operator overloads for the shift operations. Those implementations forward to the associated methods above.

## Bitwise Operations

We have methods that combine two bit-stores using the logical operations `XOR`, `AND`, and `OR`.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">❗</div>

These methods require that the two bit-stores use the same underlying word type.
They also require that the left-hand-side and right-hand-side bit-store operands are the same size.
That precondition is always checked.
Interactions between bit-stores with different word types are only possible at the cost of increased code complexity, and are not a common use case.

</div>

The methods can act in place, mutating the left-hand side caller: `lhs.xor_eq(rhs)`.
There is also non-mutating versions like `result = lhs.xor(rhs)`, which returns a new `result` _bit-vector_ in each case.

| Method               | Description                                                                         |
| -------------------- | ----------------------------------------------------------------------------------- |
| [`BitStore::xor_eq`] | In-place `XOR` operation of equal-sized bit-stores: `lhs = lhs ^ rhs`.              |
| [`BitStore::and_eq`] | In-place `AND` operation of equal-sized bit-stores: `lhs = lhs & rhs`.              |
| [`BitStore::or_eq`]  | In-place `OR` operation of equal-sized bit-stores: `lhs = lhs \| rhs`.              |
| [`BitStore::xor`]    | Returns the `XOR` of this store with another equal-sized store as a new bit-vector. |
| [`BitStore::and`]    | Returns the `AND` of this store with another equal-sized store as a new bit-vector. |
| [`BitStore::or`]     | Returns the `OR` of this store with another equal-sized store as a new bit-vector.  |

**Note:** We have also implemented the [`std::ops::BitXorAssign`], [`std::ops::BitAndAssign`], [`std::ops::BitOrAssign`], [`std::ops::BitXor`], [`std::ops::BitAnd`], and [`std::ops::BitOr`] foreign traits to provide operator overloads for the bit-wise operations. Those implementations forward to the associated methods above.

## Arithmetic Operations

In GF(2), the arithmetic operators `+` and `-` are both the `XOR` operator.

| Method                 | Description                                                                  |
| ---------------------- | ---------------------------------------------------------------------------- |
| [`BitStore::plus_eq`]  | Adds the passed (equal-sized) `rhs` bit-store to this one.                   |
| [`BitStore::minus_eq`] | Subtracts the passed (equal-sized) `rhs` bit-store from this one.            |
| [`BitStore::plus`]     | Adds two equal-sized bit-stores and returns the result as a bit-vector.      |
| [`BitStore::minus`]    | Subtracts two equal-sized bit-stores and returns the result as a bit-vector. |

**Note:** We have also implemented the [`std::ops::AddAssign`], [`std::ops::SubAssign`], [`std::ops::Add`], and [`std::ops::Sub`] foreign traits to provide operator overloads for the arithmetic operations. Those implementations forward to the associated methods above.

## Other Functions

| Method                       | Description                                                         |
| ---------------------------- | ------------------------------------------------------------------- |
| [`BitStore::dot`]            | Returns the dot product of two equal-sized bit-stores as a boolean. |
| [`BitStore::convolved_with`] | Returns the convolution of two bit-stores as a new bit-vector.      |

### Note

We have implement the [`std::ops::Mul`] foreign trait to provide an operator overload for the dot product operation.

## Foreign Traits

There are many foreign traits in the Rust standard library that we would like to implement for all bit-store types. However, Rust's orphan rules prevent us from doing this in a blanket way. If you try, you will get compiler errors and somewhat opaque complaints about "coherence and overlap" issues.

You cannot implement foreign traits for any `BitStore` type in a blanket manner. The compiler worries that someone might later define a new type that implements `BitStore` and also implements its own version of the foreign trait, leading to ambiguity. There is no way to close a trait in Rust to prevent this from happening.

Instead, you have to implement the foreign traits for each concrete type separately, leading to a lot of code duplication, particularly as many of the foreign traits of interest work on _pairs_ of bit-stores (for example, the [`std::ops::Add`] trait). We have three bit-store types, so there are nine possible pairs to consider, and that is before you consider whether each argument was passed by value or by reference.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

This is not an issue for the C++ version of this library because C++ allows for blanket implementations of operator overloads.

</div>

To this crate, to avoid code duplication, we defined macros that implemented the foreign traits for all bit-store types uniformly.

## Foreign Traits for Individual Bit-Stores

The simplest case is where a foreign trait acts on a single bit-store type:

| Trait Name              | Description                                  |
| ----------------------- | -------------------------------------------- |
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

The [`std::ops::Not`] trait is implemented for each concrete bit-store both by value and by reference.

Our `impl_unary_traits!` macro implements these foreign traits for any _concrete_ bit-store type.
For example, we can invoke `impl_unary_traits!(BitVector)` to implement all these traits for the `BitVector` type.

## Foreign Traits for Pairs of Bit-Stores

Other foreign traits act on _pairs_ of bit-store types:

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

Our `impl_binary_traits!` macro implements these foreign traits for any _concrete pair_ of bit-store types.
For example, we can invoke `impl_binary_traits!(BitVector, BitSlice)` to implement all these traits for the `BitVector` type interacting with a `BitSlice` type.

Moreover, the macro implements the traits for all combinations of references and values for the two types, so `impl_binary_traits!(BitVector, BitSlice)` implements the traits for the following pairs:

- `BitVector` and `BitSlice`
- `&BitVector` and `BitSlice`
- `BitVector` and `&BitSlice`
- `&BitVector` and `&BitSlice`

This includes all combinations of the two types being passed by either by value or by reference.
For example, if `u` and `v` are two `BitVector` instances, then the following expressions will all work:

```rust
use gf2::*;
let u: BitVector = BitVector::random(10);
let v: BitVector = BitVector::random(10);
let a = &u + &v;    // `a` is a new `BitVector`; `u` and `v` are both preserved.
let b = &u + v;     // `b` is a new `BitVector`; we cannot use `v` again.
let c = u + &b;     // `c` is a new `BitVector`; we cannot use `u` again.
let d = b + c;      // `d` is a new `BitVector`; we cannot use either `b` or `c` again.
```

This is very different from C++, where operator overloads are typically defined to preserve both arguments.

```cpp
auto u = gf2::BitVector::random(10);
auto v = gf2::BitVector::random(10);
auto a = u + v;     // `a` is a new `BitVector`; `u` and `v` are both preserved.
```

In C++, you don't have to write `a = &u + &v` to preserve both operands, instead, you just write `a = u + v` with no ampersands.
The syntax is cleaner for the most common use case.

### The Macros

The macros are lengthy but straightforward, with a few arms that funnel to a single match arm that actually does something.

The one twist is that while all of our bit-store types have a generic `Word: Unsigned` parameter, some types have an extra generic parameter (a lifetime for `BitSlice`, and a `const N: usize` for `BitArray`).

- `BitVector<Word>` has a single generic parameter.
- `BitSlice<'a, Word>` has two generic parameters, the first of which is a lifetime.
- `BitArray<const N, Word>` has two generic parameters, the first of which is `const usize`.

Handling the existence/non-existence of these extra generic parameters is the main complexity in the macros.

<!-- Internal Reference Links -->

[`BitArray`]: crate::BitArray
[`BitVector`]: crate::BitVector
[`BitSlice`]: crate::BitSlice
[`Unsigned`]: crate::Unsigned

<!-- Base64 Encoded Image: rustdoc is extremely primitive and cannot embed images in any reasonable manner -->

[bit-slice-example]: data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiIHN0YW5kYWxvbmU9Im5vIj8+CjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+CjxzdmcgeG1sbnM6ZGM9Imh0dHA6Ly9wdXJsLm9yZy9kYy9lbGVtZW50cy8xLjEvIiB4bWxuczp4bD0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB2ZXJzaW9uPSIxLjEiIHZpZXdCb3g9IjQ1LjUgMzg4LjU1MiA3NjMuNSAxMzEuNTQwMiIgd2lkdGg9Ijc2My41IiBoZWlnaHQ9IjEzMS41NDAyIj4KICA8ZGVmcy8+CiAgPGcgaWQ9IkNhbnZhc18xIiBmaWxsLW9wYWNpdHk9IjEiIHN0cm9rZT0ibm9uZSIgc3Ryb2tlLW9wYWNpdHk9IjEiIHN0cm9rZS1kYXNoYXJyYXk9Im5vbmUiIGZpbGw9Im5vbmUiPgogICAgPHRpdGxlPkNhbnZhcyAxPC90aXRsZT4KICAgIDxnIGlkPSJDYW52YXNfMV9MYXllcl8xIj4KICAgICAgPHRpdGxlPkxheWVyIDE8L3RpdGxlPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMDgiPgogICAgICAgIDxyZWN0IHg9IjYxNiIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0id2hpdGUiLz4KICAgICAgICA8cmVjdCB4PSI2MTYiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2MjEgNDY5LjkyODI2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4zNCIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPlg8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMDkiPgogICAgICAgIDxyZWN0IHg9IjYzNSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0id2hpdGUiLz4KICAgICAgICA8cmVjdCB4PSI2MzUiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2NDAgNDY5LjkyODI2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4zNCIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPlg8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMTAiPgogICAgICAgIDxyZWN0IHg9IjY1NCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0id2hpdGUiLz4KICAgICAgICA8cmVjdCB4PSI2NTQiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2NTkgNDY5LjkyODI2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4zNCIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPlg8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMTEiPgogICAgICAgIDxyZWN0IHg9IjY3MyIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0id2hpdGUiLz4KICAgICAgICA8cmVjdCB4PSI2NzMiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2NzggNDY5LjkyODI2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4zNCIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPlg8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18zNiI+CiAgICAgICAgPHJlY3QgeD0iMTk4IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9IjE5OCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgyMDMgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MDwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzM3Ij4KICAgICAgICA8cmVjdCB4PSIyMTciIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iMjE3IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDIyMiA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4xPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfMzgiPgogICAgICAgIDxyZWN0IHg9IjIzNiIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSIyMzYiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMjQxIDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjI8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18zOSI+CiAgICAgICAgPHJlY3QgeD0iMjU1IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjI1NSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgyNjAgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MzwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzQzIj4KICAgICAgICA8cmVjdCB4PSIyNzQiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjODBmZjgwIi8+CiAgICAgICAgPHJlY3QgeD0iMjc0IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDI3OSA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj40PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNDIiPgogICAgICAgIDxyZWN0IHg9IjI5MyIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSIyOTMiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMjk4IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjU8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY180MSI+CiAgICAgICAgPHJlY3QgeD0iMzEyIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjMxMiIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgzMTcgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NjwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzQwIj4KICAgICAgICA8cmVjdCB4PSIzMzEiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjODBmZjgwIi8+CiAgICAgICAgPHJlY3QgeD0iMzMxIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDMzNiA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj43PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNTEiPgogICAgICAgIDxyZWN0IHg9IjM1MCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSIzNTAiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMzU1IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjA8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY181MCI+CiAgICAgICAgPHJlY3QgeD0iMzY5IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjM2OSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgzNzQgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzQ5Ij4KICAgICAgICA8cmVjdCB4PSIzODgiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjODBmZjgwIi8+CiAgICAgICAgPHJlY3QgeD0iMzg4IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDM5MyA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4yPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNDgiPgogICAgICAgIDxyZWN0IHg9IjQwNyIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSI0MDciIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNDEyIDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjM8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY180NyI+CiAgICAgICAgPHJlY3QgeD0iNDI2IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjQyNiIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg0MzEgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NDwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzQ2Ij4KICAgICAgICA8cmVjdCB4PSI0NDUiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjODBmZjgwIi8+CiAgICAgICAgPHJlY3QgeD0iNDQ1IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDQ1MCA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj41PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNDUiPgogICAgICAgIDxyZWN0IHg9IjQ2NCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSI0NjQiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNDY5IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjY8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY180NCI+CiAgICAgICAgPHJlY3QgeD0iNDgzIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjQ4MyIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg0ODggNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NzwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzU5Ij4KICAgICAgICA8cmVjdCB4PSI1MDIiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjODBmZjgwIi8+CiAgICAgICAgPHJlY3QgeD0iNTAyIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDUwNyA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4wPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNTgiPgogICAgICAgIDxyZWN0IHg9IjUyMSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSI1MjEiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNTI2IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjE8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY181NyI+CiAgICAgICAgPHJlY3QgeD0iNTQwIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjU0MCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg1NDUgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MjwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzU2Ij4KICAgICAgICA8cmVjdCB4PSI1NTkiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjODBmZjgwIi8+CiAgICAgICAgPHJlY3QgeD0iNTU5IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDU2NCA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4zPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNTUiPgogICAgICAgIDxyZWN0IHg9IjU3OCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiM4MGZmODAiLz4KICAgICAgICA8cmVjdCB4PSI1NzgiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNTgzIDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjQ8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY181NCI+CiAgICAgICAgPHJlY3QgeD0iNTk3IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iIzgwZmY4MCIvPgogICAgICAgIDxyZWN0IHg9IjU5NyIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2MDIgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzUzIj4KICAgICAgICA8cmVjdCB4PSI2MTYiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iNjE2IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDYyMSA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj42PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNTIiPgogICAgICAgIDxyZWN0IHg9IjYzNSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGZmYzAiLz4KICAgICAgICA8cmVjdCB4PSI2MzUiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNjQwIDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjc8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY182NyI+CiAgICAgICAgPHJlY3QgeD0iNjU0IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9IjY1NCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2NTkgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MDwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzY2Ij4KICAgICAgICA8cmVjdCB4PSI2NzMiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iNjczIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDY3OCA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4xPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNjUiPgogICAgICAgIDxyZWN0IHg9IjY5MiIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGZmYzAiLz4KICAgICAgICA8cmVjdCB4PSI2OTIiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNjk3IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjI8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY182NCI+CiAgICAgICAgPHJlY3QgeD0iNzExIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9IjcxMSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg3MTYgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MzwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzYzIj4KICAgICAgICA8cmVjdCB4PSI3MzAiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iNzMwIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDczNSA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj40PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNjIiPgogICAgICAgIDxyZWN0IHg9Ijc0OSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGZmYzAiLz4KICAgICAgICA8cmVjdCB4PSI3NDkiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNzU0IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjU8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY182MSI+CiAgICAgICAgPHJlY3QgeD0iNzY4IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9Ijc2OCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg3NzMgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NjwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzYwIj4KICAgICAgICA8cmVjdCB4PSI3ODciIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iNzg3IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDc5MiA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj43PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNzUiPgogICAgICAgIDxyZWN0IHg9IjQ2IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9IjQ2IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDUxIDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjA8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY183NCI+CiAgICAgICAgPHJlY3QgeD0iNjUiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iNjUiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNzAgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzczIj4KICAgICAgICA8cmVjdCB4PSI4NCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGZmYzAiLz4KICAgICAgICA8cmVjdCB4PSI4NCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg4OSA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4yPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNzIiPgogICAgICAgIDxyZWN0IHg9IjEwMyIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGZmYzAiLz4KICAgICAgICA8cmVjdCB4PSIxMDMiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMTA4IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjM8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY183MSI+CiAgICAgICAgPHJlY3QgeD0iMTIyIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9IjEyMiIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgxMjcgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NDwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzcwIj4KICAgICAgICA8cmVjdCB4PSIxNDEiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBmZmMwIi8+CiAgICAgICAgPHJlY3QgeD0iMTQxIiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDE0NiA0MjAuMjc2KSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj41PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNjkiPgogICAgICAgIDxyZWN0IHg9IjE2MCIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGZmYzAiLz4KICAgICAgICA8cmVjdCB4PSIxNjAiIHk9IjQxNyIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMTY1IDQyMC4yNzYpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjY8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY182OCI+CiAgICAgICAgPHJlY3QgeD0iMTc5IiB5PSI0MTciIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwZmZjMCIvPgogICAgICAgIDxyZWN0IHg9IjE3OSIgeT0iNDE3IiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgxODQgNDIwLjI3NikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NzwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzg0Ij4KICAgICAgICA8cmVjdCB4PSIyMzYiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSIyMzYiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iYmxhY2siIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMjQxIDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4wPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfODMiPgogICAgICAgIDxyZWN0IHg9IjI1NSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjI1NSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJibGFjayIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgyNjAgNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjE8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY184MiI+CiAgICAgICAgPHJlY3QgeD0iMjc0IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBjMGZmIi8+CiAgICAgICAgPHJlY3QgeD0iMjc0IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImJsYWNrIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDI3OSA0NjkuOTIwMikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MjwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzgxIj4KICAgICAgICA8cmVjdCB4PSIyOTMiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSIyOTMiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iYmxhY2siIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMjk4IDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4zPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfODAiPgogICAgICAgIDxyZWN0IHg9IjMxMiIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjMxMiIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJibGFjayIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgzMTcgNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjQ8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY183OSI+CiAgICAgICAgPHJlY3QgeD0iMzMxIiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBjMGZmIi8+CiAgICAgICAgPHJlY3QgeD0iMzMxIiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImJsYWNrIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDMzNiA0NjkuOTIwMikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzc4Ij4KICAgICAgICA8cmVjdCB4PSIzNTAiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSIzNTAiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iYmxhY2siIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMzU1IDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj42PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfNzciPgogICAgICAgIDxyZWN0IHg9IjM2OSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjM2OSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJibGFjayIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSgzNzQgNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjc8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY185MiI+CiAgICAgICAgPHJlY3QgeD0iMzg4IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBjMGZmIi8+CiAgICAgICAgPHJlY3QgeD0iMzg4IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMzkzIDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4wPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfOTEiPgogICAgICAgIDxyZWN0IHg9IjQwNyIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjQwNyIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDQxMiA0NjkuOTIwMikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzkwIj4KICAgICAgICA8cmVjdCB4PSI0MjYiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSI0MjYiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg0MzEgNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjI8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY184OSI+CiAgICAgICAgPHJlY3QgeD0iNDQ1IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBjMGZmIi8+CiAgICAgICAgPHJlY3QgeD0iNDQ1IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNDUwIDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4zPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfODgiPgogICAgICAgIDxyZWN0IHg9IjQ2NCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjQ2NCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDQ2OSA0NjkuOTIwMikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NDwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzg3Ij4KICAgICAgICA8cmVjdCB4PSI0ODMiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSI0ODMiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg0ODggNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjU8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY184NiI+CiAgICAgICAgPHJlY3QgeD0iNTAyIiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBjMGZmIi8+CiAgICAgICAgPHJlY3QgeD0iNTAyIiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNTA3IDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj42PC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfODUiPgogICAgICAgIDxyZWN0IHg9IjUyMSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjUyMSIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDUyNiA0NjkuOTIwMikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+NzwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzk3Ij4KICAgICAgICA8cmVjdCB4PSI1NDAiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSI1NDAiIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg1NDUgNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjA8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY185NiI+CiAgICAgICAgPHJlY3QgeD0iNTU5IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBmaWxsPSIjYzBjMGZmIi8+CiAgICAgICAgPHJlY3QgeD0iNTU5IiB5PSI0NjYuNjQ0MiIgd2lkdGg9IjE5IiBoZWlnaHQ9IjI1IiBzdHJva2U9ImdyYXkiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIxIi8+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNTY0IDQ2OS45MjAyKSIgZmlsbD0iYmxhY2siPgogICAgICAgICAgPHRzcGFuIGZvbnQtZmFtaWx5PSJIZWx2ZXRpY2EgTmV1ZSIgZm9udC1zaXplPSIxNiIgZmlsbD0iYmxhY2siIHg9Ii4wNTIiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj4xPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfOTUiPgogICAgICAgIDxyZWN0IHg9IjU3OCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgZmlsbD0iI2MwYzBmZiIvPgogICAgICAgIDxyZWN0IHg9IjU3OCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxOSIgaGVpZ2h0PSIyNSIgc3Ryb2tlPSJncmF5IiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiIHN0cm9rZS13aWR0aD0iMSIvPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDU4MyA0NjkuOTIwMikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSIuMDUyIiB5PSIxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+MjwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzk0Ij4KICAgICAgICA8cmVjdCB4PSI1OTciIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIGZpbGw9IiNjMGMwZmYiLz4KICAgICAgICA8cmVjdCB4PSI1OTciIHk9IjQ2Ni42NDQyIiB3aWR0aD0iMTkiIGhlaWdodD0iMjUiIHN0cm9rZT0iZ3JheSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjEiLz4KICAgICAgICA8dGV4dCB0cmFuc2Zvcm09InRyYW5zbGF0ZSg2MDIgNDY5LjkyMDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iLjA1MiIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPjM8L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMDUiPgogICAgICAgIDxyZWN0IHg9IjIzNiIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iI2IxMDAxYyIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMDYiPgogICAgICAgIDxyZWN0IHg9IjM4OCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iI2IxMDAxYyIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xMTMiPgogICAgICAgIDxyZWN0IHg9IjU0MCIgeT0iNDY2LjY0NDIiIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iI2IxMDAxYyIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iTGluZV8xMjYiPgogICAgICAgIDxwYXRoIGQ9Ik0gMjM2IDQ0MiBMIDIzNiA0NDIgQyAyMzYgNDUwLjI5OTU0IDI0Mi43MjgxIDQ1Ny4wMjc2NSAyNTEuMDI3NjUgNDU3LjAyNzY1IEwgMjk3LjUxMjggNDU3LjAyNzY1IEMgMzAyLjQ1NDAzIDQ1Ny4wMjc2NSAzMDYuOTE0MzQgNDU5Ljk4ODEgMzA4LjgzMzMzIDQ2NC41NDE1IEwgMzA4LjgzMzMzIDQ2NC41NDE1IEMgMzA5LjU3MDQgNDY2LjI5MDQgMzExLjU4NTY3IDQ2Ny4xMTA2NCAzMTMuMzM0NTggNDY2LjM3MzU3IEMgMzE0LjE2MDkzIDQ2Ni4wMjUzIDMxNC44MTg0IDQ2NS4zNjc4MyAzMTUuMTY2NjcgNDY0LjU0MTUgTCAzMTUuMTY2NjcgNDY0LjU0MTUgQyAzMTcuMDg1NjYgNDU5Ljk4ODEgMzIxLjU0NTk3IDQ1Ny4wMjc2NSAzMjYuNDg3MiA0NTcuMDI3NjUgTCAzNzIuOTcyMzUgNDU3LjAyNzY1IEMgMzgxLjI3MTkgNDU3LjAyNzY1IDM4OCA0NTAuMjk5NTQgMzg4IDQ0MiBMIDM4OCA0NDIiIHN0cm9rZT0iI2E1YTVhNSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iTGluZV8xMjciPgogICAgICAgIDxwYXRoIGQ9Ik0gMzg5IDQ0MiBMIDM4OSA0NDIgQyAzODkgNDUwLjI5OTU0IDM5NS43MjgxIDQ1Ny4wMjc2NSA0MDQuMDI3NjUgNDU3LjAyNzY1IEwgNDUwLjUxMjggNDU3LjAyNzY1IEMgNDU1LjQ1NDAzIDQ1Ny4wMjc2NSA0NTkuOTE0MzQgNDU5Ljk4ODEgNDYxLjgzMzMzIDQ2NC41NDE1IEwgNDYxLjgzMzMzIDQ2NC41NDE1IEMgNDYyLjU3MDQgNDY2LjI5MDQgNDY0LjU4NTcgNDY3LjExMDY0IDQ2Ni4zMzQ2IDQ2Ni4zNzM1NyBDIDQ2Ny4xNjA5IDQ2Ni4wMjUzIDQ2Ny44MTg0IDQ2NS4zNjc4MyA0NjguMTY2NjcgNDY0LjU0MTUgTCA0NjguMTY2NjcgNDY0LjU0MTUgQyA0NzAuMDg1NjYgNDU5Ljk4ODEgNDc0LjU0NTk3IDQ1Ny4wMjc2NSA0NzkuNDg3MiA0NTcuMDI3NjUgTCA1MjUuOTcyMzUgNDU3LjAyNzY1IEMgNTM0LjI3MTkgNDU3LjAyNzY1IDU0MSA0NTAuMjk5NTQgNTQxIDQ0MiBMIDU0MSA0NDIiIHN0cm9rZT0iI2E1YTVhNSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iTGluZV8xMjgiPgogICAgICAgIDxwYXRoIGQ9Ik0gNTQyIDQ0Mi4wMzQyNiBMIDU0MiA0NDIuMDM0MjYgQyA1NDIgNDUwLjMzMzggNTQ4LjcyODEgNDU3LjA2MTkgNTU3LjAyNzY1IDQ1Ny4wNjE5IEwgNTY4LjI0NjMgNDU3LjA2MTkgQyA1NzIuNzExNCA0NTcuMDYxOSA1NzYuNTYwOSA0NjAuMjAxNzYgNTc3LjQ1ODMgNDY0LjU3NTc0IEwgNTc3LjQ1ODMgNDY0LjU3NTc0IEMgNTc3LjYzMyA0NjUuNDI3MiA1NzguNDY0OSA0NjUuOTc1OCA1NzkuMzE2MyA0NjUuODAxMSBDIDU3OS45MzMxIDQ2NS42NzQ1NCA1ODAuNDE1MSA0NjUuMTkyNTQgNTgwLjU0MTcgNDY0LjU3NTc0IEwgNTgwLjU0MTcgNDY0LjU3NTc0IEMgNTgxLjQzOTEgNDYwLjIwMTc2IDU4NS4yODg2IDQ1Ny4wNjE5IDU4OS43NTM3IDQ1Ny4wNjE5IEwgNjAwLjk3MjM1IDQ1Ny4wNjE5IEMgNjA5LjI3MTkgNDU3LjA2MTkgNjE2IDQ1MC4zMzM4IDYxNiA0NDIuMDM0MjYgTCA2MTYgNDQyLjAzNDI2IiBzdHJva2U9IiNhNWE1YTUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIgc3Ryb2tlLXdpZHRoPSIzIi8+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfMTMwIj4KICAgICAgICA8dGl0bGU+c3BhblswXTwvdGl0bGU+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoMjg2LjE0MjEgNDk2LjY0NDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iNzg4NzAyNGUtMTkiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj5zbGljZVswXTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzEzMSI+CiAgICAgICAgPHRpdGxlPnNwYW5bMV08L3RpdGxlPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDQ0MC4yNjQgNDk2LjY0NDIpIiBmaWxsPSJibGFjayI+CiAgICAgICAgICA8dHNwYW4gZm9udC1mYW1pbHk9IkhlbHZldGljYSBOZXVlIiBmb250LXNpemU9IjE2IiBmaWxsPSJibGFjayIgeD0iNzg4NzAyNGUtMTkiIHk9IjE1IiB4bWw6c3BhY2U9InByZXNlcnZlIj5zbGljZVsxXTwvdHNwYW4+CiAgICAgICAgPC90ZXh0PgogICAgICA8L2c+CiAgICAgIDxnIGlkPSJHcmFwaGljXzEzMiI+CiAgICAgICAgPHRleHQgdHJhbnNmb3JtPSJ0cmFuc2xhdGUoNTkxLjI2NCA0OTYuNjQ0MikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSI3ODg3MDI0ZS0xOSIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPnNsaWNlWzJdPC90c3Bhbj4KICAgICAgICA8L3RleHQ+CiAgICAgIDwvZz4KICAgICAgPGcgaWQ9IkdyYXBoaWNfMTM5Ij4KICAgICAgICA8cmVjdCB4PSI0Ny41IiB5PSI0MTciIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iIzE0NDk1ZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDAiPgogICAgICAgIDxyZWN0IHg9IjE5OS41IiB5PSI0MTciIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iIzE0NDk1ZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDEiPgogICAgICAgIDxyZWN0IHg9IjM1MS41IiB5PSI0MTciIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iIzE0NDk1ZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDIiPgogICAgICAgIDxyZWN0IHg9IjUwMy41IiB5PSI0MTciIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iIzE0NDk1ZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDMiPgogICAgICAgIDxyZWN0IHg9IjY1NS41IiB5PSI0MTciIHdpZHRoPSIxNTIiIGhlaWdodD0iMjUiIHN0cm9rZT0iIzE0NDk1ZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBzdHJva2Utd2lkdGg9IjMiLz4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDQiPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDExNi44NDQgMzkzLjU1MikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSI4NTk3NTY3ZS0xOSIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPndbMF08L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDUiPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDI2Ni41MjggMzkzLjU1MikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSI4NTk3NTY3ZS0xOSIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPndbMV08L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDYiPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDQxNS41MjggMzkzLjU1MikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSI4NTk3NTY3ZS0xOSIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPndbMl08L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDciPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDU2NC41MjggMzkzLjU1MikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSI4NTk3NTY3ZS0xOSIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPndbM108L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgICA8ZyBpZD0iR3JhcGhpY18xNDgiPgogICAgICAgIDx0ZXh0IHRyYW5zZm9ybT0idHJhbnNsYXRlKDcxMy41MjggMzkzLjU1MikiIGZpbGw9ImJsYWNrIj4KICAgICAgICAgIDx0c3BhbiBmb250LWZhbWlseT0iSGVsdmV0aWNhIE5ldWUiIGZvbnQtc2l6ZT0iMTYiIGZpbGw9ImJsYWNrIiB4PSI4NTk3NTY3ZS0xOSIgeT0iMTUiIHhtbDpzcGFjZT0icHJlc2VydmUiPndbNF08L3RzcGFuPgogICAgICAgIDwvdGV4dD4KICAgICAgPC9nPgogICAgPC9nPgogIDwvZz4KPC9zdmc+Cg==
