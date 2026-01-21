#![doc = include_str!("../docs/store_traits.md")]

// Crate imports.
use crate::{
    BitSlice,
    BitStore,
    BitVec,
    Unsigned,
};

// BitArray requires unstable features.
#[cfg(feature = "unstable")]
use crate::array::BitArray;

// Standard library imports.
use std::{
    fmt::{
        self,
    },
    ops::{
        Add,
        AddAssign,
        BitAnd,
        BitAndAssign,
        BitOr,
        BitOrAssign,
        BitXor,
        BitXorAssign,
        Index,
        Mul,
        Not,
        Shl,
        ShlAssign,
        Shr,
        ShrAssign,
        Sub,
        SubAssign,
    },
};

// ====================================================================================================================
// `impl_unary_traits` macro implements the following foreign traits for any single bit-store type.
//
// - `Index`
// - `Display`
// - `Binary`
// - `UpperHex`
// - `LowerHex`
// - `ShlAssign`
// - `ShrAssign`
// - `Shl`
// - `Shr`
// ====================================================================================================================
macro_rules! impl_unary_traits {

    // The `BitVec` case which has just the one generic parameter: `Word: Unsigned`.
    (BitVec) => {
        impl_unary_traits!(@impl BitVec[Word]; [Word: Unsigned]);
    };

    // The `BitSlice` case with an `'a` lifetime parameter as well as the `Word: Unsigned` parameter.
    (BitSlice) => {
        impl_unary_traits!(@impl BitSlice['a, Word]; ['a, Word: Unsigned]);
    };

    // The `BitArray` case with a `const N: usize` parameter as well as the `Word: Unsigned` parameter.
    // Until Rust gets better const generics, there is also an extra fudge `const WORDS: usize` automatic parameter.
    (BitArray) => {
        impl_unary_traits!(@impl BitArray[N, Word, WORDS]; [const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The other arms funnel to this one which does the actual work of implementing the various foreign traits:
    // This matches on the pattern `$Type[$TypeParams]; [$ImplParams]` where in our case:
    //
    // $Type:       one of `BitVec`, `BitSlice`, or `BitArray`
    // $TypeParams: some combo of `Word`, `'a, Word`, and `N, Word`.
    // $ImplParams: some combo of `Word: Unsigned`, `'a, Word:Unsigned`, and `const N: usize, Word: Unsigned, const WORDS: usize.
    //
    // The trait implementations follow and are all straightforward …
    (@impl $Type:ident[$($TypeParams:tt)*]; [$($ImplParams:tt)*]) => {

// --------------------------------------------------------------------------------------------------------------------
// The `Index` trait implementation.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("`std::ops::Index<usize>` for a [`", stringify!($Type), "`].")]
///
/// Returns the value of element `i` in the type.
///
/// # Panics
/// In debug mode, panics if the passed index is out of bounds.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v: gf2::BitVec = gf2::BitVec::ones(37);
/// assert_eq!(v[10], true);
/// v.set(10, false);
/// assert_eq!(v[10], false);
/// ```
impl<$($ImplParams)*> Index<usize> for $Type<$($TypeParams)*> {
    type Output = bool;
    #[inline]
    fn index(&self, i: usize) -> &Self::Output {
        if self.get(i) { &true } else { &false }
    }
}

// --------------------------------------------------------------------------------------------------------------------
// The various Display-like trait implementations.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("Returns a binary string representation of the bits in a [`", stringify!($Type), "`].")]
///
/// The output is a string of `0` and `1` characters without any spaces, commas, or other formatting.
///
/// # Note
/// - The output is in *vector* order, with the least significant bit printed first on the left.
/// - If the `alternate` `#` flag is set, the output is prefixed with `0b`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v: gf2::BitVec = gf2::BitVec::new();
/// assert_eq!(format!("{v:b}"), "");
/// let v: gf2::BitVec = gf2::BitVec::ones(4);
/// assert_eq!(format!("{v:b}"), "1111");
/// assert_eq!(format!("{v:#b}"), "0b1111");
/// ```
impl<$($ImplParams)*> fmt::Display for $Type<$($TypeParams)*> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0b{}", self.to_binary_string())
        }
        else {
            write!(f, "{}", self.to_binary_string())
        }
    }
}

#[doc = concat!("Returns a binary string representation of the bits in a [`", stringify!($Type), "`].")]
///
/// The output is a string of `0` and `1` characters without any spaces, commas, or other formatting.
///
/// # Note
/// - The output is in *vector* order, with the least significant bit printed first on the left.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v: gf2::BitVec = gf2::BitVec::ones(10);
/// assert_eq!(format!("{v:?}"), "1111111111");
/// ```
impl<$($ImplParams)*> fmt::Debug for $Type<$($TypeParams)*> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_binary_string())
    }
}

