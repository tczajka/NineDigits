use crate::digit::Digit;
use crate::digit_set::DigitSet;

#[test]
fn test_digit_set() {
    let mut set = DigitSet::EMPTY;
    assert_eq!(set, DigitSet::EMPTY);
    assert_ne!(set, DigitSet::ALL);

    let three = Digit::try_from('3').unwrap();
    let nine = Digit::try_from('9').unwrap();

    set.insert(three);
    let mut set2 = set;
    set2.insert(nine);
    assert_ne!(set, set2);
    set2.remove(nine);
    assert_eq!(set, set2);
}
