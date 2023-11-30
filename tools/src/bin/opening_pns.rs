use clap::Parser;
use std::{
    collections::{hash_map, HashMap, HashSet},
    fmt::{self, Display, Formatter},
    ops::Add,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};
use sudoku_game::{
    board::{Board, FullMove, Move},
    endgame::EndgameSolver,
    midgame,
    random::RandomGenerator,
    solution_table::SolutionTable,
    symmetry,
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value_t = 8)]
    threads: u32,

    #[arg(long)]
    time_limit_minutes: Option<u64>,

    #[arg(long, default_value_t = 512)]
    memory_mb: usize,

    #[arg(long, default_value_t = 60)]
    watch_secs: u64,

    #[arg(long, default_value_t = 1000000)]
    max_solutions: u32,

    #[arg(long, default_value_t = 120)]
    solve_secs: u64,
}

const ROOT: usize = 0;

struct OpeningBook {
    nodes: Vec<Node>,
    node_lookup: HashMap<Board, usize>,
    num_solved_nodes: usize,
    num_nodes_at_size: [usize; 82],
}

impl OpeningBook {
    fn new() -> Self {
        let mut book = Self {
            nodes: Vec::new(),
            node_lookup: HashMap::new(),
            num_solved_nodes: 0,
            num_nodes_at_size: [0; 82],
        };
        assert_eq!(book.add_node(&Board::new()), ROOT);
        book
    }

    fn add_node(&mut self, board: &Board) -> usize {
        match self.node_lookup.entry(*board) {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => {
                let id = self.nodes.len();
                self.nodes.push(Node::new(board));
                entry.insert(id);
                self.num_nodes_at_size[81 - usize::from(board.empty_squares().size())] += 1;
                id
            }
        }
    }

    fn done(&self) -> bool {
        !matches!(self.nodes[ROOT].outcome, Outcome::Unknown)
    }
}

struct Node {
    board: Board,
    outcome: Outcome,
    forward_edges: Vec<Edge>,
    backward_edges: Vec<Edge>,
    proof_number: Number,
    disproof_number: Number,
    virtual_proof_number: Number,
    virtual_disproof_number: Number,
}

impl Node {
    fn new(board: &Board) -> Self {
        Self {
            board: *board,
            outcome: Outcome::Unknown,
            forward_edges: Vec::new(),
            backward_edges: Vec::new(),
            proof_number: Number::Finite(1),
            disproof_number: Number::Finite(1),
            virtual_proof_number: Number::Finite(1),
            virtual_disproof_number: Number::Finite(1),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Outcome {
    Win(Move),
    Loss,
    Unknown,
}

#[derive(Copy, Clone, Debug)]
struct Edge {
    from: usize,
    to: usize,
    mov: Move,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Number {
    Finite(u128),
    Infinity,
}

impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Infinity, _) | (_, Number::Infinity) => Number::Infinity,
            (Number::Finite(a), Number::Finite(b)) => Number::Finite(a.checked_add(b).unwrap()),
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Number::Finite(n) => write!(f, "{n}"),
            Number::Infinity => write!(f, "inf"),
        }
    }
}

fn select_leaf(book: &OpeningBook) -> Option<usize> {
    if book.nodes[ROOT].virtual_proof_number == Number::Infinity
        || book.nodes[ROOT].virtual_disproof_number == Number::Infinity
        || book.nodes[ROOT].outcome != Outcome::Unknown
    {
        return None;
    }
    let mut node_id = ROOT;
    while !book.nodes[node_id].forward_edges.is_empty() {
        let best_edge = book.nodes[node_id]
            .forward_edges
            .iter()
            .min_by_key(|edge| book.nodes[edge.to].virtual_disproof_number)
            .unwrap();
        node_id = best_edge.to;
    }
    Some(node_id)
}