#[doc = concat!("Returns a binary string representation of the bits in a [`", stringify!($Type), "`].")]
///
/// The output is a string of `0` and `1` characters without any spaces, commas, or other formatting.
///
/// # Note
/// - The output is in *vector* order, with the least significant bit printed first on the left.
/// - If the `alternate` `#` flag is set, the output is prefixed with `0b`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v: gf2::BitVec = gf2::BitVec::new();
/// assert_eq!(format!("{v:b}"), "");
/// let v: gf2::BitVec = gf2::BitVec::ones(4);
/// assert_eq!(format!("{v:b}"), "1111");
/// assert_eq!(format!("{v:#b}"), "0b1111");
/// ```
impl<$($ImplParams)*> fmt::Binary for $Type<$($TypeParams)*> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0b{}", self.to_binary_string())
        }
        else {
            write!(f, "{}", self.to_binary_string())
        }
    }
}

#[doc = concat!("Returns an upper hex string representation of the bits in a [`", stringify!($Type), "`].")]
///
/// The output is a string of upper case hex characters without any spaces, commas, or other formatting.
///
/// The string may have a two character *suffix* of the form ".base" where `base` is one of 2, 4 or 8.
///
/// All hex characters encode 4 bits: "0X0" -> `0b0000`, "0X1" -> `0b0001`, ..., "0XF" -> `0b1111`.
/// The three possible ".base" suffixes allow for bit-vectors whose length is not a multiple of 4. <br>
/// Empty bit-vectors are represented as the empty string.
///
/// - `0X1`   is the hex representation of the bit-vector `0001` => length 4.
/// - `0X1.8` is the hex representation of the bit-vector `001`  => length 3.
/// - `0X1.4` is the hex representation of the bit-vector `01`   => length 2.
/// - `0X1.2` is the hex representation of the bit-vector `1`    => length 1.
///
/// # Note
/// - The output is in *vector-order* with the least significant bits printed first on the left.
/// - If the `alternate` `#` flag is set, the output is prefixed with `0X`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v: gf2::BitVec = gf2::BitVec::new();
/// assert_eq!(format!("{v:X}"), "");
/// let v: gf2::BitVec = gf2::BitVec::ones(4);
/// assert_eq!(format!("{v:X}"), "F");
/// assert_eq!(format!("{v:#X}"), "0XF");
/// ```
impl<$($ImplParams)*> fmt::UpperHex for $Type<$($TypeParams)*> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0X{}", self.to_hex_string())
        }
        else {
            write!(f, "{}", self.to_hex_string())
        }
    }
}

