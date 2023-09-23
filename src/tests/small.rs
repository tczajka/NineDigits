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
