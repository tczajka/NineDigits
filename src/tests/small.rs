use crate::small::Small;

#[test]
fn test_small() {
    let x = Small::<10>::new(5);
    assert_eq!(u8::from(x), 5);
    assert_eq!(usize::from(x), 5);
}

#[test]
#[should_panic]
fn test_small_panic() {
    let _ = Small::<10>::new(10);
}

#[test]
fn test_all() {
    assert!(Small::<5>::all().map(u8::from).eq([0, 1, 2, 3, 4]));
}

#[test]
fn test_array() {
    let mut x = [6; 5];
    x[Small::new(0)] = 1;
    assert_eq!(x[Small::new(0)], 1);
    assert_eq!(x[Small::new(2)], 6);
}
