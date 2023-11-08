use sudoku_game::small::{CartesianProduct, Small};

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

#[test]
fn test_conversions() {
    let x = Small::<3>::new(2);
    let y: Small<4> = x.into();
    assert_eq!(y, Small::<4>::new(2));

    let x = Small::<16>::new(7);
    let y: Result<Small<15>, _> = x.try_into();
    assert_eq!(y, Ok(Small::<15>::new(7)));

    let x = Small::<16>::new(15);
    let y: Result<Small<15>, _> = x.try_into();
    assert!(y.is_err());
}

#[test]
fn test_cartesian_product() {
    let x = Small::<4>::combine(Small::<2>::new(1), Small::<2>::new(0));
    assert_eq!(x, Small::<4>::new(2));
    let (y, z) = x.split();
    assert_eq!(y, Small::<2>::new(1));
    assert_eq!(z, Small::<2>::new(0));
}
