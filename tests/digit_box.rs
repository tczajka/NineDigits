use sudoku_game::{
    digit_box::{Box4x4x16, DigitBox},
    small::Small,
};

#[test]
fn test_digit_box_parse_display() {
    let s = "123|456|7|
456|789|1|2
789|123|4|39
234|567|8|13
";

    let a: DigitBox = s.parse().unwrap();

    assert_eq!(a.to_string(), s);
}

#[test]
fn test_digit_box_is_all_empty() {
    assert!(DigitBox::empty().is_all_empty());

    let a: DigitBox = "|||
||9|
|||
|||"
    .parse()
    .unwrap();

    assert!(!a.is_all_empty());
}

#[test]
fn test_digit_box_fill() {
    let a = DigitBox::fill("13".parse().unwrap());
    let expected: DigitBox = "13|13|13|13
13|13|13|13
13|13|13|13
13|13|13|13"
        .parse()
        .unwrap();
    assert_eq!(a, expected);

    let a = DigitBox::fill_rows([
        "1".parse().unwrap(),
        "2".parse().unwrap(),
        "3".parse().unwrap(),
        "4".parse().unwrap(),
    ]);
    let expected: DigitBox = "1|2|3|4
1|2|3|4
1|2|3|4
1|2|3|4"
        .parse()
        .unwrap();
    assert_eq!(a, expected);
}

#[test]
fn test_digit_box_set_clear() {
    let a: DigitBox = "123|456|789|
398|789||123
345||123|456
|123|456|789"
        .parse()
        .unwrap();

    let expected: DigitBox = "123|456|789|
3798|789||123
345||123|456
|123|456|789"
        .parse()
        .unwrap();

    let mut x = a;
    x.set(Small::new(1), Small::new(0), '7'.try_into().unwrap());
    assert_eq!(x, expected);
    x.clear(Small::new(1), Small::new(0), '7'.try_into().unwrap());
    assert_eq!(x, a);
}

#[test]
fn test_bitops() {
    let a: DigitBox = "1|23|456|123456789
12||1234|579
435||12|4
13456789|145|347|2468"
        .parse()
        .unwrap();

    let b: DigitBox = "7|34|67|123
|||
5||23|4
|||"
    .parse()
    .unwrap();

    let expected_and: DigitBox = "|3|6|123
|||
5||2|4
|||"
    .parse()
    .unwrap();

    let expected_or: DigitBox = "17|234|4567|123456789
12||1234|579
435||123|4
13456789|145|347|2468"
        .parse()
        .unwrap();

    let expected_xor: DigitBox = "17|24|457|456789
12||1234|579
43||13|
13456789|145|347|2468"
        .parse()
        .unwrap();

    let expected_and_not: DigitBox = "1|2|45|456789
12||1234|579
43||1|
13456789|145|347|2468"
        .parse()
        .unwrap();

    assert_eq!(a & b, expected_and);
    let mut x = a;
    x &= b;
    assert_eq!(x, expected_and);

    assert_eq!(a | b, expected_or);
    let mut x = a;
    x |= b;
    assert_eq!(x, expected_or);

    assert_eq!(a ^ b, expected_xor);
    let mut x = a;
    x ^= b;
    assert_eq!(x, expected_xor);

    assert_eq!(a.and_not(b), expected_and_not);
}

#[test]
fn test_counts() {
    let digit_box: DigitBox = "1|23|456|123456789
12||1234|579
435||12|4
13456789|145|347|2468"
        .parse()
        .unwrap();

    let expected: Box4x4x16 = [[1, 2, 3, 9], [2, 0, 4, 3], [3, 0, 2, 1], [8, 3, 3, 4]].into();

    assert_eq!(digit_box.counts(), expected);
}

#[test]
fn test_any_lt_gt() {
    let a: Box4x4x16 = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
    ]
    .into();
    let b: Box4x4x16 = [[1, 2, 3, 4], [5, 3, 7, 8], [9, 10, 11, 12], [13, 1, 15, 16]].into();
    assert!(!a.any_lt(a));
    assert!(!a.any_lt(b));
    assert!(b.any_lt(a));

    assert!(!a.any_gt(a));
    assert!(!b.any_gt(a));
    assert!(a.any_gt(b));
}

#[test]
fn test_masks_eq() {
    let a: DigitBox = "1|23|456|123456789
12||1234|579
435||12|4
13456789|145|347|2468"
        .parse()
        .unwrap();

    let b: DigitBox = "1||456|123456789
12|||123
1||12|4
1|145|347|2468"
        .parse()
        .unwrap();

    let expected: Box4x4x16 = [
        [0xffff, 0, 0xffff, 0xffff],
        [0xffff, 0xffff, 0, 0],
        [0, 0xffff, 0xffff, 0xffff],
        [0, 0xffff, 0xffff, 0xffff],
    ]
    .into();

    assert_eq!(a.masks_eq(b), expected);
}

#[test]
fn test_pick() {
    let a: DigitBox = "1|2|3|3
4|4|4|4
3|3|3|3
5|5|5|5"
        .parse()
        .unwrap();

    let mask: Box4x4x16 = [
        [0xffff, 0, 0xffff, 0],
        [0, 0xffff, 0, 0xffff],
        [0xffff, 0, 0xffff, 0],
        [0xffff, 0, 0xffff, 0],
    ]
    .into();

    let expected: DigitBox = "1||3|
|4||4
3||3|
5||5|"
        .parse()
        .unwrap();

    assert_eq!(a.pick(mask), expected);
}

#[test]
fn test_replace() {
    let a: DigitBox = "1|2|3|3
4|4|4|4
3|3|3|3
5|5|5|5"
        .parse()
        .unwrap();

    let b: DigitBox = "7|7|7|7
8|8|8|8
9|9|9|9
3|34|3|3"
        .parse()
        .unwrap();

    let mask: Box4x4x16 = [
        [0xffff, 0, 0xffff, 0],
        [0, 0xffff, 0, 0xffff],
        [0xffff, 0, 0xffff, 0],
        [0xffff, 0, 0xffff, 0],
    ]
    .into();

    let expected: DigitBox = "7|2|7|3
4|8|4|8
9|3|9|3
3|5|3|5"
        .parse()
        .unwrap();

    assert_eq!(a.replace(mask, b), expected);
}

#[test]
fn test_rotate() {
    let a: DigitBox = "1|2|3|4
4|5|6|7
8|9|12|13
14|15|16|17"
        .parse()
        .unwrap();

    let expected_right: DigitBox = "4|1|2|3
7|4|5|6
13|8|9|12
17|14|15|16"
        .parse()
        .unwrap();

    let expected_down: DigitBox = "14|15|16|17
1|2|3|4
4|5|6|7
8|9|12|13"
        .parse()
        .unwrap();

    assert_eq!(a.rotate_right(), expected_right);
    assert_eq!(a.rotate_down(), expected_down);
}
