use sudoku_game::{endgame::EndgameResult, transposition_table::TranspositionTable};

#[test]
fn test_transposition_table() {
    let mut table = TranspositionTable::new(1 << 20);
    table.insert(0xabcd, EndgameResult::Win(None));
    table.insert(0x1234, EndgameResult::Loss);
    assert_eq!(table.find(0xabcd), Some(EndgameResult::Win(None)));
    assert_eq!(table.find(0x1234), Some(EndgameResult::Loss));
    assert_eq!(table.find(0x5678), None);

    table.new_era();
    table.insert(0x10000000abcd, EndgameResult::Loss);
    table.insert(0x20000000abcd, EndgameResult::Loss);
    table.insert(0x30000000abcd, EndgameResult::Loss);
    assert_eq!(table.find(0xabcd), Some(EndgameResult::Win(None)));
    table.insert(0x50000000abcd, EndgameResult::Loss);
    assert_eq!(table.find(0xabcd), None);
}
