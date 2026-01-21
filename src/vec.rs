//! [`BitVec`] is a  _dynamically sized_ vector over GF(2) --- a _bit-vector_.

use crate::{
    BitSlice,
    BitStore,
    Unsigned,
};

#[doc = include_str!("../docs/vec.md")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BitVec<Word: Unsigned = usize> {
    // The number of bits in the bit-vector.
    m_len: usize,

    // The underlying store of `Unsigned` words that are used to store the bits.
    m_store: Vec<Word>,
}

/// Implement the `BitStore` trait for `BitVec`.
impl<Word: Unsigned> BitStore<Word> for BitVec<Word> {
    /// Returns the number of *bits* in the `BitVec`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u8> = BitVec::zeros(10);
    /// assert_eq!(v.len(), 10);
    /// ```
    fn len(&self) -> usize { self.m_len }

    /// Returns a pointer to the real words underlying the `BitVec` as an [`Unsigned`] slice.
    fn store(&self) -> &[Word] { self.m_store.as_slice() }

    /// Returns a pointer to the real words underlying the `BitVec` as an [`Unsigned`] mutable slice.
    fn store_mut(&mut self) -> &mut [Word] { self.m_store.as_mut_slice() }

    /// Returns the offset (in bits) of the first bit element in the `BitVec` within the first [`Unsigned`] word.
    /// This is always zero for a `BitVec`.
    fn offset(&self) -> u32 { 0 }

    /// Returns the least number of [`Unsigned`] words needed to store the bits in the `BitVec`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u8> = BitVec::zeros(6);
    /// assert_eq!(v.words(), 1);
    /// let v: BitVec<u8> = BitVec::zeros(10);
    /// assert_eq!(v.words(), 2);
    /// ```
    #[inline]
    fn words(&self) -> usize {
        // For a bit-vector the number of words is the same as the number of words in the underlying store.
        self.m_store.len()
    }

    /// Returns the [`Unsigned`] word at index `i` from the `BitVec`'s underlying store of words.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the `BitVec`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u8> = BitVec::zeros(10);
    /// assert_eq!(v.word(0), 0);
    /// ```
    #[inline]
    fn word(&self, i: usize) -> Word {
        debug_assert!(i < self.words(), "Index {i} should be less than {}", self.words());
        self.m_store[i]
    }

    /// Sets the [`Unsigned`] word at index `i` in the `BitVec` to `word`.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the `BitVec`.
    ///
    /// # Note
    /// It is careful to only set the bits that are within the vector (the last word may only be partially occupied).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::zeros(12);
    /// v.set_word(0, 0b1111_1111);
    /// v.set_word(1, 0b1111_1111);
    /// assert_eq!(v.to_string(), "111111111111");
    /// assert_eq!(v.count_ones(), 12);
    /// ```
    #[inline]
    fn set_word(&mut self, i: usize, word: Word) {
        debug_assert!(i < self.words(), "Index {i} should be less than {}", self.words());

        // If the word is not the last word, just set it.
        if i < self.words() - 1 {
            self.m_store[i] = word;
        }
        else {
            // The last word may only be partially occupied & we only set the bits that are within the vector.
            let last_bit = self.m_len - 1;
            #[allow(clippy::cast_possible_truncation)]
            let last_offset = (last_bit % Word::UBITS) as u32;
            self.m_store[i].replace_bits(0..=last_offset, word);
        }
    }
}

/// Constructors for bit-vectors.
impl<Word: Unsigned> BitVec<Word> {
    /// The default constructor creates an empty bit-vector.
    ///
    /// No capacity is reserved until elements are added.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::new();
    /// assert_eq!(v.len(), 0);
    /// assert_eq!(v.capacity(), 0);
    /// ```
    #[must_use]
    #[inline]
    pub fn new() -> Self { Self { m_len: 0, m_store: Vec::new() } }

    /// Constructs a bit-vector with `len` elements by repeatedly copying the bits from a single `Word` instance.
    ///
    /// You specify the length `len` of the bit-vector which means the final copy of `word` may be truncated and padded
    /// with zeros (unused word slots are always set to zero in this crate).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u8> = BitVec::from_word(0b01010101, 10);
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_word(word: Word, len: usize) -> Self {
        let underlying = vec![word; Word::words_needed(len)];
        let mut result = Self { m_len: len, m_store: underlying };
        result.clean();
        result
    }

