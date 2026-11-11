# cli-git subvolume migration script (proven on a throwaway, 2026-11-01)

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Purpose:
a durable copy of the one-off script that converts a Git main checkout into a btrfs subvolume in place.
It was proven on throwaway copies (the procedure and its measurements are summarized in
`doc/planning/cli-git-btrfs-worktree.md`,
"Migration of the main checkout");
the original lives outside Git under
`~/temp/agent/subvol-migrate/` and would not survive a reboot of this machine.
Where the tool lives for good (a wrapper subcommand or printed commands) is an open owner decision;
this file is the record,
not a shipped tool.

To run it,
copy the code block into a `.ts` file and run it with `node` as the README part says.

## README written with the script

Script:
`migrate.ts` in this directory (node,
`node:child_process`,
no dependencies).
Proven on throwaway copies of this repository's checkout under `/var/home/user/subvol-migration-probe-20261101`
(same filesystem as the real checkout;
the other disk under `~/temp` cannot reflink).
Nothing in the real checkout,
its `.git`,
its worktrees or the commit service was touched.

### What it does

- Dry run by default:
  preflight and plan only.
- `--apply` runs these journaled steps (journal:
  `<parent>/.migrate-<name>.journal.json`):
  1.  `btrfs subvolume create <root>.subvol-new`.
  2.  Each linked worktree under `<root>/` (from `.git/worktrees/*/gitdir`) is created as a nested subvolume inside the
      new tree,
      so a later snapshot of the root holds an empty stub for it (disable with `--no-nested-worktrees`).
  3.  `cp --archive --reflink=always --remove-destination <root>/. <root>.subvol-new/`:
      data blocks are shared,
      only metadata is copied.
  4.  `rsync --archive --delete` of whatever changed during the copy.
  5.  Hold `<root>/.git/index.lock`,
      run `rsync` once more,
      `mv --exchange --no-target-directory <root> <root>.subvol-new`
      (renameat2 `RENAME_EXCHANGE`,
      one atomic step),
      rename the old tree to `<root>.before-subvolume`,
      drop the lock
      that is now inside the old tree.
  6.  One `rsync --archive --update` (no `--delete`) from the old tree to the new root for writes that landed in the old
      tree after the exchange.
  7.  Rewrite the stat fields of every copied Git index (main and the linked worktrees inside the root).
  8.  Verify:
      root inode 256,
      `btrfs filesystem du`,
      `git status`,
      `git worktree list`.
- The old tree is never deleted unless `--remove-old` is given.

### Refusals (preflight)

- Not on btrfs;
  root is already a subvolume (inode 256);
  `.git` is not a directory;
  less than 5 GiB free.
- Any of `index.lock`,
  `MERGE_HEAD`,
  `CHERRY_PICK_HEAD`,
  `REVERT_HEAD`,
  `BISECT_LOG`,
  `rebase-merge`,
  `rebase-apply`,
  `sequencer`,
  `HEAD.lock`,
  `config.lock`,
  `packed-refs.lock`,
  `shallow.lock` in `.git` or in any `.git/worktrees/*`.
- `.git/cli-git/work/pending` or `.git/cli-git/work/claimed` non-empty.
- It warns (does not refuse) about processes whose working directory is inside the checkout,
  with their PIDs.

### Exact real command line

Dry run first (read-only,
reads only `.git` and `.git/worktrees/*/gitdir`,
never walks the tree):

```sh
# file: dry run
node /var/home/user/temp/agent/subvol-migrate/migrate.ts --root /var/home/user/Monochromatic
```

Then the migration,
in a bounded scope (the measured bounds):

```sh
# file: migration
systemd-run --user --scope --property=IOWeight=10 --property=CPUQuota=200% --property=MemoryMax=2G \
  --property=TasksMax=64 node /var/home/user/temp/agent/subvol-migrate/migrate.ts \
  --root /var/home/user/Monochromatic --apply
```

### Results on the throwaway (all with the bounds above, host load 11 to 27)