#[doc = concat!("Returns a lower hex string representation of the bits in a [`", stringify!($Type), "`].")]
///
/// The output is a string of lower case hex characters without any spaces, commas, or other formatting.
///
/// The string may have a two character *suffix* of the form ".base" where `base` is one of 2, 4 or 8.
/// All hex characters encode 4 bits: "0x0" -> `0b0000`, "0x1" -> `0b0001`, ..., "0xf" -> `0b1111`.
/// The three possible ".base" suffixes allow for bit-vectors whose length is not a multiple of 4. <br>
/// Empty bit-vectors are represented as the empty string.
///
/// - `0x1`   is the hex representation of the bit-vector `0001` => length 4.
/// - `0x1.8` is the hex representation of the bit-vector `001`  => length 3.
/// - `0x1.4` is the hex representation of the bit-vector `01`   => length 2.
/// - `0x1.2` is the hex representation of the bit-vector `1`    => length 1.
///
/// # Note
/// - The output is in *vector-order* with the least significant bits printed first on the left.
/// - If the `alternate` `#` flag is set, the output is prefixed with `0x`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v: gf2::BitVec = gf2::BitVec::new();
/// assert_eq!(format!("{v:x}"), "");
/// let v: gf2::BitVec = gf2::BitVec::ones(4);
/// assert_eq!(format!("{v:x}"), "f");
/// assert_eq!(format!("{v:#x}"), "0xf");
/// ```
impl<$($ImplParams)*> fmt::LowerHex for $Type<$($TypeParams)*> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x{}", self.to_hex_string().to_lowercase())
        }
        else {
            write!(f, "{}", self.to_hex_string().to_lowercase())
        }
    }
}

// --------------------------------------------------------------------------------------------------------------------
// The `ShlAssign`, and `ShrAssign` trait implementations for _mutable_ bit-store types.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("Shifts a [`", stringify!($Type), "`] left by a given number of bits *in-place*.")]
///
/// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v <<= 1` is `[v1,v2,v3,0]` with zeros added to the right.
///
/// # Note
/// - Left shifting in vector-order is the same as right shifting in bit-order.
/// - Only accessible bits are affected by the shift.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut bv: gf2::BitVec = gf2::BitVec::ones(10);
/// bv <<= 3;
/// assert_eq!(bv.to_string(), "1111111000");
/// ```
impl<$($ImplParams)*> ShlAssign<usize> for $Type<$($TypeParams)*> {
    #[inline] fn shl_assign(&mut self, shift: usize) { self.left_shift(shift); }
}

