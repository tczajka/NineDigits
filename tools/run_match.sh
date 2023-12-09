#/bin/bash
set -e
cargo run --release -p tools --bin match -- \
    -t 8 \
    -g 800 \
    target/release/sudoku-game \
    players/competition1