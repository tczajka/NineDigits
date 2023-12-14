use sudoku_game::{
    board::Move,
    digit::{Digit, OptionalDigit},
    small::Small,
    solution_table::{SolutionTable, SquareInfo},
};

fn example_solution_table() -> SolutionTable {
    let square_infos = vec![
        SquareInfo {
            original_square: Small::new(10),
            original_digits: [
                OptionalDigit::try_from('2').unwrap(),
                OptionalDigit::try_from('4').unwrap(),
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
            ],
        },
        SquareInfo {
            original_square: Small::new(20),
            original_digits: [
                OptionalDigit::try_from('5').unwrap(),
                OptionalDigit::try_from('7').unwrap(),
                OptionalDigit::try_from('8').unwrap(),
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
            ],
        },
        SquareInfo {
            original_square: Small::new(30),
            original_digits: [
                OptionalDigit::try_from('2').unwrap(),
                OptionalDigit::try_from('4').unwrap(),
                OptionalDigit::try_from('6').unwrap(),
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
                OptionalDigit::NONE,
            ],
        },
    ];
    let mut solution_table = SolutionTable::with_capacity(vec![2, 3, 3], square_infos, 2);
    solution_table.append(11, &['1', '3', '3'].map(|c| Digit::try_from(c).unwrap()));
    solution_table.append(22, &['2', '1', '3'].map(|c| Digit::try_from(c).unwrap()));
    solution_table
}

#[test]
fn test_solution_table_properties() {
    let solution_table = example_solution_table();
    assert_eq!(solution_table.len(), 2);
    assert_eq!(solution_table.num_squares(), 3);
    assert_eq!(solution_table.num_moves_per_square(), [2, 3, 3]);
    assert_eq!(solution_table.hash(), 11 ^ 22);
}

#[test]
fn test_original_move() {
    let solution_table = example_solution_table();
    let mov = Move {
        square: Small::new(1),
        digit: Digit::try_from('2').unwrap(),
    };
    let mov = solution_table.original_move(mov);
    assert_eq!(mov.square, Small::new(20));
    assert_eq!(mov.digit, Digit::try_from('7').unwrap());
}

#[test]
fn test_solution_table_iter() {
    let solution_table = example_solution_table();
    let mut iter = solution_table.iter();

    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 11);
    assert_eq!(
        sol.digits(),
        &['1', '3', '3'].map(|c| Digit::try_from(c).unwrap())
    );

    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 22);
    assert_eq!(
        sol.digits(),
        &['2', '1', '3'].map(|c| Digit::try_from(c).unwrap())
    );

    assert!(iter.next().is_none());
}

#[test]
fn test_solution_table_filter() {
    let solution_table = example_solution_table();

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
        &['2', '1', '3'].map(|c| Digit::try_from(c).unwrap())
    );

    assert!(iter.next().is_none());
}

#[test]
fn test_move_summaries() {
    let solution_table = example_solution_table();

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
fn test_compress_and_gen_moves() {
    let solution_table = example_solution_table();
    let move_summaries = solution_table.move_tables();
    let (solution_table, moves) = solution_table.compress_and_gen_moves(&move_summaries);

    assert_eq!(solution_table.len(), 2);
    assert_eq!(solution_table.hash(), 11 ^ 22);
    assert_eq!(solution_table.num_squares(), 2);
    assert_eq!(solution_table.num_moves_per_square(), [2, 2]);

    let mut iter = solution_table.iter();
    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 11);
    assert_eq!(
        sol.digits(),
        &['1', '2'].map(|c| Digit::try_from(c).unwrap())
    );
    let sol = iter.next().unwrap();
    assert_eq!(sol.id(), 22);
    assert_eq!(
        sol.digits(),
        &['2', '1'].map(|c| Digit::try_from(c).unwrap())
    );
    assert!(iter.next().is_none());

    assert_eq!(moves.len(), 4);
    assert_eq!(moves[0].mov.square, Small::new(0));
    assert_eq!(moves[0].mov.digit, Digit::try_from('1').unwrap());
    assert_eq!(moves[0].num_solutions, 1);
    assert_eq!(moves[0].hash, 11);
    assert_eq!(moves[1].mov.square, Small::new(0));
    assert_eq!(moves[1].mov.digit, Digit::try_from('2').unwrap());
    assert_eq!(moves[1].num_solutions, 1);
    assert_eq!(moves[1].hash, 22);
    assert_eq!(moves[2].mov.square, Small::new(1));
    assert_eq!(moves[2].mov.digit, Digit::try_from('1').unwrap());
    assert_eq!(moves[2].num_solutions, 1);
    assert_eq!(moves[2].hash, 22);
    assert_eq!(moves[3].mov.square, Small::new(1));
    assert_eq!(moves[3].mov.digit, Digit::try_from('2').unwrap());
    assert_eq!(moves[3].num_solutions, 1);
    assert_eq!(moves[3].hash, 11);
}
