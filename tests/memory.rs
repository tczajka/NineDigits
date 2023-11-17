use sudoku_game::{error::ResourcesExceeded, memory::Memory};

#[test]
fn test_memory() {
    let mut a = Memory::new(16);
    let mut remaining = a.into_remaining();
    let (x, mut rem2) = remaining.allocate_slice::<u32>(2).unwrap();
    x[0] = 1;
    x[1] = 2;
    let (y, mut rem3) = rem2.allocate_slice::<u32>(1).unwrap();
    y[0] = 3;
    assert_eq!(x, [1, 2]);
    assert_eq!(y, [3]);
    assert_eq!(
        rem3.allocate_slice::<u32>(5).unwrap_err(),
        ResourcesExceeded::Memory
    );

    remaining.allocate_slice::<u32>(3).unwrap();
}
