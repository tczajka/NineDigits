use crate::{small::Small, square_set::SquareSet};

#[test]
fn test_square_set() {
    let mut set = SquareSet::EMPTY;
    assert_eq!(set, SquareSet::EMPTY);
    assert_ne!(set, SquareSet::ALL);

    let a = Small::<81>::new(15);
    let b = Small::<81>::new(80);

    set.insert(a);
    let mut set2 = set;
    set2.insert(b);
    assert_ne!(set, set2);
    set2.remove(b);
    assert_eq!(set, set2);
}