    /// Constructs a new empty bit-vector with *at least* the specified capacity.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::with_capacity(10);
    /// assert_eq!(v.len(), 0);
    /// assert!(v.capacity() >= 10);
    /// ```
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { m_len: 0, m_store: Vec::with_capacity(Word::words_needed(capacity)) }
    }

    /// Constructs a bit-vector with `len` elements, all initialized to `0`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::zeros(10);
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.count_zeros(), 10);
    /// ```
    #[must_use]
    #[inline]
    pub fn zeros(len: usize) -> Self {
        let underlying = vec![Word::ZERO; Word::words_needed(len)];
        Self { m_len: len, m_store: underlying }
    }

    /// Constructs a bit-vector with `len` elements, all initialized to `1`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::ones(10);
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.count_ones(), 10);
    #[must_use]
    #[inline]
    pub fn ones(len: usize) -> Self {
        let underlying = vec![Word::MAX; Word::words_needed(len)];
        let mut result = Self { m_len: len, m_store: underlying };
        result.clean();
        result
    }

    /// Returns a bit-vector with `len` elements, all initialized to the boolean value `val`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::constant(true, 10);
    /// assert_eq!(v.to_string(), "1111111111");
    /// let v: BitVec = BitVec::constant(false, 10);
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    #[must_use]
    pub fn constant(val: bool, len: usize) -> Self {
        let underlying = vec![if val { Word::MAX } else { Word::ZERO }; Word::words_needed(len)];
        let mut result = Self { m_len: len, m_store: underlying };
        result.clean();
        result
    }

    /// Constructs the *unit* bit-vector where only element `i` of `len` elements is set to 1.
    ///
    /// # Panics
    /// Panics if `i` is greater than or equal to `len`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::unit(5, 10);
    /// assert_eq!(v.to_string(), "0000010000");
    /// ```
    #[must_use]
    #[inline]
    pub fn unit(i: usize, len: usize) -> Self {
        assert!(i < len, "Index {i} must be less than the length of the BitVec {len}!");
        let mut result = Self::zeros(len);
        result.set(i, true);
        result
    }

    /// Constructs a bit-vector with `len` elements, where the bits alternate between `1` and `0`.
    ///
    /// The pattern starts with a `1` so e.g. `1010101010101`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::alternating(10);
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    #[must_use]
    #[inline]
    pub fn alternating(len: usize) -> Self {
        let underlying = vec![Word::ALTERNATING; Word::words_needed(len)];
        let mut result = Self { m_len: len, m_store: underlying };
        result.clean();
        result
    }

    /// Constructs a bit-vector by copying all the bits from a single `Unsigned` instance.
    /// The length of the bit-vector will be equal to the number of bits in `Src`.
    /// The `Src` type must be an `Unsigned` but can be different from the `Word` type.
    ///
    /// # Note
    /// You probably think of the bits in a word as printing with the least significant bit on the right: ...b2,b1,b0.
    /// However, bit-vectors are first and foremost, *vectors*, so the zero bit prints on the left: v0,v1,v2,...
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u8> = BitVec::from_unsigned(0b01010101_u8);
    /// assert_eq!(v.len(), 8);
    /// assert_eq!(v.to_string(), "10101010");
    /// let v: BitVec<u8> = BitVec::from_unsigned(0b0101010101010101_u16);
    /// assert_eq!(v.len(), 16);
    /// assert_eq!(v.to_string(), "1010101010101010");
    /// let v: BitVec<u32> = BitVec::from_unsigned(0b01010101_u8);
    /// assert_eq!(v.len(), 8);
    /// assert_eq!(v.to_string(), "10101010");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_unsigned<Src>(src: Src) -> Self
    where Src: Unsigned + TryInto<Word> {
        let mut result = Self::zeros(Src::UBITS);
        result.copy_unsigned(src);
        result
    }

    /// Construct a bit-vector by *copying* the bits from any bit-store.
    ///
    /// This is one of the few methods in the library that _doesn't_ require the two stores to have the same underlying
    /// `Unsigned` word type for their storage -- i.e., the `Word` type for `self` may differ from the `SrcWord` type
    /// for the bit-store `src`. You can use it to convert between different `Word` type stores (e.g., from
    /// `BitVec<u32>` to `BitVec<u8>`) as long as the sizes match.
    ///
    /// # Note
    /// We also have implemented the [`From`] trait for [`BitVec`] from any bit-store type that forwards to this method.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let src: BitVec = BitVec::from_string("010101010101010101010101010101010101010101010101010101010101").unwrap();
    /// let mut dst: BitVec = BitVec::from_store(&src);
    /// assert_eq!(dst.to_string(), src.to_string());
    /// let src: BitVec<u8> = BitVec::from_string("1011001110001111").unwrap();
    /// let mut dst: BitVec<u32> = BitVec::from_store(&src);
    /// assert_eq!(dst.to_string(), src.to_string());
    /// let src: BitVec<u16> = BitVec::from_string("101100111000111110110011100011111011001110001111").unwrap();
    /// let mut dst: BitVec<u8> = BitVec::from_store(&src);
    /// assert_eq!(dst.to_string(), src.to_string());
    /// let v1: BitVec = BitVec::ones(10);
    /// let slice = v1.slice(0..4);
    /// let v2: BitVec = BitVec::from_store(&slice);
    /// assert_eq!(v2.to_string(), "1111");
    /// let v3: BitVec = slice.into();
    /// assert_eq!(v3.to_string(), "1111");
    /// ```
    #[must_use]
    #[inline]
    pub fn from_store<SrcWord, SrcStore>(src: &SrcStore) -> Self
    where
        SrcWord: Unsigned,
        SrcStore: BitStore<SrcWord>,
    {
        let mut result = Self::zeros(src.len());
        result.copy_store(src);
        result
    }

    /// Constructs a bit-vector with `len` elements by calling a function `f` for each bit index.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::from_fn(10, |i| i % 2 == 0);
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    #[must_use]
    pub fn from_fn(len: usize, f: impl Fn(usize) -> bool) -> Self {
        let mut result = Self::zeros(len);
        result.copy_fn(f);
        result
    }
}

