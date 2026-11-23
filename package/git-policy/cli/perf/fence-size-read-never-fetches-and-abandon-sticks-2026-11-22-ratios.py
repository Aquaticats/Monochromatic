#!/usr/bin/env python3
# What: per-round wall-clock ratios of the recheck's guest rows: branch to main, and the identical-binary control
#       branch-again to branch, with the strict reading rule fixed before any row was seen.
# Why:  a timing is read only as a ratio to the same round's other arm with the control's own range beside it.
#       The rule: the branch is RESOLVED SLOWER than main only when every round's branch/main ratio lies above the largest
#       control ratio, RESOLVED FASTER only when every round lies below the smallest control ratio; anything else is NOT
#       RESOLVED. The median ratios and the count of rounds outside the control's range are printed beside the verdict.
# Usage: python3 -I ratios-recheck.py <rows.jsonl>...
import json
import sys


def median(values):
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


for path in sys.argv[1:]:
    rows = [json.loads(line) for line in open(path, encoding='utf-8') if line.strip()]
    counted = [row for row in rows if row['counted']]
    rounds = {}
    for row in counted:
        rounds.setdefault(row['round'], {})[row['arm']] = row
    print(f'== {path.split("/")[-1]}  ({len(rounds)} rounds)')
    for arm in ('main', 'branch', 'branch-again'):
        walls = [row['wallMs'] for row in counted if row['arm'] == arm]
        print(f'  {arm}: n={len(walls)} median wall {median(walls):.1f} ms, range {min(walls):.1f} to {max(walls):.1f}')
    ratios = [rounds[number]['branch']['wallMs'] / rounds[number]['main']['wallMs'] for number in sorted(rounds)]
    control = [rounds[number]['branch-again']['wallMs'] / rounds[number]['branch']['wallMs'] for number in sorted(rounds)]
    high = max(control)
    low = min(control)
    above = len([value for value in ratios if value > high])
    below = len([value for value in ratios if value < low])
    if above == len(ratios):
        verdict = 'RESOLVED SLOWER (every round above the control range)'
    elif below == len(ratios):
        verdict = 'RESOLVED FASTER (every round below the control range)'
    else:
        verdict = 'NOT RESOLVED'
    shown = ', '.join(f'{value:.2f}' for value in ratios)
    print(f'  branch / main per round: {shown}')
    print(f'    range {min(ratios):.2f} to {max(ratios):.2f}, median {median(ratios):.2f}')
    shown_control = ', '.join(f'{value:.2f}' for value in control)
    print(f'  branch-again / branch (identical binary) per round: {shown_control}')
    print(f'    control range {low:.2f} to {high:.2f}, median {median(control):.2f}')
    print(f'  rounds above the control range: {above} of {len(ratios)}, below: {below}; verdict: {verdict}')
    slower_rounds = len([value for value in ratios if value > 1.0])
    control_slower = len([value for value in control if value > 1.0])
    print(f'  sign counts: branch slower than main in {slower_rounds} of {len(ratios)} rounds; control twin slower in {control_slower} of {len(control)}')
    for arm in ('branch', 'main'):
        waits = [row['steps'].get('fenceSizesWaitingForTheRead') for row in counted if row['arm'] == arm]
        waits = [value for value in waits if value is not None]
        work = [row['work'].get('fenceSizesBesideTheCheck') for row in counted if row['arm'] == arm]
        work = [value for value in work if isinstance(value, (int, float))]
        routes = sorted({str(row['route']) for row in counted if row['arm'] == arm})
        notes = sorted({str((row.get('notes') or {}).get('fenceSizesBesideTheCheck')) for row in counted if row['arm'] == arm})
        print(f'  {arm}: wait step {waits and median(waits)} ms over {len(waits)} rows, read work {work and median(work)} ms over {len(work)} rows, routes {routes}, read note {notes}')
    print('  failed adds per arm: ' + ', '.join(f'{arm} {len([r for r in counted if r["arm"] == arm and r["failed"]])}' for arm in ('main', 'branch', 'branch-again')))
    print('  ls-tree processes seen after an add: ' + str(sum(len(row.get('lsTreeLeftBehind') or []) for row in counted)))
