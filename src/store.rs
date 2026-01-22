//! [`BitStore`] is the core trait implemented by bit-arrays, bit-vectors, and bit-slices.
use crate::{
    BitSlice,
    BitVector,
    Bits,
    SetBits,
    UnsetBits,
    Unsigned,
    Words,
    rng,
};

// Standard library imports.
use std::{
    fmt::Write,
    ops::{
        Bound,
        RangeBounds,
    },
};

#[doc = include_str!("../docs/store.md")]
pub trait BitStore<Word: Unsigned>: Sized {
    /// Required method that should return the number of *bit elements* in the store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::zeros(10);
    /// assert_eq!(v.len(), 10);
    /// ```
    fn len(&self) -> usize;

    /// Required method that should return a pointer to the real words underlying the bit-store as an [`Unsigned`]
    /// slice.
    fn store(&self) -> &[Word];

    /// Required method that should return a pointer to the real words underlying the bit-store as an [`Unsigned`]
    /// mutable slice.
    fn store_mut(&mut self) -> &mut [Word];

    /// Required method that should return the number of bits from the least significant bit of `self.store()[0]` to the
    /// first bit in the store. This will be zero for bit-vectors and sets, but generally non-zero for bit-slices.
    fn offset(&self) -> u32;

    /// Required method that should return the _fewest_ number of [`Unsigned`] words needed to store the bits in the
    /// store. The return value will always be identical to `Word::words_needed(self.len())`.
    ///
    /// # Note
    /// This may not be the same as the number of words in the underlying vector of `Unsigned` words. <br>
    /// In particular, slices need not be aligned to the word boundaries of the underlying store so this method may
    /// return one fewer words than the underlying vector of `Unsigned` words.
    /// The return value will always be identical to `Word::words_needed(self.len())` so we could compute `words()` on
    /// the fly but all concrete bit-store types cache this value, which has a measurable performance impact.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector<u8> = BitVector::zeros(10);
    /// assert_eq!(v.words(), 2);
    /// ```
    fn words(&self) -> usize;

    /// Required method that should return the "word" `i` from the store.
    ///
    /// # Note
    /// This method is used to access the bits in the store _as if_ they were perfectly aligned with the word
    /// boundaries. It should appear a bit `0` in the store is located at the least significant bit of `word(0)`.
    /// This is trivial for the `BitVector` and `BitArray` types, but bit-slices typically need to synthesise
    /// appropriate "words" on demand from a couple of the "real" words that back the owning bit-vector or
    /// bit-array.
    ///
    /// For example, if the store has 18 elements, then `word(0)` should cover bit elements 0 through 7, `word(1)` the
    /// bit elements 8 through 15, and `word(2)` the bit elements 16 and 17 and the rest of the bits in that word
    /// should be zeros.
    ///
    /// The final word may not be fully occupied but the method must guarantee that unused bits are set to 0.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::zeros(10);
    /// assert_eq!(v.word(0), 0);
    /// ```
    fn word(&self, i: usize) -> Word;

    /// Required method that should set "word"  `i` in the store to the specified `value`.
    ///
    /// # Note
    /// This method is used to mutate the bits in the store as if they were perfectly aligned with the word boundaries.
    /// It should _look as if_ bit `0` in the store is located at the least significant bit of `word(0)`.
    /// This is trivial for the `BitVector` and `BitArray` types, but bit-slices typically need to synthesise
    /// appropriate "words" on demand from a couple of the "real" words that back the owning bit-vector or
    /// bit-array.
    ///
    /// For example, if the store has 18 elements, then `set_word(0,v)` should set bit elements 0 through 7,
    /// `set_word(1,v)` the bit elements 8 through 15, and `set_word(2,v)` the bit elements 16 and 17 and leave the
    /// rest of the bits in that word at zero
    ///
    /// The method must ensure that inaccessible bits in the underlying store are not changed by this call.
    ///
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.set_word(0, 0);
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    fn set_word(&mut self, i: usize, value: Word);

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to access individual bits in the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns `true` if element `i` in the store is set to `1`.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the object.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// assert_eq!(v.get(0), true);
    /// assert_eq!(v.get(1), false);
    /// assert_eq!(v.get(8), true);
    /// assert_eq!(v.get(9), false);
    /// ```
    #[inline]
    fn get(&self, i: usize) -> bool {
        debug_assert!(i < self.len(), "index {} is out of bounds for bit-length {}", i, self.len());
        let (word_index, mask) = Word::index_and_mask(i);
        self.word(word_index) & mask != Word::ZERO
    }

    /// Returns `true` if the first bit in the store is set.
    ///
    /// # Panics
    /// In debug mode, panics if the type is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::ones(10);
    /// assert!(v.first() == true);
    /// let v: BitVector = BitVector::zeros(10);
    /// assert!(v.first() == false);
    /// ```
    #[inline]
    fn first(&self) -> bool {
        debug_assert!(!self.is_empty(), "The store is empty");
        self.get(0)
    }

