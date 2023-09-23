use crate::{array::Array, small::Small};

#[test]
fn test_array() {
    let mut x = Array::<u32, 5>::new(6);
    x[Small::new(0)] = 1;
    assert_eq!(x[Small::new(0)], 1);
    assert_eq!(x[Small::new(2)], 6);
}
