use sudoku_game::{
    simd128::{Simd4x32, Simd8x16},
    simd256::{Simd16x16, Simd4x64},
    small::Small,
};

#[test]
fn test_simd4x32_array() {
    let arr: [u32; 4] = [1, 2, 3, 4];
    assert_eq!(<[u32; 4]>::from(Simd4x32::from(arr)), arr);
}

#[test]
fn test_simd4x32_is_all_zero() {
    assert!(Simd4x32::zero().is_all_zero());
    assert!(!Simd4x32::from([0, 0, 1, 0]).is_all_zero());
}

#[test]
fn test_simd4x32_eq() {
    let a = Simd4x32::from([1, 2, 3, 4]);
    let b = Simd4x32::from([1, 2, 5, 4]);
    assert_eq!(a, a);
    assert_ne!(a, b);
}

#[test]
fn test_simd4x32_bitops() {
    let x = Simd4x32::from([0b1111, 0b0000, 0b1111, 0b0000]);
    let y = Simd4x32::from([0b0101, 0b0101, 0b0101, 0b0111]);
    let expected_and = Simd4x32::from([0b0101, 0b0000, 0b0101, 0b0000]);
    let expected_and_not = Simd4x32::from([0b1010, 0b0000, 0b1010, 0b0000]);
    let expected_or = Simd4x32::from([0b1111, 0b0101, 0b1111, 0b0111]);
    let expected_xor = Simd4x32::from([0b1010, 0b0101, 0b1010, 0b0111]);

    assert_eq!(x & y, expected_and);
    let mut a = x;
    a &= y;
    assert_eq!(a, expected_and);

    assert_eq!(x.and_not(y), expected_and_not);

    assert_eq!(x | y, expected_or);
    let mut a = x;
    a |= y;
    assert_eq!(a, expected_or);

    assert_eq!(x ^ y, expected_xor);
    let mut a = x;
    a ^= y;
    assert_eq!(a, expected_xor);
}

#[test]
fn test_simd4x32_first_bit() {
    let x = Simd4x32::from([0b0000, 0b0100, 0b1111, 0b1111]);
    assert_eq!(x.first_bit(), Some((Small::new(1), Small::new(2))));
    assert_eq!(Simd4x32::zero().first_bit(), None);
}

#[test]
fn test_simd8x16_popcount_9() {
    let a = Simd8x16::from([
        0b000000000,
        0b000000001,
        0b000000100,
        0b000001000,
        0b000100000,
        0b101010101,
        0b010101010,
        0b111111111,
    ]);
    let expected = Simd8x16::from([0, 1, 1, 1, 1, 5, 4, 9]);
    assert_eq!(a.popcount_9(), expected);
}

#[test]
fn test_simd8x16_any_lt() {
    let a = Simd8x16::from([0, 1, 2, 3, 4, 5, 6, 0xfffe]);
    let b = Simd8x16::from([0, 1, 2, 3, 3, 5, 6, 10]);
    let c = Simd8x16::from([100, 1, 2, 3, 4, 5, 6, 0xfffe]);
    assert!(!a.any_lt(b));
    assert!(a.any_lt(c));
}

#[test]
fn test_simd4x32_rotate_bits() {
    let x = Simd4x32::from([
        0b00000000000000000000000000000100,
        0b11000000000000000000000000000001,
        0b00000000000001000000000000000000,
        0b11111000000000000000000000000000,
    ]);
    assert_eq!(
        x.rotate_bits_7(),
        Simd4x32::from([
            0b00000000000000000000001000000000,
            0b00000000000000000000000011100000,
            0b00000010000000000000000000000000,
            0b00000000000000000000000001111100,
        ])
    );
    assert_eq!(
        x.rotate_bits_12(),
        Simd4x32::from([
            0b00000000000000000100000000000000,
            0b00000000000000000001110000000000,
            0b01000000000000000000000000000000,
            0b00000000000000000000111110000000,
        ])
    );
    assert_eq!(
        x.rotate_bits_16(),
        Simd4x32::from([
            0b00000000000001000000000000000000,
            0b00000000000000011100000000000000,
            0b00000000000000000000000000000100,
            0b00000000000000001111100000000000,
        ])
    );
}

#[test]
fn test_simd4x32_rotate_words() {
    let x = Simd4x32::from([111, 222, 333, 444]);
    assert_eq!(x.rotate_words_1(), Simd4x32::from([444, 111, 222, 333]));
    assert_eq!(x.rotate_words_2(), Simd4x32::from([333, 444, 111, 222]));
    assert_eq!(x.rotate_words_3(), Simd4x32::from([222, 333, 444, 111]));
}

