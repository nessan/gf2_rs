use gf2::*;

// The type of bit-vector/slice we are testing.
type Word = u8;
type BV = BitVec<Word>;
type BS<'a> = BitSlice<'a, Word>;

/// Tests that don't mutate the underlying words.
#[test]
fn test_new() {
    let words = vec![0b10101010_u8, 0b11001100_u8];
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.to_binary_string(), "0101010100110011");
    let slice = BS::new(&words, 1, 11);
    assert_eq!(slice.to_binary_string(), "1010101001");
    let slice = BS::new(&words, 0, 11);
    assert_eq!(slice.to_binary_string(), "01010101001");
    let slice = BS::new(&words, 1, 16);
    assert_eq!(slice.to_binary_string(), "101010100110011");
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.to_binary_string(), "0101010100110011");
}

#[test]
fn test_len() {
    let words = vec![0b10101010_u8, 0b11001100_u8];
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.len(), 16);
    let slice = BS::new(&words, 1, 11);
    assert_eq!(slice.len(), 10);
}

#[test]
fn test_word_count() {
    let words = vec![0b1111_1111_u8, 0b1111_1111_u8];
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.words(), 2);
    let slice = BS::new(&words, 4, 11);
    assert_eq!(slice.words(), 1);
}

#[test]
fn test_word() {
    let words = vec![0b1010_1010_u8, 0b1100_1100_u8, 0b1111_1111_u8];
    let slice = BS::new(&words, 0, 24);
    assert_eq!(slice.word(0), 0b1010_1010_u8);
    assert_eq!(slice.word(1), 0b1100_1100_u8);
    assert_eq!(slice.word(2), 0b1111_1111_u8);
    let slice = BS::new(&words, 0, 4);
    assert_eq!(slice.word(0), 0b0000_1010_u8);
    let slice = BS::new(&words, 12, 16);
    assert_eq!(slice.word(0), 0b0000_1100_u8);
    let slice = BS::new(&words, 4, 22);
    assert_eq!(slice.word(0), 0b11001010);
    assert_eq!(slice.word(1), 0b11111100);
    assert_eq!(slice.word(2), 0b00000011);
    let words = vec![0b1111_1111_u8, 0b1111_1111_u8];
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.word(0), 0b1111_1111_u8);
    assert_eq!(slice.word(1), 0b1111_1111_u8);
    let slice = BS::new(&words, 8, 12);
    assert_eq!(slice.words(), 1);
    assert_eq!(slice.word(0), 0b0000_1111_u8);
    let slice = BS::new(&words, 6, 10);
    assert_eq!(slice.words(), 1);
    assert_eq!(slice.word(0), 0b0000_1111_u8);
}

#[test]
fn test_index() {
    let words = vec![0b10101010_u8, 0b11001100_u8];
    let slice = BS::new(&words, 0, 16);
    assert!(!slice[0], "bit 0 is not supposed to be set");
    assert!(slice[1], "bit 1 is supposed to be set");
    assert!(!slice[8], "bit 8 is not supposed to be set");
    assert!(!slice[9], "bit 9 is not supposed to be set");
}

#[test]
fn test_count_ones() {
    let words = vec![0b10101010_u8, 0b11001100_u8];
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.count_ones(), 8);
    let slice = BS::new(&words, 1, 11);
    assert_eq!(slice.count_ones(), 5);
}

#[test]
fn test_count_zeros() {
    let words = vec![0b10101010_u8, 0b11001100_u8];
    let slice = BS::new(&words, 0, 16);
    assert_eq!(slice.count_zeros(), 8);
    let slice = BS::new(&words, 1, 11);
    assert_eq!(slice.count_zeros(), 5);
}

#[test]
fn test_any() {
    let words = vec![0b0000_0000_u8, 0b1111_0000_u8];
    let slice = BS::new(&words, 1, 12);
    assert!(!slice.any(), "slice should not contain any ones");
    let slice = BS::new(&words, 12, 16);
    assert!(slice.any(), "slice should contain some ones");
}

#[test]
fn test_none() {
    let words = vec![0b0000_0000_u8, 0b1111_0000_u8];
    let slice = BS::new(&words, 1, 12);
    assert!(slice.none(), "slice should contain no ones");
    let slice = BS::new(&words, 12, 16);
    assert!(!slice.none(), "slice should contain some ones");
}

