use sudoku_game::{digit::Digit, solution_table::SolutionTable};

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

    let solution_table = solution_table.filter(2, 1, Digit::try_from('1').unwrap());

    let mut iter = solution_table.iter();

    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 22);
    assert_eq!(
        sol.digits(),
        &['2', '1', '1'].map(|c| Digit::try_from(c).unwrap())
    );

    assert!(iter.next().is_none());
}
