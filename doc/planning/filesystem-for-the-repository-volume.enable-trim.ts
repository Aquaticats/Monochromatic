// Copied unchanged on 2026-11-10T23:52:41-05:00 from ~/temp/agent/pi-runs/enable-trim-on-the-860.ts,
// so the record that cites it points at a path the repository keeps.
// The owner's answer of 2026-11-10: "On, saved in the service (Recommended)".
// Saves TRIM in mount-860.service, applies it to the live volume, runs one full trim, and samples the drive before and after.
// Never prints the key file's path or contents. Stops before changing anything if the service does not read as expected.
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { setTimeout as sleep } from 'node:timers/promises';

const UNIT = '/etc/systemd/system/mount-860.service';
const BACKUP = '/var/home/user/temp/agent/pi-runs/mount-860.service.before-trim';
const UUID = '01c308e7-06fa-4737-8a7b-3bb5fcba871d';
const MOUNT = '/var/mnt/encrypted';
const SAMPLE_SECONDS = 120;
const REFRESH_ATTEMPTS = 3;
const OPEN_BEFORE = 'cryptsetup luksOpen --key-file';
const OPEN_AFTER = 'cryptsetup luksOpen --allow-discards --key-file';
const MOUNT_BEFORE = 'mount -o compress=zstd:3,noatime,user_subvol_rm_allowed /dev/mapper/crypt_sda';
const MOUNT_AFTER = 'mount -o compress=zstd:3,noatime,user_subvol_rm_allowed,discard=async /dev/mapper/crypt_sda';

const say = (text: string): void => {
  process.stdout.write(`${new Date().toISOString()} ${text}\n`);
};
const sudo = (...arguments_: readonly string[]): string =>
  execFileSync('sudo', ['-n', ...arguments_], { encoding: 'utf8', timeout: 600_000, maxBuffer: 16 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });

interface DiskCounters {
  readonly reads: number;
  readonly readMs: number;
  readonly writes: number;
  readonly writeMs: number;
  readonly busyMs: number;
  readonly weightedMs: number;
  readonly sectorsRead: number;
  readonly sectorsWritten: number;
  readonly discards: number;
  readonly flushes: number;
  readonly flushMs: number;
}
const counters = (device: string): DiskCounters => {
  const line = readFileSync('/proc/diskstats', 'utf8').split('\n').find((candidate) => candidate.trim().split(/\s+/u)[2] === device);
  if (line === undefined) {
    throw new Error(`no ${device} in /proc/diskstats`);
  }
  const f = line.trim().split(/\s+/u).map(Number);
  const at = (index: number): number => f[index] ?? 0;
  return { reads: at(3), sectorsRead: at(5), readMs: at(6), writes: at(7), sectorsWritten: at(9), writeMs: at(10), busyMs: at(12), weightedMs: at(13), discards: at(14), flushes: at(18), flushMs: at(19) };
};
const commitStats = (): { readonly commits: number; readonly totalMs: number } => {
  const text = readFileSync(`/sys/fs/btrfs/${UUID}/commit_stats`, 'utf8');
  const value = (name: string): number => Number(new RegExp(`${name} (\\d+)`, 'u').exec(text)?.[1] ?? '0');
  return { commits: value('commits'), totalMs: value('total_commit_ms') };
};

const sample = async (label: string): Promise<void> => {
  const started = performance.now();
  const [sda0, dm0, commit0] = [counters('sda'), counters('dm-2'), commitStats()];
  await sleep(SAMPLE_SECONDS * 1000);
  const elapsedMs = performance.now() - started;
  const [sda1, dm1, commit1] = [counters('sda'), counters('dm-2'), commitStats()];
  const reads = sda1.reads - sda0.reads;
  const writes = sda1.writes - sda0.writes;
  const dmWrites = dm1.writes - dm0.writes;
  const commits = commit1.commits - commit0.commits;
  const seconds = elapsedMs / 1000;
  const pressure = readFileSync('/proc/pressure/io', 'utf8').trim().replace(/\n/gu, ' | ');
  say(
    `SAMPLE ${label}: ${seconds.toFixed(0)} s; sda ${(reads / seconds).toFixed(0)} r/s ${(writes / seconds).toFixed(0)} w/s, ${(((sda1.sectorsRead - sda0.sectorsRead) * 512) / seconds / 1e6).toFixed(1)} MB/s read ${(((sda1.sectorsWritten - sda0.sectorsWritten) * 512) / seconds / 1e6).toFixed(1)} MB/s written, read await ${reads === 0 ? 'n/a' : ((sda1.readMs - sda0.readMs) / reads).toFixed(2)} ms, write await ${writes === 0 ? 'n/a' : ((sda1.writeMs - sda0.writeMs) / writes).toFixed(2)} ms, busy ${(((sda1.busyMs - sda0.busyMs) / elapsedMs) * 100).toFixed(0)}%, mean queue ${((sda1.weightedMs - sda0.weightedMs) / elapsedMs).toFixed(1)}, discards ${String(sda1.discards - sda0.discards)}; dm-2 write await ${dmWrites === 0 ? 'n/a' : ((dm1.writeMs - dm0.writeMs) / dmWrites).toFixed(0)} ms; btrfs ${String(commits)} commits, mean ${commits === 0 ? 'n/a' : ((commit1.totalMs - commit0.totalMs) / commits).toFixed(0)} ms, committing ${(((commit1.totalMs - commit0.totalMs) / elapsedMs) * 100).toFixed(0)}% of the time; ${pressure}`,
  );
};

