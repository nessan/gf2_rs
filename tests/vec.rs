use gf2::*;

// The type of bit-vector we are testing.
type BV = BitVector<u8>;

#[test]
fn test_zeros() {
    let v: BV = BV::zeros(10);
    assert_eq!(v.to_string(), "0000000000");
}

#[test]
fn test_leading_zeros() {
    let n = 30;
    let mut v = BV::ones(n);
    for i in 0..n {
        v.set(i, false);
        assert_eq!(v.leading_zeros(), i + 1, "v = {v} so expected leading zeros to be {i}");
    }
}

#[test]
fn test_trailing_zeros() {
    let n = 30;
    let mut v = BV::zeros(n);
    for i in 0..n {
        v.set(i, true);
        assert_eq!(v.trailing_zeros(), n - i - 1, "v = {v} so expected trailing zeros to be {i}");
    }
}

#[test]
fn test_all() {
    let mut v: BV = BV::zeros(3);
    assert_eq!(v.all(), false, "{}", v);
    v.set_all(true);
    assert_eq!(v.all(), true, "{}", v);
}

#[test]
fn test_or_assign() {
    let mut v1: BV = BV::zeros(3);
    v1.set(0, true);
    let v2: BV = BV::ones(3);
    v1 |= &v2;
    assert_eq!(v1.to_string(), "111", "{}", v1);
}

#[test]
fn test_shl_assign() {
    // Bit-vector.
    let mut bv: BV = BV::ones(10);
    assert_eq!(bv.to_binary_string(), "1111111111");
    bv <<= 2;
    assert_eq!(bv.to_binary_string(), "1111111100");
    bv <<= 1;
    assert_eq!(bv.to_binary_string(), "1111111000");
    bv <<= 10;
    assert_eq!(bv.to_binary_string(), "0000000000");

    // Mutable slice.
    let mut bv: BV = BV::ones(10);
    let mut slice = bv.slice_mut(5..);
    assert_eq!(slice.to_binary_string(), "11111");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "11110");
    slice <<= 1;
    assert_eq!(slice.to_binary_string(), "11100");
    slice <<= 10;
    assert_eq!(slice.to_binary_string(), "00000");

    // Only the accessible slice bits should be affected by the shift.
    assert_eq!(bv.to_binary_string(), "1111100000");
}

#[test]
fn test_shr_assign() {
    // Bit-vector.
    let mut bv: BV = BV::ones(10);
    assert_eq!(bv.to_binary_string(), "1111111111");
    bv >>= 2;
    assert_eq!(bv.to_binary_string(), "0011111111");
    bv >>= 1;
    assert_eq!(bv.to_binary_string(), "0001111111");
    bv >>= 10;
    assert_eq!(bv.to_binary_string(), "0000000000");

    // Mutable slice.
    let mut bv: BV = BV::ones(10);
    let mut slice = bv.slice_mut(5..);
    assert_eq!(slice.to_binary_string(), "11111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "01111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "00111");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "00011");
    slice >>= 1;
    assert_eq!(slice.to_binary_string(), "00001");
    slice >>= 10;
    assert_eq!(slice.to_binary_string(), "00000");

    // Only the accessible slice bits should be affected by the shift.
    assert_eq!(bv.to_binary_string(), "1111100000");
}

#[test]
fn test_shl() {
    // Bit-vector reference.
    let bv1: BV = BV::ones(10);
    let bv2 = &bv1 << 2;
    assert_eq!(bv1.to_binary_string(), "1111111111");
    assert_eq!(bv2.to_binary_string(), "1111111100");

    // Bit-vector
    let bv3 = bv1 << 7;
    assert_eq!(bv3.to_binary_string(), "1110000000");

    // Bit-vector slice reference.
    let bv1: BV = BV::ones(10);
    let bv4 = bv1.slice(5..);
    assert_eq!(bv4.to_binary_string(), "11111");
    let bv5 = &bv4 << 2;
    assert_eq!(bv4.to_binary_string(), "11111");
    assert_eq!(bv5.to_binary_string(), "11100");

    // Bit-vector slice.
    let bv1: BV = BV::ones(10);
    let bv4 = bv1.slice(5..);
    assert_eq!(bv4.to_binary_string(), "11111");
    assert_eq!(bv5.to_binary_string(), "11100");
}

#[test]
fn test_shr() {
    // Bit-vector reference.
    let bv1: BV = BV::ones(10);
    let bv2 = &bv1 >> 2;
    assert_eq!(bv1.to_binary_string(), "1111111111");
    assert_eq!(bv2.to_binary_string(), "0011111111");

    // Bit-vector.
    let bv3 = bv1 >> 7;
    assert_eq!(bv3.to_binary_string(), "0000000111");

    // Bit-vector slice reference.
    let bv1: BV = BV::ones(10);
    let bv4 = bv1.slice(5..);
    assert_eq!(bv4.to_binary_string(), "11111");
    let bv5 = &bv4 >> 2;
    assert_eq!(bv4.to_binary_string(), "11111");
    assert_eq!(bv5.to_binary_string(), "00111");

    // Bit-vector slice.
    let bv1: BV = BV::ones(10);
    let bv4 = bv1.slice(5..);
    assert_eq!(bv4.to_binary_string(), "11111");
    assert_eq!(bv5.to_binary_string(), "00111");
}

#[test]
fn assign_from_store_copies_between_word_sizes() {
    let src: BitVector<u8> = BitVector::from_string("1011001110001111").unwrap();
    let mut dst: BitVector<u32> = BitVector::zeros(src.len());

    dst.copy_store(&src);

    assert_eq!(dst.to_string(), src.to_string());
}

#[test]
fn assign_from_store_splits_larger_words() {
    let src: BitVector<u64> = BitVector::from_string("110011001010").unwrap();
    let mut dst: BitVector<u8> = BitVector::zeros(src.len());

    dst.copy_store(&src);

    assert_eq!(dst.to_string(), src.to_string());
}

#[test]
fn assign_from_store_handles_slices() {
    let src: BitVector<u16> = BitVector::from_string("0001111001111000").unwrap();
    let slice = src.slice(3..13);
    let mut dst: BitVector<u32> = BitVector::zeros(slice.len());

    dst.copy_store(&slice);

    assert_eq!(dst.to_string(), slice.to_string());
}