#[doc = concat!("Shifts a [`", stringify!($Type), "`] right by a given number of bits *in-place*.")]
///
/// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v >>= 1` is `[0,v0,v1,v2]` with zeros added to the left.
///
/// # Note
/// - Right shifting in vector-order is the same as left shifting in bit-order.
/// - Only accessible bits are affected by the shift.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut bv: gf2::BitVec = gf2::BitVec::ones(10);
/// bv >>= 3;
/// assert_eq!(bv.to_string(), "0001111111");
/// ```
impl<$($ImplParams)*> ShrAssign<usize> for $Type<$($TypeParams)*> {
    #[inline] fn shr_assign(&mut self, shift: usize) { self.right_shift(shift); }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Not`, `Shl`, and `Shr` trait implementations for bit-store types and *reference*s to bit-store types.
//
// For example, if `v` is a bit-store type, then `u = v << 1` is a new bit-vector and `v` is consumed by the call.
// On the other hand, `u = &v << 1` is a new bit-vector but `v` is not consumed by the call.
//
// These methods all are implemented in terms of the `<<=` and `>>=` operators.
//
// The only trick is code that handles whether or not the left-hand side is consumed by the call.
//
// Types which are consumed are handled by code like:
// ```
// let mut result: BitVec<Word> = self.into();  <1>
// result <<= shift;                            <2>
// result
// ```
// <1> If self is a slice, then self.into() converts the slice to a bit-vector (there is necessary cost to the call).
//     If self is a bit-vector, then self.into() s a trivial no-op returning self at no cost.
// <2> Now that we have a bit-vector, we can shift it as required.
//
// References which are not consumed are handled by code like:
// ```
// let mut result: BitVec<Word> = self.clone().into();  <1>
// result <<= shift;                                    <2>
// result
// ```
// <1> If self is a slice, then self.clone() is cheap and the into() does the heavy lifting to get a bit-vector.
//     If self is a bit-vector, the heavy lifting is done by the clone() call and the into() is no cost.
// <2> Now that we have a bit-vector, we can shift it as required.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("Shifts a [`", stringify!($Type), "`] *reference* left, returning a new bit-vector. Leaves the left-hand side unchanged.")]
///
/// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `&v << 1 = [v1,v2,v3,0]`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v1: gf2::BitVec = gf2::BitVec::ones(10);
/// let v2  = &v1 << 3;
/// assert_eq!(v1.to_string(), "1111111111");
/// assert_eq!(v2.to_string(), "1111111000");
/// let s2 = v2.slice(0..7);
/// assert_eq!(s2.to_string(), "1111111");
/// let v3 = &s2 << 3;
/// assert_eq!(v3.to_string(), "1111000");
/// assert_eq!(s2.to_string(), "1111111");
/// ```
impl<$($ImplParams)*> Shl<usize> for &$Type<$($TypeParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn shl(self, shift: usize) -> Self::Output { self.left_shifted(shift) }
}

#[doc = concat!("Shifts a [`", stringify!($Type), "`] left, returning a new bit-vector. Consumes the left-hand side.")]
///
/// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v << 1 = [v1,v2,v3,0]`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v1: gf2::BitVec = gf2::BitVec::ones(10);
/// let v2  = v1 << 3;
/// assert_eq!(v2.to_string(), "1111111000");
/// let s2 = v2.slice(0..7);
/// assert_eq!(s2.to_string(), "1111111");
/// let v3 = s2 << 3;
/// assert_eq!(v3.to_string(), "1111000");
/// ```
impl<$($ImplParams)*> Shl<usize> for $Type<$($TypeParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn shl(self, shift: usize) -> Self::Output { self.left_shifted(shift) }
}


#[doc = concat!("Shifts a [`", stringify!($Type), "`] *reference* right, returning a new bit-vector. Leaves the left-hand side unchanged.")]
///
/// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `&v >> 1 = [0,v0,v1,v2]` with zeros added to the left.
///
/// # Note
/// - Right shifting in vector-order is the same as left shifting in bit-order.
/// - Only accessible bits in the argument are affected by the shift.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v1: gf2::BitVec = gf2::BitVec::ones(10);
/// let v2  = &v1 >> 3;
/// assert_eq!(v1.to_string(), "1111111111");
/// assert_eq!(v2.to_string(), "0001111111");
/// let s2 = v2.slice(0..7);
/// assert_eq!(s2.to_string(), "0001111");
/// let v3 = &s2 >> 3;
/// assert_eq!(v3.to_string(), "0000001");
/// assert_eq!(s2.to_string(), "0001111");
/// ```
impl<$($ImplParams)*> Shr<usize> for &$Type<$($TypeParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn shr(self, shift: usize) -> Self::Output { self.right_shifted(shift) }
}

#[doc = concat!("Shifts a [`", stringify!($Type), "`] right, returning a new bit-vector. Consumes the left-hand side.")]
///
/// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v >> 1 = [0,v0,v1,v2]` with zeros added to the left.
///
/// # Note
/// - Right shifting in vector-order is the same as left shifting in bit-order.
/// - Only accessible bits in the argument are affected by the shift.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v1: gf2::BitVec = gf2::BitVec::ones(10);
/// let v2  = v1 >> 3;
/// assert_eq!(v2.to_string(), "0001111111");
/// let s2 = v2.slice(0..7);
/// assert_eq!(s2.to_string(), "0001111");
/// let v3 = s2 >> 3;
/// assert_eq!(v3.to_string(), "0000001");
/// ```
impl<$($ImplParams)*> Shr<usize> for $Type<$($TypeParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn shr(self, shift: usize) -> Self::Output { self.right_shifted(shift) }
}

#[doc = concat!("Flips all the bits in a [`", stringify!($Type), "`] *reference*, returning a new bit-vector. Leaves the left-hand side unchanged.")]
///
/// # Examples
/// ```
/// use gf2::*;
/// let u: gf2::BitVec = gf2::BitVec::ones(3);
/// let v = !&u;
/// assert_eq!(u.to_binary_string(), "111");
/// assert_eq!(v.to_binary_string(), "000");
/// ```
impl<$($ImplParams)*> Not for &$Type<$($TypeParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn not(self) -> Self::Output { self.flipped() }
}

#[doc = concat!("Flips all the bits in a [`", stringify!($Type), "`], returning a new bit-vector. Consumes the left-hand side.")]
///
/// # Examples
/// ```
/// use gf2::*;
/// let u: gf2::BitVec = gf2::BitVec::ones(3);
/// let v = !u;
/// assert_eq!(v.to_binary_string(), "000");
/// ```
impl<$($ImplParams)*> Not for $Type<$($TypeParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn not(self) -> Self::Output { self.flipped() }
}

};} // End of the `impl_unary_traits!` macro.

