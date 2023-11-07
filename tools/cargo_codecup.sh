#!/bin/bash
set -e
RUSTFLAGS="-Ctarget-feature=+sse4.2,+avx,+popcnt,+pclmulqdq" cargo $@