/// Constructors that set the elements of a bit-vector randomly.
impl<Word: Unsigned> BitVec<Word> {
    /// Constructs a random bit-vector with `len` elements where each bit is set/unset with probability 50/50.
    ///
    /// The random number generator is seeded on first use with a scrambled version of the current time so you get
    /// different outputs for each run.
    ///
    /// See the `random_seeded` method for a way to get reproducible randomly filled bit-vectors.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::random(10);
    /// assert_eq!(v.len(), 10);
    /// ```
    #[must_use]
    pub fn random(len: usize) -> Self {
        let mut result = Self::zeros(len);
        result.fill_random();
        result
    }

    /// Constructs a random bit-vector with `len` elements where each bit is set/unset with probability 50/50.
    ///
    /// For reproducibility, the random number generator used here is seeded with the specified `seed`.
    ///
    /// # Note
    /// - The generator is reset to the previous seed after the bit-vector is constructed.
    /// - A seed of `0` is taken to seed the generator from a scrambled version of the current time.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: BitVec = BitVec::random_seeded(1000, 42);
    /// let v2: BitVec = BitVec::random_seeded(1000, 42);
    /// assert_eq!(v1, v2);
    /// ```
    #[must_use]
    pub fn random_seeded(len: usize, seed: u64) -> Self {
        let mut result = Self::zeros(len);
        result.fill_random_seeded(seed);
        result
    }

    /// Constructs a random bit-vector with `len` elements where each bit is set with probability `p`.
    ///
    /// The random number generator is seeded on first use with a scrambled version of the current time.
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-vector with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::random_biased(10, 0.5);
    /// assert_eq!(v.len(), 10);
    /// ```
    #[must_use]
    pub fn random_biased(len: usize, p: f64) -> Self {
        let mut result = Self::zeros(len);
        result.fill_random_biased(p);
        result
    }

    /// Constructs a random bit-vector with `len` elements where each bit is set with probability `p` and the RNG is
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-vector with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::random_biased_seeded(10, 1.2, 42); // All bits set since p > 1
    /// assert_eq!(v.count_ones(), 10);
    /// let u: BitVec = BitVec::random_biased_seeded(100, 0.85, 42); // Using same seed for u and v.
    /// let v: BitVec = BitVec::random_biased_seeded(100, 0.85, 42);
    /// assert_eq!(u, v);
    /// ```
    #[must_use]
    pub fn random_biased_seeded(len: usize, p: f64, seed: u64) -> Self {
        let mut result = Self::zeros(len);
        result.fill_random_biased_seeded(p, seed);
        result
    }
}

