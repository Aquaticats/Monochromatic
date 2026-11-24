#!/usr/bin/env python3
# What: this pass's guest driver for rung 1 (the btrfs snapshot) on Git 2.56.1: served (clean), dirty, refused,
#       fail-fast refused, cone-sparse refused and declined shapes, loose and packed, with four arms:
#       base (main before the feature), main (as it stands), merge (the integration commit) and merge-again
#       (the merge's identical binary, the control).
# Why:  adapted from the integration recheck's `driver-guest.py` of 2026-11-22 (same fixture, same rotation, same
#       sync and warm-up rules); the arms and the dirty and partial-clone shapes are this pass's.
# Usage: python3 driver.py setup | gc | refuse | restore | decline | undecline | cone | coneoff | dirty | undirty
#        python3 driver.py adds <label> <rounds> [failfast]
#        python3 driver.py partial <rounds>
import hashlib
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
TAR = f'{SHARE}/tree.tar.zst'
RECORD_NAME = 'monochromatic-git-policy-cli-worktree-add-timing.jsonl'
BINARIES = {'base': '/root/bin/base', 'main': '/root/bin/main', 'c': '/root/bin/c'}
ARMS = [('base', 'base'), ('main', 'main'), ('c', 'c'), ('c-again', 'c')]
# Every arm of this comparison starts no read but the planted one needs nothing; C starts none at all.
PLANT_C = {'NEVER_FETCHES_PLANT_C': '1'}
GIT = '/opt/git-2.56.1/bin'
PLAIN = {'PATH': f'{GIT}:/usr/sbin:/sbin:/usr/bin:/bin', 'HOME': f'{HOMES}/plain', 'GIT_CONFIG_NOSYSTEM': '1',
         'GIT_AUTHOR_NAME': 'Fixture', 'GIT_AUTHOR_EMAIL': 'fixture@example.invalid',
         'GIT_COMMITTER_NAME': 'Fixture', 'GIT_COMMITTER_EMAIL': 'fixture@example.invalid'}
FIXTURE_COMMIT = f'{MOUNT}/fixture-commit'
DIRTY_FILE = 'README.md'


def run(args, cwd=None, env=None, timeout=1800, check=True):
    """One command; raises on a nonzero exit when check is set."""
    done = subprocess.run(args, cwd=cwd, env=env if env is not None else PLAIN, capture_output=True, timeout=timeout)
    if check and done.returncode != 0:
        raise RuntimeError(f'{args} failed ({done.returncode}): {done.stderr.decode(errors="replace")[:600]}')
    return done


