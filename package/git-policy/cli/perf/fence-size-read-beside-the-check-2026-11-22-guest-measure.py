#!/usr/bin/env python3
# What: the guest-side measurement of rung 1 (the btrfs snapshot rung) for the shipped overlap: one
#       `git worktree add` per arm per round, driven by ONE process inside the guest so no host round trip
#       sits between arms, on a source whose objects are loose and then packed in place.
# Why:  rung 1 is where the fence's size read moved off the waiting path, so this rung is where the shipped
#       build can shorten the add; the order rotates one place per round and every second pass runs the
#       reversed list, so each arm takes each slot once per pass and follows each other arm once.
# Usage: python3 guest-measure.py setup
#        python3 guest-measure.py adds <layout> <rounds>
#        python3 guest-measure.py gc
#        python3 guest-measure.py refuse | decline | restore
import json
import os
import shutil
import subprocess
import sys
import time

IMAGE = '/root/bt.img'
MOUNT = '/mnt/bt'
SOURCE = f'{MOUNT}/src'
DESTINATIONS = f'{MOUNT}/dests'
HOMES = f'{MOUNT}/homes'
OUT = '/root/out'
SHARE = '/mnt/shared'
TAR = f'{SHARE}/tree-40783c23ea.tar.zst'
RECORD = f'{SOURCE}/.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl'
BINARIES = {'branch': '/root/bin/branch', 'main': '/root/bin/main'}
# name, binary. The shipped build starts the read beside the source check on rung 1; that is the arm under
# test. `branch-again` is the same binary as `branch` and is the identical-binary control.
ARMS = [
    ('branch', 'branch'),
    ('main', 'main'),
    ('branch-again', 'branch'),
]
# The two arms the refused and declined sets need: the shipped build and its identical twin.
COST_ARMS = [
    ('branch', 'branch'),
    ('branch-again', 'branch'),
]
PLAIN = {'PATH': '/usr/sbin:/usr/bin:/sbin:/bin', 'HOME': f'{HOMES}/plain', 'GIT_CONFIG_NOSYSTEM': '1',
         'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.invalid',
         'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.invalid'}


def run(args, cwd=None, env=None, timeout=1800, check=True):
    """One command; returns the completed process; raises on a nonzero exit when check is set."""
    done = subprocess.run(args, cwd=cwd, env=env if env is not None else PLAIN, capture_output=True,
                          timeout=timeout)
    if check and done.returncode != 0:
        raise RuntimeError(f'{args} failed ({done.returncode}): {done.stderr.decode(errors="replace")[:600]}')
    return done


def median(values):
    """The upper middle element for an even count."""
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def setup():
    """The btrfs image, the fixture subvolume, and a loose source of the repository's tree."""
    os.makedirs(f'{HOMES}/plain', exist_ok=True)
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(MOUNT, exist_ok=True)
    run(['truncate', '--size=6G', IMAGE])
    run(['mkfs.btrfs', '--quiet', '--force', IMAGE])
    run(['mount', '-o', 'loop', IMAGE, MOUNT])
    run(['btrfs', 'subvolume', 'create', SOURCE])
    os.makedirs(DESTINATIONS, exist_ok=True)
    run(['tar', '--zstd', '--extract', '--file', TAR, '--directory', SOURCE])
    attributes = f'{SOURCE}/.gitattributes'
    with open(attributes, encoding='utf-8') as handle:
        kept = [line for line in handle.read().split('\n') if 'filter=lfs' not in line]
    with open(attributes, 'w', encoding='utf-8') as handle:
        handle.write('\n'.join(kept))
    with open(f'{SOURCE}/monochromatic-git-policy-cli.config.jsonc', 'w', encoding='utf-8') as handle:
        handle.write('{\n  "policies": {}\n}\n')
    run(['git', 'init', '--quiet', '--initial-branch=main'], cwd=SOURCE)
    run(['git', 'config', 'user.name', 'Fixture'], cwd=SOURCE)
    run(['git', 'config', 'user.email', 'fixture@example.invalid'], cwd=SOURCE)
    run(['git', 'config', 'commit.gpgsign', 'false'], cwd=SOURCE)
    # No policy runs on the fixture's own commits, so a measurement never pays a scanner over this tree.
    run(['git', 'add', '--all'], cwd=SOURCE)
    tree = run(['git', 'write-tree'], cwd=SOURCE).stdout.decode().strip()
    commit = run(['git', 'commit-tree', tree, '-m', 'fixture'], cwd=SOURCE).stdout.decode().strip()
    run(['git', 'update-ref', 'refs/heads/main', commit], cwd=SOURCE)
    run(['sync', '--file-system', SOURCE])
    # The stat data of a fresh checkout is racy for a moment; the check refuses until it settles.
    time.sleep(8)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    tracked = run(['git', 'ls-files'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': 'setup', 'tracked': len(tracked), 'statusLines': len([s for s in status if s]),
                      'commit': commit}))
    print(run(['git', 'count-objects', '-v'], cwd=SOURCE).stdout.decode().strip())


