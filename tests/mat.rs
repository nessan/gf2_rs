#[test]
fn test_inverse() {
    let n = 50;
    let identity = gf2::BitMat::identity(n);
    let mut trial = 1;
    let max_trials = 100;
    loop {
        let m: gf2::BitMat<u8> = gf2::BitMat::random(n, n);
        if let Some(inv) = m.inverse() {
            assert_eq!(&m * &inv, identity);
            break;
        }
        trial += 1;
        if trial > max_trials {
            panic!("No inverse found after {max_trials} trials");
        }
    }
}
