use sudoku_game::permutation::{Permutation, ALL_PERMUTATIONS_2, ALL_PERMUTATIONS_3};

#[test]
fn test_inverse() {
    for p in &ALL_PERMUTATIONS_2 {
        assert_eq!(p.then(&p.inverse()), Permutation::identity());
    }
    for p in &ALL_PERMUTATIONS_3 {
        assert_eq!(p.then(&p.inverse()), Permutation::identity());
    }
}
