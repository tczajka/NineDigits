use sudoku_game::{
    board::{Board, Move},
    random::RandomGenerator,
    small::Small,
    symmetry::{normalize_board, Symmetry},
};

#[test]
fn test_symmetry_forward_backward() {
    let mut rng = RandomGenerator::with_nonce(0);
    for _ in 0..100 {
        let symmetry = Symmetry::random(&mut rng);
        let mov = Move {
            square: Small::random(&mut rng),
            digit: Small::random(&mut rng).into(),
        };
        assert_eq!(symmetry.backward_move(symmetry.forward_move(mov)), mov);
    }
}

#[test]
fn test_normalize_board() {
    // . . . . . . . . .
    // . . . . . . . . .
    // . . . . . . . . .
    // . . . . . . . . .
    // . . . . . . . 5 .
    // . . . . 3 . . . .
    // . . . . . . . . .
    // . . . . . . 4 3 .
    // . . . . . . . . .
    // Normalizes to:
    // 1 . . 3 . . . . .
    // 2 . . . . . . . .
    // . . . . . . . . .
    // . . . . 1 . . . .
    // . . . . . . . . .
    // . . . . . . . . .
    // . . . . . . . . .
    // . . . . . . . . .
    // . . . . . . . . .
    let board1: Board =
        "...........................................5.....3...................43.........."
            .parse()
            .unwrap();
    let board2: Board =
        "1..3.....2.....................1................................................."
            .parse()
            .unwrap();
    let (b, symmetry) = normalize_board(&board1);
    println!("{board2}");
    println!("{b}");
    assert_eq!(b, board2);
    let mov1 = "Hi5".parse().unwrap();
    let mov2 = "Ca3".parse().unwrap();
    assert_eq!(symmetry.forward_move(mov1), mov2);
    assert_eq!(symmetry.backward_move(mov2), mov1);
}
