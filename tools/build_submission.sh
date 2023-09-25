#/bin/bash
set -e
mkdir -p target/submission
tools/build_submission.py -o target/submission/submission.rs
rustc +1.66.1 -O --edition=2021 -Ctarget-cpu=x86-64 -Ctarget-feature=+sse4.2,+avx,+popcnt,+pclmulqdq target/submission/submission.rs -o target/submission/submission