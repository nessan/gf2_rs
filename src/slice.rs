//! [`BitSlice`] is a _non-owing view_ into a range of contiguous bits --- a _bit-slice_.

// Crate traits.
use crate::{
    BitStore,
    Unsigned,
};

// --------------------------------------------------------------------------------------------------------------------
// The `BitSlicePtr` helper enum.
// --------------------------------------------------------------------------------------------------------------------

/// An enum to hold either a const or mutable raw pointer to the first underlying word in the bit-slice.
///
/// # Note
/// This is a unified pointer type that can be either const or mutable depending on whether the slice is mutable or not.
/// There are other ways to achieve this, (two distinct types `BitSlice` and `BitSliceMut`) or perhaps adding an extra
/// boolean template parameter to `BitSlice` (e.g. `BitSlice<Word, IsMutable = false>`).
///
/// On the whole, this is a simpler approach. It also matches the somewhat equivalent C++ `gf2::BitSpan` type.
#[derive(PartialEq, Eq)]
enum BitSlicePtr<T: Unsigned = usize> {
    Mutable(*mut T),
    Const(*const T),
}

// --------------------------------------------------------------------------------------------------------------------
// The `BitSlice` type.
// --------------------------------------------------------------------------------------------------------------------

#[doc = include_str!("../docs/slice.md")]
#[derive(PartialEq, Eq)]
pub struct BitSlice<'a, Word: Unsigned> {
    /// A pointer to the first *word* containing bits in the slice (it may be partially occupied).
    m_store: BitSlicePtr<Word>,

    /// `slice[0]` is at this bit offset in the word `m_store[0]`.
    m_offset: u32,

    /// The number of bits in the slice.
    m_len: usize,

    // We cache the *minimum* number of words needed to store the slice (this is `slice.words()`).
    m_words: usize,

    /// Phantom data to bind the lifetime to the backing storage.
    _marker: core::marker::PhantomData<&'a Word>,
}

/// `BitSlice` constructors.
impl<'a, Word: Unsigned> BitSlice<'a, Word> {
    /// Creates a `BitSlice` encompassing the *bits* in the range `[start, end)` from contiguous [`Unsigned`] words.
    ///
    /// # Panics
    /// In debug mode, panics if the array of `words` is empty or the range is invalid. <br>
    /// The range cannot extend beyond the last bit of the array of `words`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let words = vec![0b10101010_u8, 0b11001100_u8];
    /// let slice = gf2::BitSlice::new(&words, 0, 16);
    /// assert_eq!(slice.to_binary_string(), "0101010100110011");
    /// ```
    #[inline]
    pub fn new(words: &'a [Word], start: usize, end: usize) -> Self {
        debug_assert!(!words.is_empty(), "cannot create a bit-slice from an empty vector of words");
        debug_assert!(start < end, "start: {start} should be <  end: {end}");
        debug_assert!(end <= words.len() * Word::UBITS, "bit range extends beyond the end of the vector of words");

        // The length of the slice and the minimum number of words needed to store the slice.
        let m_len = end - start;
        let m_words = Word::words_needed(m_len);

        // Location of the first element in the slice.
        let (m_store, m_offset) = Word::index_and_offset(start);

        // Create the slice.
        Self {
            m_store: BitSlicePtr::Const(unsafe { words.as_ptr().add(m_store) }),
            m_offset,
            m_len,
            m_words,
            _marker: core::marker::PhantomData,
        }
    }

