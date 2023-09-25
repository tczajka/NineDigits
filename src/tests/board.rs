use crate::{
    board::{Board, Coordinates, FilledBoard},
    error::InvalidInput,
    small::Small,
};

#[test]
fn test_coordinates() {
    for i in Small::all() {
        assert_eq!(Small::<81>::from(Coordinates::from(i)), i);
    }
}

#[test]
fn test_board() {
    let board_str =
        "000000000000000000000000000000000000000000000000000000000000000000000000000001290";
    let board: Board = board_str.parse().unwrap();
    assert_eq!(board.to_string(), board_str);
}

#[test]
fn test_filled_board() {
    let board_str =
        "000000000000000000000000000000000000000000000000000000000000000000000000000001290";
    assert_eq!(board_str.parse::<FilledBoard>().unwrap_err(), InvalidInput);
    let board_str =
        "123456789123456789123456789123456789123456789123456789123456789123456789123456789";
    let board: FilledBoard = board_str.parse().unwrap();
    assert_eq!(board.to_string(), board_str);

    let board: Board = board_str.parse().unwrap();
    // Safety: No zeroes in `board_str`.
    let filled_board = unsafe { board.to_filled() };
    assert_eq!(filled_board.to_string(), board_str);
}
