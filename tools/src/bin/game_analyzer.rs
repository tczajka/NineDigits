use clap::Parser;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use sudoku_game::{
    board::{Board, FullMove},
    endgame::{EndgameResult, EndgameSolver},
    random::RandomGenerator,
    solution_table::SolutionTable,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    database: PathBuf,

    #[arg(short, long)]
    player_id: u32,
}

const ENDGAME_MEMORY: usize = 512 << 20;
const MAX_SOLUTIONS: u32 = 100_000;

fn main() {
    let args = Args::parse();
    let database = sqlite::open(args.database).unwrap();

    let games = get_games(&database, args.player_id);
    let mut statistics = Statistics {
        num_games: 0,
        total_log_num_solutions: 0.0,
        total_log_difficulty: 0.0,
    };
    let mut endgame_solver = EndgameSolver::new(ENDGAME_MEMORY);
    let mut rng = RandomGenerator::with_nonce(0);
    for game in games {
        analyze_game(
            &game,
            args.player_id,
            &mut endgame_solver,
            &mut rng,
            &mut statistics,
        );
    }
    let avg_num_solutions =
        (statistics.total_log_num_solutions / statistics.num_games as f64).exp();
    let avg_difficulty = (statistics.total_log_difficulty / statistics.num_games as f64).exp();
    println!(
        "Average mistake num_solutions: {avg_num_solutions:.3} difficulty: {avg_difficulty:.3}"
    );
}

fn get_games(database: &sqlite::Connection, player_id: u32) -> Vec<Game> {
    let query = "
    SELECT round.competition_id, game.round_id, game.id, player_first, player_second, moves
    FROM game, round
    WHERE game.round_id = round.id
    AND (player_first = :player_id OR player_second = :player_id)
    ";
    let mut statement = database.prepare(query).unwrap();
    statement
        .bind(&[(":player_id", i64::from(player_id))][..])
        .unwrap();
    let mut games = Vec::new();
    while statement.next().unwrap() == sqlite::State::Row {
        let competition_id: u32 = statement.read::<i64, _>(0).unwrap().try_into().unwrap();
        let round_id: u32 = statement.read::<i64, _>(1).unwrap().try_into().unwrap();
        let game_id: u32 = statement.read::<i64, _>(2).unwrap().try_into().unwrap();
        let player_first: u32 = statement.read::<i64, _>(3).unwrap().try_into().unwrap();
        let player_second: u32 = statement.read::<i64, _>(4).unwrap().try_into().unwrap();
        let moves: String = statement.read::<String, _>(5).unwrap();
        games.push(Game {
            competition_id,
            round_id,
            id: game_id,
            player_ids: [player_first, player_second],
            moves: moves
                .split_whitespace()
                .map(|s| s.parse::<FullMove>().unwrap())
                .collect(),
        });
    }
    games
}

fn analyze_game(
    game: &Game,
    player_id: u32,
    endgame_solver: &mut EndgameSolver,
    rng: &mut RandomGenerator,
    statistics: &mut Statistics,
) {
    println!();
    println!(
        "Competition {} round {} game {}",
        game.competition_id, game.round_id, game.id
    );
    let mut boards = Vec::new();
    let mut board = Board::new();
    boards.push(board);
    let mut mov_iter = game.moves.iter();
    loop {
        match mov_iter.next() {
            None => {
                println!("Game never finished");
                return;
            }
            Some(FullMove::ClaimUnique) => {
                boards.push(board);
                break;
            }
            Some(FullMove::MoveClaimUnique(mov)) => {
                board.make_move(*mov).unwrap();
                boards.push(board);
                break;
            }
            Some(FullMove::Move(mov)) => {
                board.make_move(*mov).unwrap();
                boards.push(board);
            }
        }
    }
    #[derive(Debug, Copy, Clone)]
    struct StateAnalysis {
        num_solutions: u32,
        result: EndgameResult,
    }
    let mut state_analyses: Vec<Option<StateAnalysis>> = vec![None; boards.len()];
    let (result, solutions) = SolutionTable::generate(
        boards.last().unwrap(),
        0,
        2,
        Instant::now() + Duration::from_secs(3600),
        rng,
    );
    result.unwrap();
    assert!(!solutions.is_empty());
    state_analyses[boards.len() - 1] = Some(StateAnalysis {
        num_solutions: solutions.len(),
        result: if solutions.len() == 1 {
            EndgameResult::Loss
        } else {
            EndgameResult::Win { difficulty: 0 }
        },
    });

    for (i, mov) in game.moves.iter().enumerate().rev() {
        eprintln!(
            "Move {} {} {}",
            i + 1,
            mov,
            if game.player_ids[i % 2] == player_id {
                "player"
            } else {
                "opponent"
            }
        );
        eprintln!("Generating solutions");
        let (result, solutions) = SolutionTable::generate(
            &boards[i],
            0,
            MAX_SOLUTIONS,
            Instant::now() + Duration::from_secs(3600),
            rng,
        );
        match result {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Can't generate solutions: {}", e);
                statistics.num_games += 1;
                statistics.total_log_num_solutions += (MAX_SOLUTIONS as f64).ln();
                statistics.total_log_difficulty += (MAX_SOLUTIONS as f64).ln();
                return;
            }
        }
        eprintln!("Analyzing endgame num_solutions = {}", solutions.len());
        let start_time = Instant::now();
        let endgame_result =
            match endgame_solver.solve(&solutions, None, Instant::now() + Duration::from_secs(120))
            {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Can't analyze endgame: {e}");
                    statistics.num_games += 1;
                    statistics.total_log_num_solutions += (MAX_SOLUTIONS as f64).ln();
                    statistics.total_log_difficulty += (MAX_SOLUTIONS as f64).ln();
                    return;
                }
            };
        eprintln!(
            "Result: {endgame_result:?} time={:.3?}",
            start_time.elapsed()
        );
        state_analyses[i] = Some(StateAnalysis {
            num_solutions: solutions.len(),
            result: endgame_result,
        });
        if game.player_ids[i % 2] == player_id
            && matches!(
                state_analyses[i + 1].unwrap().result,
                EndgameResult::Win { .. }
            )
        {
            if let StateAnalysis {
                num_solutions,
                result: EndgameResult::Win { difficulty },
            } = state_analyses[i].unwrap()
            {
                println!(
                    ">>> MISTAKE at {} {mov} num_solutions: {num_solutions} difficulty: {difficulty}",
                    i + 1,
                );
                statistics.num_games += 1;
                statistics.total_log_num_solutions += (num_solutions.max(1) as f64).ln();
                statistics.total_log_difficulty += (difficulty.max(1) as f64).ln();
                return;
            }
        }
    }
    panic!("Whole game analyzed");
}

#[derive(Debug)]
struct Game {
    competition_id: u32,
    round_id: u32,
    id: u32,
    player_ids: [u32; 2],
    moves: Vec<FullMove>,
}

#[derive(Debug)]
struct Statistics {
    num_games: u32,
    total_log_num_solutions: f64,
    total_log_difficulty: f64,
}