/// Construct bit-vectors from strings. These constructors can fail.
impl<Word: Unsigned> BitVec<Word> {
    /// Tries to construct a bit-vector from any string `s`.
    ///
    /// `s` can contain whitespace, commas, and underscores and optionally a "0b", "0x", or "0X" prefix.
    ///
    /// If there is no prefix, and the string only contains '0' and '1' characters, we assume the string is binary.
    /// To force getting `s` interpreted as a hex string, add a prefix of "0x" or "0X".
    ///
    /// # Note
    /// A hex string can have a suffix of ".2", ".4", or ".8" to indicate the base of the last digit/character. <br>
    /// This allows for bit-vectors of any length as opposed to just a multiple of 4.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::from_string("0b1010_1010_10").unwrap();
    /// assert_eq!(v.to_string(), "1010101010");
    /// let v: BitVec = BitVec::from_string("AA").unwrap();
    /// assert_eq!(v.to_string(), "10101010");
    /// let v: BitVec = BitVec::from_string("10101010").unwrap();
    /// assert_eq!(v.to_string(), "10101010");
    /// let v: BitVec = BitVec::from_string("0x1.8").unwrap();
    /// assert_eq!(v.to_string(), "001");
    /// ```
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        // Edge case: completely empty string.
        if s.is_empty() {
            return Some(Self::new());
        }

        // Perhaps there is a "0b" prefix.
        if let Some(s) = s.strip_prefix("0b") {
            return Self::from_binary_string(s);
        }

        // Perhaps there is a "0x" or "0X" prefix.
        if let Some(s) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            return Self::from_hex_string(s);
        }

        // No prefix, but perhaps the string only contains '0' and '1' characters: Assume binary.
        if s.chars().all(|c| c == '0' || c == '1') {
            return Self::from_binary_string(s);
        }

        // Try hex.
        Self::from_hex_string(s)
    }

    /// Tries to construct a bit-vector from a "binary" string  `s` (zeros and ones).
    ///
    /// `s` can contain whitespace, commas, and underscores and optionally a "0b" prefix.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::from_binary_string("0b1010_1010_10").expect("Input is not binary!");
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    #[must_use]
    pub fn from_binary_string(s: &str) -> Option<Self> {
        // Edge case: completely empty string.
        if s.is_empty() {
            return Some(Self::new());
        }

        // Remove any "0b" prefix.
        let s = if let Some(s) = s.strip_prefix("0b") { s } else { s };

        // Remove any whitespace, commas, and underscores.
        let s = s.replace(|c: char| c.is_whitespace() || c == ',' || c == '_', "");

        // The string should now only contain '0' and '1' characters.
        if !s.chars().all(|c| c == '0' || c == '1') {
            return None;
        }

        // Construct the bit-vector.
        let mut result = Self::zeros(s.len());
        for (i, c) in s.chars().enumerate() {
            if c == '1' {
                result.set(i, true);
            }
        }
        Some(result)
    }

    /// Tries to construct a bit-vector from a hex string `s` (characters 0-9, A-F, a-f).
    ///
    /// `s` can contain whitespace, commas, and underscores and optionally a "0x" or "0X" prefix.
    ///
    /// # Note
    /// `s` can have a suffix of ".2", ".4", or ".8" to indicate the base of the last digit/character. <br>
    /// This allows for bit-vectors of any length as opposed to just a multiple of 4.
    ///
    /// # Panics
    /// Panics if there is a suffix that is not one of ".2", ".4", or ".8".
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::from_hex_string("0xAA").expect("Input is not interpretable as hex!");
    /// assert_eq!(v.to_string(), "10101010");
    /// let v: BitVec = BitVec::from_hex_string("0x1").expect("Input is not interpretable as hex!");
    /// assert_eq!(v.to_string(), "0001");
    /// let v: BitVec = BitVec::from_hex_string("0x1.8").expect("Input is not interpretable as hex!");
    /// assert_eq!(v.to_string(), "001");
    /// let v: BitVec = BitVec::from_hex_string("0x1.4").expect("Input is not interpretable as hex!");
    /// assert_eq!(v.to_string(), "01");
    /// let v: BitVec = BitVec::from_hex_string("0x1.2").expect("Input is not interpretable as hex!");
    /// assert_eq!(v.to_string(), "1");
    /// ```
    #[must_use]
    pub fn from_hex_string(s: &str) -> Option<Self> {
        // Edge case: completely empty string.
        if s.is_empty() {
            return Some(Self::new());
        }

        // Remove any "0x" or "0X" prefix.
        let s = if let Some(str) = s.strip_prefix("0x").or(s.strip_prefix("0X")) { str } else { s };

        // By default, the base of the "last" digit/character is 16 just like all the others.
        // However, there might be a suffix (one of ".2", ".4", or ".8") that changes that.
        let mut last_digit_base = 16;
        if s.ends_with(".2") {
            last_digit_base = 2;
        }
        else if s.ends_with(".4") {
            last_digit_base = 4;
        }
        else if s.ends_with(".8") {
            last_digit_base = 8;
        }

        // Remove the suffix if it exists.
        let mut s = s;
        if last_digit_base != 16 {
            s = &s[..s.len() - 2];
        }

        // Remove any whitespace, commas, and underscores & then check the string only contains hex digits.
        let s = s.replace(|c: char| c.is_whitespace() || c == ',' || c == '_', "");
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }

        // Size the result to allow for all characters being hex digits.
        let mut result = Self::with_capacity(4 * s.len());

        // Append all but the last character -- these are hex for sure.
        for c in s[..s.len() - 1].chars() {
            result.append_hex_digit(c);
        }

        // Append the last character using the appropriate base.
        result.append_digit(s.chars().last().unwrap(), last_digit_base);

        Some(result)
    }
}

