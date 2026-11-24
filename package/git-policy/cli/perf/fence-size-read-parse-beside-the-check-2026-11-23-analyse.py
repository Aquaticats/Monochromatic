#!/usr/bin/env python3
# What: the pass's statistics on this round's rows: per-round paired differences of a step, the median with a
#       95 percent percentile bootstrap interval, and the identical-twin control's same statistic.
# Why:  the reading rule fixed before any row (keep/reading-rule-fixed-before-rows.md) reads a step as resolved
#       worse only from a paired interval above zero that the control's interval does not cover.
# Usage: python3 analyse.py <rows.jsonl> <step-or-sum> [numerator-arm]
import json
import random
import statistics
import sys

def step_of(row, which):
    steps = row.get('steps') or {}
    if which == 'choose':
        return steps.get('chooseRung', 0.0)
    if which == 'step':
        return sum(steps.get(k, 0.0) for k in ('chooseRung', 'startBesideTheCheck', 'abandonRead', 'abandonTakeTheAnswer', 'abandonEndTheChild'))
    return steps.get(which, 0.0)

def bootstrap(values, resamples=10000, seed=7):
    generator = random.Random(seed)
    medians = sorted(statistics.median([values[generator.randrange(len(values))] for _ in values]) for _ in range(resamples))
    return medians[int(0.025 * resamples)], medians[int(0.975 * resamples)]

path, which = sys.argv[1], sys.argv[2]
rows = [json.loads(line) for line in open(path, encoding='utf-8') if line.strip()]
counted = [row for row in rows if row['counted']]
rounds = {}
for row in counted:
    rounds.setdefault(row['round'], {})[row['arm']] = row
arms = ('base', 'main', 'fix', 'fix-again')
print(f'== {path.split("/")[-1]}  {which}  ({len(rounds)} rounds)')
for arm in arms:
    mine = [step_of(row, which) for row in counted if row['arm'] == arm]
    if mine:
        print(f'  {arm}: median {statistics.median(mine):.3f} ms, range {min(mine):.3f} to {max(mine):.3f}, n {len(mine)}')
def paired(a, b):
    out = []
    for number in sorted(rounds):
        if a in rounds[number] and b in rounds[number]:
            out.append(step_of(rounds[number][a], which) - step_of(rounds[number][b], which))
    return out
for a, b in (('base', 'main'), ('main', 'fix'), ('base', 'fix'), ('fix', 'fix-again')):
    v = paired(a, b)
    if not v:
        continue
    low, high = bootstrap(v)
    positive = len([x for x in v if x > 0])
    print(f'  {a} minus {b}: median {statistics.median(v):+.3f} ms [{low:+.3f}, {high:+.3f}], range {min(v):+.3f} to {max(v):+.3f}, positive {positive} of {len(v)}')
