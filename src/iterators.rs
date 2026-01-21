//! [`Bits`], [`SetBits`], [`UnsetBits`], and [`Words`] iterators over any [`BitStore`].

use crate::{
    BitStore,
    Unsigned,
};

// Standard library imports.
use std::marker::PhantomData;

// ---------------------------------------------------------------------------------------------------------------------
// The `Bits` iterator.
// ---------------------------------------------------------------------------------------------------------------------

/// An iterator over *boolean* values of the bits in a [`BitStore`].
///
/// This iterator starts at the first bit and then returns *all* the bits in order. It returns `None` when it has
/// reached the end of the bit-store.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v: BitVec = BitVec::ones(10);
/// v.set(5, false);
/// let bits: Vec<bool> = v.bits().collect();
/// assert_eq!(bits, vec![true, true, true, true, true, false, true, true, true, true]);
/// ```
pub struct Bits<'a, Store: BitStore<Word>, Word: Unsigned> {
    store:    &'a Store,
    index:    usize,
    _phantom: PhantomData<Word>,
}

/// Construct a `Bits` iterator.
impl<'a, Store: BitStore<Word>, Word: Unsigned> Bits<'a, Store, Word> {
    /// Creates a new `Bits` iterator for the given `BitStore`.
    pub fn new(store: &'a Store) -> Self { Self { store, index: 0, _phantom: PhantomData } }
}

/// Implement the `Iterator` trait for `Bits`.
impl<Store: BitStore<Word>, Word: Unsigned> Iterator for Bits<'_, Store, Word> {
    type Item = bool;

    /// Returns the next bit from the bit-store as a `bool`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.store.len() {
            self.index += 1;
            return Some(self.store.get(self.index - 1));
        }
        None
    }
}

/// Implement the `ExactSizeIterator` trait for `Bits`.
impl<Store: BitStore<Word>, Word: Unsigned> ExactSizeIterator for Bits<'_, Store, Word> {
    /// Returns the number of bit-store elements that have not been iterated over.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: BitVec = BitVec::ones(15);
    /// bv.set(5, false);
    /// let mut iter = bv.bits();
    /// assert_eq!(iter.len(), 15);
    /// iter.next();
    /// assert_eq!(iter.len(), bv.len() - 1);
    /// ```
    fn len(&self) -> usize { self.store.len() - self.index }
}

/// Implement the `DoubleEndedIterator` trait for `Bits`.
impl<Store: BitStore<Word>, Word: Unsigned> DoubleEndedIterator for Bits<'_, Store, Word> {
    /// Returns the previous element from the bit-store or `None` if the iterator has reached the start.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: BitVec = BitVec::ones(15);
    /// bv.set(0, false);
    /// bv.set(5, false);
    /// bv.set(10, false);
    /// let mut iter = bv.bits();
    /// for (i, value) in iter.enumerate().rev() {
    ///     assert_eq!(value, bv[i], "at index {i} (backwards)");
    /// }
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.store.len() {
            let item = self.store.get(self.store.len() - 1 - self.index);
            self.index += 1;
            return Some(item);
        }
        None
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `SetBits` iterator.
// ---------------------------------------------------------------------------------------------------------------------

/// An iterator over the *index locations* of any *set* bits in a [`BitStore`].
///
/// This iterator returns the *indices* of the set bits in the bit-store in order. It returns `None`
/// when it has reached the end of the bit-store or when there are no more set bits.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v: BitVec = BitVec::ones(10);
/// v.set(5, false);
/// let set_indices: Vec<usize> = v.set_bits().collect();
/// assert_eq!(set_indices, vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
/// ```
pub struct SetBits<'a, Store: BitStore<Word>, Word: Unsigned> {
    store:    &'a Store,
    index:    Option<usize>,
    _phantom: PhantomData<Word>,
}

/// Construct a `SetBits` iterator.
impl<'a, Store: BitStore<Word>, Word: Unsigned> SetBits<'a, Store, Word> {
    /// Creates a new `SetBits` iterator for the given `BitStore`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: BitVec = BitVec::ones(10);
    /// bv.set(5, false);
    /// let set_indices: Vec<usize> = bv.set_bits().collect();
    /// assert_eq!(set_indices, vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
    /// ```
    pub fn new(store: &'a Store) -> Self {
        // The `index` is initialized to `usize::MAX` to indicate that the iterator has not yet found a set bit.
        // When the iterator is advanced, the `index` is set to the index of the first set bit.
        // If no set bit is found, the iterator will return `None` for all subsequent calls to `next()`.
        Self { store, index: Some(usize::MAX), _phantom: PhantomData }
    }
}

/// Implement the `Iterator` trait for `SetBits`.
impl<Store: BitStore<Word>, Word: Unsigned> Iterator for SetBits<'_, Store, Word> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == Some(usize::MAX) {
            self.index = self.store.first_set();
        }
        else if self.index.is_some() {
            self.index = self.store.next_set(self.index.unwrap());
        }
        self.index
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `UnsetBits` iterator.
// ---------------------------------------------------------------------------------------------------------------------