/// Resizing and capacity methods for bit-vectors.
impl<Word: Unsigned> BitVec<Word> {
    /// Returns the capacity of the bit-vector.
    ///
    /// This is the total number of elements that can be stored without reallocating.
    /// This includes elements that are already in the bit-vector.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u64> = BitVec::zeros(10);
    /// assert_eq!(v.capacity(), 64);
    /// ```
    #[must_use]
    #[inline]
    pub fn capacity(&self) -> usize { Word::UBITS * self.m_store.capacity() }

    /// Returns the number of *additional* elements we can store in the bit-vector without reallocating.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec<u64> = BitVec::zeros(10);
    /// assert_eq!(v.remaining_capacity(), 54);
    /// ```
    #[must_use]
    #[inline]
    pub fn remaining_capacity(&self) -> usize { self.capacity() - self.m_len }

    /// Shrinks the capacity of the bit-vector as much as possible.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u64> = BitVec::zeros(1000);
    /// v.resize(15);
    /// v.shrink_to_fit();
    /// assert_eq!(v.capacity(), 64);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) -> &mut Self {
        self.m_store.truncate(Word::words_needed(self.m_len));
        self.m_store.shrink_to_fit();
        self
    }

    /// Clears the bit-vector *without* changing its capacity.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::zeros(10);
    /// let capacity = v.capacity();
    /// v.clear();
    /// assert_eq!(v.len(), 0);
    /// assert_eq!(v.capacity(), capacity);
    /// ```
    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.m_store.clear();
        self.m_len = 0;
        self
    }

    /// Resizes the bit-vector in-place so that `len` is equal to `new_len`.
    ///
    /// - If `new_len` is greater than `len`, the new elements are initialized to `0`.
    /// - If `new_len` is less than the current length, the bit-vector is truncated.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::ones(1000);
    /// v.resize(11);
    /// assert_eq!(v.len(), 11);
    /// assert_eq!(v.to_string(), "11111111111");
    /// v.resize(17);
    /// assert_eq!(v.len(), 17);
    /// assert_eq!(v.to_string(), "11111111111000000");
    /// ```
    pub fn resize(&mut self, new_len: usize) -> &mut Self {
        if new_len != self.m_len {
            self.m_store.resize(Word::words_needed(new_len), Word::ZERO);
            let old_len = self.m_len;
            self.m_len = new_len;

            // If we have truncated we may need to clean the last block.
            if new_len < old_len {
                self.clean();
            }
        }
        self
    }

    /// Helper method that cleans the last word of the bit-vector if that word is not fully occupied.
    ///
    /// This is used to enforce the guarantee that unused bits in the store are always set to 0.
    #[inline]
    fn clean(&mut self) -> &mut Self {
        let shift = self.m_len % Word::UBITS;

        // NOTE: If len == 0 then shift = 0 so there are no issues here:
        if shift != 0 {
            let mask = !(Word::MAX << shift);
            self.m_store[Word::word_index(self.m_len - 1)] &= mask;
        }
        self
    }
}

