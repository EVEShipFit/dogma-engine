#!/bin/sh
# Usage: status.sh [MINUTES]
set -e

cd "$(dirname "$0")"
minutes=${1:-60}

printf '%-10s %7s %5s %8s  %s\n' target corpus new findings status
for source in fuzz_targets/*.rs; do
    target=$(basename "$source" .rs)
    if [ ! -d "corpus/$target" ]; then
        printf '%-10s %7s %5s %8s  %s\n' "$target" - - - "not run"
        continue
    fi
    total=$(($(find "corpus/$target" -type f | wc -l)))
    new=$(($(find "corpus/$target" -type f -mmin "-$minutes" | wc -l)))
    findings=$(($(find "artifacts/$target" -type f 2>/dev/null | wc -l)))
    status="done"
    [ "$new" -gt 0 ] && status="growing"
    printf '%-10s %7s %5s %8s  %s\n' "$target" "$total" "$new" "$findings" "$status"
done
echo "new: corpus files from the last $minutes minutes"