fn update_node(book: &mut OpeningBook, node_id: usize) {
    let mut outcome = Outcome::Loss;
    let mut pn = Number::Infinity;
    let mut dn = Number::Finite(0);
    let mut vpn = Number::Infinity;
    let mut vdn = Number::Finite(0);
    let mut seen = HashSet::new();
    for edge in &book.nodes[node_id].forward_edges {
        if !seen.insert(edge.to) {
            continue;
        }
        match book.nodes[edge.to].outcome {
            Outcome::Loss => {
                if !matches!(outcome, Outcome::Win(_)) {
                    outcome = Outcome::Win(edge.mov);
                }
            }
            Outcome::Unknown => {
                if matches!(outcome, Outcome::Loss) {
                    outcome = Outcome::Unknown;
                }
            }
            Outcome::Win(_) => {}
        }
        pn = pn.min(book.nodes[edge.to].disproof_number);
        dn = dn + book.nodes[edge.to].proof_number;
        vpn = vpn.min(book.nodes[edge.to].virtual_disproof_number);
        vdn = vdn + book.nodes[edge.to].virtual_proof_number;
    }
    book.nodes[node_id].outcome = outcome;
    book.nodes[node_id].proof_number = pn;
    book.nodes[node_id].disproof_number = dn;
    book.nodes[node_id].virtual_proof_number = vpn;
    book.nodes[node_id].virtual_disproof_number = vdn;
}

// We end up with topological ordering, starting from root.
fn add_ancestors_rec(
    book: &OpeningBook,
    node_id: usize,
    ancestors: &mut Vec<usize>,
    ancestors_set: &mut HashSet<usize>,
) {
    for edge in &book.nodes[node_id].backward_edges {
        if ancestors_set.insert(edge.from) {
            add_ancestors_rec(book, edge.from, ancestors, ancestors_set);
            ancestors.push(edge.from);
        }
    }
}

fn update_ancestors(book: &mut OpeningBook, node_id: usize) {
    let mut ancestors = Vec::new();
    let mut ancestors_set = HashSet::new();
    add_ancestors_rec(book, node_id, &mut ancestors, &mut ancestors_set);
    if node_id != ROOT {
        assert_eq!(ancestors[0], ROOT);
    }
    for &ancestor in ancestors.iter().rev() {
        update_node(book, ancestor);
    }
}

fn expand(book: &mut OpeningBook, node_id: usize) {
    assert!(book.nodes[node_id].forward_edges.is_empty());
    assert!(book.nodes[node_id].outcome == Outcome::Unknown);

    let mut board = book.nodes[node_id].board;
    let moves = midgame::generate_moves(&mut board, &SolutionTable::empty());
    for mov in moves {
        let mut board2 = board;
        board2.make_move(mov.mov).unwrap();
        // Normalize board2.
        let _ = midgame::generate_moves(&mut board2, &SolutionTable::empty());
        (board2, _) = symmetry::normalize_board(&board2);
        let node_id2 = book.add_node(&board2);
        let edge = Edge {
            from: node_id,
            to: node_id2,
            mov: mov.mov,
        };
        book.nodes[node_id].forward_edges.push(edge);
        book.nodes[node_id2].backward_edges.push(edge);
    }
}

fn solve(
    board: &Board,
    max_solutions: u32,
    time_limit: Duration,
    endgame_solver: &mut EndgameSolver,
    rng: &mut RandomGenerator,
) -> Outcome {
    eprintln!("Solving: {board}");
    let deadline = Instant::now() + time_limit;
    let (result, solutions) = SolutionTable::generate(board, 0, max_solutions, deadline, rng);
    if result.is_err() {
        return Outcome::Unknown;
    }
    eprintln!(
        "Solutions: {num_solutions}",
        num_solutions = solutions.len()
    );
    match endgame_solver.solve_with_move(&solutions, deadline) {
        Ok((true, Some(FullMove::Move(mov)))) => Outcome::Win(mov),
        Ok((true, fmov)) => {
            panic!("Unexpected move from solver: {fmov:?}");
        }
        Ok((false, _)) => Outcome::Loss,
        Err(_) => Outcome::Unknown,
    }
}

