#!/bin/sh
# Usage: run.sh TARGET [SECONDS] [JOBS]
set -e

cd "$(dirname "$0")"
target=$1
seconds=${2:-60}
jobs=${3:-1}

mkdir -p "corpus/$target"
if [ ! -e "corpus/.seeded" ]; then
    cargo run --quiet --release --example seeds
    touch corpus/.seeded
fi

[ -d "seeds/$target" ] && cp -n "seeds/$target"/* "corpus/$target"/

extra=""
[ "$target" = fit ] && extra="-len_control=0 -max_len=4096"

exec cargo +nightly fuzz run "$target" "corpus/$target" -- \
    -max_total_time="$seconds" -fork="$jobs" -ignore_crashes=1 $extra
