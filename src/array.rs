//! [`BitArray`] is a  _statically sized_ vector over GF(2) --- a _bit-array_. <br>
//! This module requires const generic arithmetic, so is gated behind the `unstable` feature.

use crate::{
    BitStore,
    Unsigned,
};

#[doc = include_str!("../docs/array.md")]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct BitArray<const N: usize, Word: Unsigned = usize, const WORDS: usize = { N.div_ceil(Word::UBITS) }> {
    // The underlying store of `Unsigned` words that are used to store the bits.
    m_store: [Word; WORDS],
}

/// Implement the `BitStore` trait for `BitArray`.
impl<const N: usize, Word: Unsigned, const WORDS: usize> BitStore<Word> for BitArray<N, Word, WORDS> {
    /// Returns the number of *bits* in the `BitArray`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::new();
    /// assert_eq!(v.len(), 10);
    /// ```
    fn len(&self) -> usize { N }

    /// Returns a pointer to the real words underlying the `BitArray` as an [`Unsigned`] slice.
    fn store(&self) -> &[Word] { self.m_store.as_slice() }

    /// Returns a pointer to the real words underlying the `BitArray` as an [`Unsigned`] mutable slice.
    fn store_mut(&mut self) -> &mut [Word] { self.m_store.as_mut_slice() }

    /// Returns the offset (in bits) of the first bit element in the `BitArray` within the first [`Unsigned`] word.
    /// This is always zero for a `BitArray`.
    fn offset(&self) -> u32 { 0 }

    /// Returns the least number of [`Unsigned`] words needed to store the bits in the `BitArray`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<6, u8> = BitArray::zeros();
    /// assert_eq!(v.words(), 1);
    /// let v: BitArray<10, u8> = BitArray::zeros();
    /// assert_eq!(v.words(), 2);
    /// ```
    #[inline]
    fn words(&self) -> usize {
        // For a bit-array the number of words is the same as the number of words in the underlying store.
        self.m_store.len()
    }

    /// Returns the [`Unsigned`] word at index `i` from the `BitArray`'s underlying store of words.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the `BitArray`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::zeros();
    /// assert_eq!(v.word(0), 0);
    /// ```
    #[inline]
    fn word(&self, i: usize) -> Word {
        debug_assert!(i < self.words(), "Index {i} should be less than {}", self.words());
        self.m_store[i]
    }

    /// Sets the [`Unsigned`] word at index `i` in the `BitArray` to `word`.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the `BitArray`.
    ///
    /// # Note
    /// It is careful to only set the bits that are within the vector (the last word may only be partially occupied).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitArray<12, u8> = BitArray::zeros();
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
            let last_bit = self.len() - 1;
            #[allow(clippy::cast_possible_truncation)]
            let last_offset = (last_bit % Word::UBITS) as u32;
            self.m_store[i].replace_bits(0..=last_offset, word);
        }
    }
}

/// Constructors for bit-arrays.
impl<const N: usize, Word: Unsigned, const WORDS: usize> BitArray<N, Word, WORDS> {
    /// The default constructor constructs a bit-array with `N` elements that are all zeros.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::new();
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    #[inline]
    pub fn new() -> Self { Self { m_store: [Word::ZERO; WORDS] } }

    /// Constructs a bit-array with `N` elements by repeatedly copying the bits from a single `Word` instance.
    ///
    /// The final copy of `word` may be truncated and padded with zeros (unused bits are always set to zero in this
    /// crate).
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::from_word(0b01010101);
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    #[inline]
    pub fn from_word(word: Word) -> Self {
        let mut result = Self { m_store: [word; WORDS] };
        result.clean();
        result
    }

    /// Constructs a bit-array with ``N` elements, all initialized to `0`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::zeros();
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    #[inline]
    pub fn zeros() -> Self { Self { m_store: [Word::ZERO; WORDS] } }

    /// Constructs a bit-array with `N` elements, all initialized to `1`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::ones();
    /// assert_eq!(v.to_string(), "1111111111");
    /// ```
    #[inline]
    pub fn ones() -> Self {
        let mut result = Self { m_store: [Word::MAX; WORDS] };
        result.clean();
        result
    }

    /// Returns a bit-array with `N` elements, all initialized to the boolean value `val`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::constant(true);
    /// assert_eq!(v.to_string(), "1111111111");
    /// let v: BitArray<10> = BitArray::constant(false);
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    pub fn constant(val: bool) -> Self {
        let word = if val { Word::MAX } else { Word::ZERO };
        let mut result = Self { m_store: [word; WORDS] };
        result.clean();
        result
    }