fn main() {
    let args = Args::parse();
    let deadline = args
        .time_limit_minutes
        .map(|m| Instant::now() + Duration::from_secs(60 * m));
    let reporting_interval = Duration::from_secs(args.watch_secs);
    let solve_time_limit = Duration::from_secs(args.solve_secs);
    let memory = args.memory_mb << 20;
    let update_condvar = Arc::new(Condvar::new());

    let book = Arc::new(Mutex::new(OpeningBook::new()));
    let join_handles: Vec<_> = (0..args.threads)
        .map(|_| {
            let book = book.clone();
            let update_condvar = update_condvar.clone();
            thread::spawn(move || {
                pn_search_thread(
                    &book,
                    &update_condvar,
                    deadline,
                    memory,
                    args.max_solutions,
                    solve_time_limit,
                )
            })
        })
        .collect();

    loop {
        thread::sleep(reporting_interval);
        if let Some(deadline) = deadline {
            let t = Instant::now();
            if t >= deadline {
                break;
            }
            eprintln!(
                "Time remaining: {:.1?}",
                deadline.saturating_duration_since(t)
            );
        }
        report(&book);
        if book.lock().unwrap().done() {
            break;
        }
    }

    for join_handle in join_handles {
        join_handle.join().unwrap();
    }

    report(&book);
}

fn pn_search_thread(
    book: &Mutex<OpeningBook>,
    update_condvar: &Condvar,
    deadline: Option<Instant>,
    memory: usize,
    max_solutions: u32,
    solve_time_limit: Duration,
) {
    let mut rng = RandomGenerator::with_nonce(0);
    let mut endgame_solver = EndgameSolver::new(memory);
    loop {
        if let Some(deadline) = deadline {
            if Instant::now() > deadline {
                break;
            }
        }
        let (node_id, board) = {
            let mut book = book.lock().unwrap();
            let node_id = loop {
                if book.done() {
                    return;
                }
                match select_leaf(&book) {
                    None => {
                        book = update_condvar.wait(book).unwrap();
                    }
                    Some(node_id) => break node_id,
                }
            };
            book.nodes[node_id].virtual_proof_number = Number::Infinity;
            book.nodes[node_id].virtual_disproof_number = Number::Infinity;
            update_ancestors(&mut book, node_id);
            (node_id, book.nodes[node_id].board)
        };
        let outcome = solve(
            &board,
            max_solutions,
            solve_time_limit,
            &mut endgame_solver,
            &mut rng,
        );
        {
            let mut book = book.lock().unwrap();
            book.nodes[node_id].outcome = outcome;
            match outcome {
                Outcome::Win(_) => {
                    book.nodes[node_id].proof_number = Number::Finite(0);
                    book.nodes[node_id].disproof_number = Number::Infinity;
                    book.nodes[node_id].virtual_proof_number = Number::Finite(0);
                    book.nodes[node_id].virtual_disproof_number = Number::Infinity;
                    book.num_solved_nodes += 1;
                }
                Outcome::Loss => {
                    book.nodes[node_id].proof_number = Number::Infinity;
                    book.nodes[node_id].disproof_number = Number::Finite(0);
                    book.nodes[node_id].virtual_proof_number = Number::Infinity;
                    book.nodes[node_id].virtual_disproof_number = Number::Finite(0);
                    book.num_solved_nodes += 1;
                }
                Outcome::Unknown => {
                    expand(&mut book, node_id);
                    update_node(&mut book, node_id);
                }
            }
            update_ancestors(&mut book, node_id);
            update_condvar.notify_all();
        }
    }
}

fn report(book: &Mutex<OpeningBook>) {
    let book = book.lock().unwrap();
    eprintln!("Nodes: {}", book.nodes.len());
    eprintln!("Solved nodes: {}", book.num_solved_nodes);
    for (size, &num_nodes) in book.num_nodes_at_size.iter().enumerate() {
        if num_nodes != 0 {
            eprintln!("Nodes at size {}: {}", size, num_nodes);
        }
    }
    match book.nodes[ROOT].outcome {
        Outcome::Win(_) => eprintln!("Win"),
        Outcome::Loss => eprintln!("Loss"),
        _ => {}
    }
    eprintln!(
        "proof {} disproof {}",
        book.nodes[ROOT].proof_number, book.nodes[ROOT].disproof_number
    );
}