#[test]
fn test_all() {
    let words = vec![0b1111_1001_u8, 0b1111_0000_u8];
    let slice = BS::new(&words, 4, 8);
    assert!(slice.all(), "slice should contain all ones");
    let slice = BS::new(&words, 0, 4);
    assert!(!slice.all(), "slice should contain some zeros");
}

#[test]
fn test_to_binary_string() {
    let words = vec![0b1010_1010_u8, 0b1100_1100_u8, 0b1111_1111_u8];
    let slice = BS::new(&words, 0, 24);
    assert_eq!(slice.to_binary_string(), "010101010011001111111111");
    let slice = BS::new(&words, 0, 4);
    println!("BitSlice len: {}, slice words {}", slice.len(), slice.words());
    assert_eq!(slice.to_binary_string(), "0101");
    let slice = BS::new(&words, 12, 16);
    assert_eq!(slice.to_binary_string(), "0011");
    let slice = BS::new(&words, 4, 16);
    assert_eq!(slice.to_binary_string(), "010100110011");
}

#[test]
fn test_to_binary_string_with_alternate() {
    let words = vec![0b1010_1010_u8, 0b1100_1100_u8, 0b1111_1111_u8];
    let slice = BS::new(&words, 0, 24);
    assert_eq!(format!("{:b}", slice), "010101010011001111111111");
    assert_eq!(format!("{:#b}", slice), "0b010101010011001111111111");
}

#[test]
fn test_to_hex_string() {
    let words = vec![0b1010_1010_u8, 0b1100_1100_u8, 0b1111_1111_u8];
    let slice = BS::new(&words, 0, 24);
    assert_eq!(format!("{:X}", slice), "5533FF");
    assert_eq!(format!("{:x}", slice), "5533ff");
    assert_eq!(format!("{:#X}", slice), "0X5533FF");
    assert_eq!(format!("{:#x}", slice), "0x5533ff");
}

/// Tests that mutate the underlying words.
#[test]
fn test_set_word() {
    let mut words = vec![0b0000_0000_u8, 0b0000_0000_u8];
    let mut slice = BS::new_mut(&mut words, 5, 10);
    assert_eq!(slice.to_binary_string(), "00000");
    slice.set_word(0, 0b1111_1111_u8);
    assert_eq!(slice.to_binary_string(), "11111");
    assert_eq!(words, vec![0b1110_0000_u8, 0b0000_0011_u8]);
}

#[test]
fn test_shift_right_assign() {
    let mut words = vec![0b1111_1111_u8, 0b1111_1111_u8, 0b1111_1111_u8];
    let mut slice = BS::new_mut(&mut words, 4, 11);
    assert_eq!(slice.to_binary_string(), "1111111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "0111111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "0011111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "0001111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "0000111");

    assert_eq!(words, vec![0b0000_1111_u8, 0b1111_1111_u8, 0b1111_1111_u8]);

    let mut bv = BV::ones(10);
    let mut slice = bv.slice_mut(5..);
    assert_eq!(slice.to_binary_string(), "11111", "expected 11111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "01111", "expected 01111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "00111", "expected 00111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "00011", "expected 00011");

    assert_eq!(bv.to_binary_string(), "1111100011", "expected 1111100011");
}

#[test]
fn test_shift_left_assign() {
    let mut words = vec![0b1111_1111_u8, 0b1111_1111_u8, 0b1111_1111_u8];
    let mut slice = BS::new_mut(&mut words, 4, 11);

    assert_eq!(slice.to_binary_string(), "1111111", "expected 1111111");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "1111110", "expected 1111110");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "1111100", "expected 1111100");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "1111000", "expected 1111000");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "1110000", "expected 1110000");

    assert_eq!(words, vec![0b0111_1111_u8, 0b1111_1000_u8, 0b1111_1111_u8]);

    let mut bv = BV::ones(10);
    let mut slice = bv.slice_mut(5..);
    assert_eq!(slice.to_binary_string(), "11111", "expected 11111");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "11110", "expected 11110");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "11100", "expected 11100");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "11000", "expected 11000");

    assert_eq!(bv.to_binary_string(), "1111111000", "expected 1111111000");
}
