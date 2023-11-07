use sudoku_game::{digit_box::DigitBox, small_set::DigitSet};

#[test]
fn test_digit_box_array() {
    let arr: [[DigitSet; 4]; 4] = [
        ["123", "456", "789", ""],
        ["398", "789", "", "123"],
        ["435", "", "123", "456"],
        ["", "123", "456", "789"],
    ]
    .map(|row| row.map(|s| s.parse().unwrap()));

    assert_eq!(<[[DigitSet; 4]; 4]>::from(DigitBox::from(arr)), arr);
}
