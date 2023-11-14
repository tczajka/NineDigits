use sudoku_game::{digit::Digit, digit_set::DigitSet};

#[test]
fn test_digit_set() {
    assert_eq!(DigitSet::EMPTY.to_string(), "");
    assert_eq!(DigitSet::all().to_string(), "123456789");

    let mut set = DigitSet::EMPTY;
    assert_eq!(set, DigitSet::EMPTY);
    assert_ne!(set, DigitSet::all());

    let three = Digit::try_from('3').unwrap();
    let nine = Digit::try_from('9').unwrap();

    set.insert(three);
    let mut set2 = set;
    set2.insert(nine);
    assert_ne!(set, set2);
    assert_eq!(set2.to_string(), "39");
    assert!(set2.contains(three));
    set2.remove(nine);
    assert!(!set2.contains(nine));
    assert_eq!(set, set2);

    let set = "142".parse::<DigitSet>().unwrap();
    assert_eq!(set.to_string(), "124");
    assert_eq!(set.size(), 3);
}

#[test]
fn test_digit_set_bitops() {
    let a: DigitSet = "142".parse().unwrap();
    let b: DigitSet = "245".parse().unwrap();
    let expected_and: DigitSet = "24".parse().unwrap();
    let expected_or: DigitSet = "1245".parse().unwrap();
    let expected_and_not: DigitSet = "1".parse().unwrap();

    assert_eq!(a & b, expected_and);
    let mut x = a;
    x &= b;
    assert_eq!(x, expected_and);

    assert_eq!(a | b, expected_or);
    let mut x = a;
    x |= b;
    assert_eq!(x, expected_or);

    assert_eq!(a.and_not(b), expected_and_not);
}
