//! [`Unsigned`] is a trait implemented for all Rust's primitive unsigned integer types.

#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use core::{
    fmt::{
        Binary,
        Debug,
        Display,
        LowerHex,
        Octal,
        UpperHex,
    },
    hash::Hash,
    iter::{
        Product,
        Sum,
    },
    num::ParseIntError,
    ops::{
        Add,
        AddAssign,
        BitAnd,
        BitAndAssign,
        BitOr,
        BitOrAssign,
        BitXor,
        BitXorAssign,
        Bound,
        Div,
        DivAssign,
        Mul,
        MulAssign,
        Not,
        RangeBounds,
        Rem,
        RemAssign,
        Shl,
        ShlAssign,
        Shr,
        ShrAssign,
        Sub,
        SubAssign,
    },
    str::FromStr,
};

// Produces a doc-string that forwards to a sample entry in the standard library pertaining to `u32`.
// This can be a method e.g. `u32.count_ones()` or an associated function e.g. `u32::from_str_radix()`.
macro_rules! primitive_url {
    ($f:ident) => {
        concat!(
            "See for example, [u32.",
            stringify!($f),
            "](https://doc.rust-lang.org/std/primitive.u32.html#method.",
            stringify!($f),
            ").",
        )
    };
    (::$f:ident) => {
        concat!(
            "See for example, [u32::",
            stringify!($f),
            "](https://doc.rust-lang.org/std/primitive.u32.html#method.",
            stringify!($f),
            ").",
        )
    };
}

