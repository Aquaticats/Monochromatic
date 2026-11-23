#!/usr/bin/env python3
# What: the per-round ratios and medians of one row file of the shipped-overlap measurement.
# Why:  a timing is only read as a ratio to the same round's control arm, with the identical-binary
#       pair's own range beside it, so the script that makes those ratios is kept with the rows.
# Usage: python3 ratios.py <rows.jsonl> [<rows.jsonl>...]
import json
import statistics
import sys


def load(path):
    """Every row of one file, counted or not."""
    with open(path, encoding='utf-8') as handle:
        return [json.loads(line) for line in handle if line.strip()]


def median(values):
    """The upper middle element for an even count, as every round's own script uses."""
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def summarise(path):
    """One file's medians, per-round ratios against each arm, and step medians."""
    rows = load(path)
    counted = [row for row in rows if row['counted']]
    arms = sorted({row['arm'] for row in counted})
    print(f'== {path}')
    for arm in arms:
        mine = [row['totalMs'] for row in counted if row['arm'] == arm]
        walls = [row['wallMs'] for row in counted if row['arm'] == arm]
        failures = len([row for row in counted if row['arm'] == arm and row['failed']])
        print(f'  {arm}: n={len(mine)} failed={failures} median={median(mine):.1f} range=({min(mine):.1f} to {max(mine):.1f})')
        print(f'    walls: median={median(walls):.1f} range=({min(walls):.1f} to {max(walls):.1f})')
        steps = {}
        work = {}
        for row in counted:
            if row['arm'] != arm or row['failed']:
                continue
            for step, spent in row['steps'].items():
                steps.setdefault(step, []).append(spent)
            for step, spent in row['work'].items():
                work.setdefault(step, []).append(spent)
        for step in sorted(steps):
            print(f'    step {step}: median={median(steps[step]):.1f} range=({min(steps[step]):.1f} to {max(steps[step]):.1f}) calls={len(steps[step])}')
        for step in sorted(work):
            print(f'    work {step}: median={median(work[step]):.1f} range=({min(work[step]):.1f} to {max(work[step]):.1f}) calls={len(work[step])}')
    # Per-round ratios, only for rounds that hold every arm.
    rounds = {}
    for row in counted:
        rounds.setdefault(row['round'], {})[row['arm']] = row
    base = arms[0]
    for arm in arms[1:]:
        ratios = []
        for number in sorted(rounds):
            held = rounds[number]
            if base not in held or arm not in held:
                continue
            ratios.append((number, held[arm]['totalMs'] / held[base]['totalMs']))
        if ratios:
            values = [ratio for _number, ratio in ratios]
            print(f'  {arm} / {base}: ' + ', '.join(f'{value:.2f}' for value in values)
                  + f'  range=({min(values):.2f} to {max(values):.2f})')
    orders = {}
    for row in counted:
        orders.setdefault(row['round'], []).append((row['position'], row['arm']))
    for number in sorted(orders):
        print(f'  round {number}: ' + ', '.join(arm for _position, arm in sorted(orders[number])))


if __name__ == '__main__':
    for argument in sys.argv[1:]:
        summarise(argument)
    if len(sys.argv) < 2:
        raise SystemExit('usage: ratios.py <rows.jsonl>...')
    print('spread of medians above is what every difference must be read against')
    _ = statistics
