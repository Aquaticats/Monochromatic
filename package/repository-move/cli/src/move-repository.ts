// Moves the repository and its worktree subvolumes to the 860 with extent sharing kept, failing closed.
// Modes: --print (default) prints every step and runs nothing; --run executes the steps in order and stops at the first
// failure. Needs root for --run (btrfs send and receive). The state folder records every completed step; a received
// subvolume is placed only when its record exists.
// Usage: node move-repository.ts --pass=first|final --main=<path> --snapshots=<NVMe folder> --received=<860 folder>
//   --final=<860 path of the main subvolume> --state=<folder> [--inventory=<btrfs subvolume list -o output>]
//   --inventory-main=<main's path as `btrfs subvolume list` prints it> [--inventory-prefix=home/=/var/home/] [--deadline-seconds=<per transfer>] [--min-free-gib=<n>]
//   [--clone-sources=all|main] [--limit=<n>] [--freeze] [--print|--run] [--no-scope --test-marker (tests only)]
import { spawnSync, } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, renameSync, statSync, statfsSync, writeFileSync, } from 'node:fs';
import { join, relative, } from 'node:path';

function flag(name: string,): string | undefined {
  const found: string | undefined = process.argv.find((argument,) => argument.startsWith(`--${name}=`,),);
  return found?.slice(name.length + 3,);
}
function need(name: string,): string {
  const value: string | undefined = flag(name,);
  if (value === undefined) {
    throw new Error(`missing --${name}=`,);
  }
  return value;
}
const pass: string = need('pass',);
if (pass !== 'first' && pass !== 'final') {
  throw new Error('--pass must be first or final',);
}
const mainPath: string = need('main',);
const snapshotFolder: string = need('snapshots',);
const receivedFolder: string = need('received',);
const finalPath: string = need('final',);
const stateFolder: string = need('state',);
const deadlineSeconds: number = Number(flag('deadline-seconds',) ?? '14400',);
const minFreeGiB: number = Number(flag('min-free-gib',) ?? '150',);
const cloneSources: string = flag('clone-sources',) ?? 'all';
const limit: number | undefined = flag('limit',) === undefined ? undefined : Number(flag('limit',),);
const freeze: boolean = process.argv.includes('--freeze',);
const run: boolean = process.argv.includes('--run',);
const noScope: boolean = process.argv.includes('--no-scope',);
const inventoryFile: string | undefined = flag('inventory',);
const [prefixFrom, prefixTo,] = (flag('inventory-prefix',) ?? 'home/=/var/home/').split('=',);
const scopeBounds: string[] = ['--scope', '--collect', '--property=CPUQuota=200%', '--property=MemoryMax=2G',
  '--property=TasksMax=32', '--property=IOWeight=10',];