// ====================================================================================================================
// `impl_binary_traits` implements the following foreign traits for pairs of bit-store types:
//
// - `BitXorAssign`
// - `BitAndAssign`
// - `BitOrAssign`
// - `AddAssign`
// - `SubAssign`
// - `MulAssign`
// - `BitXor`
// - `BitAnd`
// - `BitOr`
// - `Add`
// - `Sub`
// - `Mul`
//
// In each case, the two bit-stores must have the same underlying `Word` type and the same length.
// ====================================================================================================================
macro_rules! impl_binary_traits {

    // The `BitVec-BitVec` case which has just the one generic parameter: `Word: Unsigned`.
    (BitVec, BitVec) => {
        impl_binary_traits!(@impl BitVec[Word]; BitVec[Word]; [Word: Unsigned]);
    };

    // The `BitVec-BitSlice` case which adds a lifetime parameter for the `BitSlice`.
    (BitVec, BitSlice) => {
        impl_binary_traits!(@impl BitVec[Word]; BitSlice['a, Word]; ['a, Word: Unsigned]);
    };

    // The `BitVec-BitArray` case which adds const generic parameters for the `BitArray`.
    (BitVec, BitArray) => {
        impl_binary_traits!(@impl BitVec[Word]; BitArray[N, Word, WORDS]; [const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The `BitSlice-BitVec` case which has two generic parameters: `'a` and `Word: Unsigned`.
    (BitSlice, BitVec) => {
        impl_binary_traits!(@impl BitSlice['a, Word]; BitVec[Word]; ['a, Word: Unsigned]);
    };

    // The `BitSlice-BitSlice` case which adds a second lifetime parameter for the rhs `BitSlice`.
    (BitSlice, BitSlice) => {
        impl_binary_traits!(@impl BitSlice['a, Word]; BitSlice['b, Word]; ['a, 'b, Word: Unsigned]);
    };

    // The `BitSlice-BitArray` case which adds const generic parameters for the `BitArray`.
    (BitSlice, BitArray) => {
        impl_binary_traits!(@impl BitSlice['a, Word]; BitArray[N, Word, WORDS]; ['a, const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The `BitArray-BitVec` case which has the generic parameters for the `BitArray` as the `Word` type is shared.
    (BitArray, BitVec) => {
        impl_binary_traits!(@impl BitArray[N, Word, WORDS]; BitVec[Word]; [const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The `BitArray-BitSlice` case which adds a lifetime parameter for the `BitSlice`.
    (BitArray, BitSlice) => {
        impl_binary_traits!(@impl BitArray[N, Word, WORDS]; BitSlice['a, Word]; ['a, const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The `BitArray-BitArray` case which adds no more generic parameters as both sides already must have the same ones!
    (BitArray, BitArray) => {
        impl_binary_traits!(@impl BitArray[N, Word, WORDS]; BitArray[N, Word, WORDS]; [const N: usize, Word: Unsigned, const WORDS: usize]);
    };

    // The other arms funnel to this one which does the actual work of implementing the various foreign traits:
    // This matches on the pattern `$Type[$TypeParams]; [$ImplParams]` where in our case:
    //
    // $LhsType:    one of `BitVec`, `BitSlice`, or `BitArray`
    // $LhsParams:  one of `Word`, `'a, Word`, or `N, Word, WORDS`.
    // $RhsType:    one of `BitVec`, `BitSlice`, or `BitArray`
    // $RhsParams:  one of `Word`, `a, Word`, `'b, Word`, or `N, Word, WORDS`.
    // $ImplParams: some combo of `Word: Unsigned`, `'a`, `b`, `const N: usize, const WORDS: usize.
    //
    // The trait implementations follow and are all straightforward …
    (@impl $Lhs:ident[$($LhsParams:tt)*]; $Rhs:ident[$($RhsParams:tt)*]; [$($ImplParams:tt)*]) => {

// --------------------------------------------------------------------------------------------------------------------
// Implementations of the BitXorAssign, BitAndAssign, BitOrAssign traits for pairs of bit-store types.
//
// We have implemented the traits where right-hand side may or may not be consumed by the call.
// For example if u and v are bit-store types, then for the pairwise XOR operator we have implemented:
//
// - u ^= &v leaves v untouched.
// - u ^= v  consumes v.
// --------------------------------------------------------------------------------------------------------------------
#[doc = concat!("In-place XOR's a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`] *reference*.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// let v2: gf2::BitVec = gf2::BitVec::from_string("0101010101").unwrap();
/// v1 ^= &v2;
/// assert_eq!(v1.to_string(), "1111111111");
/// assert_eq!(v2.to_string(), "0101010101");
/// ```
impl<$($ImplParams)*> BitXorAssign<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn bitxor_assign(&mut self, rhs: &$Rhs<$($RhsParams)*>) { self.xor_eq(rhs); }
}

#[doc = concat!("In-place XOR's a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`].")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// v1 ^= gf2::BitVec::from_string("0101010101").unwrap();
/// assert_eq!(v1.to_string(), "1111111111");
/// ```
impl<$($ImplParams)*> BitXorAssign<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn bitxor_assign(&mut self, rhs: $Rhs<$($RhsParams)*>) { self.xor_eq(&rhs); }
}

#[doc = concat!("In-place AND's a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`] *reference*.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// let v2: gf2::BitVec = gf2::BitVec::from_string("0101010101").unwrap();
/// v1 &= &v2;
/// assert_eq!(v1.to_string(), "0000000000");
/// assert_eq!(v2.to_string(), "0101010101");
/// ```
impl<$($ImplParams)*> BitAndAssign<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn bitand_assign(&mut self, rhs: &$Rhs<$($RhsParams)*>) { self.and_eq(rhs); }
}

#[doc = concat!("In-place AND's a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`].")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// v1 &= gf2::BitVec::from_string("0101010101").unwrap();
/// assert_eq!(v1.to_string(), "0000000000");
/// ```
impl<$($ImplParams)*> BitAndAssign<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn bitand_assign(&mut self, rhs: $Rhs<$($RhsParams)*>) { self.and_eq(&rhs); }
}

#[doc = concat!("In-place OR's a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`] *reference*.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// let v2: gf2::BitVec = gf2::BitVec::from_string("0101010101").unwrap();
/// v1 |= &v2;
/// assert_eq!(v1.to_string(), "1111111111");
/// assert_eq!(v2.to_string(), "0101010101");
/// ```
impl<$($ImplParams)*> BitOrAssign<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn bitor_assign(&mut self, rhs: &$Rhs<$($RhsParams)*>) { self.or_eq(rhs); }
}

#[doc = concat!("In-place OR's a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`].")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// v1 |= gf2::BitVec::from_string("0101010101").unwrap();
/// assert_eq!(v1.to_string(), "1111111111");
/// ```
impl<$($ImplParams)*> BitOrAssign<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn bitor_assign(&mut self, rhs: $Rhs<$($RhsParams)*>) { self.or_eq(&rhs); }
}

// --------------------------------------------------------------------------------------------------------------------
// Implementations of the AddAssign, SubAssign traits for pairs of bit-store types.
//
// We have implemented the traits where right-hand side may or may not be consumed by the call.
// For example if u and v are bit-store types, then for the pairwise `+=` operator we have implemented:
//
// - u += &v leaves v untouched.
// - u += v  consumes v.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("In-place addition of a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`] *reference*.")]
///
/// # Note
/// Addition in GF(2) is the same as the XOR operation.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// let v2: gf2::BitVec = gf2::BitVec::from_string("0101010101").unwrap();
/// v1 += &v2;
/// assert_eq!(v1.to_string(), "1111111111");
/// assert_eq!(v2.to_string(), "0101010101");
/// ```
impl<$($ImplParams)*> AddAssign<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn add_assign(&mut self, rhs: &$Rhs<$($RhsParams)*>) { self.xor_eq(rhs); }
}

#[doc = concat!("In-place addition of a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`].")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Note
/// Addition in GF(2) is the same as the XOR operation.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// v1 += gf2::BitVec::from_string("0101010101").unwrap();
/// assert_eq!(v1.to_string(), "1111111111");
/// ```
impl<$($ImplParams)*> AddAssign<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn add_assign(&mut self, rhs: $Rhs<$($RhsParams)*>) { self.xor_eq(&rhs); }
}

#[doc = concat!("In-place subtraction of a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`] *reference*.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Note
/// Subtraction in GF(2) is the same as addition which is the same as the XOR operation.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// let v2: gf2::BitVec = gf2::BitVec::from_string("0101010101").unwrap();
/// v1 -= &v2;
/// assert_eq!(v1.to_string(), "1111111111");
/// assert_eq!(v2.to_string(), "0101010101");
/// ```
impl<$($ImplParams)*> SubAssign<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn sub_assign(&mut self, rhs: &$Rhs<$($RhsParams)*>) { self.xor_eq(rhs); }
}

#[doc = concat!("In-place subtraction of a [`", stringify!($Lhs), "`] with a [`", stringify!($Rhs), "`].")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
///
/// # Note
/// Subtraction in GF(2) is the same as addition which is the same as the XOR operation.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v1: gf2::BitVec = gf2::BitVec::from_string("1010101010").unwrap();
/// v1 -= gf2::BitVec::from_string("0101010101").unwrap();
/// assert_eq!(v1.to_string(), "1111111111");
/// ```
impl<$($ImplParams)*> SubAssign<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    #[inline] fn sub_assign(&mut self, rhs: $Rhs<$($RhsParams)*>) { self.xor_eq(&rhs); }
}

// --------------------------------------------------------------------------------------------------------------------
// Implementations of the BitXor, BitAnd, BitOr traits for pairs of vector-like types.
//
// We have implemented the traits for all four combinations of bit-store types and *reference*s to bit-store types.
// For example if u and v are bit-store types, then for the pairwise XOR operator we have implemented:
//
// - &u ^ &v leaves u and v untouched
// - &u ^ v  leaves u untouched, consumes v
// - u ^ &v  leaves v untouched, consumes u
// - u ^ v   consumes both u and v
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("XOR's a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitXor<&$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitxor(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.xor(rhs) }
}

#[doc = concat!("XOR's a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitXor<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitxor(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.xor(rhs) }
}

#[doc = concat!("XOR's a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitXor<$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitxor(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.xor(&rhs) }
}

#[doc = concat!("XOR's a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitXor<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitxor(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.xor(&rhs) }
}

#[doc = concat!("AND's a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitAnd<&$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitand(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.and(rhs) }
}

