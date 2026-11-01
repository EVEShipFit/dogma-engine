#!/usr/bin/env python3
"""Runs every fuzz target, and stops each once its corpus stops growing.

Usage: manage.py [--jobs N] [--quiet MINUTES] [TARGET ...]
"""

import argparse
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

FUZZ = Path(__file__).resolve().parent
LOGS = FUZZ / "logs"
FOREVER = 10 * 365 * 24 * 3600


def kill_group(group):
    for sig in (signal.SIGINT, signal.SIGKILL):
        try:
            os.killpg(group, sig)
            for _ in range(5):
                time.sleep(1)
                os.killpg(group, 0)
        except ProcessLookupError:
            return


class Target:
    def __init__(self, name):
        self.name = name
        self.state = "waiting"
        self.process = None
        self.jobs = 0
        self.started = None
        self.group_file = LOGS / f"{name}.pgid"
        self.kill_leftover()

    def kill_leftover(self):
        """Kills the fuzzer of an earlier manager that did not get to stop it."""
        try:
            group = int(self.group_file.read_text())
        except (FileNotFoundError, ValueError):
            return
        leftover = subprocess.run(["pgrep", "-g", str(group), "-f", f"release/{self.name} "], capture_output=True)
        if leftover.returncode == 0:
            kill_group(group)
        self.group_file.unlink()

    def newest(self):
        """Modification time of the newest corpus file."""
        try:
            return max((entry.stat().st_mtime for entry in os.scandir(FUZZ / "corpus" / self.name)), default=None)
        except FileNotFoundError:
            return None

    def count(self, directory):
        try:
            return sum(1 for _ in os.scandir(FUZZ / directory / self.name))
        except FileNotFoundError:
            return 0

    def start(self, jobs):
        LOGS.mkdir(exist_ok=True)
        with open(LOGS / f"{self.name}.log", "ab") as log:
            self.process = subprocess.Popen(
                [str(FUZZ / "run.sh"), self.name, str(FOREVER), str(jobs)],
                cwd=FUZZ, stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True,
            )
        self.group_file.write_text(str(self.process.pid))
        self.jobs = jobs
        self.state = "running"
        self.started = self.started or time.time()

    def stop(self, state):
        if self.process:
            kill_group(self.process.pid)
            self.process.wait()
            self.group_file.unlink(missing_ok=True)
        self.process = None
        self.jobs = 0
        self.state = state


def minutes(seconds):
    return "-" if seconds is None else f"{int(seconds // 60)}m"


def draw(targets, args):
    now = time.time()
    lines = [
        f"fuzz manager: {args.jobs} jobs, a target is done after {args.quiet}m without new corpus. Ctrl-C stops all.",
        "",
        f"{'target':<10} {'state':<8} {'jobs':>4} {'corpus':>7} {'last new':>9} {'running':>8} {'findings':>8}",
    ]
    for target in targets:
        newest = target.newest()
        lines.append(
            f"{target.name:<10} {target.state:<8} {target.jobs or '-':>4} {target.count('corpus'):>7} "
            f"{minutes(newest and now - newest):>9} {minutes(target.started and now - target.started):>8} "
            f"{target.count('artifacts'):>8}"
        )
    lines += ["", f"logs: {LOGS}/<target>.log, findings: {FUZZ}/artifacts/<target>/"]
    sys.stdout.write("\033[H\033[J" + "\n".join(lines) + "\n")
    sys.stdout.flush()


def balance(targets, budget):
    """Spreads the jobs over the targets not done, restarting those whose share changed."""
    active = [target for target in targets if target.state in ("waiting", "running")]
    for index, target in enumerate(active):
        jobs = max(1, budget // len(active) + (index < budget % len(active)))
        if target.jobs != jobs:
            target.stop("waiting")
            target.start(jobs)


def main():
    names = sorted(path.stem for path in (FUZZ / "fuzz_targets").glob("*.rs"))
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--jobs", type=int, default=os.cpu_count(), help="jobs over all targets (default: all CPUs)")
    parser.add_argument("--quiet", type=int, default=60, metavar="MINUTES", help="minutes without new corpus before a target is done (default: 60)")
    parser.add_argument("targets", nargs="*", metavar="TARGET", help=f"one of {', '.join(names)} (default: all)")
    args = parser.parse_args()
    for name in args.targets:
        if name not in names:
            parser.error(f"unknown target {name}")

    def interrupt(*_):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, interrupt)
    signal.signal(signal.SIGHUP, interrupt)

    subprocess.run(["cargo", "run", "--quiet", "--release", "--example", "seeds"], cwd=FUZZ, check=True)
    targets = [Target(name) for name in args.targets or names]
    try:
        balance(targets, args.jobs)
        while any(target.state == "running" for target in targets):
            now = time.time()
            changed = False
            for target in targets:
                if target.state != "running":
                    continue
                if target.process.poll() is not None:
                    target.stop("failed")
                    changed = True
                    continue
                newest = max(target.newest() or 0, target.started)
                if now - target.started > args.quiet * 60 and now - newest > args.quiet * 60:
                    target.stop("done")
                    changed = True
            if changed:
                balance(targets, args.jobs)
            draw(targets, args)
            time.sleep(5)
    except KeyboardInterrupt:
        pass
    finally:
        for target in targets:
            if target.state == "running":
                target.stop("stopped")
        draw(targets, args)


if __name__ == "__main__":
    main()
