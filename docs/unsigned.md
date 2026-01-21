# The `Unsigned` Trait

## Introduction

While computer hardware famously operates on bits, computers don't really provide _direct_ access to _single_ bits.

Instead, computers have memory registers for _words_ where the smallest addressable unit is an eight-bit word, a _byte_.
Other "native" word lengths vary by computer architecture, but 8, 16, 32, 64, and even 128-bit words are widely supported.
Computers perform operations on and between those short word types optimally.

Computers have lots of primitive word types --- bytes, characters, various sized integers which can be positive or negative, floating point numbers in various degrees of precision.
Blocks of zeros and ones are best modelled by the simplest primitive _unsigned integer_ types: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, etc.

In this library we pack contiguous bit elements into arrays of one of those primitive unsigned word types

For example, if we have a bit-vector of size 200, and the underlying word is a `u64`, the bit elements will be packed into four words (a total of 256 bits), and there will be 56 bits of unused capacity.
The library will efficiently perform almost all operations on that vector 64 bits at a time in an inherently parallel manner.

In C++ one uses the standard `std::unsigned_integral` concept to build objects that are generic over the primitive unsigned types.

Unfortunately, Rust does not have an equivalent trait in its _standard_ library.
While there are third party crates that fill the void we decided to roll our own with the relatively simple [`Unsigned`] trait that captures all the functionality we need from words that are just "bit blocks" and which support all needed arithmetic and bit twiddling operations.

The `Unsigned` trait is implemented for the primitive unsigned types `u8`, `u16`, `u32`, `u64`, `u128`, and `usize`.

The rest of the library is _generic_ over the particular `Unsigned` type you choose to use.
The default is `usize`, which on most modern computers will have 64 bits and a good choice for most applications.

You might pick a different word if, for example, your application used a vast number of smaller bit-vectors and the unused capacity is a concern.

<div style="border: 2px solid #ccc; border-radius: 8px; padding: 16px; margin: 16px 0; display: flex; align-items: center;">
<div style="font-size: 48px; margin-right: 12px; color: #666;">📝</div>

Unless otherwise noted, the library assumes you use a single word type for all objects.
It is theoretically possible to support operations between bit-vectors where some are based on `u64` words and others on `u32` words, but only at the expense of greatly increasing the complexity of the library's code.
We do not support that except in a very limited part of the API.

</div>

## Constants

The trait defines the following associated constants:

| Name          | Description                                            |
| ------------- | ------------------------------------------------------ |
| `BITS`        | The number of bits in the type as a `u32`.             |
| `UBITS`       | The number of bits in the type as a `usize`.           |
| `ZERO`        | The additive identity (0).                             |
| `ONE`         | The multiplicative identity (1).                       |
| `MAX`         | The maximum value representable by the type.           |
| `ALTERNATING` | A value with alternating bits set (e.g., `0b1010...`). |

## Associated Methods

In our library we pack bit elements into arrays and vectors of words of an `Unsigned` type and need to be able to compute which word holds a particular bit element and where within that word the bit element is located etc.

We define several methods that are useful for working with bit-stores that pack bit elements into arrays of words:

| Name                            | Description                                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------------------------ |
| [`Unsigned::words_needed`]      | Returns the number of words needed to store a certain number of bits.                            |
| [`Unsigned::word_index`]        | Returns the index of the word holding a given bit element.                                       |
| [`Unsigned::bit_offset`]        | Returns the bit position within the containing word for a bit element.                           |
| [`Unsigned::index_and_offset`]  | Returns a pair of the word-index and the bit position within the word for bit element.           |
| [`Unsigned::index_and_mask`]    | Returns a pair of the word-index and a mask to isolate a bit within that word for bit element.   |
| [`Unsigned::lowest_set_bit`]    | Returns the _index_ of the lowest set bit or `None` if there are no set bits.                    |
| [`Unsigned::highest_set_bit`]   | Returns the _index_ of the highest set bit or `None` if there are no set bits.                   |
| [`Unsigned::lowest_unset_bit`]  | Returns the _index_ of the lowest unset bit or `None` if there are no set bits.                  |
| [`Unsigned::highest_unset_bit`] | Returns the _index_ of the highest unset bit or `None` if there are no set bits.                 |
| [`Unsigned::min_digits`]        | Returns the minimum number of binary digits needed to represent a value.                         |
| [`Unsigned::prev_power_of_two`] | Returns the greatest power of two less than or equal to the value.                               |
| [`Unsigned::with_set_bits`]     | Returns an `Unsigned` with all the bits in a passed set to one and the other set to zero.        |
| [`Unsigned::with_unset_bits`]   | Returns an `Unsigned` with all the bits in a passed set to zero and the other set to one.        |
| [`Unsigned::set_bits`]          | Sets all the bits in a passed range to one.                                                      |
| [`Unsigned::reset_bits`]        | Sets all the bits in a passed range to zero.                                                     |
| [`Unsigned::set_except_bits`]   | Sets all the bits outside a passed range to one.                                                 |
| [`Unsigned::reset_except_bits`] | Sets all the bits outside a passed range to zero.                                                |
| [`Unsigned::replace_bits`]      | Copy the bits in a passed range from a passed source value and don't touch the other bits.       |
| [`Unsigned::riffle`]            | Riffle a value into a pair of others containing the bits in the original interleaved with zeros. |

## Methods that Forward to the Standard Library

The trait defines lots of methods that simply forward to the corresponding methods on the primitive unsigned types in the standard library.
Here is a list of those methods, where we are using `usize` as a placeholder for any of the primitive unsigned types:

- [`usize::as_bool`]
- [`usize::as_u8`]
- [`usize::as_u16`]
- [`usize::as_u32`]
- [`usize::as_u64`]
- [`usize::as_u128`]
- [`usize::as_usize`]
- [`usize::from_str_radix`]
- [`usize::count_ones`]
- [`usize::count_zeros`]
- [`usize::leading_zeros`]
- [`usize::trailing_zeros`]
- [`usize::leading_ones`]
- [`usize::trailing_ones`]
- [`usize::rotate_left`]
- [`usize::rotate_right`]
- [`usize::swap_bytes`]
- [`usize::reverse_bits`]
- [`usize::unbounded_shl`]
- [`usize::unbounded_shr`]
- [`usize::pow`]
- [`usize::div_euclid`]
- [`usize::rem_euclid`]
- [`usize::to_be_bytes`]
- [`usize::to_le_bytes`]
- [`usize::to_ne_bytes`]
- [`usize::from_be_bytes`]
- [`usize::from_le_bytes`]
- [`usize::from_ne_bytes`]
- [`usize::from_be`]
- [`usize::from_le`]
- [`usize::to_be`]
- [`usize::to_le`]
- [`usize::is_power_of_two`]
- [`usize::next_power_of_two`]
- [`usize::checked_next_power_of_two`]

You can find documentation for those methods in the Rust standard library documentation for the primitive unsigned integer types such as [`usize`].

## Acknowledgement

The trait is a simplified version of [`funty::Unsigned`], with some additional methods useful for working with bit-vectors and bit-matrices.

[`funty::Unsigned`]: https://docs.rs/funty/2.0.0/funty/trait.Unsigned.html
[GF(2)]: https://en.wikipedia.org/wiki/Finite_field_arithmetic
[`usize`]: https://doc.rust-lang.org/std/primitive.usize.html
