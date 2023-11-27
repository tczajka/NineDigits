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

#[test]
fn test_then_array() {
    let p = &ALL_PERMUTATIONS_3[1]; // 0, 1, 2 -> 0, 2, 1
    let array = [10, 20, 30];
    assert_eq!(p.then_array(&array), [10, 30, 20]);
}