/// An iterator over the *index locations* of any *unset* bits in a [`BitStore`].
///
/// This iterator returns the *indices* of the unset bits in the bit-store in order. It returns `None`
/// when it has reached the end of the bit-store or when there are no more unset bits.
///
/// # Examples
/// ```
/// use gf2::*;
/// let mut v: BitVec = BitVec::zeros(10);
/// v.set(5, true);
/// let unset_indices: Vec<usize> = v.unset_bits().collect();
/// assert_eq!(unset_indices, vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
/// ```
pub struct UnsetBits<'a, Store: BitStore<Word>, Word: Unsigned> {
    store:    &'a Store,
    index:    Option<usize>,
    _phantom: PhantomData<Word>,
}

/// Construct a `UnsetBits` iterator.
impl<'a, Store: BitStore<Word>, Word: Unsigned> UnsetBits<'a, Store, Word> {
    /// Creates a new `UnsetBits` iterator for the given `BitStore`.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: BitVec = BitVec::zeros(10);
    /// bv.set(5, true);
    /// let unset_indices: Vec<usize> = bv.unset_bits().collect();
    /// assert_eq!(unset_indices, vec![0, 1, 2, 3, 4, 6, 7, 8, 9]);
    /// ```
    pub fn new(store: &'a Store) -> Self {
        // The `index` is initialized to `usize::MAX` to indicate that the iterator has not yet found a unset bit.
        // When the iterator is advanced, the `index` is set to the index of the first unset bit.
        // If no unset bit is found, the iterator will return `None` for all subsequent calls to `next()`.
        Self { store, index: Some(usize::MAX), _phantom: PhantomData }
    }
}

/// Implement the `Iterator` trait for `UnsetBits`.
impl<Store: BitStore<Word>, Word: Unsigned> Iterator for UnsetBits<'_, Store, Word> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == Some(usize::MAX) {
            self.index = self.store.first_unset();
        }
        else if self.index.is_some() {
            self.index = self.store.next_unset(self.index.unwrap());
        }
        self.index
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// The `Words` iterator.
// ---------------------------------------------------------------------------------------------------------------------

/// An iterator over the [`Unsigned`]s holding the bits of a [`BitStore`].
///
/// This *behaves as if* the [`BitStore`] words were copied into a vector of [`Unsigned`] words starting at bit 0 of
/// word 0. The iterator then returns the words from that vector in order until it has reached the end of the
/// store [`BitStore`].
///
/// The final `Unsigned` word may not be fully occupied but any unused bits are set to `0`.
///
/// # Examples
/// ```
/// use gf2::*;
/// let v: BitVec<u8> = BitVec::ones(10);
/// let words: Vec<u8> = v.store_words().collect();
/// assert_eq!(words, vec![0b1111_1111_u8, 0b0000_0011_u8]);
/// let slice = gf2::BitSlice::new(&words, 0, 8);
/// let slice_words: Vec<u8> = slice.store_words().collect();
/// assert_eq!(slice_words, vec![0b1111_1111_u8]);
/// ```
pub struct Words<'a, Store: BitStore<Word>, Word: Unsigned> {
    store:    &'a Store,
    index:    usize,
    _phantom: PhantomData<Word>,
}

/// Construct a `Words` iterator.
impl<'a, Store: BitStore<Word>, Word: Unsigned> Words<'a, Store, Word> {
    /// Creates a new `Words` iterator for the given `BitStore`.
    pub fn new(store: &'a Store) -> Self { Self { store, index: 0, _phantom: PhantomData } }
}

/// Implement the `Iterator` trait for `Words`.
impl<Store: BitStore<Word>, Word: Unsigned> Iterator for Words<'_, Store, Word> {
    type Item = Word;

    /// Returns the next word in the bit-store as a `Word`.
    ///
    /// Acts as if the bit-store was copied into a vector of `Unsigned` words starting at bit 0 of word 0.
    /// The iterator then returns the words in that vector in order.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.store.words() {
            self.index += 1;
            return Some(self.store.word(self.index - 1));
        }
        None
    }
}

/// Implement the `ExactSizeIterator` trait for `Words`.
impl<Store: BitStore<Word>, Word: Unsigned> ExactSizeIterator for Words<'_, Store, Word> {
    /// Returns the number of *words* in the bit-store that have not been iterated over.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: BitVec<u8> = BitVec::ones(125);
    /// let mut iter = bv.store_words();
    /// assert_eq!(iter.len(), 16);
    /// iter.next();
    /// assert_eq!(iter.len(), 15);
    /// ```
    fn len(&self) -> usize { self.store.words() - self.index }
}

/// Implement the `DoubleEndedIterator` trait for `Words`.
impl<Store: BitStore<Word>, Word: Unsigned> DoubleEndedIterator for Words<'_, Store, Word> {
    /// Returns the previous element from the bit-store or `None` if the iterator has reached the start.
    ///
    /// # Examples
    /// ```
    /// use gf2::*;
    /// let mut bv: BitVec<u8> = BitVec::random(125);
    /// let mut iter = bv.store_words();
    /// assert_eq!(iter.len(), 16);
    /// for (i, word) in iter.enumerate().rev() {
    ///     assert_eq!(word, bv.word(i), "{word:08b} != {:08b} at index {i} (backwards)", bv.word(i));
    /// }
    /// ```
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index < self.store.words() {
            let item = self.store.word(self.store.words() - 1 - self.index);
            self.index += 1;
            return Some(item);
        }
        None
    }
}
