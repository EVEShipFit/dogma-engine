#!/usr/bin/env python3
"""Line coverage of the crates by the fuzz corpus, per file, over all targets.

Usage: coverage.py [--missed FILE] [TARGET ...]
"""

import argparse
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

FUZZ = Path(__file__).resolve().parent
CRATES = FUZZ.parent / "crates"
TARGETS = ["dna", "eft", "esf", "esi", "fit", "fit_json", "killmail", "link"]
IGNORE = r"(\.cargo|rustc|/fuzz/|_generated\.rs)"


def run(*args, **kwargs):
    return subprocess.run(args, cwd=FUZZ, check=True, **kwargs)


def quiet(*args):
    result = subprocess.run(args, cwd=FUZZ, capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f"{' '.join(args)} failed:\n{result.stderr[-4000:]}")


def llvm_tools():
    sysroot = run("rustc", "+nightly", "--print", "sysroot", capture_output=True, text=True).stdout.strip()
    host = next(
        line.split(": ")[1]
        for line in run("rustc", "+nightly", "-vV", capture_output=True, text=True).stdout.splitlines()
        if line.startswith("host: ")
    )
    return Path(sysroot) / "lib" / "rustlib" / host / "bin", host


def collect(target, tools, host):
    """Per file, per line, whether the corpus of `target` reached it."""
    corpus = FUZZ / "corpus" / target
    corpus.mkdir(parents=True, exist_ok=True)
    quiet("cargo", "+nightly", "fuzz", "cmin", target, str(corpus))
    quiet("cargo", "+nightly", "fuzz", "coverage", target, str(corpus))

    binary = FUZZ / "target" / host / "coverage" / host / "release" / target
    profile = FUZZ / "coverage" / target / "coverage.profdata"
    lcov = run(
        str(tools / "llvm-cov"), "export", "-format=lcov", str(binary),
        f"-instr-profile={profile}", f"--ignore-filename-regex={IGNORE}",
        capture_output=True, text=True,
    ).stdout

    lines = defaultdict(dict)
    filename = None
    for line in lcov.splitlines():
        if line.startswith("SF:"):
            filename = str(Path(line[3:]).relative_to(CRATES))
        elif line.startswith("DA:"):
            number, hits = line[3:].split(",")[:2]
            lines[filename][int(number)] = lines[filename].get(int(number), False) or int(hits) > 0
    return lines


def ranges(numbers):
    out = []
    for number in sorted(numbers):
        if out and out[-1][1] == number - 1:
            out[-1][1] = number
        else:
            out.append([number, number])
    return ", ".join(str(a) if a == b else f"{a}-{b}" for a, b in out)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--missed", metavar="FILE", help="list the lines no target reached in FILE, like esf-format/src/eft.rs")
    parser.add_argument("targets", nargs="*", default=TARGETS)
    args = parser.parse_args()

    tools, host = llvm_tools()
    merged = defaultdict(lambda: defaultdict(bool))
    per_target = {}
    for target in args.targets:
        print(f"collecting {target} ...", file=sys.stderr)
        lines = collect(target, tools, host)
        per_target[target] = sum(hit for file in lines.values() for hit in file.values())
        for filename, file in lines.items():
            for number, hit in file.items():
                merged[filename][number] |= hit

    if args.missed:
        missed = [number for number, hit in merged[args.missed].items() if not hit]
        print(ranges(missed) or "none")
        return

    rows = sorted(merged.items(), key=lambda item: sum(item[1].values()) / len(item[1]))
    width = max(len(filename) for filename in merged)
    total = hit = 0
    for filename, file in rows:
        reached = sum(file.values())
        total += len(file)
        hit += reached
        print(f"{filename:<{width}} {reached:>5}/{len(file):<5} {100 * reached / len(file):5.1f}%")
    print(f"{'TOTAL':<{width}} {hit:>5}/{total:<5} {100 * hit / total:5.1f}%")
    print()
    for target, reached in per_target.items():
        print(f"{target:<10} {reached:>5} lines")

    (FUZZ / "coverage").mkdir(exist_ok=True)
    (FUZZ / "coverage" / "lines.json").write_text(json.dumps(merged))


if __name__ == "__main__":
    main()
