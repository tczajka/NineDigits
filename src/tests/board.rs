use crate::{board::Coordinates, small::Small};

#[test]
fn test_coordinates() {
    for i in Small::all() {
        assert_eq!(Small::<81>::from(Coordinates::from(i)), i);
    }
}
