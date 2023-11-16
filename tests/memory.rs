use sudoku_game::{error::ResourcesExceeded, memory::Memory};

#[test]
fn test_memory() {
    let mut a = Memory::new(16);
    let mut remaining = a.into_remaining();
    let (x, mut rem2) = remaining.allocate_slice(2, 3u32).unwrap();
    let (y, mut rem3) = rem2.allocate_slice(1, 4u32).unwrap();
    assert_eq!(x, [3, 3]);
    assert_eq!(y, [4]);
    assert_eq!(
        rem3.allocate_slice(5, 5u32).unwrap_err(),
        ResourcesExceeded::Memory
    );
}
