#/bin/bash
set -e
cargo run --release --example solver_benchmark -- \
  -i data/kaggle.in \
  -i data/most_17.in \
  -i data/magic_1465.in \
  -i data/hardest26.in \
  -o /dev/null \
  -s basic