def median(values):
    """The upper middle element for an even count."""
    ordered = sorted(values)
    return ordered[len(ordered) // 2]


def loadavg():
    """The guest's one-minute load average."""
    with open('/proc/loadavg', encoding='utf-8') as handle:
        return handle.read().split()[0]


def arm_environment(name, binary, tag, extra=None):
    """One arm's shim, home and runtime directory for one add.

    `/usr/bin` and `/bin` are left off the path so the wrapper's candidate sequence cannot promote
    the guest's 2.43.0 ahead of the 2.56.1 built from source.
    """
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
    environment = {'PATH': f'{shim}:{GIT}:/usr/sbin:/sbin', 'HOME': home, 'XDG_RUNTIME_DIR': runtime,
                   'GIT_CONFIG_NOSYSTEM': '1'}
    if name in ('c', 'c-again'):
        environment.update(PLANT_C)
    if extra:
        environment.update(extra)
    return environment


def setup():
    """The btrfs image, the fixture subvolume, and a loose source of the repository's tree."""
    os.makedirs(f'{HOMES}/plain', exist_ok=True)
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(MOUNT, exist_ok=True)
    os.makedirs('/root/bin', exist_ok=True)
    for name in BINARIES:
        shutil.copyfile(f'{SHARE}/bin-{name}', f'/root/bin/{name}')
        os.chmod(f'/root/bin/{name}', 0o755)
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
    run(['git', 'add', '--all'], cwd=SOURCE)
    tree = run(['git', 'write-tree'], cwd=SOURCE).stdout.decode().strip()
    commit = run(['git', 'commit-tree', tree, '-m', 'fixture'], cwd=SOURCE).stdout.decode().strip()
    run(['git', 'update-ref', 'refs/heads/main', commit], cwd=SOURCE)
    with open(FIXTURE_COMMIT, 'w', encoding='utf-8') as handle:
        handle.write(commit)
    run(['sync', '--file-system', SOURCE])
    time.sleep(8)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    tracked = run(['git', 'ls-files'], cwd=SOURCE).stdout.decode().strip().split('\n')
    version = run(['git', '--version']).stdout.decode().strip()
    bin_hashes = {name: hashlib.sha256(open(f'/root/bin/{name}', 'rb').read()).hexdigest() for name in BINARIES}
    lstree = run(['git', 'ls-tree', '-r', '-l', '-z', commit], cwd=SOURCE).stdout
    total_bytes = sum(int(entry.split(b'\t')[0].split()[3]) for entry in lstree.split(b'\0') if entry and entry.split(b'\t')[0].split()[3].isdigit())
    print(json.dumps({'phase': 'setup', 'tracked': len(tracked), 'statusLines': len([s for s in status if s]),
                      'commit': commit, 'git': version, 'binaries': bin_hashes,
                      'lsTreeOutputBytes': len(lstree), 'blobBytesTotal': total_bytes,
                      'meanBlobBytes': round(total_bytes / max(1, len(tracked)), 1)}), flush=True)
    print(run(['git', 'count-objects', '-v'], cwd=SOURCE).stdout.decode().strip(), flush=True)


def record_lines(source):
    """Every line of the add-timing file, or none before the first add."""
    path = f'{source}/.git/{RECORD_NAME}'
    if not os.path.exists(path):
        return []
    with open(path, 'rb') as handle:
        return [line for line in handle.read().decode().split('\n') if line.strip()]


def processes_named(fragment):
    """The `pid cmdline` lines of live processes whose command line holds `fragment`, read from /proc."""
    found = []
    me = str(os.getpid())
    for entry in os.listdir('/proc'):
        if not entry.isdigit() or entry == me:
            continue
        try:
            with open(f'/proc/{entry}/cmdline', 'rb') as handle:
                text = handle.read().replace(b'\0', b' ').decode(errors='replace').strip()
        except OSError:
            continue
        if fragment in text and 'python3' not in text:
            found.append(f'{entry} {text[:160]}')
    return found


def one_add(arm, tag, layout, counted, failfast=False, source=SOURCE, verbose=False):
    """One add through one arm, with its record and wall time."""
    name, binary = arm
    extra = {'MONOCHROMATIC_VERBOSE': 'true'} if verbose else None
    env = arm_environment(name, binary, tag, extra)
    destination = f'{DESTINATIONS}/{name}-{tag}'
    branch = 'main' if failfast else f'wt-{name}-{tag}'
    run(['sync', '--file-system', source])
    before = len(record_lines(source))
    load_before = loadavg()
    began = time.perf_counter_ns()
    done = run(['git', 'worktree', 'add', '--quiet', '-b', branch, destination], cwd=source, env=env,
               timeout=300, check=False)
    wall = (time.perf_counter_ns() - began) / 1e6
    lines = record_lines(source)
    record = json.loads(lines[-1]) if len(lines) > before else {}
    failed = done.returncode != 0
    steps = {step: value['ms'] for step, value in (record.get('steps') or {}).items()}
    work = {step: (value['ms'] if isinstance(value, dict) else value) for step, value in (record.get('work') or {}).items()}
    status_lines = None
    destination_files = None
    if not failed and os.path.isdir(destination):
        destination_files = sum(len(files) for walked, _dirs, files in os.walk(destination) if '/.git' not in walked)
        status_lines = len([s for s in run(['git', 'status', '--porcelain'], cwd=destination, check=False)
                            .stdout.decode().split('\n') if s])
    normalised = done.stderr.decode(errors='replace').replace(destination, 'DEST').replace(branch, 'BRANCH')
    for other_arm in ('c-again', 'c', 'main', 'base'):
        normalised = normalised.replace(other_arm + '-', 'ARM-')
    row = {'layout': layout, 'arm': name, 'tag': tag, 'counted': counted, 'failfast': failfast,
           'stderrNormalised': normalised[-600:], 'guestLoad': load_before,
           'wallMs': round(wall, 1), 'totalMs': record.get('totalMs'),
           'recordAppended': len(lines) > before,
           'route': (record.get('notes') or {}).get('route'), 'notes': record.get('notes'), 'exitCode': done.returncode,
           'failed': failed, 'stderr': done.stderr.decode(errors='replace')[-200:] if failed else '',
           'destinationStatusLines': status_lines, 'destinationFiles': destination_files,
           'lsTreeLeftBehind': processes_named('ls-tree'),
           'steps': steps, 'work': work}
    if verbose:
        row['verboseLines'] = [line for line in done.stderr.decode(errors='replace').split('\n')
                               if 'worktreeCloneConversionSizeRead' in line]
    deleted = run(['btrfs', 'subvolume', 'delete', '--commit-after', destination], check=False)
    if deleted.returncode != 0 and os.path.exists(destination):
        run(['btrfs', 'subvolume', 'delete', '--commit-after', '--recursive', destination], check=False)
    if os.path.exists(destination):
        shutil.rmtree(destination, ignore_errors=True)
    run(['git', 'worktree', 'prune'], cwd=source, check=False)
    if not failfast:
        run(['git', 'branch', '-D', branch], cwd=source, check=False)
    shutil.rmtree(env['HOME'], ignore_errors=True)
    shutil.rmtree(env['XDG_RUNTIME_DIR'], ignore_errors=True)
    return row


def adds(label, rounds, failfast=False):
    """The counted rounds, with one uncounted warm-up add per arm; every layout runs all four arms."""
    rows = []
    for arm in ARMS:
        rows.append(one_add(arm, f'warm-{arm[0]}', label, False, failfast))
    for round_number in range(rounds):
        shift = round_number % len(ARMS)
        base = list(ARMS) if (round_number // len(ARMS)) % 2 == 0 else list(reversed(ARMS))
        order = base[shift:] + base[:shift]
        for position, arm in enumerate(order):
            row = one_add(arm, f'r{round_number + 1}p{position + 1}', label, True, failfast)
            row['round'] = round_number + 1
            row['position'] = position + 1
            rows.append(row)
    tagged = f'{label}{"-failfast" if failfast else ""}-{rounds}rounds-{int(time.time())}'
    with open(f'{OUT}/{tagged}-adds.jsonl', 'w', encoding='utf-8') as handle:
        for row in rows:
            handle.write(json.dumps(row) + '\n')
    shutil.copyfile(f'{OUT}/{tagged}-adds.jsonl', f'{SHARE}/{tagged}-adds.jsonl')
    counted = [row for row in rows if row['counted']]
    summary = {}
    for name, _binary in ARMS:
        mine = [row for row in counted if row['arm'] == name]
        walls = [row['wallMs'] for row in mine]
        steps = {}
        for row in mine:
            for step, spent in row['steps'].items():
                steps.setdefault(step, []).append(spent)
        summary[name] = {
            'n': len(mine), 'failed': len([row for row in mine if row['failed']]),
            'routes': sorted({str(row['route']) for row in mine}),
            'destinationStatusLines': sorted({str(row['destinationStatusLines']) for row in mine}),
            'destinationFiles': sorted({str(row['destinationFiles']) for row in mine}),
            'lsTreeLeftBehind': sum(len(row['lsTreeLeftBehind']) for row in mine),
            'medianWallMs': median(walls), 'minWallMs': min(walls), 'maxWallMs': max(walls),
            'stepMedians': {step: round(median(values), 2) for step, values in steps.items()},
        }
    print(json.dumps({'phase': 'adds', 'label': tagged, 'summary': summary}), flush=True)


def gc():
    """Pack the source's objects in place, the same tree, no new commit."""
    run(['git', 'gc', '--quiet'], cwd=SOURCE)
    run(['sync', '--file-system', SOURCE])
    time.sleep(4)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': 'gc', 'statusLines': len([s for s in status if s])}))
    print(run(['git', 'count-objects', '-v'], cwd=SOURCE).stdout.decode().strip(), flush=True)


def refresh_and_report(phase):
    """Let the files settle, refresh the index, and print the status line count."""
    run(['sync', '--file-system', SOURCE])
    time.sleep(8)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': phase, 'statusLines': len([line for line in status if line])}), flush=True)


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
    refresh_and_report('decline')


def undecline():
    """Take the decline commit away again, so the next shape starts from the fixture commit."""
    with open(FIXTURE_COMMIT, encoding='utf-8') as handle:
        commit = handle.read().strip()
    run(['git', 'reset', '--hard', '--quiet', commit], cwd=SOURCE)
    refresh_and_report('undecline')


def dirty():
    """An ordinary uncommitted change: one tracked file edited, so the add snapshots and restores."""
    with open(f'{SOURCE}/{DIRTY_FILE}', 'a', encoding='utf-8') as handle:
        handle.write('\nan uncommitted line\n')
    run(['sync', '--file-system', SOURCE])
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': 'dirty', 'statusLines': len([line for line in status if line])}), flush=True)


def undirty():
    """Take the edit away again."""
    run(['git', 'checkout', '--quiet', '--', DIRTY_FILE], cwd=SOURCE)
    refresh_and_report('undirty')


def cone():
    """A real cone sparse checkout in the source: the check refuses it, and Git's checkout writes only the cone."""
    path = f'{SOURCE}/.git/info/sparse-checkout'
    if os.path.exists(path):
        os.remove(path)
    run(['git', 'sparse-checkout', 'init', '--cone'], cwd=SOURCE)
    run(['git', 'sparse-checkout', 'set', 'doc/decision'], cwd=SOURCE)
    run(['sync', '--file-system', SOURCE])
    time.sleep(4)
    present = sum(len(files) for walked, _dirs, files in os.walk(SOURCE) if '/.git' not in walked)
    print(json.dumps({'phase': 'cone', 'sourceFilesPresent': present}))


def clear_sparse():
    """Take every trace of a sparse checkout away: the file, the worktree config and the extension."""
    run(['git', 'sparse-checkout', 'disable'], cwd=SOURCE, check=False)
    path = f'{SOURCE}/.git/info/sparse-checkout'
    if os.path.exists(path):
        os.remove(path)
    worktree_config = f'{SOURCE}/.git/config.worktree'
    if os.path.exists(worktree_config):
        os.remove(worktree_config)
    run(['git', 'config', '--unset', 'extensions.worktreeConfig'], cwd=SOURCE, check=False)


def coneoff():
    """Leave the cone again, so the next shape starts from a whole checkout."""
    clear_sparse()
    run(['sync', '--file-system', SOURCE])
    time.sleep(4)
    present = sum(len(files) for walked, _dirs, files in os.walk(SOURCE) if '/.git' not in walked)
    run(['git', 'update-index', '--really-refresh'], cwd=SOURCE, check=False)
    status = run(['git', 'status', '--porcelain'], cwd=SOURCE).stdout.decode().strip().split('\n')
    print(json.dumps({'phase': 'coneoff', 'sourceFilesPresent': present,
                      'sparseFile': os.path.exists(f'{SOURCE}/.git/info/sparse-checkout'),
                      'worktreeConfig': os.path.exists(f'{SOURCE}/.git/config.worktree'),
                      'statusLines': len([line for line in status if line])}), flush=True)


def unpack():
    """Make the packed source loose again: unpack every pack in place."""
    pack_dir = f'{SOURCE}/.git/objects/pack'
    for name in sorted(os.listdir(pack_dir)):
        if name.endswith('.pack'):
            base = name[:-5]
            moved = f'/root/unpack-{base}.pack'
            shutil.move(f'{pack_dir}/{name}', moved)
            for extension in ('.idx', '.rev', '.bitmap', '.mtimes'):
                if os.path.exists(f'{pack_dir}/{base}{extension}'):
                    os.remove(f'{pack_dir}/{base}{extension}')
            with open(moved, 'rb') as handle:
                subprocess.run(['git', 'unpack-objects', '-q'], cwd=SOURCE, env=PLAIN, stdin=handle, check=True)
            os.remove(moved)
    run(['sync', '--file-system', SOURCE])
    time.sleep(4)
    print(json.dumps({'phase': 'unpack'}))
    print(run(['git', 'count-objects', '-v'], cwd=SOURCE).stdout.decode().strip(), flush=True)


ROUNDS_SERVED = 12
ROUNDS_REFUSED = 12
ROUNDS_FAILFAST = 40
ROUNDS_CONE = 20
ROUNDS_DECLINED = 12
ROUNDS_DIRTY = 12


def one_layout(layout):
    """The shapes this round must show, in an order that leaves the source as the next shape expects it."""
    adds(f'served-{layout}', ROUNDS_SERVED)
    dirty()
    adds(f'dirty-{layout}', ROUNDS_DIRTY)
    undirty()
    decline()
    adds(f'declined-{layout}', ROUNDS_DECLINED)
    undecline()


def all_shapes():
    """Every shape, on loose objects and then on packed ones."""
    setup()
    one_layout('loose')
    gc()
    one_layout('packed')


def positive_control_first():
    """The pass's positive control, before any counted shape: main's declined step against base's.

    The recheck measured +4 to +6 ms there; a null on fix minus main afterwards is only read as "cost gone"
    when this reproduces.
    """
    decline()
    adds('declined-positive-control-loose', 6)
    undecline()


if __name__ == '__main__':
    command = sys.argv[1]
    if command == 'all':
        all_shapes()
    elif command == 'adds':
        adds(sys.argv[2], int(sys.argv[3]), len(sys.argv) > 4 and sys.argv[4] == 'failfast')
    elif command == 'layout':
        one_layout(sys.argv[2])
    else:
        globals()[command]()