/// Methods to add or remove single elements from the end of a bit-vector.
impl<Word: Unsigned> BitVec<Word> {
    /// Appends a single `bool` element to the end of the bit-vector.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec = BitVec::zeros(10);
    /// v.push(true);
    /// assert_eq!(v.to_string(), "00000000001");
    /// v.push(false);
    /// assert_eq!(v.to_string(), "000000000010");
    /// ```
    #[inline]
    pub fn push(&mut self, val: bool) -> &mut Self {
        self.resize(self.len() + 1);
        if val {
            self.set(self.len() - 1, true);
        }
        self
    }

    /// Removes the last element and returns it, or `None` if the bit-vector is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec = BitVec::zeros(10);
    /// v.push(true);
    /// assert_eq!(v.to_string(), "00000000001");
    /// assert_eq!(v.pop(), Some(true));
    /// assert_eq!(v.to_string(), "0000000000");
    /// assert_eq!(v.pop(), Some(false));
    /// assert_eq!(v.to_string(), "000000000");
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<bool> {
        if self.is_empty() {
            return None;
        }
        let result = self[self.len() - 1];
        self.resize(self.len() - 1);
        Some(result)
    }
}

/// Methods that append bits from various sources to the end of a bit-vector.
impl<Word: Unsigned> BitVec<Word> {
    /// Appends *all* the bits from *any* unsigned type `src` to the end of the bit-vector.
    ///
    /// # Note
    /// The size of `src` need not match the size of the words that hold the elements of the bit-vector.
    /// For example, if `src` is a `u64` and `Word` is a `u32` then the `u64` will be added to the end as two `u32`
    /// words.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::zeros(6);
    /// v.append_unsigned(u8::MAX);
    /// assert_eq!(v.len(), 14);
    /// assert_eq!(v.to_string(), "00000011111111");
    /// v.append_unsigned(u16::MAX);
    /// assert_eq!(v.len(), 30);
    /// assert_eq!(v.to_string(), "000000111111111111111111111111");
    /// ```
    pub fn append_unsigned<Src>(&mut self, src: Src) -> &mut Self
    where Src: Unsigned + TryInto<Word> {
        // Resize the underlying store to accommodate a new `Src`'s worth of elements.
        let old_len = self.len();
        self.resize(old_len + Src::UBITS);
        self.slice_mut(old_len..).copy_unsigned(src);
        self
    }

    /// Appends all the bits from *any* bit-store `src` to the end of the bit-vector.
    ///
    /// # Note
    /// Generally, we do not support interactions between bit-stores that use different underlying unsigned word
    /// types. This method is an exception, and the `src` bit-store may use a different unsigned type from the one used
    /// here.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut dst: BitVec<u8> = BitVec::zeros(10);
    /// let src: BitVec<u16> = BitVec::ones(10);
    /// dst.append_store(&src);
    /// assert_eq!(dst.to_string(), "00000000001111111111");
    /// ```
    pub fn append_store<SrcWord, SrcStore>(&mut self, src: &SrcStore) -> &mut Self
    where
        SrcWord: Unsigned,
        SrcStore: BitStore<SrcWord>,
    {
        // Resize the underlying store to accommodate a new `Src`'s worth of elements.
        let old_len = self.len();
        self.resize(old_len + src.len());
        self.slice_mut(old_len..).copy_store(src);
        self
    }

    /// Appends a single character `x` interpreted as a digit in some `base` to the end of the bit-vector.
    ///
    /// The `base` argument **must** be one of 2, 4, 8, or 16. <br>
    /// Does nothing if `base` is not in that set or if `x` is not a valid digit.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::new();
    /// v.append_digit('A', 16);
    /// assert_eq!(v.to_string(), "1010");
    /// v.append_digit('X', 16);
    /// assert_eq!(v.to_string(), "1010");
    /// v.append_digit('1', 8);
    /// assert_eq!(v.to_string(), "1010001");
    /// v.append_digit('1', 4);
    /// assert_eq!(v.to_string(), "101000101");
    /// v.append_digit('1', 2);
    /// assert_eq!(v.to_string(), "1010001011");
    /// ```
    pub fn append_digit(&mut self, x: char, base: u32) -> &mut Self {
        const BASES: &[u32] = &[2, 4, 8, 16];
        if BASES.contains(&base)
            && let Some(digit) = x.to_digit(base)
        {
            // Resize to accommodate the bits in `digit`. This adds zero elements.
            let digit_bits = base.ilog2() as usize;
            let old_len = self.m_len;
            self.resize(old_len + digit_bits);

            // If a `digit` bit is set then set the corresponding slot in the bit-vector.
            for i in 0..digit_bits {
                let digit_mask = 1 << (digit_bits - 1 - i);
                if digit & digit_mask != 0 {
                    self.set(old_len + i, true);
                }
            }
        }
        self
    }

