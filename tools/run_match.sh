#/bin/bash
set -e
cargo run --release -p tools --bin match -- \
    -t 8 \
    -g 200 \
    target/release/sudoku-game \
    players/main