// Copied unchanged on 2026-11-10T23:52:41-05:00 from ~/temp/agent/pi-runs/compare-deletes-on-three-filesystems.ts,
// so the record that cites it points at a path the repository keeps.
// Times creating and deleting one small tree on fresh btrfs, XFS and ext4 images and on the real volume, all on the same drive.
// Each image is a no-copy-on-write file behind a direct-I/O loop device, so its writes reach the same dm-crypt and SATA drive.
// Arms alternate so that the host's own load falls on all of them, and the rounds give the spread.
// Runs as root in a private mount namespace. Argument: rounds.
import { execFileSync } from 'node:child_process';
import { chownSync, closeSync, mkdirSync, openSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const ROUNDS = Number(process.argv[2] ?? '3');
const SCRATCH = `/var/home/user/temp/agent/fs-compare-${String(process.pid)}`;
const IMAGE_BYTES = '8G';
const DIRECTORIES = 300;
const FILES_PER_DIRECTORY = 100;
const FILE_BYTES = 1024;
const USER_ID = 1000;

const run = (command: string, ...arguments_: readonly string[]): string =>
  execFileSync(command, arguments_, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 300_000 }).trim();

interface Arm {
  readonly name: string;
  readonly directory: string;
  readonly cleanup: () => void;
}

const makeImageArm = (name: string, format: readonly string[], mountOptions: string): Arm => {
  const image = join(SCRATCH, `${name}.img`);
  closeSync(openSync(image, 'w'));
  // The attribute has to be set while the file is still empty, or btrfs copies on write anyway.
  run('chattr', '+C', image);
  run('fallocate', '--length', IMAGE_BYTES, image);
  const device = run('losetup', '--find', '--show', '--direct-io=on', image);
  run(format[0] ?? '', ...format.slice(1), device);
  const mountPoint = join(SCRATCH, `${name}-mount`);
  mkdirSync(mountPoint);
  run('mount', '--options', mountOptions, device, mountPoint);
  return {
    name,
    directory: mountPoint,
    cleanup: () => {
      run('umount', mountPoint);
      run('losetup', '--detach', device);
    },
  };
};

const milliseconds = (work: () => void): number => {
  const started = performance.now();
  work();
  return performance.now() - started;
};

const syncOf = (directory: string): void => {
  run('sync', '--file-system', directory);
};

interface Sample {
  readonly create: number;
  readonly delete_: number;
}

const measure = (directory: string): Sample => {
  const tree = join(directory, 'tree');
  const contents = 'x'.repeat(FILE_BYTES);
  const create = milliseconds(() => {
    mkdirSync(tree);
    for (let folder = 0; folder < DIRECTORIES; folder += 1) {
      const path = join(tree, `d${String(folder)}`);
      mkdirSync(path);
      for (let file = 0; file < FILES_PER_DIRECTORY; file += 1) {
        writeFileSync(join(path, `f${String(file)}`), contents);
      }
    }
    syncOf(directory);
  });
  // A delete is only finished once the filesystem has made it durable, so the sync counts as part of it.
  const delete_ = milliseconds(() => {
    rmSync(tree, { recursive: true });
    syncOf(directory);
  });
  return { create, delete_ };
};

mkdirSync(SCRATCH, { recursive: true });
chownSync(SCRATCH, USER_ID, USER_ID);
const realDirectory = join(SCRATCH, 'real-volume');
mkdirSync(realDirectory);

const arms: Arm[] = [];
try {
  if (process.argv[3] === 'metadata-profiles') {
    // Deletes are nearly all metadata, and DUP writes every metadata block twice.
    arms.push(makeImageArm('btrfs-metadata-dup', ['mkfs.btrfs', '--quiet', '--metadata', 'dup', '--data', 'single'], 'noatime,compress=zstd:3'));
    arms.push(makeImageArm('btrfs-metadata-single', ['mkfs.btrfs', '--quiet', '--metadata', 'single', '--data', 'single'], 'noatime,compress=zstd:3'));
  } else {
    arms.push(makeImageArm('btrfs', ['mkfs.btrfs', '--quiet', '--metadata', 'dup', '--data', 'single'], 'noatime,compress=zstd:3'));
    arms.push(makeImageArm('xfs', ['mkfs.xfs', '-q'], 'noatime'));
    arms.push(makeImageArm('ext4', ['mkfs.ext4', '-q'], 'noatime'));
    arms.push({ name: 'real-volume', directory: realDirectory, cleanup: () => undefined });
  }

  const results = new Map<string, Sample[]>();
  for (let round = 0; round < ROUNDS; round += 1) {
    // Each round starts one arm later, so no arm always runs first or last.
    const order = [...arms.slice(round % arms.length), ...arms.slice(0, round % arms.length)];
    for (const arm of order) {
      const sample = measure(arm.directory);
      results.set(arm.name, [...(results.get(arm.name) ?? []), sample]);
      process.stdout.write(`round ${String(round + 1)} ${arm.name}: create ${sample.create.toFixed(0)} ms, delete ${sample.delete_.toFixed(0)} ms\n`);
    }
  }
  const files = DIRECTORIES * FILES_PER_DIRECTORY;
  process.stdout.write(`== ${String(files)} files of ${String(FILE_BYTES)} bytes, ${String(ROUNDS)} rounds, median (min to max) ms\n`);
  for (const [name, samples] of results) {
    const summary = (values: readonly number[]): string => {
      const sorted = values.toSorted((left, right) => left - right);
      return `${(sorted[Math.floor(sorted.length / 2)] ?? 0).toFixed(0)} (${(sorted[0] ?? 0).toFixed(0)} to ${(sorted.at(-1) ?? 0).toFixed(0)})`;
    };
    process.stdout.write(`${name}: create ${summary(samples.map(({ create }) => create))}; delete ${summary(samples.map(({ delete_ }) => delete_))}\n`);
  }
} finally {
  for (const arm of arms) {
    arm.cleanup();
  }
  rmSync(SCRATCH, { recursive: true, force: true });
}