    /// Appends a single character `x` interpreted as a hex digit to the end of the bit-vector.
    ///
    /// Does nothing if `x` is not a valid hex digit.
    ///
    /// # Note
    /// This is the same as `append_digit(x, 16)` but we provide a specialized version as we push hex characters
    /// much more often than other bases and want to skip some checks for efficiency.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::new();
    /// v.append_hex_digit('F');
    /// assert_eq!(v.to_string(), "1111", "v.append_hex_digit('F') = {v}");
    /// v.append_hex_digit('X');
    /// assert_eq!(v.to_string(), "1111", "v.append_hex_digit('X') = {v}");
    /// v.append_hex_digit('1');
    /// assert_eq!(v.to_string(), "11110001", "v.append_hex_digit('1') = {v}");
    /// ```
    pub fn append_hex_digit(&mut self, x: char) -> &mut Self {
        if let Some(digit) = x.to_digit(16) {
            // Resize to accommodate four extra bits -- initially all zeros
            let old_len = self.m_len;
            self.resize(old_len + 4);
            // If a `digit` bit is set then set the corresponding slot in the bit-vector.
            for i in 0..4 {
                let mask = 1 << (3 - i);
                if digit & mask != 0 {
                    self.set(old_len + i, true);
                }
            }
        }
        self
    }
}

///  Methods to remove items from the end of a bit-vector.
impl<Word: Unsigned> BitVec<Word> {
    /// Splits a bit-vector into two at the given index. The second part is returned as a new bit-vector.
    ///
    /// On return, `dst` contains the bits from `at` to the end of the bit-vector. The `self` bit-vector is modified.
    ///
    /// # Panics
    /// This method panics if the split point is beyond the end of the bit-vector.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec = BitVec::alternating(10);
    /// let dst = v.split_off(5);
    /// assert_eq!(v.to_string(), "10101");
    /// assert_eq!(dst.to_string(), "01010");
    /// ```
    #[must_use]
    pub fn split_off(&mut self, at: usize) -> BitVec<Word> {
        let mut dst = BitVec::new();
        self.split_off_into(at, &mut dst);
        dst
    }

    /// Splits a bit-vector into two at the given index and puts the second part into the provided `dst` bit-vector.
    ///
    /// On return, `dst` contains the bits from `at` to the end of the bit-vector. The `self` bit-vector is modified.
    ///
    /// # Panics
    /// This method panics if the split point is beyond the end of the bit-vector.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec = BitVec::alternating(10);
    /// let mut dst: BitVec = BitVec::new();
    /// v.split_off_into(5, &mut dst);
    /// assert_eq!(v.to_string(), "10101");
    /// assert_eq!(dst.to_string(), "01010");
    /// ```
    pub fn split_off_into(&mut self, at: usize, dst: &mut BitVec<Word>) {
        assert!(at <= self.len(), "split point {at} is beyond the end of the bit-vector");
        dst.clear();
        dst.append_store(&self.slice(at..self.len()));
        self.resize(at);
    }

    /// Removes a single word's worth of bits from the end of the bit-vector and returns it as a `Word` or `None` if the
    /// bit-vector is empty.
    ///
    /// The bit-vector shrinks by the number of bits in a `Word`.
    ///
    /// Note:
    /// If the number of bits in the bit-vector is less than `Word::UBITS` then this will clear out the vector and the
    /// returned word will be padded appropriately with zero bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::zeros(10);
    /// v.append_unsigned(u8::MAX);
    /// assert_eq!(v.split_off_word(), Some(u8::MAX));
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    pub fn split_off_word(&mut self) -> Option<Word> {
        if self.is_empty() {
            return None;
        }

        // Easy case: There is just one word in the bit-vector.
        let n_words = self.words();
        if n_words == 1 {
            let result = self.m_store[0];
            self.clear();
            return Some(result);
        }

        // Otherwise, we may need to handle the case where the last word is not fully occupied.
        let shift = self.m_len % Word::UBITS;
        let result = if shift == 0 {
            // The last word is fully occupied so we can just return it.
            self.m_store[n_words - 1]
        }
        else {
            // Need to combine the last two words in the bit-vector.
            let lo = self.m_store[n_words - 2] >> shift;
            let hi = self.m_store[n_words - 1] << (Word::UBITS - shift);
            lo | hi
        };

        // Resize the bit-vector to remove the last word's worth of bits.
        self.resize(self.m_len - Word::UBITS);
        Some(result)
    }

