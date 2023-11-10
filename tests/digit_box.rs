use sudoku_game::{
    digit_box::{Box4x4x16, DigitBox},
    digit_set::DigitSet,
    small::Small,
};

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

#[test]
fn test_counts() {
    let digit_box: DigitBox = [
        ["1", "23", "456", "123456789"],
        ["12", "", "1234", "579"],
        ["435", "", "12", "4"],
        ["13456789", "145", "347", "2468"],
    ]
    .map(|row| row.map(|s| s.parse().unwrap()))
    .into();

    let expected: Box4x4x16 = [[1, 2, 3, 9], [2, 0, 4, 3], [3, 0, 2, 1], [8, 3, 3, 4]].into();

    assert_eq!(digit_box.counts(), expected);
}
