#!/usr/bin/env python3
# What: the whole add's per-round ratio of fix to main, and the identical twin's ratio to fix, per shape.
import json
import random
import statistics
import sys

def bootstrap(values, resamples=10000, seed=7):
    generator = random.Random(seed)
    medians = sorted(statistics.median([values[generator.randrange(len(values))] for _ in values]) for _ in range(resamples))
    return medians[int(0.025 * resamples)], medians[int(0.975 * resamples)]

for path in sys.argv[1:]:
    rows = [json.loads(line) for line in open(path, encoding='utf-8') if line.strip()]
    counted = [row for row in rows if row['counted']]
    rounds = {}
    for row in counted:
        rounds.setdefault(row['round'], {})[row['arm']] = row
    ratios, control = [], []
    for number in sorted(rounds):
        arms = rounds[number]
        if 'fix' in arms and 'main' in arms:
            ratios.append(arms['fix']['totalMs'] / arms['main']['totalMs'])
        if 'fix-again' in arms and 'fix' in arms:
            control.append(arms['fix-again']['totalMs'] / arms['fix']['totalMs'])
    low, high = bootstrap(ratios)
    clow, chigh = bootstrap(control)
    print(f'{path.split("/")[-1][:34]:34} fix/main {statistics.median(ratios):.3f} [{low:.3f}, {high:.3f}] range {min(ratios):.3f} to {max(ratios):.3f} | twin/fix {statistics.median(control):.3f} [{clow:.3f}, {chigh:.3f}] range {min(control):.3f} to {max(control):.3f}')