#[doc = include_str!("../docs/unsigned.md")]
pub trait Unsigned:
    'static
    + Sized
    + Send
    + Sync
    + Unpin
    + Clone
    + Copy
    + Default
    + FromStr
    + PartialEq<Self>
    + PartialOrd<Self>
    + Debug
    + Display
    + Product<Self>
    + for<'a> Product<&'a Self>
    + Sum<Self>
    + for<'a> Sum<&'a Self>
    + Add<Self, Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + AddAssign<Self>
    + for<'a> AddAssign<&'a Self>
    + Sub<Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + SubAssign<Self>
    + for<'a> SubAssign<&'a Self>
    + Mul<Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + MulAssign<Self>
    + for<'a> MulAssign<&'a Self>
    + Div<Self, Output = Self>
    + for<'a> Div<&'a Self, Output = Self>
    + DivAssign<Self>
    + for<'a> DivAssign<&'a Self>
    + Rem<Self, Output = Self>
    + for<'a> Rem<&'a Self, Output = Self>
    + RemAssign<Self>
    + for<'a> RemAssign<&'a Self>
    + Hash
    + Eq
    + Ord
    + Binary
    + LowerHex
    + UpperHex
    + Octal
    + BitAnd<Self, Output = Self>
    + for<'a> BitAnd<&'a Self, Output = Self>
    + BitAndAssign<Self>
    + for<'a> BitAndAssign<&'a Self>
    + BitOr<Self, Output = Self>
    + for<'a> BitOr<&'a Self, Output = Self>
    + BitOrAssign<Self>
    + for<'a> BitOrAssign<&'a Self>
    + BitXor<Self, Output = Self>
    + for<'a> BitXor<&'a Self, Output = Self>
    + BitXorAssign<Self>
    + for<'a> BitXorAssign<&'a Self>
    + Not<Output = Self>
    + TryFrom<u8>
    + TryFrom<u16>
    + TryFrom<u32>
    + TryFrom<u64>
    + TryFrom<u128>
    + TryFrom<usize>
    + TryInto<u8>
    + TryInto<u16>
    + TryInto<u32>
    + TryInto<u64>
    + TryInto<u128>
    + TryInto<usize>
    + Shl<u32, Output = Self>
    + for<'a> Shl<&'a u32, Output = Self>
    + ShlAssign<u32>
    + for<'a> ShlAssign<&'a u32>
    + Shr<u32, Output = Self>
    + for<'a> Shr<&'a u32, Output = Self>
    + ShrAssign<u32>
    + for<'a> ShrAssign<&'a u32>
    + Shl<usize, Output = Self>
    + for<'a> Shl<&'a usize, Output = Self>
    + ShlAssign<usize>
    + for<'a> ShlAssign<&'a usize>
    + Shr<usize, Output = Self>
    + for<'a> Shr<&'a usize, Output = Self>
    + ShrAssign<usize>
    + for<'a> ShrAssign<&'a usize>
{
    // ----------------------------------------------------------------------------------------------------------------
    // Associated constants
    // ----------------------------------------------------------------------------------------------------------------

    /// The number 0 as a value of this type.
    const ZERO: Self;

    /// The number 1 as a value of this type.
    const ONE: Self;

    /// The maximum value of this type.
    const MAX: Self;

    /// The value of this type with alternating bits set to one: `0b01010101...`
    const ALTERNATING: Self;

    /// The number of bits in this type.
    const BITS: u32;

    /// The number of bits in this type as a `usize`.
    const UBITS: usize;

    // ----------------------------------------------------------------------------------------------------------------
    // Casting methods
    // ----------------------------------------------------------------------------------------------------------------

    /// Tests `self != 0`.
    fn as_bool(self) -> bool { self != Self::ZERO }

    /// Performs `self as u8`.
    fn as_u8(self) -> u8;

    /// Performs `self as u16`.
    fn as_u16(self) -> u16;

    /// Performs `self as u32`.
    fn as_u32(self) -> u32;

    /// Performs `self as u64`.
    fn as_u64(self) -> u64;

    /// Performs `self as u128`.
    fn as_u128(self) -> u128;

    /// Performs `self as usize`.
    fn as_usize(self) -> usize;

    // ----------------------------------------------------------------------------------------------------------------
    // Methods that work for all Rust's numerical primitive types
    // ----------------------------------------------------------------------------------------------------------------

    #[doc = primitive_url!(::from_str_radix)]
    #[allow(clippy::missing_errors_doc)]
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError>;

    #[doc = primitive_url!(count_ones)]
    fn count_ones(self) -> u32;

    #[doc = primitive_url!(count_zeros)]
    fn count_zeros(self) -> u32;

    #[doc = primitive_url!(leading_zeros)]
    fn leading_zeros(self) -> u32;

    #[doc = primitive_url!(trailing_zeros)]
    fn trailing_zeros(self) -> u32;

    #[doc = primitive_url!(leading_ones)]
    fn leading_ones(self) -> u32;

    #[doc = primitive_url!(trailing_ones)]
    fn trailing_ones(self) -> u32;

    #[doc = primitive_url!(rotate_left)]
    #[must_use]
    fn rotate_left(self, n: u32) -> Self;

    #[doc = primitive_url!(rotate_right)]
    #[must_use]
    fn rotate_right(self, n: u32) -> Self;

    #[doc = primitive_url!(swap_bytes)]
    #[must_use]
    fn swap_bytes(self) -> Self;

    #[doc = primitive_url!(reverse_bits)]
    #[must_use]
    fn reverse_bits(self) -> Self;

    #[doc = primitive_url!(unbounded_shl)]
    #[must_use]
    fn unbounded_shl(self, rhs: u32) -> Self;

    #[doc = primitive_url!(unbounded_shr)]
    #[must_use]
    fn unbounded_shr(self, rhs: u32) -> Self;

    #[doc = primitive_url!(pow)]
    #[must_use]
    fn pow(self, rhs: u32) -> Self;

    #[doc = primitive_url!(div_euclid)]
    #[must_use]
    fn div_euclid(self, rhs: Self) -> Self;

    #[doc = primitive_url!(rem_euclid)]
    #[must_use]
    fn rem_euclid(self, rhs: Self) -> Self;

    // ----------------------------------------------------------------------------------------------------------------
    // Methods that convert Rust primitive types to/from a fixed size array of bytes.
    // ----------------------------------------------------------------------------------------------------------------

    /// The `[u8; N]` byte array that stores values of `Self`.
    type Bytes;

    #[doc = primitive_url!(to_be_bytes)]
    fn to_be_bytes(self) -> Self::Bytes;

    #[doc = primitive_url!(to_le_bytes)]
    fn to_le_bytes(self) -> Self::Bytes;

    #[doc = primitive_url!(to_ne_bytes)]
    fn to_ne_bytes(self) -> Self::Bytes;

    #[doc = primitive_url!(::from_be_bytes)]
    fn from_be_bytes(bytes: Self::Bytes) -> Self;

    #[doc = primitive_url!(::from_le_bytes)]
    #[must_use]
    fn from_le_bytes(bytes: Self::Bytes) -> Self;

    #[doc = primitive_url!(::from_ne_bytes)]
    #[must_use]
    fn from_ne_bytes(bytes: Self::Bytes) -> Self;

    #[doc = primitive_url!(from_be)]
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    fn from_be(self) -> Self;

    #[doc = primitive_url!(from_le)]
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    fn from_le(self) -> Self;

    #[doc = primitive_url!(to_be)]
    #[must_use]
    fn to_be(self) -> Self;

    #[doc = primitive_url!(to_le)]
    #[must_use]
    fn to_le(self) -> Self;

    // ----------------------------------------------------------------------------------------------------------------
    // Methods that only work for Rust's unsigned integer primitive types
    // ----------------------------------------------------------------------------------------------------------------

    #[doc = primitive_url!(is_power_of_two)]
    fn is_power_of_two(self) -> bool;

    #[doc = primitive_url!(next_power_of_two)]
    #[must_use]
    fn next_power_of_two(self) -> Self;

    #[doc = primitive_url!(checked_next_power_of_two)]
    #[must_use]
    fn checked_next_power_of_two(self) -> Option<Self>;

    // ----------------------------------------------------------------------------------------------------------------
    // Some extra associated methods that are useful for working with arrays of bits.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns the number of words of this type needed to store `n_bits` bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::words_needed(0), 0);
    /// assert_eq!(u8::words_needed(1), 1);
    /// assert_eq!(u8::words_needed(8), 1);
    /// assert_eq!(u8::words_needed(10), 2);
    /// assert_eq!(u8::words_needed(16), 2);
    /// assert_eq!(u8::words_needed(17), 3);
    /// assert_eq!(u16::words_needed(17), 2);
    /// assert_eq!(u32::words_needed(17), 1);
    /// ```
    #[must_use]
    #[inline]
    fn words_needed(n_bits: usize) -> usize { n_bits.div_ceil(Self::UBITS) }

    /// Returns the index of the `Unsigned` word holding bit element `i`.
    ///
    /// If you are storing `n_bits` in a contiguous array of `Unsigned`s, this returns the index of the *word* that
    /// holds a bit indexed by `i`. The return value is a `usize` in the range `[0, Word::words_needed(n_bits))`.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::word_index(0), 0);
    /// assert_eq!(u8::word_index(1), 0);
    /// assert_eq!(u8::word_index(8), 1);
    /// assert_eq!(u8::word_index(9), 1);
    /// assert_eq!(u8::word_index(16), 2);
    /// assert_eq!(u8::word_index(17), 2);
    /// assert_eq!(u16::word_index(17), 1);
    /// assert_eq!(u32::word_index(17), 0);
    /// ```
    #[must_use]
    #[inline]
    fn word_index(i: usize) -> usize { i / Self::UBITS }

    /// Returns the bit position within the containing word for bit element `i`.
    ///
    /// If you are storing bits in a contiguous array of `Unsigned`s, this returns the offset of bit element `i` within
    /// its containing `Unsigned` word. The returned bit index is a `u32` that is in the range `[0, Word::UBITS)`.
    ///
    /// # Note
    /// We return a `u32` because that is the type Rust uses for its native bit-twiddling methods.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::bit_offset(0), 0);
    /// assert_eq!(u8::bit_offset(1), 1);
    /// assert_eq!(u8::bit_offset(8), 0);
    /// assert_eq!(u8::bit_offset(9), 1);
    /// assert_eq!(u8::bit_offset(16), 0);
    /// assert_eq!(u8::bit_offset(17), 1);
    /// assert_eq!(u16::bit_offset(17), 1);
    /// assert_eq!(u32::bit_offset(17), 17);
    /// ```
    #[must_use]
    #[inline]
    fn bit_offset(i: usize) -> u32 { (i % Self::UBITS) as u32 }

    /// Returns a pair of the index of the word and the bit position within the word for bit element `i`.
    ///
    /// If you are storing bits in a contiguous array of `Unsigned`s, this returns the pair: `(index, offset)` that
    /// locates bit element `i` within the array --- `index` is the index of the containing word and `offset` is the
    /// bit position within that word.
    ///
    /// # Note
    /// We return a `u32` for `offset` because that is the type Rust uses for its native bit-twiddling methods.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::index_and_offset(0), (0, 0));
    /// assert_eq!(u8::index_and_offset(1), (0, 1));
    /// assert_eq!(u8::index_and_offset(10), (1, 2));
    /// assert_eq!(u8::index_and_offset(11), (1, 3));
    /// assert_eq!(u8::index_and_offset(12), (1, 4));
    /// assert_eq!(u8::index_and_offset(13), (1, 5));
    /// assert_eq!(u8::index_and_offset(14), (1, 6));
    /// assert_eq!(u8::index_and_offset(15), (1, 7));
    /// assert_eq!(u8::index_and_offset(16), (2, 0));
    /// assert_eq!(u16::index_and_offset(16), (1, 0));
    /// assert_eq!(u32::index_and_offset(16), (0, 16));
    /// ```
    #[must_use]
    #[inline]
    fn index_and_offset(i: usize) -> (usize, u32) { (Self::word_index(i), Self::bit_offset(i)) }

    /// If you are storing bits in a contiguous array of `Unsigned`s, this returns the pair: `(word, mask)` that can
    /// isolate a bit at the given `bit_offset`.
    ///
    /// This assumes we are storing bits in a contiguous array of `Unsigned` words.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// type Word = u8;
    /// assert_eq!(Word::index_and_mask(0), (0, 0b00000001));
    /// assert_eq!(Word::index_and_mask(1), (0, 0b00000010));
    /// assert_eq!(Word::index_and_mask(8), (1, 0b00000001));
    /// assert_eq!(Word::index_and_mask(9), (1, 0b00000010));
    /// assert_eq!(Word::index_and_mask(16), (2, 0b00000001));
    /// assert_eq!(Word::index_and_mask(17), (2, 0b00000010));
    /// ```
    #[must_use]
    #[inline]
    fn index_and_mask(bit_offset: usize) -> (usize, Self) {
        (Self::word_index(bit_offset), Self::ONE << Self::bit_offset(bit_offset))
    }

    /// Returns the *index* of the lowest set bit in `self` or `None` if there are no set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(0b0000_0000_u8.lowest_set_bit(), None);
    /// assert_eq!(0b0001_1001_u8.lowest_set_bit(), Some(0));
    /// assert_eq!(0b1111_1000_u8.lowest_set_bit(), Some(3));
    /// ```
    #[must_use]
    #[inline]
    fn lowest_set_bit(&self) -> Option<u32> {
        if *self != Self::ZERO {
            return Some(self.trailing_zeros());
        }
        None
    }

    /// Returns the *index* of the highest set bit in `self` or `None` if there are no set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(0b0000_0000_u8.highest_set_bit(), None);
    /// assert_eq!(0b0001_1001_u8.highest_set_bit(), Some(4));
    /// assert_eq!(0b1111_1000_u8.highest_set_bit(), Some(7));
    /// ```
    #[inline]
    fn highest_set_bit(&self) -> Option<u32> {
        if *self != Self::ZERO {
            return Some(Self::BITS - self.leading_zeros() - 1);
        }
        None
    }

    /// Returns the *index* of the lowest unset bit in `self` or `None` if there are no unset bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(0b1111_1111_u8.lowest_unset_bit(), None);
    /// assert_eq!(0b0001_1000_u8.lowest_unset_bit(), Some(0));
    /// assert_eq!(0b0000_0011_u8.lowest_unset_bit(), Some(2));
    /// ```
    #[inline]
    fn lowest_unset_bit(&self) -> Option<u32> {
        if *self != Self::MAX {
            return Some(self.trailing_ones());
        }
        None
    }

    /// Returns the *index* of the highest unset bit in `self` or `None` if there are no unset bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(0b1111_1111_u8.highest_unset_bit(), None);
    /// assert_eq!(0b1111_1001_u8.highest_unset_bit(), Some(2));
    /// assert_eq!(0b0011_1001_u8.highest_unset_bit(), Some(7));
    /// ```
    #[inline]
    fn highest_unset_bit(&self) -> Option<u32> {
        if *self != Self::MAX {
            return Some(Self::BITS - self.leading_ones() - 1);
        }
        None
    }

    /// Returns the minimum number of digits required to represent the `self` value.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(0u32.min_digits(), 0);
    /// assert_eq!(1u32.min_digits(), 1);
    /// assert_eq!(2u32.min_digits(), 2);
    /// assert_eq!(3u32.min_digits(), 2);
    /// assert_eq!(4u32.min_digits(), 3);
    /// assert_eq!(8u32.min_digits(), 4);
    /// assert_eq!(9u32.min_digits(), 4);
    /// ```
    #[inline]
    fn min_digits(&self) -> u32 { Self::BITS - self.leading_zeros() }

    /// Returns the greatest power of two less than or equal to `self`, or 0 otherwise.
    ///
    /// Note:
    /// - This is the companion to `next_power_of_two` supplied as standard for Rust's primitive unsigned types.
    /// - In C++ terms this is the `bit_floor` function while the `next_power_of_two` is `bit_ceil`.
    /// - Returning 0 if `self` is 0 is a convention we choose to use here rather than panicking etc.
    #[inline]
    #[must_use]
    fn prev_power_of_two(&self) -> Self {
        // If `*self == 0` then then the following sets `highest_set_bit` to 0.
        let highest_set_bit = Self::BITS - 1 - (*self | Self::ONE).leading_zeros();
        (Self::ONE << highest_set_bit) & *self
    }

    /// Consumes the `range` argument and returns a pair of `u32` values `(start, end)` where the corresponding *bits*
    /// of interest in the `Unsigned` are those in the half-open interval `[start, end)`.
    ///
    /// If the range is unbounded on the left, we set `start` to `0`. <br>
    /// If the range is unbounded on the right, we set `end` to `Word::BITS`.
    ///
    /// # Note
    /// In Rust all primitive bit operations are defined in terms of `u32` values which is what we return here.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::bit_start_and_end_from(4..), (4, 8));
    /// assert_eq!(u8::bit_start_and_end_from(..=3), (0, 4));
    /// assert_eq!(u8::bit_start_and_end_from(..), (0, 8));
    /// assert_eq!(u32::bit_start_and_end_from(7..), (7, 32));
    /// ```
    #[inline]
    fn bit_start_and_end_from<R: RangeBounds<u32>>(range: R) -> (u32, u32) {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => Self::BITS,
        };
        (start, end)
    }

    /// Returns an `Unsigned` with all the bits in `range` set to one, all other bits set to zero.
    ///
    /// # Panics
    ///
    /// In debug mode, this method will panic if the start of the range is greater than the end of the range or if the
    /// end of the range is greater than the number of bits in the word.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::with_set_bits(0..0), 0b0000_0000_u8);
    /// assert_eq!(u8::with_set_bits(..1), 0b0000_0001_u8);
    /// assert_eq!(u8::with_set_bits(1..3), 0b0000_0110_u8);
    /// assert_eq!(u8::with_set_bits(..), 0b1111_1111_u8);
    /// ```
    #[inline]
    #[must_use]
    fn with_set_bits<R: RangeBounds<u32>>(range: R) -> Self {
        let (start, end) = Self::bit_start_and_end_from(range);
        debug_assert!(start <= end, "start: {start} must be less than or equal to end: {end}");
        debug_assert!(end <= Self::BITS, "end: {end} cannot be greater than the number of word bits: {}", Self::BITS);
        Self::MAX.unbounded_shl(start) & Self::MAX.unbounded_shr(Self::BITS - end)
    }

    /// Returns an `Unsigned` with all the bits in `range` set to zero, all other bits set to one.
    ///
    /// # Panics
    ///
    /// In debug mode, this method will panic if the start of the range is greater than the end of the range or if the
    /// end of the range is greater than the number of bits in the word.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// assert_eq!(u8::with_unset_bits(..2), 0b1111_1100_u8);
    /// assert_eq!(u8::with_unset_bits(..3), 0b1111_1000_u8);
    /// ```
    #[inline]
    fn with_unset_bits<R: RangeBounds<u32>>(range: R) -> Self { !Self::with_set_bits(range) }

    /// Set all the bits in a `range` to one.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// let mut word = 0b0000_0000_u8;
    /// word.set_bits(1..3);
    /// assert_eq!(word, 0b0000_0110_u8);
    /// ```
    #[inline]
    fn set_bits<R: RangeBounds<u32>>(&mut self, range: R) { *self |= Self::with_set_bits(range); }

    /// Reset all the bits in a `range` to zero.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// let mut word = 0b1111_1111_u8;
    /// word.reset_bits(1..3);
    /// assert_eq!(word, 0b1111_1001_u8);
    /// ```
    #[inline]
    fn reset_bits<R: RangeBounds<u32>>(&mut self, range: R) { *self &= !Self::with_set_bits(range); }

    /// Sets all the bits *except* the bits in a `range`.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// let mut word = 0b0000_0000_u8;
    /// word.set_except_bits(1..3);
    /// assert_eq!(word, 0b1111_1001_u8);
    /// ```
    #[inline]
    fn set_except_bits<R: RangeBounds<u32>>(&mut self, range: R) { *self |= !Self::with_set_bits(range); }

    /// Resets all the bits *except* the bits in a `range`.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// let mut word = 0b1111_1111_u8;
    /// word.reset_except_bits(1..3);
    /// assert_eq!(word, 0b0000_0110_u8);
    /// ```
    #[inline]
    fn reset_except_bits<R: RangeBounds<u32>>(&mut self, range: R) { *self &= Self::with_set_bits(range); }

    /// Replace a range of bits in `self` with the corresponding bits from `with`.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// let mut word = 0b1111_1111_u8;
    /// word.replace_bits(1..3, 0b0000_0000_u8);
    /// assert_eq!(word, 0b1111_1001_u8);
    /// ```
    #[inline]
    fn replace_bits<R: RangeBounds<u32>>(&mut self, range: R, with: Self) {
        let with_mask = Self::with_set_bits(range);
        let self_mask = !with_mask;
        *self = (*self & self_mask) | (with & with_mask);
    }

    /// Riffle an unsigned integer into a pair of others containing the bits in the original word interleaved with
    /// zeros.
    ///
    /// For example, if `self` is a `u8` with the binary representation `abcdefgh`, then on return `lo` will have the
    /// bits `0a0b0c0d` and `hi` will have the bits `0e0f0g0h`. The `lo` and `hi` words are returned in a tuple.
    ///
    /// # Examples
    /// ```
    /// use gf2::Unsigned;
    /// let word = u8::MAX;
    /// let (lo, hi) = word.riffle();
    /// assert_eq!(lo, 0b0101_0101_u8, "Expected lo: 01010101, got: {lo:08b}");
    /// assert_eq!(hi, 0b0101_0101_u8, "Expected hi: 01010101, got: {hi:08b}");
    /// ```
    fn riffle(&self) -> (Self, Self) {
        let half_bits = Self::BITS / 2;
        let mut lo = *self & (Self::MAX >> half_bits);
        let mut hi = *self >> half_bits;

        // Some magic to interleave the respective halves with zeros.
        let mut i = Self::BITS / 4;
        while i > 0 {
            let div = (Self::ONE << i) | Self::ONE;
            let mask = Self::MAX / div;
            lo = (lo ^ (lo << i)) & mask;
            hi = (hi ^ (hi << i)) & mask;
            i /= 2;
        }
        (lo, hi)
    }
}

