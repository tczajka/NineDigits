use sudoku_game::{digit_box::DigitBox, digit_set::DigitSet, small::Small};

#[test]
fn test_digit_box_array() {
    let arr: [[DigitSet; 4]; 4] = [
        ["123", "456", "789", ""],
        ["398", "789", "", "123"],
        ["435", "", "123", "456"],
        ["", "123", "456", "789"],
    ]
    .map(|row| row.map(|s| s.parse().unwrap()));

    let arr2: [[DigitSet; 4]; 4] = [
        ["123", "456", "789", ""],
        ["3798", "789", "", "123"],
        ["435", "", "123", "456"],
        ["", "123", "456", "789"],
    ]
    .map(|row| row.map(|s| s.parse().unwrap()));

    let mut digit_box: DigitBox = arr.into();
    digit_box.set(Small::new(1), Small::new(0), '7'.try_into().unwrap());
    assert_eq!(digit_box, arr2.into());
    digit_box.clear(Small::new(1), Small::new(0), '7'.try_into().unwrap());
    assert_eq!(digit_box, arr.into());
}