- Layout:
  20546 tracked files,
  `.git` of 700 MiB,
  one to two linked worktrees,
  a 5000-file ignored `node_modules`.
- 45472 entries:
  copy 2.14 s,
  each `rsync` pass 1.5 s,
  exchange under 0.01 s,
  root inode afterwards 256.
- 68149 entries:
  copy 8.46 s,
  `rsync` passes 2.52 s,
  2.37 s and 2.15 s,
  exchange 0.00 s,
  index rewrite 0.62 s for
  three indexes of 20546 entries each.
- `btrfs filesystem du --summarize` of the new root:
  total 3.85 GiB,
  exclusive 0.00 B,
  set shared 2.89 GiB.
- Without the index rewrite the first `git status` in the new root took 22.1 s (every copied entry has a new inode and
  change time,
  so Git hashes every file and runs the LFS clean filter);
  with it,
  0.17 s,
  and 0.07 s in a linked worktree.
- Linked worktrees:
  inode 256 (nested subvolumes),
  `.git` file and `.git/worktrees/<name>/gitdir` unchanged because the
  path is unchanged.
  `git worktree list`,
  `git commit` from the main checkout and from a worktree,
  `git worktree add`,
  `git worktree remove` (deleted the nested subvolume),
  `git worktree prune`,
  `git gc --auto`:
  all worked (plain Git;
  a wrapped commit was not run because it would go through the production commit service).
- A snapshot of the migrated root (`btrfs subvolume snapshot`) took 1.08 s and showed each nested worktree as an empty
  directory (inode 2);
  deleting the snapshot took 2 ms (`user_subvol_rm_allowed` is active on this host).
- The snapshot of a root still contains `.git` and every ignored tree in full (shared blocks,
  but real directory
  entries),
  so the root is not a good snapshot source for a new worktree unless `.git` and the ignored trees are
  nested subvolumes too.
  See the planning document.

### Effects on running processes

- A process whose working directory is inside the old tree keeps that directory:
  after the exchange its relative writes
  land in `<root>.before-subvolume`,
  not in the new root.
  Demonstrated:
  a loop writing `tick.txt` once a second from a
  shell started in the checkout wrote 60 in the old tree while the new root stopped at 26 (the copy made by the last
  `rsync --update` pass);
  the file written after the pass exists only in the old tree.
- Processes that open paths by absolute name (`/var/home/user/Monochromatic/...`) resolve to the new root from the
  exchange on.
  Open descriptors stay on the old inodes.
- Restart every shell,
  editor and agent session whose working directory is inside the checkout.
- `cli-git.service` (read only check on this host):
  working directory `/var/home/user`,
  no open descriptor under the
  checkout;
  it resolves repository paths per job,
  so it needs no restart if `pending` and `claimed` are empty.
  While the
  script holds `.git/index.lock` (the length of one `rsync` pass) index writers wait or fail on the lock.
- Rerun the catch-up by hand while old sessions linger:
  `rsync --archive --update --exclude=/.git/index.lock <root>.before-subvolume/ <root>/`.

### Crash and rollback

- Kill at any point,
  then `--resume`:
  proven by simulated SIGKILL after the copy,
  with the lock held before the
  exchange,
  and between the exchange and the rename (environment `MIGRATE_KILL_AT=after-copy|before-exchange|after-exchange-mv`).
  Every resume ended with root inode 256 and a clean `git status`.
- `--rollback`:
  carries newer files back with `rsync --update`,
  exchanges the trees again,
  deletes the new subvolume.
  Proven:
  a file written to the new root after the migration was present in the restored tree.
- Before the exchange,
  nothing in the original tree changed except the temporary `.git/index.lock`.

### Extrapolation to the real checkout

- Rates measured:
  copy 47 to 124 microseconds per entry (47 unloaded-ish,
  124 at load 20),
  `rsync` 33 to 37 microseconds
  per entry per pass,
  three passes,
  index rewrite about 10 microseconds per entry.