#[doc = concat!("AND's a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitAnd<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitand(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.and(rhs) }
}

#[doc = concat!("AND's a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitAnd<$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitand(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.and(&rhs) }
}

#[doc = concat!("AND's a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitAnd<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitand(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.and(&rhs) }
}

#[doc = concat!("OR's a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitOr<&$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitor(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.or(rhs) }
}

#[doc = concat!("OR's a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitOr<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitor(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.or(rhs) }
}

#[doc = concat!("OR's a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitOr<$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitor(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.or(&rhs) }
}

#[doc = concat!("OR's a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> BitOr<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn bitor(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.or(&rhs) }
}

// --------------------------------------------------------------------------------------------------------------------
// Implementations of the Add & Sub traits for pairs of bit-store types.
//
// These are implement for all four combinations of bit-store types and *reference*s to bit-store types.
// For example, for Add:
//
// - &u + &v leaving u and v untouched
// - &u + v  leaving u untouched, but v is consumed by the call
// - u + &v  leaving v untouched, but u is consumed by the call
// - u + v   both u and v are consumed by the call
//
// In GF(2), addition and subtraction are the same as bitwise XOR.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("Adds a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// In GF(2), addition is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Add<&$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn add(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.xor(rhs) }
}

#[doc = concat!("Adds a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// In GF(2), addition is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Add<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn add(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.xor(rhs) }
}

