use sudoku_game::{board::Move, random::RandomGenerator, small::Small, symmetry::Symmetry};

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