    /// Returns `true` if the last bit in the store is set.
    ///
    /// # Panics
    /// In debug mode, panics if the type is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::ones(10);
    /// assert!(v.last() == true);
    /// let v: BitVector = BitVector::zeros(10);
    /// assert!(v.last() == false);
    /// ```
    #[inline]
    fn last(&self) -> bool {
        debug_assert!(!self.is_empty(), "The store is empty");
        self.get(self.len() - 1)
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to change individual bits in the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Sets the bit at index `i` to the boolean `val` and returns a *reference* to the store.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the store for chaining.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.set(0, true);
    /// assert_eq!(v.to_string(), "1000000000");
    /// v.set(1, true);
    /// assert_eq!(v.to_string(), "1100000000");
    /// v.set(1, false);
    /// assert_eq!(v.to_string(), "1000000000");
    /// ```
    #[inline]
    fn set(&mut self, i: usize, val: bool) -> &mut Self {
        debug_assert!(i < self.len(), "index {i} is out of bounds for a store of length {}", self.len());
        let (word_index, mask) = Word::index_and_mask(i);
        let word = self.word(word_index);
        let current_value = (word & mask) != Word::ZERO;
        if current_value != val {
            self.set_word(word_index, word ^ mask);
        }
        self
    }

    /// Flips the bit at index `i` and returns a *reference* to the store for chaining.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the store for chaining.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::ones(10);
    /// v.flip(0);
    /// assert_eq!(v.to_string(), "0111111111");
    /// v.flip(1);
    /// assert_eq!(v.to_string(), "0011111111");
    /// v.flip(9);
    /// assert_eq!(v.to_string(), "0011111110");
    /// ```
    #[inline]
    fn flip(&mut self, i: usize) -> &mut Self {
        debug_assert!(i < self.len(), "index {i} is out of bounds for a store of length {}", self.len());
        let (word_index, mask) = Word::index_and_mask(i);
        self.set_word(word_index, self.word(word_index) ^ mask);
        self
    }

    /// Swaps the bits at indices `i0` and `i1` and returns a *reference* to the store for chaining.
    ///
    /// # Panics
    /// In debug mode, panics if either of the indices is out of bounds for the store for chaining.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.set(0, true);
    /// assert_eq!(v.to_string(), "1000000000");
    /// v.swap(0, 1);
    /// assert_eq!(v.to_string(), "0100000000");
    /// v.swap(0, 1);
    /// assert_eq!(v.to_string(), "1000000000");
    /// v.swap(0, 9);
    /// assert_eq!(v.to_string(), "0000000001");
    /// v.swap(0, 9);
    /// assert_eq!(v.to_string(), "1000000000");
    /// ```
    #[inline]
    fn swap(&mut self, i0: usize, i1: usize) -> &mut Self {
        debug_assert!(i0 < self.len(), "index {i0} is out of bounds for a store of length {}", self.len());
        debug_assert!(i1 < self.len(), "index {i1} is out of bounds for a store of length {}", self.len());
        if i0 != i1 {
            let (word0, mask0) = Word::index_and_mask(i0);
            let (word1, mask1) = Word::index_and_mask(i1);
            let val0 = (self.word(word0) & mask0) != Word::ZERO;
            let val1 = (self.word(word1) & mask1) != Word::ZERO;
            if val0 != val1 {
                if word0 == word1 {
                    // Both bits are in the same word
                    self.set_word(word0, self.word(word0) ^ mask0 ^ mask1);
                }
                else {
                    // Bits are in different words
                    self.set_word(word0, self.word(word0) ^ mask0);
                    self.set_word(word1, self.word(word1) ^ mask1);
                }
            }
        }
        self
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to query the overall state of the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns `true` if the store is empty.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::new();
    /// assert!(v.is_empty());
    /// v.push(true);
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }

    /// Returns `true` if at least one bit is set to `1` in the store.
    ///
    /// # Note
    /// Empty types are considered to have no set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(12);
    /// assert!(v.any() == false);
    /// v.set_all(true);
    /// assert!(v.any() == true);
    /// ```
    #[inline]
    fn any(&self) -> bool {
        // If the type is empty, it trivially has no set bits.
        // NOTE: Formally, the "logical connective" for any() is `OR` with the identity `false`.
        for i in 0..self.words() {
            if self.word(i) != Word::ZERO {
                return true;
            }
        }
        false
    }

    /// Returns `true` if all of the bits are set to `1` in the store.
    ///
    /// # Note
    /// Empty types are considered to have all set bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(12);
    /// assert!(v.all() == false);
    /// v.set(1, true);
    /// assert!(v.all() == false);
    /// v.set_all(true);
    /// assert!(v.all() == true);
    /// ```
    fn all(&self) -> bool {
        // NOTE: Formally, the "logical connective" for all() is `AND` with the identity `TRUE`.
        if self.is_empty() {
            return true;
        }

        // Check the fully occupied words ...
        for i in 0..self.words() - 1 {
            if self.word(i) != Word::MAX {
                return false;
            }
        }

        // Handle the last word which may not be fully occupied.
        #[allow(clippy::cast_possible_truncation)]
        let unused_bits = (Word::UBITS - self.len() % Word::UBITS) as u32;
        let last_word_max = Word::MAX.unbounded_shr(unused_bits);
        if self.word(self.words() - 1) != last_word_max {
            return false;
        }
        true
    }

    /// Returns `true` if none of the bits are set to `1` in the store.
    ///
    /// # Note
    /// Empty types are considered to have no unset bits.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(12);
    /// assert!(v.none() == true);
    /// v.set(1, true);
    /// assert!(v.none() == false);
    /// ```
    #[inline]
    fn none(&self) -> bool {
        // If the bit-vector is empty, it trivially has no unset bits.
        // NOTE: Formally, the "logical connective" for none() is `AND` with the identity `TRUE`.
        !self.any()
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to change all bits in the store in one call.
    // ----------------------------------------------------------------------------------------------------------------

    /// Sets all bits in the store to the boolean value `v` and returns a *reference* to the store for chaining.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.set_all(true);
    /// assert_eq!(v.to_string(), "1111111111");
    /// ```
    fn set_all(&mut self, v: bool) -> &mut Self {
        let value = if v { Word::MAX } else { Word::ZERO };
        for word_index in 0..self.words() {
            self.set_word(word_index, value);
        }
        self
    }

    /// Flips all bits in the store and returns a *reference* to it for chaining.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::ones(10);
    /// v.flip_all();
    /// assert_eq!(v.to_string(), "0000000000");
    /// ```
    fn flip_all(&mut self) -> &mut Self {
        for word_index in 0..self.words() {
            self.set_word(word_index, !self.word(word_index));
        }
        self
    }

    /// Returns a new bit-vector that is the result of flipping all the bits in this bit-store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let u: gf2::BitVector = gf2::BitVector::ones(3);
    /// let v = u.flipped();
    /// assert_eq!(u.to_binary_string(), "111");
    /// assert_eq!(v.to_binary_string(), "000");
    /// ```
    fn flipped(&self) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.flip_all();
        result
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to copy bits into the store from other sources.
    // ----------------------------------------------------------------------------------------------------------------

    /// Copies the bits from an unsigned `src` value.
    ///
    /// # Notes:
    /// 1. The size of the store *must* match the number of bits in the source type.
    /// 2. We allow *any* unsigned word source, e.g. copying a single `u64` into a `BitVector<u8>` of size 64.
    /// 3. The least-significant bit of the source becomes the bit at index 0 in the store.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(16);
    /// let src: u16 = 0b1010101010101010;
    /// v.copy_unsigned(src);
    /// assert_eq!(v.to_string(), "0101010101010101");
    /// ```
    fn copy_unsigned<Src>(&mut self, src: Src) -> &mut Self
    where Src: Unsigned + TryInto<Word> {
        assert_eq!(
            self.len(),
            Src::UBITS,
            "Bit-length mismatch: store has {} bits but source has {} bits",
            self.len(),
            Src::UBITS
        );

        // Try to convert `src` into a `Word` if possible.
        if let Ok(word) = src.try_into() {
            // Conversion succeeded: The `src` was converted to a `Word` without loss of information. Use it.
            self.set_word(0, word);
        }
        else {
            // The `src` word is too big to fit into a `Word` so nibble bits from it one `Word` at a time.
            let num_words = Src::UBITS / Word::UBITS;
            let mut word: Word;
            let mut src = src;
            for word_index in 0..num_words {
                // Extract the next `Word` from `src`. This works because `Word` is smaller than `Src`.
                unsafe { word = std::mem::transmute_copy(&src) };

                // Store the extracted `Word`.
                self.set_word(word_index, word);

                // Shift `src` down to get the next `Word` into position.
                src >>= Word::BITS;
            }
        }
        self
    }

    /// Fills this bit-store with the bits from _any_ other bit-store `src` of the same length.
    ///
    /// # Note
    /// This is one of the few methods in the library that _doesn't_ require the two stores to have the same underlying
    /// `Unsigned` word type for their storage -- i.e., the `Word` type for `self` may differ from the `SrcWord` type
    /// for the bit-store `src`. You can use it to convert between different `Word` type stores (e.g., from
    /// `BitVector<u32>` to `BitVector<u8>`) as long as the sizes match.
    ///
    /// # Panics
    /// This method panics if the number of elements in this store and the `src` do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let src: BitVector =
    ///     BitVector::from_string("010101010101010101010101010101010101010101010101010101010101").unwrap();
    /// let mut dst: BitVector = BitVector::zeros(src.len());
    /// dst.copy_store(&src);
    /// assert_eq!(dst.to_string(), src.to_string());
    /// let src: BitVector<u8> = BitVector::from_string("1011001110001111").unwrap();
    /// let mut dst: BitVector<u32> = BitVector::zeros(src.len());
    /// dst.copy_store(&src);
    /// assert_eq!(dst.to_string(), src.to_string());
    /// let src: BitVector<u16> = BitVector::from_string("101100111000111110110011100011111011001110001111").unwrap();
    /// let mut dst: BitVector<u8> = BitVector::zeros(src.len());
    /// dst.copy_store(&src);
    /// assert_eq!(dst.to_string(), src.to_string());
    /// ```
    fn copy_store<SrcWord, SrcStore>(&mut self, src: &SrcStore) -> &mut Self
    where
        SrcWord: Unsigned,
        SrcStore: BitStore<SrcWord>,
    {
        assert_eq!(self.len(), src.len(), "Number of bits mismatch {} != {}", self.len(), src.len());

        // Edge case.
        if self.is_empty() {
            return self;
        }

        // Fast path: source and destination words have the same bit-width so we can copy a word at a time.
        if Word::UBITS == SrcWord::UBITS {
            for i in 0..self.words() {
                let src_word = src.word(i);
                let dst_word: Word = unsafe { std::mem::transmute_copy(&src_word) };
                self.set_word(i, dst_word);
            }
            return self;
        }

        // General case: source and destination words have different bit-widths.
        // We need to nibble bits from the source as needed.
        #[allow(clippy::cast_possible_truncation)]
        for dst_word_index in 0..self.words() {
            // Determine the bit-range in the overall store covered by this destination word.
            let start_bit = dst_word_index * Word::UBITS;
            let end_bit = (start_bit + Word::UBITS).min(self.len());
            let bits_needed = end_bit - start_bit;

            // Nibble the required bits from the source store to create the destination word `dst_word`.
            let mut dst_word = Word::ZERO;
            let mut remaining = bits_needed;
            let mut src_bit = start_bit;
            while remaining > 0 {
                // Determine the source word and offset within that word for the current source bit.
                let src_word_index = src_bit / SrcWord::UBITS;
                let src_offset = src_bit % SrcWord::UBITS;
                let src_word = src.word(src_word_index);

                // Determine how many bits we can grab from the current source word.
                let available = (SrcWord::UBITS - src_offset).min(remaining);

                // Grab the required bits from the current source word.
                let mask = SrcWord::with_set_bits(0..available as u32);
                let chunk = (src_word >> (src_offset as u32)) & mask;
                let chunk_as_word: Word = match Word::try_from(chunk.as_u128()) {
                    Ok(val) => val,
                    Err(_) => unreachable!("Oops --- chunk should always fit into destination word!"),
                };

                // Place the chunk into the destination word.
                let dst_offset = src_bit - start_bit;
                dst_word |= chunk_as_word << (dst_offset as u32);

                // Update counters.
                remaining -= available;
                src_bit += available;
            }

            // Store the constructed destination word into the appropriate location.
            self.set_word(dst_word_index, dst_word);
        }
        self
    }

    /// Fills a bit-store by calling a function `f` for each bit index.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.copy_fn(|i| i % 2 == 0);
    /// assert_eq!(v.len(), 10);
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    fn copy_fn(&mut self, f: impl Fn(usize) -> bool) -> &mut Self {
        self.set_all(false);
        for i in 0..self.len() {
            if f(i) {
                self.set(i, true);
            }
        }
        self
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to fill the store with random bits.
    // ----------------------------------------------------------------------------------------------------------------

    /// Fills the store with random bits where each bit is set with probability `p`, and the RNG is seeded to `seed`.
    /// A seed of `0` indicates we should randomly seed the RNG.
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-vector with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.fill_random_biased_seeded(1.2, 42); // All bits set
    /// assert_eq!(v.count_ones(), 10);
    /// let mut u: BitVector = BitVector::zeros(10);
    /// u.fill_random_biased_seeded(0.5, 42); // Using same seed for u and v.
    /// v.fill_random_biased_seeded(0.5, 42);
    /// assert_eq!(u, v);
    /// ```
    fn fill_random_biased_seeded(&mut self, p: f64, seed: u64) -> &mut Self {
        // Note: Need `LazyLock` to make `TWO_POWER_64` `static` as `powi` is not `const`.
        static TWO_POWER_64: std::sync::LazyLock<f64> = std::sync::LazyLock::new(|| 2.0_f64.powi(64));

        if p <= 0.0 {
            return self.set_all(false);
        }
        if p >= 1.0 {
            return self.set_all(true);
        }

        // If given a non-zero seed we need to save and restore the old seed.
        let old_seed = rng::seed();
        if seed != 0 {
            rng::set_seed(seed);
        }

        // Scale p by 2^64 to remove floating point arithmetic from the main loop below.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let scaled_p = (*TWO_POWER_64 * p) as u64;

        // Start with all zeros and set each bit with probability `p`.
        self.set_all(false);
        for i in 0..self.len() {
            if rng::u64() < scaled_p {
                self.set(i, true);
            }
        }

        // Restore the old RNG seed.
        if seed != 0 {
            rng::set_seed(old_seed);
        }

        self
    }

    /// Fills the store with random bits where each bit is set with probability `p`, and the RNG is seeded using the
    /// system clock.
    ///
    /// # Note
    /// Probability `p` should be in the range `[0, 1]`. If `p` is outside this range, the function will return a
    /// bit-vector with all elements set or unset as appropriate.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// v.fill_random_biased(0.5);
    /// assert_eq!(v.len(), 10);
    /// v.fill_random_biased(1.2); // All bits set
    /// assert_eq!(v.count_ones(), 10);
    /// ```
    fn fill_random_biased(&mut self, p: f64) -> &mut Self { self.fill_random_biased_seeded(p, 0) }

    /// Fills the store with random bits where each bit is set with probability `0.5`, and the RNG is seeded to `seed`.
    /// A seed of `0` indicates we should randomly seed the RNG.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(10);
    /// let mut u: BitVector = BitVector::zeros(10);
    /// u.fill_random_seeded(42); // Using same seed for u and v.
    /// v.fill_random_seeded(42);
    /// assert_eq!(u, v);
    /// ```
    fn fill_random_seeded(&mut self, seed: u64) -> &mut Self { self.fill_random_biased_seeded(0.5, seed) }

    /// Fills the store with random bits where each bit is set with probability `0.5`, and the RNG is seeded using the
    /// clock.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut u: BitVector = BitVector::zeros(10);
    /// u.fill_random();
    /// assert_eq!(u.len(), 10);
    /// ```
    fn fill_random(&mut self) -> &mut Self { self.fill_random_biased_seeded(0.5, 0) }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to count the number of set and unset bits in the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns the number of set bits in the store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(12);
    /// assert_eq!(v.count_ones(), 0);
    /// v.set_all(true);
    /// assert_eq!(v.count_ones(), 12);
    /// ```
    fn count_ones(&self) -> usize {
        let mut count = 0;
        for i in 0..self.words() {
            count += self.word(i).count_ones() as usize;
        }
        count
    }

    /// Returns the number of unset bits in the store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(12);
    /// assert_eq!(v.count_zeros(), 12);
    /// v.set(1, true);
    /// assert_eq!(v.count_zeros(), 11);
    /// v.set_all(true);
    /// assert_eq!(v.count_zeros(), 0);
    /// ```
    fn count_zeros(&self) -> usize { self.len() - self.count_ones() }

    /// Returns the number of leading zeros in the store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(37);
    /// assert_eq!(v.leading_zeros(), 37);
    /// v.set(27, true);
    /// assert_eq!(v.leading_zeros(), 27);
    /// let v: BitVector = BitVector::ones(10);
    /// assert_eq!(v.leading_zeros(), 0, "v = {v} so expected leading zeros to be 0");
    /// let v: BitVector = BitVector::unit(3, 41);
    /// assert_eq!(v.leading_zeros(), 3);
    /// ```
    fn leading_zeros(&self) -> usize {
        // Note: Even if the last word is partially occupied, we know any unused bits are zeros.
        for i in 0..self.words() {
            let word = self.word(i);
            if word != Word::ZERO {
                return i * Word::UBITS + word.trailing_zeros() as usize;
            }
        }
        // Note: This handles the case where the bit-vector or slice is all zeros.
        self.len()
    }

    /// Returns the number of trailing zeros in the store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(27);
    /// assert_eq!(v.trailing_zeros(), 27);
    /// v.set(0, true);
    /// assert_eq!(v.trailing_zeros(), 26);
    /// ```
    fn trailing_zeros(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        // The last occupied word may have some unused bits that we need to subtract.
        let last_word = self.words() - 1;
        let unused_bits = Word::UBITS - (self.len() % Word::UBITS);
        for i in (0..=last_word).rev() {
            if self.word(i) != Word::ZERO {
                return (last_word - i) * Word::UBITS + self.word(i).leading_zeros() as usize - unused_bits;
            }
        }
        self.len()
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to find set bits in the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns the index of the first *set* bit in the store or `None` if no bits are set.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector<u8> = BitVector::zeros(37);
    /// assert_eq!(v.first_set(), None);
    /// v.set(2, true);
    /// assert_eq!(v.first_set(), Some(2));
    /// v.set(2, false);
    /// assert_eq!(v.first_set(), None);
    /// v.set(27, true);
    /// assert_eq!(v.first_set(), Some(27));
    /// v.clear();
    /// assert_eq!(v.first_set(), None);
    /// ```
    fn first_set(&self) -> Option<usize> {
        // Iterate forward looking for a word with a set bit and use the lowest of those ...
        // Remember that any unused bits in the final word are guaranteed to be unset.
        for i in 0..self.words() {
            let word = self.word(i);
            if let Some(loc) = word.lowest_set_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    /// Returns the index of the last *set* bit in the store or `None` if no bits are set.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(37);
    /// assert_eq!(v.last_set(), None);
    /// v.set(2, true);
    /// assert_eq!(v.last_set(), Some(2));
    /// v.set(2, false);
    /// assert_eq!(v.last_set(), None);
    /// v.set(27, true);
    /// assert_eq!(v.last_set(), Some(27));
    /// v.clear();
    /// assert_eq!(v.last_set(), None);
    /// ```
    fn last_set(&self) -> Option<usize> {
        // Iterate backwards looking for a word with a set bit and use the highest of those ...
        // Remember that any unused bits in the final word are guaranteed to be unset.
        for i in (0..self.words()).rev() {
            let word = self.word(i);
            if let Some(loc) = word.highest_set_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    /// Returns the index of the next set bit after `index` in the store or `None` if no more set bits exist.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(37);
    /// assert_eq!(v.next_set(0), None);
    /// v.set(2, true);
    /// v.set(27, true);
    /// assert_eq!(v.next_set(0), Some(2));
    /// assert_eq!(v.next_set(2), Some(27));
    /// assert_eq!(v.next_set(27), None);
    /// ```
    fn next_set(&self, index: usize) -> Option<usize> {
        // Start our search at index + 1.
        let index = index + 1;

        // Perhaps we are off the end? (This also handles the case of an empty type).
        if index >= self.len() {
            return None;
        }

        // Where is that starting index located in the word store?
        let (word_index, bit) = Word::index_and_offset(index);

        // Iterate forward looking for a word with a new set bit and use the lowest one ...
        for i in word_index..self.words() {
            let mut word = self.word(i);
            if i == word_index {
                // First word -- turn all the bits before our starting bit into zeros so we don't see them as set.
                word.reset_bits(0..bit);
            }
            if let Some(loc) = word.lowest_set_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    /// Returns the index of the previous set bit before `index` in the store or `None` if no previous set bits exist.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::zeros(37);
    /// assert_eq!(v.previous_set(0), None);
    /// v.set(2, true);
    /// v.set(27, true);
    /// assert_eq!(v.previous_set(37), Some(27));
    /// assert_eq!(v.previous_set(27), Some(2));
    /// assert_eq!(v.previous_set(2), None);
    /// ```
    fn previous_set(&self, index: usize) -> Option<usize> {
        // Edge case: If the type is empty or we are at the start, there are no previous set bits.
        if self.is_empty() || index == 0 {
            return None;
        }

        // Silently fix large indices and also adjust the index down a slot.
        let index = if index >= self.len() { self.len() - 1 } else { index - 1 };

        // Where is that starting index located in the word store?
        let (word_index, bit) = Word::index_and_offset(index);

        // Iterate backwards looking for a word with a *new* set bit and use the highest of those ...
        for i in (0..=word_index).rev() {
            let mut word = self.word(i);
            if i == word_index {
                // First word -- turn all higher bits after our starting bit into zeros so we don't see them as set.
                word.reset_bits(bit + 1..);
            }
            if let Some(loc) = word.highest_set_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to find unset bits in the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns the index of the first unset bit in the store or `None` if all bits are set.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector<u8> = BitVector::ones(39);
    /// assert_eq!(v.first_unset(), None);
    /// v.set(2, false);
    /// assert_eq!(v.first_unset(), Some(2));
    /// v.set(2, true);
    /// assert_eq!(v.first_unset(), None);
    /// v.set(27, false);
    /// assert_eq!(v.first_unset(), Some(27));
    /// v.clear();
    /// assert_eq!(v.first_unset(), None);
    /// ```
    fn first_unset(&self) -> Option<usize> {
        // Iterate forward looking for a word with an unset bit and use the lowest of those ...
        for i in 0..self.words() {
            let mut word = self.word(i);
            if i == self.words() - 1 {
                // Final word may have some unused zero bits that we need to replace with ones.
                let last_occupied_bit = Word::bit_offset(self.len() - 1);
                word.set_bits(last_occupied_bit + 1..);
            }
            if let Some(loc) = word.lowest_unset_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    /// Returns the index of the last unset bit in the store or `None` if all bits are set.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::ones(37);
    /// assert_eq!(v.last_unset(), None);
    /// v.set(2, false);
    /// assert_eq!(v.last_unset(), Some(2));
    /// v.set(2, true);
    /// assert_eq!(v.last_unset(), None);
    /// v.set(27, false);
    /// assert_eq!(v.last_unset(), Some(27));
    /// v.clear();
    /// assert_eq!(v.last_unset(), None);
    /// ```
    fn last_unset(&self) -> Option<usize> {
        // Iterate backwards looking for a word with an unset bit and use the highest of those ...
        for i in (0..self.words()).rev() {
            let mut word = self.word(i);
            if i == self.words() - 1 {
                // Final word may have some unused zero bits that we need to replace with ones.
                let last_occupied_bit = Word::bit_offset(self.len() - 1);
                word.set_bits(last_occupied_bit + 1..);
            }
            if let Some(loc) = word.highest_unset_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    /// Returns the index of the next unset bit after `index` in the store or `None` if no more set bits exist.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector<u8> = BitVector::ones(37);
    /// assert_eq!(v.next_unset(0), None);
    /// v.set(2, false);
    /// v.set(27, false);
    /// assert_eq!(v.next_unset(0), Some(2));
    /// assert_eq!(v.next_unset(2), Some(27));
    /// assert_eq!(v.next_unset(27), None);
    /// ```
    fn next_unset(&self, index: usize) -> Option<usize> {
        // Start our search at index + 1.
        let index = index + 1;

        // Perhaps we are off the end of the type (also handles the case of an empty type).
        if index >= self.len() {
            return None;
        }

        // Where is that starting index in the word store?
        let (word_index, bit) = Word::index_and_offset(index);

        // Iterate forward looking for a word with a new unset bit and use the lowest of those ...
        for i in word_index..self.words() {
            let mut word = self.word(i);
            if i == word_index {
                // Current word -- turn all the bits before our starting bit into ones so we don't see them as unset.
                word.set_bits(0..bit);
            }
            if i == self.words() - 1 {
                // Final word may have some unused zero bits that we need to replace with ones.
                let last_occupied_bit = Word::bit_offset(self.len() - 1);
                word.set_bits(last_occupied_bit + 1..);
            }
            if let Some(loc) = word.lowest_unset_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    /// Returns the index of the previous unset bit before `index` in the store or `None` if no more set bits exist.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector<u8> = BitVector::ones(37);
    /// assert_eq!(v.previous_unset(0), None);
    /// v.set(2, false);
    /// v.set(27, false);
    /// assert_eq!(v.previous_unset(37), Some(27));
    /// assert_eq!(v.previous_unset(27), Some(2));
    /// assert_eq!(v.previous_unset(2), None);
    /// ```
    fn previous_unset(&self, index: usize) -> Option<usize> {
        // Edge case: If the type is empty or we are at the start, there are no previous set bits.
        if self.is_empty() || index == 0 {
            return None;
        }

        // Silently fix large indices and also adjust the index down a slot.
        let index = if index >= self.len() { self.len() - 1 } else { index - 1 };

        // Where is that starting index in the word store?
        let (word_index, bit) = Word::index_and_offset(index);

        // Iterate backwards looking for a word with a new *unset* bit and use the highest of those ...
        for i in (0..=word_index).rev() {
            let mut word = self.word(i);
            if i == word_index {
                // Current word -- turn all higher bits after our starting bit into ones so we don't see them as unset.
                word.set_bits(bit + 1..);
            }
            if i == self.words() - 1 {
                // Final word may have some unused high order zero bits that we need to replace with ones.
                let last_occupied_bit = Word::bit_offset(self.len() - 1);
                word.set_bits(last_occupied_bit + 1..);
            }
            if let Some(loc) = word.highest_unset_bit() {
                return Some(i * Word::UBITS + loc as usize);
            }
        }
        None
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to create iterators over the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns an iterator over all the bits in the bit-store with the associated type `bool`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::ones(10);
    /// v.set(5, false);
    /// let bits: Vec<bool> = v.bits().collect();
    /// assert_eq!(bits, vec![true, true, true, true, true, false, true, true, true, true]);
    /// ```
    #[inline]
    fn bits(&self) -> Bits<'_, Self, Word> { Bits::new(self) }

    /// Returns an iterator over all the bits in the bit-store that are set to `true`. The associated type is `usize`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// type BV = BitVector<u8>;
    /// let v = BV::ones(10);
    /// let set_indices: Vec<usize> = v.set_bits().collect();
    /// assert_eq!(set_indices, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    #[inline]
    fn set_bits(&self) -> SetBits<'_, Self, Word> { SetBits::new(self) }

    /// Returns an iterator over all the bits in the bit-store that are set to `false`. The associated type is `usize`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// type BV = BitVector<u8>;
    /// let v = BV::zeros(10);
    /// let unset_indices: Vec<usize> = v.unset_bits().collect();
    /// assert_eq!(unset_indices, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    #[inline]
    fn unset_bits(&self) -> UnsetBits<'_, Self, Word> { UnsetBits::new(self) }

    /// Returns an iterator over the "words" in the bit-store with some [`Unsigned`] associated type.
    ///
    /// # Note
    /// This *behaves as if* the bits were copied into a vector of `Unsigned` words starting at bit 0 of word 0.
    /// The iterator returns the words from that vector in order.
    ///
    /// The final `Unsigned` word may not be fully occupied but any unused bits will be zeros.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector<u8> = BitVector::ones(10);
    /// let words: Vec<u8> = v.store_words().collect();
    /// assert_eq!(words, vec![0b1111_1111_u8, 0b0000_0011_u8]);
    /// ```
    #[inline]
    fn store_words(&self) -> Words<'_, Self, Word> { Words::new(self) }

    /// Returns a copy of the words underlying this bit-store.
    ///
    /// # Note
    /// The last word in the vector may not be fully occupied but unused slots will be all zeros.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector<u8> = BitVector::ones(10);
    /// let words = v.to_words();
    /// assert_eq!(words, vec!(255, 3));
    /// ```
    #[inline]
    fn to_words(&self) -> Vec<Word> { self.store_words().collect() }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to create slices of the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns a [`BitSlice`] of this store for the bits in the half-open range `[range.start, range.end)`.
    ///
    /// # Panics
    /// This method panics if `self` is empty or if the range is not valid.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// let s1 = v.slice(1..5);
    /// assert_eq!(s1.to_string(), "0101");
    /// ```
    fn slice<R: RangeBounds<usize>>(&self, range: R) -> BitSlice<'_, Word> {
        let (start, end) = self.start_and_end_for(range);
        BitSlice::new(self.store(), start, end)
    }

    /// Returns a mutable [`BitSlice`] of this store for the bits in the half-open range `[range.start, range.end)`.
    ///
    /// # Panics
    /// This method panics if `self` is empty or if the range is not valid.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector = BitVector::alternating(10);
    /// let mut slice = v.slice_mut(1..5);
    /// assert_eq!(slice.to_string(), "0101");
    /// ```
    fn slice_mut<R: RangeBounds<usize>>(&mut self, range: R) -> BitSlice<'_, Word> {
        let (start, end) = self.start_and_end_for(range);
        BitSlice::new_mut(self.store_mut(), start, end)
    }

    /// Helper method: Consumes a range and returns the corresponding `start` and `end` as a pair of `usize`s.
    ///
    /// The bits of interest are in the half-open interval `[start, end)`
    ///
    /// # Panics
    /// Panics if the vector of `words` is empty or the `range` is invalid. <br>
    /// The `range` cannot extend beyond the last bit of the vector of `words`.
    fn start_and_end_for<R: RangeBounds<usize>>(&self, range: R) -> (usize, usize) {
        assert!(!self.is_empty(), "cannot create a bit-slice from an empty bit-vector");

        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => *start + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(end) => *end + 1,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => self.len(),
        };

        // Check that the range is non-empty and does not extend beyond the end of the vector of words.
        assert!(start < end, "bit range [{start}, {end}) is invalid");
        assert!(end <= self.len(), "bit range extends beyond the end of the type");

        (start, end)
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods for sub-vector extraction.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns a *clone* of the elements in the half-open range `[begin, end)` as a new bit-vector.
    ///
    /// # Panics
    /// This method panics if the range is not valid.
    ///
    /// # Example
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// let mut s = v.sub(1..5);
    /// assert_eq!(s.to_string(), "0101");
    /// s.set_all(true);
    /// assert_eq!(s.to_string(), "1111");
    /// assert_eq!(v.to_string(), "1010101010");
    /// ```
    fn sub<R: RangeBounds<usize>>(&self, range: R) -> BitVector<Word> { BitVector::from_store(&self.slice(range)) }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods that split the store into two parts.
    // ----------------------------------------------------------------------------------------------------------------

    /// Splits a bit-store into two parts at the given index.
    // The parts are stored in the bit-vectors `left` and `right`.
    ///
    /// On return, `left` contains the bits from the start of the bit-store up to but not including `at` and `right`
    /// contains the bits from `at` to the end of the bit-store. The `self` bit-store is not modified.
    ///
    /// # Note
    /// This allows one to reuse the `left` and `right` outputs without having to allocate new bit-vectors.
    /// This is useful when implementing iterative algorithms that need to split a bit-store into two parts repeatedly.
    ///
    /// # Panics
    /// This method panics if the split point is beyond the end of the bit-store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// let mut left: BitVector = BitVector::new();
    /// let mut right: BitVector = BitVector::new();
    /// v.split_at_into(5, &mut left, &mut right);
    /// assert_eq!(left.to_string(), "10101");
    /// assert_eq!(right.to_string(), "01010");
    /// ```
    fn split_at_into(&self, at: usize, left: &mut BitVector<Word>, right: &mut BitVector<Word>) {
        assert!(at <= self.len(), "split point {at} is beyond the end of the bit-vector");
        left.clear();
        right.clear();
        left.append_store(&self.slice(0..at));
        right.append_store(&self.slice(at..self.len()));
    }

    /// Splits a bit-store into two parts at the given index. The parts are returned as a pair of new bit-vectors.
    ///
    /// On return, `left` contains the bits from the start of the bit-store up to but not including `at` and `right`
    /// contains the bits from `at` to the end of the bit-store. The `self` bit-store is not modified.
    ///
    /// # Panics
    /// This method panics if the split point is beyond the end of the bit-store.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// let (left, right) = v.split_at(5);
    /// assert_eq!(left.to_string(), "10101");
    /// assert_eq!(right.to_string(), "01010");
    /// ```
    fn split_at(&self, at: usize) -> (BitVector<Word>, BitVector<Word>) {
        let mut left = BitVector::new();
        let mut right = BitVector::new();
        self.split_at_into(at, &mut left, &mut right);
        (left, right)
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods tto return copies of the store interleaved with zeros.
    // ----------------------------------------------------------------------------------------------------------------

    /// Riffle this bit-vector into another bit-vector `dst` with the bits in the original vector interleaved with
    /// zeros.
    ///
    /// If this bit-vector in binary is `abcde` then the destination bit-vector will have the elements `a0b0c0d0e`.
    /// Note there is no last `0` bit in the destination bit-vector here.
    ///
    /// # Note
    /// This method is useful in various repeated squaring algorithms where we want to re-use the same `dst` bit-vector
    /// for each iteration.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v: BitVector<u8> = BitVector::ones(10);
    /// let mut dst: BitVector<u8> = BitVector::zeros(10);
    /// v.riffled_into(&mut dst);
    /// assert_eq!(dst.to_string(), "1010101010101010101");
    /// ```
    fn riffled_into(&self, dst: &mut BitVector<Word>) {
        // Not a lot to do if the bit-vector is empty or has only one bit. With two bits `ab` we return `a0b`.
        let ln = self.len();
        if ln < 2 {
            dst.resize(ln);
            dst.copy_store(self);
            return;
        }

        // Make sure `dst` is large enough to hold the riffled bits (a bit too big but we fix that below).
        dst.resize(2 * ln);
        let dst_words = dst.words();

        // Riffle each word in the store into two adjacent words in `dst`.
        for i in 0..self.words() {
            let (lo, hi) = self.word(i).riffle();
            dst.set_word(2 * i, lo);

            // Note that `hi` may be completely superfluous ...
            if 2 * i + 1 < dst_words {
                dst.set_word(2 * i + 1, hi);
            }
        }

        // If this bit-store was say `abcde` then `dst` will now be `a0b0c0d0e0`. Pop the last 0.
        dst.pop();
    }

    /// Returns a new bit-vector that is the result of riffling the bits in this bit-store with zeros.
    ///
    /// If this bit-store in binary is say `abcde` then the output bit-vector will have the elements `a0b0c0d0e`.
    /// Note there is no last `0` bit in the output bit-vector here.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector<u8> = BitVector::ones(10);
    /// let dst = v.riffled();
    /// assert_eq!(dst.to_string(), "1010101010101010101");
    /// ```
    #[must_use]
    fn riffled(&self) -> BitVector<Word> {
        let mut dst = BitVector::new();
        self.riffled_into(&mut dst);
        dst
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods for vector-vector dot products and convolutions.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns the scalar dot product of this bit-store and another bit-store.
    ///
    /// For any pair of vector-like types, the dot product is the "sum" of the element-wise products of the two
    /// operands. In GF(2) the sum is modulo 2 and, by convention, the scalar output is a boolean value.
    /// ```text
    /// u * v = \sum_i u[i] v[i]
    /// ```
    /// where the sum is over all the indices so the two operands must have the same length.
    ///
    /// # Note
    /// - We have also implemented the `Mul` trait to overload the `*` operator to return the same thing.
    /// - See the `BitMatrix` documentation for matrix-vector, vector-matrix, and matrix-matrix multiplication.
    ///
    /// # Panics
    /// In debug mode, panics if the lengths of `self` and `rhs` do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: BitVector = BitVector::alternating(10);
    /// let v2: BitVector = BitVector::alternating(10) >> 1;
    /// assert_eq!(v1.dot(&v1), true);
    /// assert_eq!(v1.dot(&v2), false);
    /// ```
    fn dot<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> bool {
        debug_assert_eq!(self.len(), rhs.len(), "Length mismatch {} != {}", self.len(), rhs.len());
        let mut sum = Word::ZERO;
        for i in 0..self.words() {
            sum ^= self.word(i) & rhs.word(i);
        }
        sum.count_ones() % 2 == 1
    }

    /// Returns the convolution of this bit-store and another bit-store as a new bit-vector.
    ///
    /// The *convolution* of any two vector-like objects `u` & `v` is defined as:
    ///
    /// ```text
    ///   (u * v)[k] = \sum_j u[j] v[k-j+1]
    /// ```
    ///
    /// where the sum is taken over all `j` such that the indices in the formula are valid.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let lhs: BitVector = BitVector::ones(3);
    /// let rhs: BitVector = BitVector::ones(2);
    /// let result = lhs.convolved_with(&rhs);
    /// assert_eq!(result.to_string(), "1001");
    /// ```
    fn convolved_with<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        // Edge case: if either vector is empty then the convolution is empty.
        if self.is_empty() || rhs.is_empty() {
            return BitVector::new();
        }

        // Generally the result will have length `self.len() + other.len() - 1` (could be all zeros).
        let mut result = BitVector::zeros(self.len() + rhs.len() - 1);

        // If either vector is all zeros then the convolution is all zeros.
        if self.none() || rhs.first_set().is_none() {
            return result;
        }

        // Only need to consider words in `rhs` up to and including the one holding its final set bit.
        // We have already checked that `rhs` is not all zeros so we know there is a last set bit!
        let rhs_words_end = Word::word_index(rhs.last_set().unwrap()) + 1;

        // Initialize `result` by copying the live words from `rhs`
        for i in 0..rhs_words_end {
            result.set_word(i, rhs.word(i));
        }

        // Work backwards from our last set bit (which we know exists as we checked `self` is not all zeros).
        for i in (0..self.last_set().unwrap()).rev() {
            let mut prev = Word::ZERO;
            for j in 0..result.words() {
                let left = prev >> (Word::UBITS - 1);
                prev = result.word(j);
                result.set_word(j, prev << 1_u32 | left);
            }
            if self.get(i) {
                for j in 0..rhs_words_end {
                    result.set_word(j, result.word(j) ^ rhs.word(j));
                }
            }
        }
        result
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated string representation methods.
    // ----------------------------------------------------------------------------------------------------------------

    /// Returns the "binary" string representation of the bits in the bit-store.
    ///
    /// The output is a string of 0's and 1's without any spaces, commas, or other formatting.
    ///
    /// # Note
    /// The output is in *vector-order* e.g. `v_0v_1v_2v_3...` where `v_0` is the first element in the vector/slice.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// assert_eq!(v.to_binary_string(), "1010101010");
    /// ```
    fn to_binary_string(&self) -> String { self.to_custom_binary_string("", "", "") }

    /// Returns the "pretty" string representation of the bits in the bit-store.
    ///
    /// The output is a string of 0's and 1's with spaces between each bit, and the whole thing enclosed in square
    /// brackets.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// assert_eq!(v.to_pretty_string(), "[1 0 1 0 1 0 1 0 1 0]");
    /// let v: BitVector = BitVector::new();
    /// assert_eq!(v.to_pretty_string(), "[]");
    /// ```
    fn to_pretty_string(&self) -> String { self.to_custom_binary_string(" ", "[", "]") }

    /// Returns a customised "binary" string representation of the bits in the bit-store.
    ///
    /// The elements are output as 0's and 1's with a custom `separator` between them.
    /// You can also provide custom `left` and `right` delimiters for the whole vector or slice.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(10);
    /// assert_eq!(v.to_custom_binary_string(" ", "[", "]"), "[1 0 1 0 1 0 1 0 1 0]");
    fn to_custom_binary_string(&self, separator: &str, left: &str, right: &str) -> String {
        // Edge case: No elements in the type, return the empty string.
        if self.is_empty() {
            return format!("{left}{right}");
        }

        // Start with the pure binary string representation.
        // We preallocate space for that string -- generally this is a little more than strictly necessary.
        let n_words = self.words();
        let mut binary_string = String::with_capacity(n_words * Word::UBITS);

        // Reverse each word to vector-order and get its binary string representation.
        for i in 0..n_words {
            let word = self.word(i).reverse_bits();
            write!(binary_string, "{:0width$b}", word, width = Word::UBITS).unwrap();
        }

        // The last block may not be fully occupied and padded with spurious zeros so we truncate to the correct length.
        binary_string.truncate(self.len());

        // If we were given a custom separator, add it between the elements ...
        let str = if separator.is_empty() {
            binary_string
        }
        else {
            binary_string.chars().fold(String::new(), |mut acc, c| {
                if !acc.is_empty() {
                    acc.push_str(separator);
                }
                acc.push(c);
                acc
            })
        };

        // Add any custom left and right delimiters ...
        let mut result = String::with_capacity(str.len() + left.len() + right.len());
        write!(result, "{left}{str}{right}").unwrap();
        result
    }

    /// Returns the "hex" string representation of the bits in the bit-store.
    ///
    /// The output is a string of hex characters without any spaces, commas, or other formatting. <br>
    /// The string may have a two character *suffix* of the form ".base" where `base` is one of 2, 4 or 8. <br>
    /// All hex characters encode 4 bits: "0X0" -> `0b0000`, "0X1" -> `0b0001`, ..., "0XF" -> `0b1111`. <br>
    /// The three possible ".base" suffixes allow for bit-vectors whose length is not a multiple of 4. <br>
    /// Empty bit-vectors are represented as the empty string.
    ///
    /// - `0X1`   is the hex representation of the bit-vector `0001` => length 4.
    /// - `0X1.8` is the hex representation of the bit-vector `001`  => length 3.
    /// - `0X1.4` is the hex representation of the bit-vector `01`   => length 2.
    /// - `0X1.2` is the hex representation of the bit-vector `1`    => length 1.
    ///
    /// # Note
    /// The output is in *vector-order*.
    /// If "h0" is the first hex digit in the output string, you can print it as four binary digits `v_0v_1v_2v_3`. <br>
    /// For example, if h0 = "A" which is `1010` in binary, then v = 1010.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::new();
    /// assert_eq!(v.to_hex_string(), "");
    /// let v: BitVector = BitVector::ones(4);
    /// assert_eq!(v.to_hex_string(), "F");
    /// let v: BitVector = BitVector::ones(5);
    /// assert_eq!(v.to_hex_string(), "F1.2");
    /// let v: BitVector = BitVector::alternating(8);
    /// assert_eq!(v.to_hex_string(), "AA");
    /// ```
    fn to_hex_string(&self) -> String {
        // Edge case: No bits in the type, return the empty string.
        if self.is_empty() {
            return String::new();
        }

        // The number of digits in the output string. Generally hexadecimal but the last may be to a lower base.
        let len = self.len();
        let digits = len.div_ceil(4);

        // Preallocate space allowing for a possible lower base on the last digit such as "_2".
        let mut result = String::with_capacity(digits + 2);

        // The number of hex digits per word.
        let hex_digits_per_word = Word::UBITS / 4;

        // Reverse each word to vector-order and get its hex string rep (fully padded with zeros to the left).
        for i in 0..self.words() {
            let word = self.word(i).reverse_bits();
            write!(result, "{word:0hex_digits_per_word$X}").unwrap();
        }

        // Last word may not be fully occupied and padded with spurious zeros so we truncate the output string.
        result.truncate(digits);

        // Every four elements in the bit-vector is encoded by a single hex digit but `len` may not be a multiple of 4.
        let k = len % 4;
        if k != 0 {
            // That last hex digit should really be encoded to a lower base -- 2, 4 or 8.
            // We compute the number represented by the trailing `k` elements in the bit-vector.
            let mut num = 0;
            for i in 0..k {
                if self.get(len - 1 - i) {
                    num |= 1 << i;
                }
            }

            // Convert that number to hex & use it to *replace* the last hex digit in our `result` string.
            let result_len = result.len();
            result.truncate(result_len - 1);
            write!(result, "{num:X}").unwrap();

            // Append the appropriate base to the output string so that the last digit can be interpreted properly.
            write!(result, ".{}", 1 << k).unwrap();
        }
        result
    }

    /// Returns a multi-line string describing the bit-store in some detail.
    ///
    /// # Note
    /// This method is useful for debugging and testing but you should not rely on the output format which may change.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v: BitVector = BitVector::alternating(20);
    /// println!("{}", v.describe());
    /// ```
    fn describe(&self) -> String {
        let mut result = String::new();
        writeln!(result, "binary format:         {}", self.to_binary_string()).unwrap();
        writeln!(result, "hex format:            {}", self.to_hex_string()).unwrap();
        writeln!(result, "number of bits:        {}", self.len()).unwrap();
        writeln!(result, "number of set bits:    {}", self.count_ones()).unwrap();
        writeln!(result, "number of unset bits:  {}", self.count_zeros()).unwrap();
        writeln!(result, "bits per word:         {}", Word::UBITS).unwrap();
        writeln!(result, "word count:            {}", self.words()).unwrap();
        writeln!(result, "words in hex:         [").unwrap();
        for i in 0..self.words() {
            writeln!(result, "  {:0width$X}", self.word(i), width = Word::UBITS / 4).unwrap();
        }
        writeln!(result, "]").unwrap();
        result
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to perform bit-shifts of the store.
    // ----------------------------------------------------------------------------------------------------------------

    /// Shifts all bits in the bit-store to the left by `shift` places.
    ///
    /// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v.left_shift(1)` is `[v1,v2,v3,0]`.
    /// New bits entering from the right are set to zero.
    ///
    /// # Note
    /// - Left shifting in vector-order is the same as right shifting in bit-order.
    /// - Only accessible bits are affected by the shift.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: gf2::BitVector = gf2::BitVector::ones(10);
    /// bv.left_shift(3);
    /// assert_eq!(bv.to_string(), "1111111000");
    /// ```
    fn left_shift(&mut self, shift: usize) {
        // Edge cases where there is nothing to do.
        if shift == 0 || self.len() == 0 {
            return;
        }

        // Perhaps we have shifted all the contents out and we are left with all zeros.
        if shift >= self.len() {
            self.set_all(false);
            return;
        }

        // For larger shifts, we can efficiently shift by whole words first.
        let word_shift = shift / Word::UBITS;
        let end_word = self.words() - word_shift;

        // Do the whole word shifts first, pushing in zero words to fill the empty slots.
        let mut shift = shift;
        if word_shift > 0 {
            // Shift the words.
            for i in 0..end_word {
                self.set_word(i, self.word(i + word_shift));
            }

            // Fill in the high order words with zeros.
            for i in end_word..self.words() {
                self.set_word(i, Word::ZERO);
            }

            // How many bits are left to shift?
            shift -= word_shift * Word::UBITS;
        }

        // Perhaps there are some partial word shifts left to do.
        if shift != 0 {
            // Do the "interior" words where the shift moves bits from one word to the next.
            if end_word > 0 {
                let shift_complement = Word::UBITS - shift;
                for i in 0..end_word - 1 {
                    let lo = self.word(i) >> shift;
                    let hi = self.word(i + 1) << shift_complement;
                    self.set_word(i, lo | hi);
                }
            }

            // Do the last word.
            let value = self.word(end_word - 1);
            self.set_word(end_word - 1, value >> shift);
        }
    }

    /// Returns a new bit-vector that is the result of left-shifting this bit-store by `shift` places.
    ///
    /// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v.left_shifted(1)` returns `[v1,v2,v3,0]`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::ones(10);
    /// let v2 = v1.left_shifted(3);
    /// assert_eq!(v1.to_string(), "1111111111");
    /// assert_eq!(v2.to_string(), "1111111000");
    /// let s2 = v2.slice(0..7);
    /// assert_eq!(s2.to_string(), "1111111");
    /// let v3 = s2.left_shifted(3);
    /// assert_eq!(v3.to_string(), "1111000");
    /// assert_eq!(s2.to_string(), "1111111");
    /// ```
    fn left_shifted(&self, shift: usize) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.left_shift(shift);
        result
    }

    /// Shifts all bits in the bit-store to the right by `shift` places.
    ///
    /// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v.right_shift(1)` is `[0,v0,v1,v2]`.
    /// New bits entering from the right are set to zero.
    ///
    /// # Note
    /// - Right shifting in vector-order is the same as left shifting in bit-order.
    /// - Only accessible bits are affected by the shift.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: gf2::BitVector = gf2::BitVector::ones(10);
    /// bv.right_shift(3);
    /// assert_eq!(bv.to_string(), "0001111111");
    /// ```
    fn right_shift(&mut self, shift: usize) {
        // Edge cases where there is nothing to do.
        if shift == 0 || self.len() == 0 {
            return;
        }

        // Perhaps we have shifted all the contents out and we are left with all zeros.
        if shift >= self.len() {
            self.set_all(false);
            return;
        }

        // For larger shifts, we can efficiently shift by whole words first.
        let word_shift = shift / Word::UBITS;

        // Do the whole word shifts first, pushing in zero words to fill the empty slots.
        let mut shift = shift;
        if word_shift > 0 {
            // Shift whole words -- starting at the end of the vector.
            for i in (word_shift..self.words()).rev() {
                self.set_word(i, self.word(i - word_shift));
            }

            // Fill in the low order words with zeros.
            for i in 0..word_shift {
                self.set_word(i, Word::ZERO);
            }

            // How many bits are left to shift?
            shift -= word_shift * Word::UBITS;
        }

        // Perhaps there are some partial word shifts left to do.
        if shift != 0 {
            // Do the "interior" words where the shift moves bits from one word to the next.
            let shift_complement = Word::UBITS - shift;
            for i in (word_shift + 1..self.words()).rev() {
                let lo = self.word(i - 1) >> shift_complement;
                let hi = self.word(i) << shift;
                self.set_word(i, lo | hi);
            }

            // Do the "first" word.
            let value = self.word(word_shift);
            self.set_word(word_shift, value << shift);
        }
    }

    /// Returns a new bit-vector that is the result of right-shifting this bit-store by `shift` places.
    ///
    /// Shifting is in *vector-order* so if `v = [v0,v1,v2,v3]` then `v.left_shifted(1)` returns `[v1,v2,v3,0]`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::ones(10);
    /// let v2 = v1.right_shifted(3);
    /// assert_eq!(v1.to_string(), "1111111111");
    /// assert_eq!(v2.to_string(), "0001111111");
    /// let s2 = v2.slice(0..7);
    /// assert_eq!(s2.to_string(), "0001111");
    /// let v3 = s2.right_shifted(3);
    /// assert_eq!(v3.to_string(), "0000001");
    /// assert_eq!(s2.to_string(), "0001111");
    /// ```
    fn right_shifted(&self, shift: usize) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.right_shift(shift);
        result
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated methods to perform bitwise operations between stores.
    // ----------------------------------------------------------------------------------------------------------------

    /// Performs an in-place bitwise XOR of this bit-store with another bit-store.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// v1.xor_eq(&v2);
    /// assert_eq!(v1.to_string(), "1111111111");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// ```
    fn xor_eq<Rhs: BitStore<Word>>(&mut self, rhs: &Rhs) {
        assert_eq!(self.len(), rhs.len(), "Length mismatch {} != {}", self.len(), rhs.len());
        for i in 0..self.words() {
            let word = self.word(i) ^ rhs.word(i);
            self.set_word(i, word);
        }
    }

    /// Returns a new bit-vector that is the result of XOR'ing this bit-store with another.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// let v3 = v1.xor(&v2);
    /// assert_eq!(v1.to_string(), "1010101010");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// assert_eq!(v3.to_string(), "1111111111");
    /// ```
    fn xor<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.xor_eq(rhs);
        result
    }

    /// Performs an in-place bitwise AND of this bit-store with another bit-store.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// v1.and_eq(&v2);
    /// assert_eq!(v1.to_string(), "0000000000");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// ```
    fn and_eq<Rhs: BitStore<Word>>(&mut self, rhs: &Rhs) {
        assert_eq!(self.len(), rhs.len(), "Length mismatch {} != {}", self.len(), rhs.len());
        for i in 0..self.words() {
            let word = self.word(i) & rhs.word(i);
            self.set_word(i, word);
        }
    }

    /// Returns a new bit-vector that is the result of AND'ing this bit-store with another.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// let v3 = v1.and(&v2);
    /// assert_eq!(v1.to_string(), "1010101010");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// assert_eq!(v3.to_string(), "0000000000");
    /// ```
    fn and<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.and_eq(rhs);
        result
    }

    /// Performs an in-place bitwise OR of this bit-store with another bit-store.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// v1.or_eq(&v2);
    /// assert_eq!(v1.to_string(), "1111111111");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// ```
    fn or_eq<Rhs: BitStore<Word>>(&mut self, rhs: &Rhs) {
        assert_eq!(self.len(), rhs.len(), "Length mismatch {} != {}", self.len(), rhs.len());
        for i in 0..self.words() {
            let word = self.word(i) | rhs.word(i);
            self.set_word(i, word);
        }
    }

    /// Returns a new bit-vector that is the result of OR'ing this bit-store with another.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// let v3 = v1.or(&v2);
    /// assert_eq!(v1.to_string(), "1010101010");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// assert_eq!(v3.to_string(), "1111111111");
    /// ```
    fn or<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.or_eq(rhs);
        result
    }

    // ----------------------------------------------------------------------------------------------------------------
    // Associated arithmetic-operations-in-place methods.
    // ----------------------------------------------------------------------------------------------------------------

    /// Performs an in-place addition of this bit-store with another bit-store.
    ///
    /// # Note
    /// In GF(2), addition is equivalent to bitwise XOR.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// v1.plus_eq(&v2);
    /// assert_eq!(v1.to_string(), "1111111111");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// ```
    fn plus_eq<Rhs: BitStore<Word>>(&mut self, rhs: &Rhs) { self.xor_eq(rhs); }

    /// Returns a new bit-vector that is the result of adding this bit-store to another.
    ///
    /// # Note
    /// In GF(2), addition is equivalent to bitwise XOR.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// let v3 = v1.plus(&v2);
    /// assert_eq!(v1.to_string(), "1010101010");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// assert_eq!(v3.to_string(), "1111111111");
    /// ```
    fn plus<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.xor_eq(rhs);
        result
    }

    /// Performs an in-place subtraction of another bit-store from this bit-store.
    ///
    /// # Note
    /// In GF(2), addition and subtraction are both equivalent to bitwise XOR.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// v1.minus_eq(&v2);
    /// assert_eq!(v1.to_string(), "1111111111");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// ```
    fn minus_eq<Rhs: BitStore<Word>>(&mut self, rhs: &Rhs) { self.xor_eq(rhs); }

    /// Returns a new bit-vector that is the result of subtracting a bit-store from this one.
    ///
    /// # Note
    /// In GF(2), addition and subtraction are both equivalent to bitwise XOR.
    ///
    /// # Panics
    /// This method panics if the lengths of the input operands do not match.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let v1: gf2::BitVector = gf2::BitVector::from_string("1010101010").unwrap();
    /// let v2: gf2::BitVector = gf2::BitVector::from_string("0101010101").unwrap();
    /// let v3 = v1.minus(&v2);
    /// assert_eq!(v1.to_string(), "1010101010");
    /// assert_eq!(v2.to_string(), "0101010101");
    /// assert_eq!(v3.to_string(), "1111111111");
    /// ```
    fn minus<Rhs: BitStore<Word>>(&self, rhs: &Rhs) -> BitVector<Word> {
        let mut result: BitVector<Word> = BitVector::from_store(self);
        result.xor_eq(rhs);
        result
    }
}