    /// Constructs the *unit* bit-array where only element `i` of `N` elements is set to 1.
    ///
    /// # Panics
    /// Panics if `i` is greater than or equal to `N`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::unit(5);
    /// assert_eq!(v.to_string(), "0000010000");
    /// ```
    #[inline]
    pub fn unit(i: usize) -> Self {
        assert!(i < N, "Index {i} must be less than the length of the BitArray {N}!");
        let mut result = Self::zeros();
        result.set(i, true);
        result
    }

    /// Constructs a bit-array with `N` elements, where the bits alternate between `1` and `0`.
    ///
    /// The pattern starts with a `1` so e.g. `1010101010101`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::alternating();
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    pub fn alternating() -> Self {
        let mut result = Self { m_store: [Word::ALTERNATING; WORDS] };
        result.clean();
        result
    }

    /// Constructs a bit-array with `N` elements by calling a function `f` for each bit index.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::from_fn(|i| i % 2 == 0);
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    pub fn from_fn(f: impl Fn(usize) -> bool) -> Self {
        let mut result = Self::zeros();
        result.copy_fn(f);
        result
    }

    /// Helper method that cleans the last word of the bit-array if that word is not fully occupied.
    ///
    /// This is used to enforce the guarantee that unused bits in the store are always set to 0.
    #[inline]
    fn clean(&mut self) -> &mut Self {
        let shift = self.len() % Word::UBITS;

        // NOTE: If len == 0 then shift = 0 so there are no issues here:
        if shift != 0 {
            let mask = !(Word::MAX << shift);
            self.m_store[Word::word_index(self.len() - 1)] &= mask;
        }
        self
    }
}

/// Constructors that set the elements of a bit-array randomly.
impl<const N: usize, Word: Unsigned, const WORDS: usize> BitArray<N, Word, WORDS> {
    /// Constructs a random bit-array with `N` elements where each bit is set/unset with probability 50/50.
    ///
    /// The random number generator is seeded on first use with a scrambled version of the current time so you get
    /// different outputs for each run.
    ///
    /// See the `random_seeded` method for a way to get reproducible randomly filled bit-arrays.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::random();
    /// assert_eq!(v.len(), 10);
    /// ```
    pub fn random() -> Self {
        let mut result = Self::zeros();
        result.fill_random();
        result
    }

    /// Constructs a random bit-array with `N` elements where each bit is set/unset with probability 50/50.
    ///
    /// For reproducibility, the random number generator used here is seeded with the specified `seed`.
    ///
    /// # Note
    /// - The generator is reset to the previous seed after the bit-array is constructed.
    /// - A seed of `0` is taken to seed the generator from a scrambled version of the current time.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: BitArray<1000, u8> = BitArray::random_seeded(42);
    /// let v2: BitArray<1000, u8> = BitArray::random_seeded(42);
    /// assert_eq!(v1, v2);
    /// ```
    pub fn random_seeded(seed: u64) -> Self {
        let mut result = Self::zeros();
        result.fill_random_seeded(seed);
        result
    }

    /// Constructs a random bit-array with `N` elements where each bit is set with probability `p`.
    ///
    /// The random number generator is seeded on first use with a scrambled version of the current time.
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-array with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::random_biased(0.578);
    /// assert_eq!(v.len(), 10);
    /// ```
    pub fn random_biased(p: f64) -> Self {
        let mut result = Self::zeros();
        result.fill_random_biased(p);
        result
    }

    /// Constructs a random bit-array with `N` elements where each bit is set with probability `p` and the RNG is
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-array with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10> = BitArray::random_biased_seeded(1.2, 42); // All bits set since p > 1
    /// assert_eq!(v.count_ones(), 10);
    /// let u: BitArray<100> = BitArray::random_biased_seeded(0.85, 42); // Using same seed for u and v.
    /// let v: BitArray<100> = BitArray::random_biased_seeded(0.85, 42);
    /// assert_eq!(u, v);
    /// ```
    pub fn random_biased_seeded(p: f64, seed: u64) -> Self {
        let mut result = Self::zeros();
        result.fill_random_biased_seeded(p, seed);
        result
    }
}

// --------------------------------------------------------------------------------------------------------------------
// The `Default` trait for bit-arrays
// --------------------------------------------------------------------------------------------------------------------

/// Implement the `Default` trait for a bit-array.
impl<const N: usize, Word: Unsigned, const WORDS: usize> Default for BitArray<N, Word, WORDS> {
    /// The default constructor creates a bit-array with all the bits set to 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitArray<10, u8> = BitArray::default();
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.words(), 2);
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    fn default() -> Self { Self::new() }
}
