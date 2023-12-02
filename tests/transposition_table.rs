use sudoku_game::transposition_table::TranspositionTable;

#[test]
fn test_transposition_table() {
    let mut table = TranspositionTable::new(1 << 20);
    table.insert(0xabcd, 100, true);
    table.insert(0x1234, 120, false);
    assert_eq!(table.find(0xabcd), Some((true, 100)));
    assert_eq!(table.find(0x1234), Some((false, 120)));
    assert_eq!(table.find(0x5678), None);

    table.insert(0x10000000abcd, 130, false);
    table.insert(0x20000000abcd, 130, false);
    table.insert(0x30000000abcd, 90, false);
    table.insert(0x40000000abcd, 200, false);
    assert_eq!(table.find(0xabcd), Some((true, 100)));
    table.insert(0x50000000abcd, 200, false);
    assert_eq!(table.find(0xabcd), None);
}
