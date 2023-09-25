use crate::digit::Digit;
use crate::digit_set::DigitSet;

#[test]
fn test_digit_set() {
    assert_eq!(DigitSet::EMPTY.to_string(), "");
    assert_eq!(DigitSet::ALL.to_string(), "123456789");

    let mut set = DigitSet::EMPTY;
    assert_eq!(set, DigitSet::EMPTY);
    assert_ne!(set, DigitSet::ALL);

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
