use sudoku_game::simd::{Simd4x32, Simd4x4x16};

#[test]
fn test_simd4x32_array() {
    let arr: [u32; 4] = [1, 2, 3, 4];
    assert_eq!(<[u32; 4]>::from(Simd4x32::from(arr)), arr);
}

#[test]
fn test_simd4x32_is_all_zero() {
    assert!(Simd4x32::from([0, 0, 0, 0]).is_all_zero());
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
fn test_simd4x32_xor() {
    let x = Simd4x32::from([0b1111, 0b0000, 0b1111, 0b0000]);
    let y = Simd4x32::from([0b0101, 0b0101, 0b0101, 0b0111]);
    let z = Simd4x32::from([0b1010, 0b0101, 0b1010, 0b0111]);
    assert_eq!(x ^ y, z);
    let mut a = x;
    a ^= y;
    assert_eq!(a, z);
}

#[test]
fn test_simd4x4x16_array() {
    let arr: [[u16; 4]; 4] = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
    ];
    assert_eq!(<[[u16; 4]; 4]>::from(Simd4x4x16::from(arr)), arr);
}
