#!/bin/sh
# Usage: run.sh TARGET [SECONDS] [JOBS]
set -e

cd "$(dirname "$0")"
target=$1
seconds=${2:-60}
jobs=${3:-1}

cargo run --quiet --release --example seeds
for seed in "seeds/$target"/*; do
    [ -e "$seed" ] || continue
    [ -e "corpus/$target/${seed##*/}" ] || cp "$seed" "corpus/$target/"
done

extra=""
[ "$target" = fit ] && extra="-len_control=0 -max_len=4096"

exec cargo +nightly fuzz run "$target" "corpus/$target" -- \
    -max_total_time="$seconds" -fork="$jobs" -ignore_crashes=1 -ignore_timeouts=1 -timeout=10 $extra
