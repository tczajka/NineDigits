use sudoku_game::{
    board::Move,
    digit::{Digit, OptionalDigit},
    small::Small,
    solution_table::SolutionTable,
};

#[test]
fn test_solution_table_iter() {
    let mut solution_table = SolutionTable::with_capacity(vec![2, 2, 3], 2);
    solution_table.append(11, &['1', '2', '2'].map(|c| Digit::try_from(c).unwrap()));
    solution_table.append(22, &['2', '1', '1'].map(|c| Digit::try_from(c).unwrap()));

    let mut iter = solution_table.iter();

    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 11);
    assert_eq!(
        sol.digits(),
        &['1', '2', '2'].map(|c| Digit::try_from(c).unwrap())
    );

    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 22);
    assert_eq!(
        sol.digits(),
        &['2', '1', '1'].map(|c| Digit::try_from(c).unwrap())
    );

    assert!(iter.next().is_none());
}

#[test]
fn test_solution_table_filter() {
    let mut solution_table = SolutionTable::with_capacity(vec![2, 2, 3], 2);
    solution_table.append(11, &['1', '2', '2'].map(|c| Digit::try_from(c).unwrap()));
    solution_table.append(22, &['2', '1', '1'].map(|c| Digit::try_from(c).unwrap()));

    let solution_table = solution_table.filter(
        2,
        Move {
            square: Small::new(1),
            digit: Digit::try_from('1').unwrap(),
        },
    );

    let mut iter = solution_table.iter();

    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 22);
    assert_eq!(
        sol.digits(),
        &['2', '1', '1'].map(|c| Digit::try_from(c).unwrap())
    );

    assert!(iter.next().is_none());
}

#[test]
fn test_move_summaries() {
    let mut solution_table = SolutionTable::with_capacity(vec![3, 3, 3], 2);
    solution_table.append(11, &['1', '2', '3'].map(|c| Digit::try_from(c).unwrap()));
    solution_table.append(22, &['2', '1', '3'].map(|c| Digit::try_from(c).unwrap()));

    let move_summaries = solution_table.move_tables();
    assert_eq!(move_summaries.len(), 3);
    assert_eq!(move_summaries[0].num_solutions[0], 1);
    assert_eq!(move_summaries[0].hash[0], 11);
    assert_eq!(move_summaries[1].num_solutions[0], 1);
    assert_eq!(move_summaries[1].hash[0], 22);
    assert_eq!(move_summaries[2].num_solutions[2], 2);
    assert_eq!(move_summaries[2].hash[2], 11 ^ 22);
}

#[test]
fn test_compress() {
    let mut solution_table = SolutionTable::with_capacity(vec![7, 7, 7], 3);
    solution_table.append(11, &['6', '3', '5'].map(|c| Digit::try_from(c).unwrap()));
    solution_table.append(22, &['2', '3', '3'].map(|c| Digit::try_from(c).unwrap()));
    solution_table.append(33, &['6', '3', '2'].map(|c| Digit::try_from(c).unwrap()));

    let move_summaries = solution_table.move_tables();
    let (solution_table, square_compressions) = solution_table.compress(&move_summaries);

    assert_eq!(solution_table.num_moves_per_square(), [2, 3]);

    let mut iter = solution_table.iter();
    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 11);
    assert_eq!(
        sol.digits(),
        &['2', '3'].map(|c| Digit::try_from(c).unwrap())
    );
    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 22);
    assert_eq!(
        sol.digits(),
        &['1', '2'].map(|c| Digit::try_from(c).unwrap())
    );
    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 33);
    assert_eq!(
        sol.digits(),
        &['2', '1'].map(|c| Digit::try_from(c).unwrap())
    );
    assert!(iter.next().is_none());

    assert_eq!(square_compressions.len(), 2);
    assert_eq!(square_compressions[0].prev_square, Small::new(0));
    assert_eq!(
        square_compressions[0].digit_map,
        ['0', '1', '0', '0', '0', '2', '0', '0', '0'].map(|c| OptionalDigit::try_from(c).unwrap())
    );
    assert_eq!(square_compressions[0].num_solutions[0], 1);
    assert_eq!(square_compressions[0].hash[0], 22);
    assert_eq!(square_compressions[0].num_solutions[1], 2);
    assert_eq!(square_compressions[0].hash[1], 11 ^ 33);
    assert_eq!(square_compressions[1].num_solutions[0], 1);
    assert_eq!(square_compressions[1].hash[0], 33);
    assert_eq!(square_compressions[1].num_solutions[1], 1);
    assert_eq!(square_compressions[1].hash[1], 22);
    assert_eq!(square_compressions[1].num_solutions[2], 1);
    assert_eq!(square_compressions[1].hash[2], 11);
}
