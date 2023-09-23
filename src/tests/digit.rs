use crate::{
    digit::{Digit, OptionalDigit},
    small::Small,
};

#[test]
fn test_digit() {
    assert_eq!(Digit::try_from_char('?'), None);
    let digit = Digit::try_from_char('3').unwrap();
    assert_eq!(digit.to_char(), '3');
    assert_eq!(Small::<9>::from(digit), Small::new(2));
    assert_eq!(digit, Digit::from(Small::new(2)));
}

#[test]
fn test_optional_digit() {
    assert_eq!(OptionalDigit::try_from_char('?'), None);

    assert_eq!(
        OptionalDigit::try_from_char('.').unwrap(),
        OptionalDigit::NONE
    );
    assert_eq!(OptionalDigit::NONE.to_char(), '.');
    assert_eq!(OptionalDigit::NONE.to_digit(), None);

    let digit = OptionalDigit::try_from_char('3').unwrap();
    assert_eq!(digit.to_char(), '3');
    assert_eq!(
        digit.to_digit().unwrap(),
        Digit::try_from_char('3').unwrap()
    );

    let digit = OptionalDigit::try_from_char('9').unwrap();
    assert_eq!(digit.to_char(), '9');
}
