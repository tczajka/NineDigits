use sudoku_game::{
    board::{box_major_coordinates, Board, Coordinates, FilledBoard, FullMove, Move},
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
fn test_box_major_coordinates() {
    let v: Vec<Coordinates> = box_major_coordinates().collect();
    assert_eq!(v.len(), 81);
    for (sq, coord) in Small::<81>::all().zip(v.iter()) {
        assert_eq!(Small::<81>::from(*coord), sq);
    }
}

#[test]
fn test_move() {
    let mov: Move = "Bd7".parse().unwrap();
    let coord = Coordinates::from(mov.square);
    assert_eq!(coord.big[0], Small::new(0));
    assert_eq!(coord.small[0], Small::new(1));
    assert_eq!(coord.big[1], Small::new(1));
    assert_eq!(coord.small[1], Small::new(0));
    assert_eq!(Small::from(mov.digit), Small::new(6));
    assert_eq!(mov.to_string(), "Bd7");
}

#[test]
fn test_full_move() {
    let fmov = "Bd7".parse::<FullMove>().unwrap();
    assert_eq!(fmov.to_string(), "Bd7");

    let fmov = "Bd7!".parse::<FullMove>().unwrap();
    assert_eq!(fmov.to_string(), "Bd7!");

    let fmov = "!".parse::<FullMove>().unwrap();
    assert_eq!(fmov.to_string(), "!");

    assert!("?".parse::<FullMove>().is_err());
}

#[test]
fn test_board() {
    let board_str =
        "000000000000000000000000000000000000000000000000000000000000000000000000000001290";
    let mut board: Board = board_str.parse().unwrap();
    assert_eq!(board.to_string(), board_str);

    assert!(board.empty_squares().contains(Small::new(3)));
    assert!(!board.empty_squares().contains(Small::new(79)));

    let mov = "Cd7".parse::<Move>().unwrap();
    board.make_move(mov).unwrap();
    assert_eq!(
        board.to_string(),
        "000000000000000000000700000000000000000000000000000000000000000000000000000001290"
    );
    // Note: box-wise order.
    assert!(!board.empty_squares().contains(Small::new(15)));
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
    let filled_board = board.into_filled().unwrap();
    assert_eq!(filled_board.to_string(), board_str);
}