function log(message: string,): void {
  console.error(`${new Date().toISOString()} ${message}`,);
}
function fail(message: string,): never {
  log(`STOP: ${message}`,);
  process.exit(1);
}
// Runs one command; in print mode only shows it. Returns stdout.
function exec(command: string, args: readonly string[], options: { scope?: boolean; deadline?: number; } = {},): string {
  const wrapped: string[] = [command, ...args,];
  let program: string = command;
  let argv: string[] = [...args,];
  if (options.deadline !== undefined) {
    argv = ['--kill-after=30', String(options.deadline,), program, ...argv,];
    program = 'timeout';
  }
  if (options.scope === true && !noScope) {
    argv = [...scopeBounds, program, ...argv,];
    program = 'systemd-run';
  }
  console.log([program, ...argv,].map((word,) => (/^[\w@%+=:,./-]+$/.test(word,) ? word : `'${word.replaceAll("'", `'\\''`,)}'`),).join(' ',),);
  if (!run) {
    return '';
  }
  const result = spawnSync(program, argv, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);
  if (result.status !== 0) {
    fail(`${wrapped.join(' ',)} exited ${String(result.status,)}${result.error ? ` (${result.error.message})` : ''}: ${result.stderr.trim().slice(0, 300,)}`,);
  }
  return result.stdout;
}
function marker(name: string, step: string,): string {
  return join(stateFolder, `${name}.${step}.json`,);
}
function done(name: string, step: string,): boolean {
  return existsSync(marker(name, step,),);
}
function record(name: string, step: string, data: object,): void {
  if (!run) {
    return;
  }
  const temporary: string = `${marker(name, step,)}.tmp`;
  writeFileSync(temporary, JSON.stringify({ at: new Date().toISOString(), ...data, },),);
  renameSync(temporary, marker(name, step,),);
}
function field(text: string, label: string,): string {
  const line: string | undefined = text.split('\n',).find((row,) => row.trim().startsWith(`${label}:`,),);
  return line?.split(':',).slice(1,).join(':',).trim() ?? '';
}
function freeGiB(path: string,): number {
  const info = statfsSync(path,);
  return (info.bavail * info.bsize) / 2 ** 30;
}

// Inventory: the Git registry, reconciled with the filesystem's own list when one is given.
const listing = spawnSync('/usr/bin/git', ['--no-optional-locks', '-C', mainPath, 'worktree', 'list', '--porcelain',], { encoding: 'utf8', },);
if (listing.status !== 0) {
  fail(`cannot list worktrees: ${listing.stderr}`,);
}
const registered: string[] = listing.stdout.split('\n',).filter((line,) => line.startsWith('worktree ',),)
  .map((line,) => line.slice('worktree '.length,),).filter((path,) => path.startsWith(`${mainPath}/`,),);
const testMarker: boolean = process.argv.includes('--test-marker',);
// A btrfs subvolume root always has inode 256; tests mark fake ones with a file instead.
function isSubvolume(path: string,): boolean {
  try {
    return testMarker ? existsSync(join(path, '.stub-subvolume',),) : statSync(path, { bigint: true, },).ino === 256n;
  } catch {
    return false;
  }
}
const registeredSubvolumes: string[] = registered.filter(isSubvolume,).sort();
if (inventoryFile !== undefined) {
  // `btrfs subvolume list <filesystem>` lists every subvolume of the filesystem with its path from the top level; only
  // those below the main checkout's own listing path count, the rest are other data and are skipped.
  const mainListed: string = flag('inventory-main',) ?? '';
  const listed: string[] = readFileSync(inventoryFile, 'utf8',).split('\n',).filter((row,) => row.trim() !== '',).flatMap((row,) => {
    const path: string | undefined = / path (.+)$/.exec(row,)?.[1];
    if (path === undefined) {
      fail(`inventory line not understood: ${row}`,);
    }
    if (!path.startsWith(`${mainListed}/`,)) {
      return [];
    }
    if (!path.startsWith(prefixFrom ?? '',)) {
      fail(`inventory path outside ${prefixFrom ?? ''}: ${row}`,);
    }
    return [`${prefixTo ?? ''}${path.slice((prefixFrom ?? '').length,)}`,];
  },).sort();
  const missingFromRegistry: string[] = listed.filter((path,) => !registeredSubvolumes.includes(path,),);
  const missingFromDisk: string[] = registeredSubvolumes.filter((path,) => !listed.includes(path,),);
  if (missingFromRegistry.length > 0 || missingFromDisk.length > 0) {
    fail(`inventory differs from the Git registry; subvolumes Git does not know: ${missingFromRegistry.join(' ',) || 'none'}; `
      + `registered but not subvolumes: ${missingFromDisk.join(' ',) || 'none'}. Give each an owner-approved disposition first.`,);
  }
  log(`inventory agrees: ${String(listed.length,)} nested subvolumes`,);
} else if (run) {
  fail('--run needs --inventory (root: btrfs subvolume list of the whole filesystem)',);
}
const chosen: string[] = limit === undefined ? registeredSubvolumes : registeredSubvolumes.slice(0, limit,);
const subvolumes: string[] = [mainPath, ...chosen,];

if (run) {
  mkdirSync(stateFolder, { recursive: true, mode: 0o700, },);
}
const first: Map<string, string> = new Map<string, string>();
if (pass === 'final') {
  for (const row of readFileSync(join(stateFolder, 'first-pass-manifest.tsv',), 'utf8',).split('\n',).filter((line,) => line !== '',)) {
    const [name, path,] = row.split('\t',);
    first.set(path ?? '', name ?? '',);
  }
} else if (run) {
  writeFileSync(join(stateFolder, 'first-pass-manifest.tsv',), `${subvolumes.map((path, index,) => `s${String(index,).padStart(3, '0',)}\t${path}`,).join('\n',)}\n`,);
}

if (freeze) {
  for (const path of subvolumes) {
    exec('btrfs', ['property', 'set', '-ts', path, 'ro', 'true',],);
  }
}
const prefix: string = pass === 'first' ? 's' : 'f';
const sent: string[] = [];
subvolumes.forEach((path, index,) => {
  const name: string = `${prefix}${String(index,).padStart(3, '0',)}`;
  const snapshot: string = join(snapshotFolder, name,);
  const received: string = join(receivedFolder, name,);
  log(`${name} ${path}`,);
  if (run && freeGiB(receivedFolder,) < minFreeGiB) {
    fail(`the 860 has under ${String(minFreeGiB,)} GiB free before ${name}; stopping with every earlier step recorded`,);
  }
  if (!done(name, 'snapshot',)) {
    exec('btrfs', ['subvolume', 'snapshot', '-r', path, snapshot,],);
    const shown: string = run ? exec('btrfs', ['subvolume', 'show', snapshot,],) : '';
    record(name, 'snapshot', { path, uuid: field(shown, 'UUID',), },);
  }
  if (!done(name, 'received',)) {
    if (run && existsSync(received,)) {
      fail(`${received} exists without a completion record: a receive was interrupted. Inspect it, delete it with `
        + `'btrfs subvolume delete ${received}' yourself, and run again. Nothing was deleted by this script.`,);
    }
    const parentName: string | undefined = first.get(path,);
    const options: string[] = [
      ...(parentName === undefined ? [] : ['-p', join(snapshotFolder, parentName,),]),
      ...(cloneSources === 'main' ? sent.slice(0, 1,) : sent).flatMap((earlier,) => ['-c', join(snapshotFolder, earlier,),]),
    ];
    const pipeline: string = `btrfs send ${[...options, snapshot,].join(' ',)} | btrfs receive ${receivedFolder}`;
    exec('bash', ['-o', 'pipefail', '-c', pipeline,], { scope: true, deadline: deadlineSeconds, },);
    if (run) {
      const readOnly: string = exec('btrfs', ['property', 'get', '-ts', received, 'ro',],).trim();
      const shown: string = exec('btrfs', ['subvolume', 'show', received,],);
      const sentUuid: string = field(exec('btrfs', ['subvolume', 'show', snapshot,],), 'UUID',);
      if (readOnly !== 'ro=true' || field(shown, 'Received UUID',) !== sentUuid || sentUuid === '' || sentUuid === '-') {
        fail(`${received} is not a complete read-only copy of ${snapshot} (${readOnly}, received UUID `
          + `${field(shown, 'Received UUID',)}, sent UUID ${sentUuid})`,);
      }
      record(name, 'received', { path, uuid: sentUuid, },);
    }
  }
  if (pass === 'final' && !done(name, 'placed',)) {
    if (run && !done(name, 'received',)) {
      fail(`refusing to place ${name}: no completion record for its receive`,);
    }
    const target: string = index === 0 ? finalPath : join(finalPath, relative(mainPath, path,),);
    if (index !== 0) {
      // The stub of a nested subvolume in the received parent is an empty directory of inode 2 (measured on the 860).
      exec('rmdir', [target,],);
    }
    exec('btrfs', ['subvolume', 'snapshot', received, target,],);
    if (run && exec('btrfs', ['property', 'get', '-ts', target, 'ro',],).trim() !== 'ro=false') {
      fail(`${target} is not writable after placement`,);
    }
    record(name, 'placed', { target, },);
  }
  sent.push(name,);
},);
log(run ? `pass ${pass} finished; ${String(subvolumes.length,)} subvolumes recorded in ${stateFolder}` : 'print mode: nothing was run',);
