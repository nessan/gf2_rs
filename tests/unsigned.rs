use gf2::Unsigned;

#[test]
fn test_words_needed() {
    assert_eq!(u8::words_needed(0), 0);
    assert_eq!(u8::words_needed(1), 1);
    assert_eq!(u8::words_needed(8), 1);
    assert_eq!(u8::words_needed(10), 2);
    assert_eq!(u8::words_needed(16), 2);
}

#[test]
fn test_alternating() {
    assert_eq!(u8::ALTERNATING, 0b01010101);
    assert_eq!(u16::ALTERNATING, 0b0101010101010101);
    assert_eq!(u32::ALTERNATING, 0b01010101010101010101010101010101);
    assert_eq!(u64::ALTERNATING, 0b0101010101010101010101010101010101010101010101010101010101010101);
}

#[test]
fn test_lowest_set_bit() {
    assert_eq!(0u8.lowest_set_bit(), None);
    assert_eq!(1u8.lowest_set_bit(), Some(0));
    assert_eq!(2u8.lowest_set_bit(), Some(1));
}

#[test]
fn test_highest_set_bit() {
    assert_eq!(0u8.highest_set_bit(), None);
    assert_eq!(1u8.highest_set_bit(), Some(0));
    assert_eq!(2u8.highest_set_bit(), Some(1));
}

#[test]
fn test_bit_width() {
    assert_eq!(0u8.min_digits(), 0);
    assert_eq!(1u8.min_digits(), 1);
    assert_eq!(2u8.min_digits(), 2);
    assert_eq!(3u8.min_digits(), 2);
    assert_eq!(4u8.min_digits(), 3);
    assert_eq!(5u8.min_digits(), 3);
    assert_eq!(6u8.min_digits(), 3);
    assert_eq!(7u8.min_digits(), 3);
    assert_eq!(8u8.min_digits(), 4);
    assert_eq!(9u8.min_digits(), 4);
    assert_eq!(10u8.min_digits(), 4);
    assert_eq!(15u8.min_digits(), 4);
    assert_eq!(16u8.min_digits(), 5);
}

#[test]
fn test_with_set_bits() {
    assert_eq!(u8::with_set_bits(..1), 0b0000_0001_u8, "with_set_bits(..1) should return 0b0000_0001_u8");
    assert_eq!(u8::with_set_bits(0..0), 0b0000_0000_u8, "with_set_bits(0..0) should return 0b0000_0000_u8");
    assert_eq!(u8::with_set_bits(1..=2), 0b0000_0110_u8, "with_set_bits(1..=2) should return 0b0000_0110_u8");
    assert_eq!(u8::with_set_bits(..), 0b1111_1111_u8, "with_set_bits(..) should return 0b1111_1111_u8");
}

#[test]
fn test_with_unset_bits() {
    assert_eq!(u8::with_unset_bits(..5), 0b1110_0000_u8, "with_unset_bits(..5) should return 0b1110_0000_u8");
    assert_eq!(u8::with_unset_bits(0..0), 0b1111_1111_u8, "with_unset_bits(0..0) should return 0b1111_1111_u8");
    assert_eq!(u8::with_unset_bits(1..=2), 0b1111_1001_u8, "with_unset_bits(1..=2) should return 0b1111_1001_u8");
    assert_eq!(u8::with_unset_bits(..), 0b0000_0000_u8, "with_unset_bits(..) should return 0b0000_0000_u8");
}

#[test]
fn test_set_bits() {
    let mut word = 0b0000_0000_u8;
    word.set_bits(1..3);
    assert_eq!(word, 0b0000_0110_u8, "set_bits(1..3) should return 0b0000_0110_u8");
    word.set_bits(..);
    assert_eq!(word, 0b1111_1111_u8, "set_bits(..) should return 0b1111_1111_u8");
}

#[test]
fn test_reset_bits() {
    let mut word = 0b1111_1111_u8;
    word.reset_bits(1..3);
    assert_eq!(word, 0b1111_1001_u8, "reset_bits(1..3) should return 0b1111_1001_u8");
    word.reset_bits(..);
    assert_eq!(word, 0b0000_0000_u8, "reset_bits(..) should return 0b0000_0000_u8");
}

#[test]
fn test_set_except_bits() {
    let mut word = 0b0000_0000_u8;
    word.set_except_bits(1..3);
    assert_eq!(word, 0b1111_1001_u8, "set_except_bits(1..3) should return 0b1111_1001_u8");
    word.set_except_bits(..);
    assert_eq!(word, 0b1111_1001_u8, "set_except_range(..) should return 0b1111_1001_u8");
    word = 0b0000_0000_u8;
    word.set_except_bits(4..);
    assert_eq!(word, 0b0000_1111_u8, "set_except_bits(4..) should return 0b0000_1111_u8");
}

#[test]
fn test_reset_except_bits() {
    let mut word = 0b1111_1111_u8;
    word.reset_except_bits(1..3);
    assert_eq!(word, 0b0000_0110_u8, "reset_except_bits(1..3) should return 0b0000_0110_u8");
    word.reset_except_bits(..);
    assert_eq!(word, 0b0000_0110_u8, "reset_except_range(..) should keep the same value");
    word = 0b1111_1111_u8;
    word.reset_except_bits(4..);
    assert_eq!(word, 0b1111_0000_u8, "reset_except_bits(4..) should return 0b1111_0000_u8");
}

#[test]
fn test_replace_bits() {
    let mut word = 0b1111_1111_u8;
    word.replace_bits(1..3, 0b0000_0000_u8);
    assert_eq!(word, 0b1111_1001_u8, "replace_bits(1..3, 0b0000_0000_u8) should return 0b1111_1001_u8");
    word.replace_bits(1..3, 0b0000_0110_u8);
    assert_eq!(word, 0b1111_1111_u8, "replace_bits(1..3, 0b0000_0110_u8) should return 0b1111_1111_u8");
    word.replace_bits(1..5, 0b0000_0110_u8);
    assert_eq!(word, 0b1110_0111_u8, "replace_bits(1..5, 0b0000_0110_u8) should return 0b1110_0111_u8");
}

#[test]
fn test_riffle() {
    let word = u8::MAX;
    let (lo, hi) = word.riffle();
    assert_eq!(lo, 0b0101_0101_u8, "Expected lo: 01010101, got: {lo:08b}");
    assert_eq!(hi, 0b0101_0101_u8, "Expected hi: 01010101, got: {hi:08b}");
}