#[test]
fn test_simd4x32_add() {
    let x = Simd4x32::from([444, 222, 0xffffffff, 333]);
    let y = Simd4x32::from([555, 666, 3, 111]);
    let sum = Simd4x32::from([999, 888, 2, 444]);
    assert_eq!(x + y, sum);
    let mut a = x;
    a += y;
    assert_eq!(a, sum);
}

#[test]
fn test_simd4x32_total_popcount() {
    let x = Simd4x32::from([
        0b11111111111111111111111111111111,
        0b00000000000000000000000010101010,
        0b10101100000000000000000000000000,
        0b11111111011111111111111111111111,
    ]);
    assert_eq!(x.total_popcount(), 32 + 4 + 4 + 31);
}

#[test]
fn test_simd4x64_array() {
    let arr: [u64; 4] = [1, 2, 3, 4];
    assert_eq!(<[u64; 4]>::from(Simd4x64::from(arr)), arr);
}

#[test]
fn test_simd4x64_is_all_zero() {
    assert!(Simd4x64::zero().is_all_zero());
    assert!(!Simd4x64::from([0, 0, 1, 0]).is_all_zero());
}

#[test]
fn test_simd4x64_eq() {
    let a = Simd4x64::from([1, 2, 3, 4]);
    let b = Simd4x64::from([1, 2, 5, 4]);
    assert_eq!(a, a);
    assert_ne!(a, b);
}

#[test]
fn test_simd4x64_bitops() {
    let x = Simd4x64::from([0b1111, 0b0000, 0b1111, 0b0000]);
    let y = Simd4x64::from([0b0101, 0b0101, 0b0101, 0b0111]);
    let expected_and = Simd4x64::from([0b0101, 0b0000, 0b0101, 0b0000]);
    let expected_and_not = Simd4x64::from([0b1010, 0b0000, 0b1010, 0b0000]);
    let expected_or = Simd4x64::from([0b1111, 0b0101, 0b1111, 0b0111]);
    let expected_xor = Simd4x64::from([0b1010, 0b0101, 0b1010, 0b0111]);

    assert_eq!(x & y, expected_and);
    let mut a = x;
    a &= y;
    assert_eq!(a, expected_and);

    assert_eq!(x.and_not(y), expected_and_not);

    assert_eq!(x | y, expected_or);
    let mut a = x;
    a |= y;
    assert_eq!(a, expected_or);

    assert_eq!(x ^ y, expected_xor);
    let mut a = x;
    a ^= y;
    assert_eq!(a, expected_xor);
}

#[test]
fn test_simd4x64_first_bit() {
    let x = Simd4x64::from([0b0000, 0b0100, 0b1111, 0b1111]);
    let y = Simd4x64::from([0b0000, 0b0000, 0b1000, 0b1111]);
    assert_eq!(x.first_bit(), Some((Small::new(1), Small::new(2))));
    assert_eq!(y.first_bit(), Some((Small::new(2), Small::new(3))));
    assert_eq!(Simd4x64::zero().first_bit(), None);
}

#[test]
fn test_simd4x64_total_popcount() {
    let x = Simd4x64::from([
        0b1111111111111111111111111111111111111111111111111111111111111111,
        0b0000000000000000000000001010101000001000000000000000000000100010,
        0b1010110000000000000000000000000000000000000000000000000000000001,
        0b1111111101111111111111111111111111111111111111111111111111111111,
    ]);
    assert_eq!(x.total_popcount(), 64 + 7 + 5 + 63);
}

#[test]
fn test_simd16x16_popcount_9() {
    let a = Simd16x16::from([
        0b000000000,
        0b000000001,
        0b000000010,
        0b000000100,
        0b000001000,
        0b000010000,
        0b000100000,
        0b001000000,
        0b010000000,
        0b100000000,
        0b111100000,
        0b000011111,
        0b101010101,
        0b010101010,
        0b110011001,
        0b111111111,
    ]);
    let expected = Simd16x16::from([0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 4, 5, 5, 4, 5, 9]);
    assert_eq!(a.popcount_9(), expected);
}

#[test]
fn test_simd16x16_any_lt() {
    let a = Simd16x16::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 0xfffe]);
    let b = Simd16x16::from([0, 1, 2, 3, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 10]);
    let c = Simd16x16::from([100, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 0xfffe]);
    assert!(!a.any_lt(b));
    assert!(a.any_lt(c));
}
