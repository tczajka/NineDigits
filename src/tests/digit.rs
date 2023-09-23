use crate::{
    digit::{Digit, OptionalDigit},
    error::InvalidInput,
    small::Small,
};

#[test]
fn test_digit() {
    assert_eq!(Digit::try_from('?'), Err(InvalidInput));
    assert_eq!(Digit::try_from('0'), Err(InvalidInput));
    let digit = Digit::try_from('3').unwrap();
    assert_eq!(char::from(digit), '3');
    assert_eq!(Small::<9>::from(digit), Small::new(2));
    assert_eq!(digit, Digit::from(Small::new(2)));
}

#[test]
fn test_optional_digit() {
    assert_eq!(OptionalDigit::try_from('?'), Err(InvalidInput));

    assert_eq!(OptionalDigit::try_from('0').unwrap(), OptionalDigit::NONE);
    assert_eq!(char::from(OptionalDigit::NONE), '0');
    assert_eq!(OptionalDigit::NONE.to_digit(), None);

    let digit = OptionalDigit::try_from('3').unwrap();
    assert_eq!(char::from(digit), '3');
    assert_eq!(digit.to_digit().unwrap(), Digit::try_from('3').unwrap());

    let digit = OptionalDigit::try_from('9').unwrap();
    assert_eq!(char::from(digit), '9');
}
