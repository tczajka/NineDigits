#/bin/bash
set -e
cargo run --release --example solver_benchmark -- \
  -i data/kaggle.in \
  -i data/magic_1465.in \
  -i data/hardest26.in \
  -i data/most_17.in \
  -i data/in1000x100 \
  -i data/in1000x1000 \
  -i data/in100x10000 \
  -i data/in10x100000 \
  -i data/in5x1000000 \
  -o /dev/null \
  -s fast
#   -s basic
  