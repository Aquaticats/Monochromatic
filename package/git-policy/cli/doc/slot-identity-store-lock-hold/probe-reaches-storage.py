# Review probe: does an fsync'd append reach storage, shown against controls whose window is as long as the fsync arm's?
#
# The round's probe counted a sample "moved a block device" when any device's written-sector counter rose between two
# readings, and its no-fsync control ran windows about a hundred times shorter than the fsync arm's, so a busy host could
# explain the difference. This probe adds controls that do the same append without fsync and then wait exactly as long as
# the fsync sample before it took (and a bare wait of that length), and it reads the kernel's ZIL counters of the `home`
# dataset (the dataset the fixture folder sits on) around every sample.
#
# Run once: `python3 -I probe-reaches-storage.py <fixture folder> <rounds> [--pool-device <partition>] [--pool-disk <disk>]`
# inside a capped systemd-run scope.
#
# The dataset the fixture folder is on is read from `findmnt` at run time, so the probe asks the same question on any
# host with a ZFS pool. The pool's partition and whole disk are options: a dataset's mount names the dataset and never
# its vdev, and reading the vdev from ZFS means opening `/dev/zfs`, which this repository's rules for the host bar.
# (An earlier attempt in Node failed: this host's Node refuses `fsync` under its permission model.)
import os
import shutil
import statistics
import subprocess
import sys
import time

fixture_folder = sys.argv[1]
rounds = int(sys.argv[2])
options = sys.argv[3:]


def option(name, fallback):
    if name in options:
        return options[options.index(name) + 1]
    return fallback


pool_device = option('--pool-device', 'nvme1n1p2')
pool_disk = option('--pool-disk', 'nvme1n1')  # the whole disk: the kernel counts flush requests on whole disks only (block/stat.rst)


def dataset_of(folder):
    source = subprocess.run(
        ['findmnt', '--target', folder, '--output', 'SOURCE', '--noheadings', '--first-only'],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    dataset = source.split('[', 1)[0]
    if '/' not in dataset:
        raise SystemExit('the fixture folder is not on a dataset whose pool can be named: ' + source)
    return dataset


home_dataset_name = dataset_of(fixture_folder)
pool_kstat_folder = os.path.join('/proc/spl/kstat/zfs', home_dataset_name.split('/', 1)[0])


def sectors_written():
    counters = {}
    with open('/proc/diskstats') as stream:
        for line in stream:
            fields = line.split()
            if len(fields) >= 10:
                counters[fields[2]] = int(fields[9])
    return counters


def flushes_completed():
    counters = {}
    with open('/proc/diskstats') as stream:
        for line in stream:
            fields = line.split()
            if len(fields) >= 19:
                counters[fields[2]] = int(fields[18])
    return counters


def find_home_objset_file():
    for name in sorted(os.listdir(pool_kstat_folder)):
        if not name.startswith('objset-'):
            continue
        path = os.path.join(pool_kstat_folder, name)
        with open(path) as stream:
            for line in stream:
                if line.startswith('dataset_name') and line.split()[-1] == home_dataset_name:
                    return path
    raise SystemExit('no objset kstat names ' + home_dataset_name)


home_objset_file = find_home_objset_file()
print('PROBE counters read from', home_objset_file, '(dataset', home_dataset_name + ')')


def zil_counters():
    counters = {}
    with open(home_objset_file) as stream:
        for line in stream.read().splitlines()[2:]:
            fields = line.split()
            if len(fields) == 3:
                counters[fields[0]] = int(fields[2]) if fields[2].isdigit() else 0
    return counters


def measured(work):
    disk_before = sectors_written()
    flush_before = flushes_completed()
    zil_before = zil_counters()
    began = time.perf_counter()
    work()
    milliseconds = (time.perf_counter() - began) * 1000.0
    disk_after = sectors_written()
    flush_after = flushes_completed()
    zil_after = zil_counters()
    any_moved = any(disk_after[name] > disk_before.get(name, disk_after[name]) for name in disk_after)
    return {
        'ms': milliseconds,
        'any_moved': any_moved,
        'pool_sectors': disk_after.get(pool_device, 0) - disk_before.get(pool_device, 0),
        'disk_flushes': flush_after.get(pool_disk, 0) - flush_before.get(pool_disk, 0),
        'zil_commits': zil_after['zil_commit_count'] - zil_before['zil_commit_count'],
        'zil_written': zil_after['zil_itx_metaslab_normal_write'] - zil_before['zil_itx_metaslab_normal_write'],
    }


line = ('slot measured ' + 'x' * 237 + '\n').encode()
shutil.rmtree(fixture_folder, ignore_errors=True)
os.makedirs(fixture_folder)
with_fsync = os.path.join(fixture_folder, 'append-with-two-fsyncs')
without_fsync = os.path.join(fixture_folder, 'append-without-fsync')
waited = os.path.join(fixture_folder, 'append-without-fsync-then-wait')


def append_line(path, sync):
    descriptor = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o600)
    os.write(descriptor, line)
    if sync:
        os.fsync(descriptor)
    os.close(descriptor)
    if sync:
        folder = os.open(fixture_folder, os.O_RDONLY | os.O_DIRECTORY)
        os.fsync(folder)
        os.close(folder)


synced = []
bare = []
bare_then_wait = []
idle = []
for _ in range(rounds):
    sample = measured(lambda: append_line(with_fsync, True))
    synced.append(sample)
    bare.append(measured(lambda: append_line(without_fsync, False)))

    def bare_append_then_wait():
        began = time.perf_counter()
        append_line(waited, False)
        remaining = sample['ms'] / 1000.0 - (time.perf_counter() - began)
        if remaining > 0:
            time.sleep(remaining)

    bare_then_wait.append(measured(bare_append_then_wait))
    idle.append(measured(lambda: time.sleep(sample['ms'] / 1000.0)))


def report(label, samples):
    milliseconds = [sample['ms'] for sample in samples]
    print(
        f'PROBE {label}: n={len(samples)} ms min={min(milliseconds):.3f} median={statistics.median(milliseconds):.3f}'
        f' max={max(milliseconds):.3f}'
        f' | samples where any device moved={sum(1 for s in samples if s["any_moved"])}'
        f' | samples where {pool_device} moved={sum(1 for s in samples if s["pool_sectors"] > 0)}'
        f' | median {pool_device} sectors={statistics.median(s["pool_sectors"] for s in samples)}'
        f' | samples where {pool_disk} completed a flush request={sum(1 for s in samples if s["disk_flushes"] > 0)}'
        f' | samples with a ZIL commit of the home dataset={sum(1 for s in samples if s["zil_commits"] > 0)}'
        f' | median ZIL commits={statistics.median(s["zil_commits"] for s in samples)}'
        f' | samples whose ZIL wrote bytes to the pool={sum(1 for s in samples if s["zil_written"] > 0)}'
    )


report('append with fsync of the file and the folder', synced)
report("same append without fsync, window as short as the round's control", bare)
report('same append without fsync, then a wait as long as the fsync sample', bare_then_wait)
report('nothing but a wait as long as the fsync sample', idle)
shutil.rmtree(fixture_folder, ignore_errors=True)