#[doc = concat!("Adds a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// In GF(2), addition is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Add<$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn add(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.xor(&rhs) }
}

#[doc = concat!("Adds a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// In GF(2), addition is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Add<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn add(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.xor(&rhs) }
}

#[doc = concat!("Subtracts a [`", stringify!(&$Lhs), "`] *reference* and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// In GF(2), subtraction is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Sub<&$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn sub(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.xor(rhs) }
}

#[doc = concat!("Subtracts a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`] *reference*, returning a new bit-vector.")]
///
/// In GF(2), subtraction is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Sub<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn sub(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.xor(rhs) }
}

#[doc = concat!("Subtracts a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// In GF(2), subtraction is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Sub<$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn sub(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.xor(&rhs) }
}

#[doc = concat!("Subtracts a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`], returning a new bit-vector.")]
///
/// In GF(2), subtraction is the same as bitwise XOR.
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Sub<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = BitVec<Word>;
    #[inline] fn sub(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.xor(&rhs) }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Mul` trait implementation for pairs of bit-store types -- we use `*` to denote the dot product.
// --------------------------------------------------------------------------------------------------------------------

#[doc = concat!("The dot product a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`] *reference*, returning a `bool`.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Mul<&$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = bool;
    #[inline] fn mul(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.dot(rhs) }
}