- One real worktree,
  `.claude/worktrees/integ` (read only source):
  `cp --archive --reflink=always` took 1.94 s wall for
  22921 entries (20570 files,
  831 MiB:
  exclusive 0.00 B,
  shared 826 MiB) at IOWeight 10,
  200% CPU,
  load 11.7.
- The real checkout was not walked,
  so its entry count is unknown.
  It holds at least 86 trees of about 22.9 thousand
  entries,
  so at least 2 million entries before `node_modules`;
  at 160 to 250 microseconds per entry that is 5 to 8 minutes
  for 2 million and about 10 to 17 minutes for 4 million.
  The final `rsync` pass,
  the one that runs under the
  `index.lock`,
  is 33 to 37 microseconds per entry:
  about 1 to 2.5 minutes for 2 to 4 million entries.
- Btrfs metadata grows by the copied entries (about 1 KiB per entry is an assumption,
  not measured here);
  358 GiB were
  free during the proof.

## Change of 2026-11-01 for the real run

The first real apply stopped in the copy step on 127 root-owned mode-600 files the user may not read.
`runCopy` now tolerates only unreadable files (`cp` "cannot open ... for reading:
Permission denied" and rsync
"send_files failed to open ...:
Permission denied (13)"),
records each once relative to its tree in
`<parent>/.migrate-<name>.unreadable.txt` and leaves it in the old tree;
every other error still stops the run.
The `--delete` passes carry `--ignore-errors` so deletions still happen,
and an unreadable directory is not tolerated,
because with deletions forced it would wipe that subtree in the new tree.
Proven on a throwaway (`retest.ts` beside the script:
mode-000 files in the main tree and a nested worktree,
11 checks),
then used for the real checkout;
see the handover section "2026-11-01:
the checkout is a btrfs subvolume".
## The script

```ts
// file: migrate.ts
// Converts an ordinary btrfs directory (a Git main checkout) into a subvolume at the same path.
// Dry-run by default. Run `node migrate.ts --help` for the flags. See README.md beside this file.
//
// What it does, in order (each step is journaled; a rerun resumes, `--rollback` undoes):
//   1. preflight: refuse unless every safety condition holds (read-only).
//   2. create:    `btrfs subvolume create <root>.subvol-new`.
//   3. copy:      `cp --archive --reflink=always <root>/. <root>.subvol-new/` (data shared, metadata copied).
//   4. delta:     `rsync --archive --delete` of whatever changed during the copy.
//   5. exchange:  hold `<root>/.git/index.lock`, last delta, `mv --exchange <root> <root>.subvol-new`,
//                 rename the old tree to `<root>.before-subvolume`, drop the lock inside the old tree.
//   6. verify:    inode 256, `btrfs filesystem du`, `git status`, `git worktree list`.
// The old tree is never deleted unless `--remove-old` is given.
import { spawnSync } from 'node:child_process';
import {
  closeSync, existsSync, openSync, readdirSync, readFileSync, readlinkSync, renameSync, statSync, statfsSync, unlinkSync,
  writeFileSync,
} from 'node:fs';
import { basename, dirname, join, relative, resolve } from 'node:path';
import { closeSync as closeFd, lstatSync, mkdirSync, openSync as openFd, writeSync } from 'node:fs';
import { createHash } from 'node:crypto';

const MIB = 1024 * 1024;
const MIN_FREE_BYTES = 5 * 1024 * MIB;
const BTRFS_SUBVOLUME_ROOT_INODE = 256;
const BTRFS_SUPER_MAGIC = 0x9123683e;
const IN_PROGRESS_NAMES = [
  'index.lock', 'MERGE_HEAD', 'CHERRY_PICK_HEAD', 'REVERT_HEAD', 'BISECT_LOG', 'rebase-merge', 'rebase-apply', 'sequencer',
  'HEAD.lock', 'config.lock', 'packed-refs.lock', 'shallow.lock',
];

const argv = process.argv.slice(2);
const flag = (name: string): boolean => argv.includes(name);
const value = (name: string): string | undefined => {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : undefined;
};

if (flag('--help') || value('--root') === undefined) {
  console.log(`usage: node migrate.ts --root <checkout> [--apply] [--resume] [--rollback] [--remove-old] [--skip-process-scan]
  --root <checkout>   the main checkout directory to convert (absolute path)
  (no other flag)     dry run: preflight and the plan; nothing is created, renamed or deleted
  --apply             run the migration (steps 2 to 6)
  --resume            continue an interrupted --apply from its journal (implies --apply)
  --rollback          undo an interrupted or finished migration (exchanges back, deletes the new subvolume)
  --remove-old        after a verified migration, delete <checkout>.before-subvolume (slow: unlinks every file)
  --no-refresh-indexes leave the Git indexes as copied (the first git status of each worktree then re-hashes every file)
  --no-nested-worktrees keep linked worktrees under the checkout as ordinary directories (default: nested subvolumes)
  --skip-process-scan do not scan /proc for processes whose working directory is inside the checkout`);
  process.exit(value('--root') === undefined && !flag('--help') ? 2 : 0);
}

const root = resolve(value('--root')!);
const parent = dirname(root);
const newPath = `${root}.subvol-new`;
const oldPath = `${root}.before-subvolume`;
const journalPath = join(parent, `.migrate-${basename(root)}.journal.json`);
const apply = flag('--apply') || flag('--resume');

type Journal = { steps: Record<string, string>; started: string };
const journal: Journal = existsSync(journalPath)
  ? JSON.parse(readFileSync(journalPath, 'utf8'))
  : { steps: {}, started: new Date().toISOString() };

// Test hook: MIGRATE_KILL_AT=<point> makes the script SIGKILL itself there, to prove that a rerun recovers.
function maybeDie(point: string): void {
  if (process.env.MIGRATE_KILL_AT === point) { say(`simulated crash at ${point}`); process.kill(process.pid, 'SIGKILL'); }
}
function say(message: string): void { console.log(`[migrate] ${message}`); }
function mark(step: string, state: string): void {
  journal.steps[step] = `${state} ${new Date().toISOString()}`;
  if (apply || flag('--rollback')) { writeFileSync(journalPath, JSON.stringify(journal, null, 2)); }
}
const done = (step: string): boolean => (journal.steps[step] ?? '').startsWith('done');

function run(command: string, args: string[], allowFailure = false): number {
  say(`$ ${[command, ...args].join(' ')}`);
  if (!apply && !flag('--rollback')) { return 0; }
  const started = process.hrtime.bigint();
  const result = spawnSync(command, args, { stdio: 'inherit' });
  say(`  took ${(Number(process.hrtime.bigint() - started) / 1e9).toFixed(2)} s: ${command} ${args[0] ?? ''}`);
  if (result.status !== 0 && !allowFailure) {
    throw new Error(`${command} exited with ${result.status}; journal: ${journalPath}`);
  }
  return result.status ?? -1;
}

// Files this user may not read (for example root-owned mode 600 logs a privileged tool wrote into the checkout) can
// be neither copied nor reflinked without privilege. They are left in the old tree, listed in `unreadablePath`, and
// every other error still stops the run. Proven on a throwaway with a mode-000 file before the real retry.
const unreadablePath = join(parent, `.migrate-${basename(root)}.unreadable.txt`);
const unreadable = new Set<string>(existsSync(unreadablePath)
  ? readFileSync(unreadablePath, 'utf8').split('\n').filter((line) => line.length > 0) : []);
const UNREADABLE_LINES = [
  /^cp: cannot open '(.+)' for reading: Permission denied$/,
  /^rsync: \[sender\] send_files failed to open "(.+)": Permission denied \(13\)$/,
  // Not tolerated on purpose: an unreadable directory ("opendir ... failed"). It lists no children, and with
  // --ignore-errors forcing deletions on, rsync would wipe that subtree from the new tree. It fails the run instead.
];
const SUMMARY_LINES = [
  /^rsync error: some files\/attrs were not transferred \(see previous errors\) \(code 23\)/,
  /^file has vanished: "(.+)"$/,
  /^rsync warning: some files vanished before they could be transferred \(code 24\)/,
];
function runCopy(command: string, args: string[]): void {
  say(`$ ${[command, ...args].join(' ')}`);
  const started = process.hrtime.bigint();
  const result = spawnSync(command, args, { stdio: ['inherit', 'inherit', 'pipe'], maxBuffer: 256 * MIB });
  const stderr = result.stderr?.toString('utf8') ?? '';
  process.stderr.write(stderr);
  say(`  took ${(Number(process.hrtime.bigint() - started) / 1e9).toFixed(2)} s: ${command} ${args[0] ?? ''}`);
  if (result.status === 0) { return; }
  const unexplained: string[] = [];
  for (const line of stderr.split('\n').filter((entry) => entry.trim().length > 0)) {
    const match = UNREADABLE_LINES.map((pattern) => pattern.exec(line)).find((found) => found !== null);
    if (match) {
      // One file is reported as <root>/./x by cp, <root>/x by rsync and <old tree>/x by the catch-up: record x.
      const absolute = resolve(match[1]);
      const tree = [oldPath, newPath, root].find((prefix) => absolute.startsWith(`${prefix}/`));
      unreadable.add(tree === undefined ? absolute : relative(tree, absolute));
      continue;
    }
    if (SUMMARY_LINES.some((pattern) => pattern.test(line))) { continue; }
    unexplained.push(line);
  }
  const tolerated = command === 'cp' ? [1] : [23, 24];
  if (unexplained.length > 0 || !tolerated.includes(result.status ?? -1)) {
    throw new Error(`${command} exited with ${result.status}; unexplained: ${unexplained.slice(0, 5).join(' | ')}; journal: ${journalPath}`);
  }
  writeFileSync(unreadablePath, `${[...unreadable].sort().join('\n')}\n`);
  say(`  tolerated: ${unreadable.size} file(s) this user may not read stay in the old tree; list: ${unreadablePath}`);
}

function inode(path: string): number { return Number(statSync(path, { bigint: true }).ino); }
function isSubvolume(path: string): boolean { return inode(path) === BTRFS_SUBVOLUME_ROOT_INODE; }

function preflight(): string[] {
  const refusals: string[] = [];
  if (!existsSync(root)) { return [`${root} does not exist`]; }
  if (Number(statfsSync(root).type) !== BTRFS_SUPER_MAGIC) { refusals.push(`${root} is not on btrfs`); }
  if (isSubvolume(root) && !existsSync(newPath) && !done('exchange')) { refusals.push(`${root} is already a subvolume (inode 256)`); }
  if (!isSubvolume(root) && existsSync(newPath) && !done('create')) { refusals.push(`${newPath} exists and the journal did not create it`); }
  if (existsSync(oldPath) && !done('exchange') && !(journal.steps.exchange ?? '').startsWith('started')) { refusals.push(`${oldPath} exists already`); }
  if (statfsSync(parent).bavail * statfsSync(parent).bsize < MIN_FREE_BYTES) { refusals.push('less than 5 GiB free'); }
  const gitDir = join(root, '.git');
  if (!existsSync(gitDir) || !statSync(gitDir).isDirectory()) { refusals.push(`${gitDir} is not a directory`); } else {
    const directories = [gitDir];
    const admin = join(gitDir, 'worktrees');
    if (existsSync(admin)) { for (const name of readdirSync(admin)) { directories.push(join(admin, name)); } }
    for (const directory of directories) {
      for (const name of IN_PROGRESS_NAMES) {
        // A lock this script left behind in the main Git directory is its own, not another program's.
        const ownLock = directory === gitDir && name === 'index.lock' && (journal.steps.exchange ?? '').startsWith('started');
        if (existsSync(join(directory, name)) && !ownLock) { refusals.push(`Git operation in progress: ${join(directory, name)}`); }
      }
    }
    for (const queue of ['pending', 'claimed']) {
      const path = join(gitDir, 'cli-git', 'work', queue);
      if (existsSync(path) && readdirSync(path).length > 0) { refusals.push(`commit service queue not empty: ${path}`); }
    }
  }
  return refusals;
}

function processesInside(): string[] {
  const found: string[] = [];
  for (const name of readdirSync('/proc')) {
    if (!/^\d+$/.test(name) || Number(name) === process.pid) { continue; }
    try {
      const cwd = readlinkSync(`/proc/${name}/cwd`);
      if (cwd === root || cwd.startsWith(`${root}/`)) { found.push(`${name} (${cwd})`); }
    } catch { /* process ended or is not ours */ }
  }
  return found;
}

function linkedWorktreesInside(): string[] {
  const admin = join(root, '.git', 'worktrees');
  if (!existsSync(admin)) { return []; }
  const found: string[] = [];
  for (const name of readdirSync(admin)) {
    try {
      const recorded = readFileSync(join(admin, name, 'gitdir'), 'utf8').trim();
      const worktree = dirname(recorded);
      if (worktree.startsWith(`${root}/`) && existsSync(worktree)) { found.push(worktree); }
    } catch { /* an administrative directory without a gitdir file is pruned by Git, not migrated */ }
  }
  return found;
}


// Reflink copies get new inode numbers and change times, so every copied index entry looks stat-dirty and the first
// `git status` of each worktree would hash every file (and run the LFS clean filter on LFS files). This rewrites the
// stat fields of an index from the files now on disk, only where size and modification time still equal the entry's.
const UNTRUSTED_EXTENSIONS = new Set(['UNTR', 'FSMN']);
function refreshIndexStat(indexPath: string, tree: string): string {
  const bytes = readFileSync(indexPath);
  if (bytes.toString('latin1', 0, 4) !== 'DIRC' || bytes.readUInt32BE(4) !== 2) { return 'skipped (not index version 2)'; }
  const body = bytes.subarray(0, bytes.length - 20);
  if (!createHash('sha1').update(body).digest().equals(bytes.subarray(bytes.length - 20))) { return 'skipped (checksum is not SHA-1)'; }
  const count = bytes.readUInt32BE(8);
  const parts: Buffer[] = [Buffer.from(bytes.subarray(0, 12))];
  let offset = 12;
  let updated = 0;
  for (let index = 0; index < count; index++) {
    const flags = bytes.readUInt16BE(offset + 60);
    if ((flags & 0x4000) !== 0) { return 'skipped (extended flags)'; }
    const end = bytes.indexOf(0, offset + 62);
    const length = (62 + (end - offset - 62) + 8) & ~7;
    const entry = Buffer.from(bytes.subarray(offset, offset + length));
    const path = bytes.toString('utf8', offset + 62, end);
    if (((flags >> 12) & 3) === 0 && entry.readUInt32BE(36) !== 0) {
      try {
        const stat = lstatSync(join(tree, path), { bigint: true });
        if (Number(stat.size & 0xffffffffn) === entry.readUInt32BE(36) && Number(stat.mtimeNs / 1000000000n) === entry.readUInt32BE(8)
          && Number(stat.mtimeNs % 1000000000n) === entry.readUInt32BE(12)) {
          entry.writeUInt32BE(Number(stat.ctimeNs / 1000000000n), 0);
          entry.writeUInt32BE(Number(stat.ctimeNs % 1000000000n), 4);
          entry.writeUInt32BE(Number(stat.dev & 0xffffffffn), 16);
          entry.writeUInt32BE(Number(stat.ino & 0xffffffffn), 20);
          entry.writeUInt32BE(Number(stat.uid), 28);
          entry.writeUInt32BE(Number(stat.gid), 32);
          updated++;
        }
      } catch { /* a missing file stays as Git recorded it */ }
    }
    parts.push(entry);
    offset += length;
  }
  while (offset < body.length) {
    const size = bytes.readUInt32BE(offset + 4);
    if (!UNTRUSTED_EXTENSIONS.has(bytes.toString('latin1', offset, offset + 4))) { parts.push(Buffer.from(bytes.subarray(offset, offset + 8 + size))); }
    offset += 8 + size;
  }
  const next = Buffer.concat(parts);
  const lock = `${indexPath}.lock`;
  let descriptor: number;
  try { descriptor = openFd(lock, 'wx'); } catch { return 'skipped (index.lock exists: a Git command is running)'; }
  writeSync(descriptor, Buffer.concat([next, createHash('sha1').update(next).digest()]));
  closeFd(descriptor);
  renameSync(lock, indexPath);
  return `${updated} of ${count} entries refreshed`;
}

function refreshAllIndexes(): void {
  say(`index ${root}/.git/index: ${refreshIndexStat(join(root, '.git', 'index'), root)}`);
  const admin = join(root, '.git', 'worktrees');
  if (!existsSync(admin)) { return; }
  for (const name of readdirSync(admin)) {
    try {
      const worktree = dirname(readFileSync(join(admin, name, 'gitdir'), 'utf8').trim());
      if (existsSync(join(admin, name, 'index')) && worktree.startsWith(`${root}/`) && existsSync(worktree)) {
        say(`index ${join(admin, name, 'index')}: ${refreshIndexStat(join(admin, name, 'index'), worktree)}`);
      }
    } catch { /* pruned or unreadable administrative directory */ }
  }
}

function mountFlags(): string {
  const mounts = readFileSync('/proc/self/mounts', 'utf8').split('\n').map((entry) => entry.split(' '))
    .filter((fields) => fields[2] === 'btrfs' && (root === fields[1] || root.startsWith(`${fields[1]}/`)))
    .sort((left, right) => right[1].length - left[1].length);
  return mounts[0]?.join(' ') ?? '';
}

function exchangeBack(): void {
  run('mv', ['--exchange', '--no-target-directory', root, oldPath]);
}

if (flag('--rollback')) {
  say(`rollback; journal ${journalPath}`);
  if (done('exchange') && existsSync(oldPath)) {
    // The migrated root is at <root>; the original is at <root>.before-subvolume.
    // First carry back anything written to the new root since the exchange (no --delete, newer files win).
    run('rsync', ['--archive', '--update', '--hard-links', '--acls', '--xattrs', '--numeric-ids', `${root}/`, `${oldPath}/`]);
    exchangeBack();
    renameSync(oldPath, newPath);
    mark('exchange', 'rolled-back');
  }
  if (existsSync(newPath) && isSubvolume(newPath)) {
    run('btrfs', ['subvolume', 'delete', newPath], true);
    if (existsSync(newPath)) { say(`could not delete ${newPath} in one step; empty it and run rmdir (see README)`); }
  }
  mark('create', 'rolled-back');
  process.exit(0);
}

const refusals = preflight();
say(`root ${root}; parent ${parent}; mode ${apply ? 'APPLY' : 'dry run'}`);
say(`mount: ${mountFlags()}`);
say(`user_subvol_rm_allowed: ${mountFlags().includes('user_subvol_rm_allowed') ? 'yes' : 'no (subvolume deletion needs an emptied subvolume and rmdir)'}`);
if (refusals.length > 0) {
  for (const refusal of refusals) { say(`REFUSED: ${refusal}`); }
  process.exit(1);
}
if (!flag('--skip-process-scan')) {
  const inside = processesInside();
  say(inside.length === 0 ? 'no process has its working directory inside the checkout'
    : `WARNING: ${inside.length} process(es) have a working directory inside the checkout and will stay in the OLD tree:\n  ${inside.join('\n  ')}`);
}
say(`linked worktrees inside the checkout that become nested subvolumes: ${flag('--no-nested-worktrees') ? 0 : linkedWorktreesInside().length}`);
say(`plan: create ${newPath}; reflink-copy ${root}/. into it; rsync delta; exchange; old tree becomes ${oldPath}`);
if (!apply) { say('dry run complete; nothing was changed. Add --apply to run it.'); process.exit(0); }

const rsyncArgs = [
  '--archive', '--hard-links', '--acls', '--xattrs', '--numeric-ids', '--delete', '--ignore-errors',
  '--exclude=/.git/index.lock', '--info=stats2',
];

if (!isSubvolume(root) || !done('exchange')) {
  if (!done('create')) {
    if (!existsSync(newPath)) { run('btrfs', ['subvolume', 'create', newPath]); }
    mark('create', 'done');
  }
  if (!done('nested') && !flag('--no-nested-worktrees')) {
    // Linked worktrees inside the checkout become nested subvolumes, so a snapshot of the root holds an empty stub
    // for each of them instead of a full copy. The copy below fills them: cp merges into an existing directory.
    for (const worktree of linkedWorktreesInside()) {
      const target = join(newPath, relative(root, worktree));
      mkdirSync(dirname(target), { recursive: true });
      if (!existsSync(target)) { run('btrfs', ['subvolume', 'create', target]); }
    }
    mark('nested', 'done');
  }
  if (!done('copy')) {
    mark('copy', 'started');
    runCopy('cp', ['--archive', '--reflink=always', '--remove-destination', '--no-target-directory', `${root}/.`, `${newPath}/`]);
    maybeDie('after-copy');
    mark('copy', 'done');
  }
  if (!done('delta1')) {
    runCopy('rsync', [...rsyncArgs, `${root}/`, `${newPath}/`]);
    mark('delta1', 'done');
  }
  if ((journal.steps.exchange ?? '').startsWith('started') && isSubvolume(root)) {
    // An earlier run was killed between the exchange and the rename or the lock removal.
    if (existsSync(newPath)) { renameSync(newPath, oldPath); }
    mark('exchange', 'done');
  }
  if (!done('exchange')) {
    const lock = join(root, '.git', 'index.lock');
    say(`hold ${lock} so no index writer starts during the last delta`);
    if (!existsSync(lock)) { closeSync(openSync(lock, 'wx')); }
    runCopy('rsync', [...rsyncArgs, `${root}/`, `${newPath}/`]);
    mark('exchange', 'started');
    maybeDie('before-exchange');
    run('mv', ['--exchange', '--no-target-directory', root, newPath]);
    // <root> is now the subvolume; <root>.subvol-new is the old tree, which still holds the lock file.
    maybeDie('after-exchange-mv');
    renameSync(newPath, oldPath);
    say(`old tree is now ${oldPath}`);
    mark('exchange', 'done');
  }
}
if (existsSync(join(oldPath, '.git', 'index.lock'))) {
  unlinkSync(join(oldPath, '.git', 'index.lock'));
  say(`removed ${join(oldPath, '.git', 'index.lock')}`);
}

if (!done('catchup')) {
  // Writers that held descriptors in the old tree may have written after the exchange; copy those once more.
  // No --delete and --update: files the new root created or changed since the exchange must survive.
  runCopy('rsync', ['--archive', '--update', '--hard-links', '--acls', '--xattrs', '--numeric-ids', '--exclude=/.git/index.lock', `${oldPath}/`, `${root}/`]);
  mark('catchup', 'done');
}

if (!done('indexes') && !flag('--no-refresh-indexes')) {
  const started = process.hrtime.bigint();
  refreshAllIndexes();
  say(`  took ${(Number(process.hrtime.bigint() - started) / 1e9).toFixed(2)} s: index stat refresh`);
  mark('indexes', 'done');
}
say(`root inode: ${inode(root)} (256 means subvolume)`);
run('btrfs', ['filesystem', 'du', '--summarize', root], true);
run('git', ['-C', root, 'status', '--short'], true);
run('git', ['-C', root, 'worktree', 'list'], true);
mark('verify', 'done');

if (flag('--remove-old')) {
  run('rm', ['--recursive', '--force', '--one-file-system', '--', oldPath]);
  mark('remove-old', 'done');
} else {
  say(`kept ${oldPath}; delete it later with --remove-old or by hand after checking the result`);
}
```