/// A macro that implements the `Unsigned` trait for the given types -- it just forwards the required methods to the
/// versions that are available for Rust's primitive unsigned integer types.
macro_rules! impl_unsigned {
	($($t:ty),+ $(,)?) => { $(
		impl Unsigned for $t {
            const ZERO: Self = 0;
			const ONE: Self = 1;
			const MAX: Self = Self::MAX;
            const ALTERNATING: Self = Self::MAX / 3;
			const BITS: u32 = Self::BITS;
			const UBITS: usize = Self::BITS as usize;

            type Bytes = [u8; core::mem::size_of::<Self>()];

			#[inline]
			fn as_u8(self) -> u8 { self as u8 }

			#[inline]
			fn as_u16(self) -> u16 { self as u16 }

			#[inline]
			fn as_u32(self) -> u32 { self as u32 }

			#[inline]
			fn as_u64(self) -> u64 { self as u64 }

			#[inline]
			fn as_u128(self) ->u128 { self as u128 }

			#[inline]
			fn as_usize(self) -> usize { self as usize }

            #[inline]
            fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError> {
                Self::from_str_radix(src, radix)
            }

            #[inline]
            fn count_ones(self) -> u32 {
                Self::count_ones(self)
            }

            #[inline]
            fn count_zeros(self) -> u32 {
                Self::count_zeros(self)
            }

            #[inline]
            fn leading_zeros(self) -> u32 {
                Self::leading_zeros(self)
            }

            #[inline]
            fn trailing_zeros(self) -> u32 {
                Self::trailing_zeros(self)
            }

            #[inline]
            fn leading_ones(self) -> u32 {
                Self::leading_ones(self)
            }

            #[inline]
            fn trailing_ones(self) -> u32 {
                Self::trailing_ones(self)
            }

            #[inline]
            fn rotate_left(self, n: u32) -> Self {
                Self::rotate_left(self, n)
            }

            #[inline]
            fn rotate_right(self, n: u32) -> Self {
                Self::rotate_right(self, n)
            }

            #[inline]
            fn swap_bytes(self) -> Self {
                Self::swap_bytes(self)
            }

            #[inline]
            fn reverse_bits(self) -> Self {
                Self::reverse_bits(self)
            }

            #[inline]
            fn from_be(self) -> Self {
                Self::from_be(self)
            }

            #[inline]
            fn from_le(self) -> Self {
                Self::from_le(self)
            }

            #[inline]
            fn unbounded_shl(self, rhs: u32) -> Self {
                Self::unbounded_shl(self, rhs)
            }

            #[inline]
            fn unbounded_shr(self, rhs: u32) -> Self {
                Self::unbounded_shr(self, rhs)
            }

            #[inline]
            fn pow(self, rhs: u32) -> Self {
                Self::pow(self, rhs)
            }

            #[inline]
            fn div_euclid(self, rhs: Self) -> Self {
                Self::div_euclid(self, rhs)
            }

            #[inline]
            fn rem_euclid(self, rhs: Self) -> Self {
                Self::rem_euclid(self, rhs)
            }

            #[inline]
            fn to_be_bytes(self) -> Self::Bytes {
                self.to_be_bytes()
            }

            #[inline]
            fn to_le_bytes(self) -> Self::Bytes {
                self.to_le_bytes()
            }

            #[inline]
            fn to_ne_bytes(self) -> Self::Bytes {
                self.to_ne_bytes()
            }

            #[inline]
            fn from_be_bytes(bytes: Self::Bytes) -> Self {
                Self::from_be_bytes(bytes)
            }

            #[inline]
            fn from_le_bytes(bytes: Self::Bytes) -> Self {
                Self::from_le_bytes(bytes)
            }

            #[inline]
            fn from_ne_bytes(bytes: Self::Bytes) -> Self {
                Self::from_ne_bytes(bytes)
            }

            #[inline]
            fn to_be(self) -> Self {
                Self::to_be(self)
            }

            #[inline]
            fn to_le(self) -> Self {
                Self::to_le(self)
            }

            #[inline]
            fn is_power_of_two(self) -> bool {
                Self::is_power_of_two(self)
            }

            #[inline]
            fn next_power_of_two(self) -> Self {
                Self::next_power_of_two(self)
            }

            #[inline]
            fn checked_next_power_of_two(self) -> Option<Self> {
                Self::checked_next_power_of_two(self)
            }
		}
	)+ };
}

// Call the macro to implement the `Unsigned` trait for all the unsigned integer types.
impl_unsigned!(u8, u16, u32, u64, u128, usize);