#[doc = concat!("The dot product a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`] *reference*, returning a `bool`.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Mul<&$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = bool;
    #[inline] fn mul(self, rhs: &$Rhs<$($RhsParams)*>) -> Self::Output { self.dot(rhs) }
}

#[doc = concat!("The dot product a [`", stringify!($Lhs), "`] *reference* and a [`", stringify!($Rhs), "`], returning a `bool`.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Mul<$Rhs<$($RhsParams)*>> for &$Lhs<$($LhsParams)*> {
    type Output = bool;
    #[inline] fn mul(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.dot(&rhs) }
}

#[doc = concat!("The dot product a [`", stringify!($Lhs), "`] and a [`", stringify!($Rhs), "`], returning a `bool`.")]
///
/// # Panics
/// This method panics if the lengths of the input operands do not match.
impl<$($ImplParams)*> Mul<$Rhs<$($RhsParams)*>> for $Lhs<$($LhsParams)*> {
    type Output = bool;
    #[inline] fn mul(self, rhs: $Rhs<$($RhsParams)*>) -> Self::Output { self.dot(&rhs) }
}

};} // End of the `impl_binary_traits` macro.

// ====================================================================================================================
// Invoke the `impl_unary_traits` macro to implement common foreign traits for individual concrete bit-store types.
// ====================================================================================================================

// Implement for BitVec, BitSlice, and BitArray.
impl_unary_traits!(BitVec);
impl_unary_traits!(BitSlice);
#[cfg(feature = "unstable")]
impl_unary_traits!(BitArray);

// ====================================================================================================================
// Invoke the `impl_binary_traits` macro to implement common foreign traits for pairs of concrete bit-store types.
// ====================================================================================================================

// BitVec with other bit-store types.
impl_binary_traits!(BitVec, BitVec);
impl_binary_traits!(BitVec, BitSlice);
#[cfg(feature = "unstable")]
impl_binary_traits!(BitVec, BitArray);

// BitSlice with other bit-store types.
impl_binary_traits!(BitSlice, BitVec);
impl_binary_traits!(BitSlice, BitSlice);
#[cfg(feature = "unstable")]
impl_binary_traits!(BitSlice, BitArray);

// BitArray with other bit-store types.
#[cfg(feature = "unstable")]
impl_binary_traits!(BitArray, BitVec);
#[cfg(feature = "unstable")]
impl_binary_traits!(BitArray, BitSlice);
#[cfg(feature = "unstable")]
impl_binary_traits!(BitArray, BitArray);