def arm_environment(name, binary, tag):
    """One arm's shim, home and runtime directory for one add."""
    shim = f'/root/shims/{name}'
    os.makedirs(shim, exist_ok=True)
    link = f'{shim}/git'
    if os.path.islink(link):
        os.remove(link)
    os.symlink(BINARIES[binary], link)
    home = f'{HOMES}/{name}-{tag}'
    os.makedirs(home, exist_ok=True)
    runtime = f'{HOMES}/runtime-{name}-{tag}'
    os.makedirs(runtime, mode=0o700, exist_ok=True)
    return {'PATH': f'{shim}:/usr/bin:/bin', 'HOME': home, 'XDG_RUNTIME_DIR': runtime, 'GIT_CONFIG_NOSYSTEM': '1'}


def last_record():
    """The record line the add appended last."""
    with open(RECORD, 'rb') as handle:
        lines = handle.read().decode().strip().split('\n')
    return json.loads(lines[-1])


def one_add(arm, tag, layout, counted):
    """One add through one arm, with its record and wall time."""
    name, binary = arm
    env = arm_environment(name, binary, tag)
    destination = f'{DESTINATIONS}/{name}-{tag}'
    branch = f'wt-{name}-{tag}'
    run(['sync', '--file-system', SOURCE])
    began = time.perf_counter_ns()
    done = run(['git', 'worktree', 'add', '--quiet', '-b', branch, destination], cwd=SOURCE, env=env,
               timeout=300, check=False)
    wall = (time.perf_counter_ns() - began) / 1e6
    record = last_record()
    failed = done.returncode != 0
    steps = {step: value['ms'] for step, value in (record.get('steps') or {}).items()}
    work = {step: value['ms'] for step, value in (record.get('work') or {}).items()}
    row = {'layout': layout, 'arm': name, 'tag': tag, 'counted': counted,
           'wallMs': round(wall, 1), 'totalMs': record.get('totalMs'),
           'route': (record.get('notes') or {}).get('route'), 'exitCode': record.get('exitCode'),
           'failed': failed, 'stderr': done.stderr.decode(errors='replace')[-300:] if failed else '',
           'steps': steps, 'work': work}
    deleted = run(['btrfs', 'subvolume', 'delete', '--commit-after', destination], check=False)
    if deleted.returncode != 0 and os.path.exists(destination):
        run(['btrfs', 'subvolume', 'delete', '--commit-after', '--recursive', destination], check=False)
    if os.path.exists(destination):
        shutil.rmtree(destination, ignore_errors=True)
    run(['git', 'worktree', 'prune'], cwd=SOURCE, check=False)
    run(['git', 'branch', '-D', branch], cwd=SOURCE, check=False)
    shutil.rmtree(env['HOME'], ignore_errors=True)
    shutil.rmtree(env['XDG_RUNTIME_DIR'], ignore_errors=True)
    return row