await sample('before');

const original = sudo('cat', UNIT);
const occurrences = (text: string, part: string): number => text.split(part).length - 1;
if (occurrences(original, OPEN_BEFORE) !== 1 || occurrences(original, MOUNT_BEFORE) !== 1) {
  throw new Error('mount-860.service does not hold exactly one of each expected command, so nothing was changed');
}
const keyFile = /cryptsetup luksOpen --key-file\s+(\S+)/u.exec(original)?.[1];
if (keyFile === undefined) {
  throw new Error('no key file in the unlock command, so nothing was changed');
}
writeFileSync(BACKUP, original, { mode: 0o600 });
const updated = original.replace(OPEN_BEFORE, OPEN_AFTER).replace(MOUNT_BEFORE, MOUNT_AFTER);
const written = spawnSync('sudo', ['-n', 'tee', UNIT], { input: updated, encoding: 'utf8', stdio: ['pipe', 'ignore', 'pipe'] });
if (written.status !== 0) {
  throw new Error(`could not write the service: ${written.stderr.slice(0, 200)}`);
}
sudo('systemctl', 'daemon-reload');
say(`service saved; backup ${BACKUP}; unlock has --allow-discards: ${String(sudo('cat', UNIT).includes(OPEN_AFTER))}; mount has discard=async: ${String(sudo('cat', UNIT).includes(MOUNT_AFTER))}`);

let refreshed = false;
for (let attempt = 1; attempt <= REFRESH_ATTEMPTS && !refreshed; attempt += 1) {
  const started = performance.now();
  const result = spawnSync('sudo', ['-n', 'cryptsetup', 'refresh', 'crypt_sda', '--allow-discards', '--key-file', keyFile], { encoding: 'utf8', timeout: 300_000 });
  say(`refresh attempt ${String(attempt)}: status ${String(result.status)} after ${(performance.now() - started).toFixed(0)} ms${result.status === 0 ? '' : ` (${result.stderr.replaceAll(keyFile, '<key file>').trim().slice(0, 200)})`}`);
  refreshed = result.status === 0;
  if (!refreshed) {
    await sleep(10_000);
  }
}
if (!refreshed) {
  throw new Error('the live refresh failed each time; the service is saved, so TRIM starts at the next unlock');
}
const flags = sudo('cryptsetup', 'status', 'crypt_sda').split('\n').find((line) => line.includes('flags')) ?? '(no flags line)';
say(`after refresh: ${flags.trim()}; dm-2 discard_max_bytes ${readFileSync('/sys/block/dm-2/queue/discard_max_bytes', 'utf8').trim()}`);

const remountStarted = performance.now();
sudo('mount', '-o', 'remount,discard=async', MOUNT);
say(`remounted in ${(performance.now() - remountStarted).toFixed(0)} ms; options: ${execFileSync('findmnt', ['--noheadings', '--output=OPTIONS', MOUNT], { encoding: 'utf8' }).trim()}`);

await sample('after enabling, before the full trim');

const trimStarted = performance.now();
const trim = spawnSync(
  'sudo',
  ['-n', 'systemd-run', '--unit=fstrim-860-first', '--collect', '--wait', '--pipe', '--property=IOWeight=10', '--property=CPUQuota=100%', '--property=MemoryMax=512M', '--property=TasksMax=32', 'ionice', '--class', 'idle', 'fstrim', '--verbose', MOUNT],
  { encoding: 'utf8', timeout: 5_400_000 },
);
say(`full trim: status ${String(trim.status)} after ${((performance.now() - trimStarted) / 1000).toFixed(0)} s: ${`${trim.stdout}${trim.stderr}`.replace(/\s+/gu, ' ').trim().slice(0, 300)}`);

await sample('after the full trim');
say(`discardable bytes now ${readFileSync(`/sys/fs/btrfs/${UUID}/discard/discardable_bytes`, 'utf8').trim()}`);
