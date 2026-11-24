#!/usr/bin/env python3
# What: the declined add's step cost before and after the map-drop fix, beside main and the two
#       identical-binary controls, from the stepcost driver's rows.
# Why:  a timing is read as a ratio to the same round's control arm, with its range; a step's
#       cost is the record's own exclusive step times, so a real change has to sit in a named step.
# Usage: python3 -I step-ratios.py <rows.jsonl>...
import json
import math
import random
import statistics
import sys

STEPS = ('chooseRung', 'startBesideTheCheck', 'abandonRead', 'abandonTakeTheAnswer', 'abandonEndTheChild')
ARMS = ('main', 'before', 'after', 'before-again', 'after-again')


def median(values):
    return statistics.median(values)


def bootstrap_interval(values, resamples=4000, seed=7):
    generator = random.Random(seed)
    medians = [median([values[generator.randrange(len(values))] for _ in values]) for _ in range(resamples)]
    medians.sort()
    return medians[int(0.025 * resamples)], medians[int(0.975 * resamples)]


def ratios(rounds, numerator, denominator, pick):
    out = []
    for number in sorted(rounds):
        arms = rounds[number]
        if numerator in arms and denominator in arms:
            top = pick(arms[numerator])
            bottom = pick(arms[denominator])
            if top is not None and bottom is not None and bottom > 0:
                out.append(top / bottom)
    return out


def report(values):
    if not values:
        return 'none'
    logs = [math.log(value) for value in values]
    low, high = bootstrap_interval(logs)
    return f'median {median(values):.3f} (range {min(values):.3f} to {max(values):.3f}, 95% {math.exp(low):.3f} to {math.exp(high):.3f})'


def step_of(row, step):
    value = row.get('steps', {}).get(step)
    return value


def whole_of(row):
    own = row.get('steps', {})
    return (own.get('chooseRung', 0.0) + own.get('startBesideTheCheck', 0.0) + own.get('abandonRead', 0.0) + own.get('abandonTakeTheAnswer', 0.0) + own.get('abandonEndTheChild', 0.0))


def total_of(row):
    return row.get('totalMs')


for path in sys.argv[1:]:
    rows = [json.loads(line) for line in open(path, encoding='utf-8') if line.strip()]
    counted = [row for row in rows if row['counted']]
    rounds = {}
    for row in counted:
        rounds.setdefault(row['round'], {})[row['arm']] = row
    print(f'== {path.split("/")[-1]}  ({len(rounds)} rounds)')
    medians = {}
    for arm in ARMS:
        mine = [row for row in counted if row['arm'] == arm]
        medians[arm] = {}
        for step in STEPS:
            values = [step_of(row, step) for row in mine if step_of(row, step) is not None]
            if values:
                medians[arm][step] = median(values)
        medians[arm]['chosenWhole'] = median([whole_of(row) for row in mine])
        totals = [total_of(row) for row in mine if total_of(row) is not None]
        if totals:
            medians[arm]['totalMs'] = median(totals)
    for step in ('chosenWhole',) + STEPS + ('totalMs',):
        print(f'  {step}: ' + ' | '.join(f'{arm} {medians[arm].get(step, float("nan")):.2f}' for arm in ARMS))
        if step == 'chosenWhole':
            print(f'    before/main {report(ratios(rounds, "before", "main", lambda row: whole_of(row)))}')
            print(f'    after/main {report(ratios(rounds, "after", "main", lambda row: whole_of(row)))}')
            print(f'    before-again/before {report(ratios(rounds, "before-again", "before", lambda row: whole_of(row)))}')
            print(f'    after-again/after {report(ratios(rounds, "after-again", "after", lambda row: whole_of(row)))}')
        elif step != 'totalMs':
            pick = (lambda row, s=step: step_of(row, s))
            print(f'    before/main {report(ratios(rounds, "before", "main", pick))}')
            print(f'    after/main {report(ratios(rounds, "after", "main", pick))}')
            print(f'    before-again/before {report(ratios(rounds, "before-again", "before", pick))}')
            print(f'    after-again/after {report(ratios(rounds, "after-again", "after", pick))}')
        else:
            print(f'    before/main {report(ratios(rounds, "before", "main", total_of))}')
            print(f'    after/main {report(ratios(rounds, "after", "main", total_of))}')
            print(f'    before-again/before {report(ratios(rounds, "before-again", "before", total_of))}')
            print(f'    after-again/after {report(ratios(rounds, "after-again", "after", total_of))}')
    for arm in ARMS:
        mine = [row for row in counted if row['arm'] == arm]
        print(f'  {arm}: {len(mine)} adds, {len([row for row in mine if row["failed"]])} failed, routes {sorted({str(row["route"]) for row in mine})}, read notes {sorted({str((row.get("notes") or {}).get("fenceSizesBesideTheCheck")) for row in mine})}')