def adds(layout, rounds):
    """The counted rounds, with one uncounted warm-up add per arm."""
    arms = COST_ARMS if layout in ('refused', 'declined') else ARMS
    rows = []
    for arm in arms:
        rows.append(one_add(arm, f'warm-{arm[0]}', layout, False))
    for round_number in range(rounds):
        shift = round_number % len(arms)
        base = list(arms) if (round_number // len(arms)) % 2 == 0 else list(reversed(arms))
        order = base[shift:] + base[:shift]
        for position, arm in enumerate(order):
            row = one_add(arm, f'r{round_number + 1}p{position + 1}', layout, True)
            row['round'] = round_number + 1
            row['position'] = position + 1
            rows.append(row)
    tagged = f'{layout}-{rounds}rounds-{int(time.time())}'
    with open(f'{OUT}/{tagged}-adds.jsonl', 'w', encoding='utf-8') as handle:
        for row in rows:
            handle.write(json.dumps(row) + '\n')
    shutil.copyfile(f'{OUT}/{tagged}-adds.jsonl', f'{SHARE}/{tagged}-adds.jsonl')
    counted = [row for row in rows if row['counted']]
    summary = {}
    for name, _binary in arms:
        mine = [row for row in counted if row['arm'] == name]
        good = [row for row in mine if not row['failed']]
        steps = {}
        work = {}
        for row in good:
            for step, spent in row['steps'].items():
                steps.setdefault(step, []).append(spent)
            for step, spent in row['work'].items():
                work.setdefault(step, []).append(spent)
        summary[name] = {
            'n': len(good), 'failed': len([row for row in mine if row['failed']]),
            'routes': sorted({str(row['route']) for row in good}),
            'medianTotalMs': median([row['totalMs'] for row in good]) if good else None,
            'totals': [row['totalMs'] for row in good],
            'walls': [row['wallMs'] for row in good],
            'stepMedians': {step: round(median(values), 1) for step, values in steps.items()},
            'workMedians': {step: round(median(values), 1) for step, values in work.items()},
        }
    print(json.dumps({'phase': 'adds', 'layout': layout, 'summary': summary}), flush=True)


def gc():
    """Pack the source's objects in place, the same tree, no new commit."""
    run(['git', 'gc', '--quiet'], cwd=SOURCE)
    run(['sync', '--file-system', SOURCE])
    time.sleep(8)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': 'gc', 'statusLines': len([s for s in status if s])}))
    print(run(['git', 'count-objects', '-v'], cwd=SOURCE).stdout.decode().strip())


def refuse():
    """Make the source one the check refuses: the sparse-checkout file no rung serves."""
    with open(f'{SOURCE}/.git/info/sparse-checkout', 'w', encoding='utf-8') as handle:
        handle.write('')
    print(json.dumps({'phase': 'refuse'}))


def restore():
    """Take the refusal away again."""
    path = f'{SOURCE}/.git/info/sparse-checkout'
    if os.path.exists(path):
        os.remove(path)
    print(json.dumps({'phase': 'restore'}))


def decline():
    """Make the fence decline every path: one `ident` rule converts all of them, and no byte changes."""
    with open(f'{SOURCE}/.gitattributes', 'a', encoding='utf-8') as handle:
        handle.write('* ident\n')
    run(['git', 'add', '--', '.gitattributes'], cwd=SOURCE)
    run(['git', 'commit', '--quiet', '--message=ident'], cwd=SOURCE)
    run(['sync', '--file-system', SOURCE])
    time.sleep(8)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': 'decline', 'statusLines': len([s for s in status if s])}))


if __name__ == '__main__':
    phase = sys.argv[1]
    if phase == 'setup':
        setup()
    elif phase == 'adds':
        adds(sys.argv[2], int(sys.argv[3]))
    elif phase == 'gc':
        gc()
    elif phase == 'refuse':
        refuse()
    elif phase == 'restore':
        restore()
    elif phase == 'decline':
        decline()
    else:
        raise SystemExit(f'unknown phase {phase}')