    /// Creates a _mutable_ `BitSlice` encompassing the *bits* in the range `[start, end)` from contiguous mutable
    /// [`Unsigned`] words.
    ///
    /// # Panics
    /// In debug mode, panics if the array of `words` is empty or the range is invalid. <br>
    /// The range cannot extend beyond the last bit of the array of `words`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let words = vec![0b10101010_u8, 0b11001100_u8];
    /// let slice = gf2::BitSlice::new(&words, 0, 16);
    /// assert_eq!(slice.to_binary_string(), "0101010100110011");
    /// ```
    #[inline]
    pub fn new_mut(words: &'a mut [Word], start: usize, end: usize) -> Self {
        debug_assert!(!words.is_empty(), "cannot create a bit-slice from an empty vector of words");
        debug_assert!(start < end, "start: {start} should be <  end: {end}");
        debug_assert!(end <= words.len() * Word::UBITS, "bit range extends beyond the end of the vector of words");

        let m_len = end - start;
        let m_words = Word::words_needed(m_len);
        let (m_store, m_offset) = Word::index_and_offset(start);

        Self {
            m_store: BitSlicePtr::Mutable(unsafe { words.as_mut_ptr().add(m_store) }),
            m_offset,
            m_len,
            m_words,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<Word: Unsigned> BitStore<Word> for BitSlice<'_, Word> {
    /// Returns the number of bits in the slice.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let words = vec![0b10101010_u8, 0b11001100_u8];
    /// let slice = gf2::BitSlice::new(&words, 1, 11);
    /// assert_eq!(slice.len(), 10);
    /// ```
    #[inline]
    fn len(&self) -> usize { self.m_len }

    /// Returns the data underlying the `BitSLice` as a slice of [`Unsigned`] words.
    fn store(&self) -> &[Word] {
        let backing_words = Word::words_needed(self.m_len + self.m_offset as usize);
        unsafe {
            match self.m_store {
                BitSlicePtr::Const(ptr) => std::slice::from_raw_parts(ptr, backing_words),
                BitSlicePtr::Mutable(ptr) => std::slice::from_raw_parts(ptr.cast_const(), backing_words),
            }
        }
    }

    /// Returns the data underlying the `BitSlice` as a mutable slice of [`Unsigned`] words.
    fn store_mut(&mut self) -> &mut [Word] {
        let backing_words = Word::words_needed(self.m_len + self.m_offset as usize);
        unsafe {
            match self.m_store {
                BitSlicePtr::Mutable(ptr) => std::slice::from_raw_parts_mut(ptr, backing_words),
                BitSlicePtr::Const(_) => panic!("cannot mutably access data of immutable BitSlice"),
            }
        }
    }

    /// Returns the offset (in bits) of the first bit element in the `BitSlice` within the first [`Unsigned`] word.
    fn offset(&self) -> u32 { self.m_offset }

    /// Returns the number of [`Unsigned`] words needed to store the accessible bits in the slice.
    ///
    /// # Note
    /// This may not be the same as the number of words in the underlying vector of `Unsigned` words. <br>
    /// In particular, slices need not be aligned to the underlying word-boundaries.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let words = vec![0b10101010_u8, 0b11001100_u8];
    /// let slice = gf2::BitSlice::new(&words, 0, 16);
    /// assert_eq!(slice.words(), 2);
    /// ```
    #[inline]
    fn words(&self) -> usize { self.m_words }

    /// Synthesizes slice "word" `i` from the underlying store.
    ///
    /// # Note
    /// Slices need not be aligned to the word boundaries of the underlying store.
    /// To synthesize "word" `i` we may use two contiguous words from the underlying store, `u0` and `u1`.
    /// Here `u0 = self.underlying[i]` and `u1 = self.underlying[i+1]`.
    ///
    /// We copy `u0_bits` high-order bits from `u0` and they become the low-order bits in the slice word.
    /// We copy `u1_bits` low-order bits from `u1` and they become the high-order bits in the slice word.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the slice.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let words = vec![0b1010_1010_u8, 0b1100_1100_u8, 0b1111_1111_u8];
    /// let slice = gf2::BitSlice::new(&words, 0, 4);
    /// assert_eq!(slice.word(0), 0b0000_1010_u8);
    /// let slice = gf2::BitSlice::new(&words, 12, 16);
    /// assert_eq!(slice.word(0), 0b0000_1100_u8);
    /// let slice = gf2::BitSlice::new(&words, 4, 22);
    /// assert_eq!(slice.word(0), 0b11001010);
    /// assert_eq!(slice.word(1), 0b11111100);
    /// assert_eq!(slice.word(2), 0b00000011);
    /// ```
    #[inline]
    fn word(&self, i: usize) -> Word {
        // Slice word `i` is generally synthesized from two underlying words `u0` and `u1` (`u1` may not be needed).
        // The slice word will use `u0_bits` high-order bits from `u0` and `u1_bits` low-order bits from `u1`.
        let (u0_bits, u1_bits) = self.recipe_for_word(i);

        // The underlying words are located at the store indices `i` and `i + 1` respectively -- grab a pointer to `u0`.
        let u0_ptr: *const Word = match self.m_store {
            BitSlicePtr::Const(ptr) => unsafe { ptr.add(i) },
            BitSlicePtr::Mutable(ptr) => unsafe { ptr.add(i) },
        };

        // Our return value -- start with all unset bits.
        let mut result = Word::ZERO;

        // Replace `u0_bits` low-order bits in `result` with the same number of high-order bits from `u0`.
        // Shift the bits down to align them with the start of the slice.
        let u0 = unsafe { *u0_ptr };
        result.replace_bits(0..u0_bits, u0.unbounded_shr(self.m_offset));

        // Perhaps we need to fold in some bits from the next word?
        if u1_bits != 0 {
            // Grab the next word `u1`, shift it left by `u0_bits`, and fold those high bits into `result`.
            let u1 = unsafe { *u0_ptr.add(1) };
            result.replace_bits(u0_bits..u0_bits + u1_bits, u1.unbounded_shl(u0_bits));
        }
        result
    }

    /// Sets the [`Unsigned`] word at index `i` in the `BitSlice` to `word` & returns a reference to `self`.
    ///
    /// # Note
    /// This *acts as if* the view was copied into a vector of `Unsigned` words starting at bit 0 of word 0 and
    /// sets the word at index `i` in that vector, being careful to only set the bits that are within the view.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the slice.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut words = vec![0b0000_0000_u8, 0b0000_0000_u8];
    /// let mut slice = gf2::BitSlice::new_mut(&mut words, 5, 10);
    /// assert_eq!(slice.to_binary_string(), "00000", "expected 00000");
    /// slice.set_word(0, 0b1111_1111_u8);
    /// assert_eq!(slice.to_binary_string(), "11111", "expected 11111");
    /// assert_eq!(words, vec![0b1110_0000_u8, 0b0000_0011_u8]);
    /// ```
    #[inline]
    fn set_word(&mut self, i: usize, value: Word) {
        // Slice word `i` is generally synthesized from two underlying words `u0` and `u1` (`u1` may not be needed).
        // The slice word will use `u0_bits` high-order bits from `u0` and `u1_bits` low-order bits from `u1`.
        let (u0_bits, u1_bits) = self.recipe_for_word(i);

        // The underlying words are located at the store indices `i` and `i + 1` respectively -- grab a pointer to `u0`.
        let u0_ptr: *mut Word = match self.m_store {
            BitSlicePtr::Mutable(ptr) => unsafe { ptr.add(i) },
            BitSlicePtr::Const(_) => {
                panic!("This should never happen -- cannot set word on immutable slice");
            },
        };

        // Replace `u0_bits` bits starting at `start_offset` with the same number of low-order bits from `value`.
        // Shift the value to the left to align its bits with the start of the slice.
        let shift = self.m_offset;
        let u0 = unsafe { &mut *u0_ptr };
        u0.replace_bits(shift..shift + u0_bits, value.unbounded_shl(shift));

        // u1 (if needed)
        if u1_bits != 0 {
            let u1 = unsafe { &mut *u0_ptr.add(1) };
            u1.replace_bits(0..u1_bits, value.unbounded_shr(u0_bits));
        }
    }
}

impl<Word: Unsigned> BitSlice<'_, Word> {
    /// Private helper method that returns the "recipe" for synthesizing a slice word from the underlying store.
    ///
    /// # Note
    /// Slices need not be aligned to the word boundaries of the underlying store.
    /// To synthesize "word" `i` we may use two contiguous words from the underlying store, `u0` and `u1`.
    /// Here `u0 = self.underlying[i]` and `u1 = self.underlying[i+1]`.
    ///
    /// We copy `u0_bits` high-order bits from `u0` and they become the low-order bits in the slice word.
    /// We copy `u1_bits` low-order bits from `u1` and they become the high-order bits in the slice word.
    ///
    /// This "recipe" method returns the pair of `u32`s: `(u0_bits, u1_bits)`.
    ///
    /// # Panics
    /// In debug mode, panics if `i` is out of bounds for the slice.
    ///
    /// Note:
    /// The only tricky part is that the last slice word may not be fully occupied so we need to handle it
    /// differently from the others.
    #[inline]
    fn recipe_for_word(&self, i: usize) -> (u32, u32) {
        // In debug mode, panic if the index is out of bounds.
        debug_assert!(i < self.m_words, "Index {i} should be less than {}", self.m_words);

        // The default values for the recipe (these sum to a whole word of bits):
        let mut u0_bits = Word::BITS - self.m_offset;
        let mut u1_bits = self.m_offset;

        // The last slice word may not contain a full word of bits so we need to handle it differently.
        #[allow(clippy::cast_possible_truncation)]
        if i == self.m_words - 1 {
            let last = self.m_offset as usize + self.m_len - 1;
            let last_offset = (last % Word::UBITS) as u32;
            if last_offset < self.m_offset {
                // The last slice word still needs two words of the underlying vector but `u1_bits` may shrink.
                u1_bits = last_offset + 1;
            }
            else {
                // The last slice word fits inside a single word of the underlying vector & we don't need `u1`
                u0_bits = last_offset - self.m_offset + 1;
                u1_bits = 0;
            }
        }
        (u0_bits, u1_bits)
    }
}