    /// Removes a single primitive *unsigned* word of any size from the end of the bit-vector and returns it or `None`
    /// if the bit-vector is empty.
    ///
    /// The bit-vector shrinks by the number of bits in the unsigned type `Dst`.
    ///
    /// # Note
    /// You can remove a primitive unsigned word of *any size* from the end of the bit-vector. For example, you can
    /// remove a `u16` from the end of a `BitVec<u8>` with 24 bit elements and it will be converted to a `u16`
    /// appropriately leaving the remaining 8 bits untouched in the bit-vector.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVec<u8> = BitVec::ones(24);
    /// assert_eq!(v.split_off_unsigned::<u16>(), Some(u16::MAX));
    /// assert_eq!(v.to_string(), "11111111");
    /// assert_eq!(v.split_off_unsigned::<u16>(), Some(u8::MAX as u16));
    /// assert_eq!(v.is_empty(), true);
    /// ```
    pub fn split_off_unsigned<Dst>(&mut self) -> Option<Dst>
    where Dst: Unsigned + TryFrom<Word> + From<Word> {
        if self.is_empty() {
            return None;
        }

        // Check if we need multiple Words to represent Dst
        let words_needed = Word::words_needed(Dst::UBITS);

        if words_needed == 1 {
            // The destination type Dst fits in a single Word, so we can just pop a single word & convert it.
            Some(self.split_off_word()?.into())
        }
        else {
            // The destination type Dst is bigger than Word, so we need to pop multiple words.
            let mut words = Vec::with_capacity(words_needed);

            // Pop the required number of words
            for _ in 0..words_needed {
                if let Some(word) = self.split_off_word() {
                    words.push(word);
                }
                else {
                    break;
                }
            }

            // Reconstruct the result by combining words in the correct order
            let mut result = Dst::ZERO;
            for &word in words.iter().rev() {
                result = (result << Word::BITS) | Dst::from(word);
            }
            Some(result)
        }
    }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Default` trait for bit-vectors
// --------------------------------------------------------------------------------------------------------------------

/// Implement the `Default` trait for a bit-vector.
impl<Word: Unsigned> Default for BitVec<Word> {
    /// The default constructor creates an empty bit-vector.
    ///
    /// No capacity is reserved until elements are added.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVec = BitVec::default();
    /// assert_eq!(v.len(), 0);
    /// assert_eq!(v.words(), 0);
    /// ```
    fn default() -> Self { Self::new() }
}

// --------------------------------------------------------------------------------------------------------------------
// Implement the `From` trait for a `BitVec` from a `BitSlice`.
// --------------------------------------------------------------------------------------------------------------------

/// Convert a [`BitSlice`] into a bit-vector. The slice is consumed by the operation.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v1: BitVec = BitVec::alternating(10);
/// let slice = v1.slice(0..5);
/// let mut v2: BitVec = slice.into();
/// assert_eq!(v2.to_string(), "10101");
/// v2.flip_all();
/// assert_eq!(v2.to_string(), "01010");
/// assert_eq!(v1.to_string(), "1010101010");
/// ```
impl<'a, Word: Unsigned> From<BitSlice<'a, Word>> for BitVec<Word> {
    fn from(src: BitSlice<'a, Word>) -> Self { BitVec::from_store(&src) }
}

/// Convert a *reference* to a [`BitSlice`] into a bit-vector. The slice continues unchanged.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v1: BitVec = BitVec::alternating(10);
/// let slice = v1.slice(0..5);
/// let mut v2: BitVec = slice.into();
/// assert_eq!(v2.to_string(), "10101");
/// v2.flip_all();
/// assert_eq!(v2.to_string(), "01010");
/// assert_eq!(v1.to_string(), "1010101010");
/// ```
impl<'a, Word: Unsigned> From<&BitSlice<'a, Word>> for BitVec<Word> {
    fn from(src: &BitSlice<'a, Word>) -> Self { BitVec::from_store(src) }
}
