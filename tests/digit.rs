use sudoku_game::{
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
    assert_eq!(digit.to_string(), "3");
}

#[test]
fn test_digit_all() {
    let v: Vec<String> = Digit::all().map(|d| d.to_string()).collect();
    assert_eq!(v, ["1", "2", "3", "4", "5", "6", "7", "8", "9"]);
}

#[test]
fn test_optional_digit() {
    assert_eq!(OptionalDigit::try_from('?'), Err(InvalidInput));

    assert_eq!(OptionalDigit::try_from('0').unwrap(), OptionalDigit::NONE);
    assert_eq!(char::from(OptionalDigit::NONE), '0');
    assert_eq!(OptionalDigit::NONE.to_string(), "0");
    assert_eq!(OptionalDigit::NONE.to_digit(), None);

    let digit = OptionalDigit::try_from('3').unwrap();
    assert_eq!(char::from(digit), '3');
    assert_eq!(digit.to_string(), "3");
    assert_eq!(digit.to_digit().unwrap(), Digit::try_from('3').unwrap());

    let digit = OptionalDigit::try_from('9').unwrap();
    assert_eq!(char::from(digit), '9');
}
