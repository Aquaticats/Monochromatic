#!/usr/bin/env node
// Count the inotify watches the language servers take, confined as in production, on large disposable
// projects: a dependency-free Rust workspace of many crates with a filled `target` and `.git`, whose first
// crate also holds a `node_modules` as a package with both manifests does, and a TypeScript 7 project with a
// large `node_modules` and `.git`. Watched directories are classified by inode into source, target,
// `.git`, `node_modules`, and outside the project. Every process the headless Language module
// starts is sampled from /proc (inotify descriptors and their `wd:` lines) while the servers load.
// The IDE watches the project's source folders for the servers and forwards changes; each production case
// also changes a file the displayed file imports, outside the IDE, and waits for the displayed file's
// diagnostics to follow. The cases without forwarding are the positive control: the same change must leave
// the diagnostics stale. SERVER_WATCH_BEFORE_BINARY names an older ide-language-inspect build to measure as
// "before" (case rust-before, which only opens, waits, and closes, since older builds know fewer steps).
// No server is ever pointed at this repository; it is read only to copy the TypeScript 7 packages.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { cpSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, readlinkSync, realpathSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';

const binary = resolve('target/debug/ide-language-inspect');
const store = resolve('../../../node_modules/.pnpm');
const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
const base = realpathSync(mkdtempSync(join(privateRoot, 'ide-server-watches-')));
const results = join(base, 'results');
mkdirSync(results);
console.log('SERVER_WATCHES_ARTIFACT=' + base);
// Production cases wait for the servers to register watchers and for the baseline diagnostics before this
// pause, so it only has to cover the scan and the watches; an older build's rust-analyzer adds its own
// watches only after loading and building compile-time dependencies, which took up to 60 s on a loaded host.
const sampleSeconds = Number(process.env.SERVER_WATCH_SECONDS ?? '20');
// An older build's cases only open, wait, and close; its rust-analyzer needs the long wait to add its watches.
const beforeSeconds = Number(process.env.SERVER_WATCH_BEFORE_SECONDS ?? '150');

const write = ({ root, relative, text }: { root: string; relative: string; text: string }): void => {
  const target = join(root, relative);
  mkdirSync(join(target, '..'), { recursive: true });
  writeFileSync(target, text);
};
let directories = 0;
const folder = ({ root, relative }: { root: string; relative: string }): void => {
  mkdirSync(join(root, relative), { recursive: true });
  directories++;
};
// Directories that should not affect results: a build tree and a git directory of realistic shape.
const filler = (root: string): void => {
  for (let index = 0; index < 3000; index++) folder({ root, relative: 'target/debug/build/dep-' + String(index).padStart(4, '0') + '/out' });
  for (let index = 0; index < 256; index++) folder({ root, relative: '.git/objects/' + index.toString(16).padStart(2, '0') });
  folder({ root, relative: '.git/refs/heads' });
};

// Rust: forty member crates of twenty-five module directories each, no dependencies, with a lockfile.
const rust = join(base, 'rust-workspace');
const crates: string[] = [];
for (let crate = 0; crate < 40; crate++) {
  const name = 'c' + String(crate).padStart(3, '0');
  crates.push(name);
  write({ root: rust, relative: 'crates/' + name + '/Cargo.toml', text: ['[package]', 'name = "' + name + '"', 'version = "0.1.0"', 'edition = "2024"', ''].join('\n') });
  const modules: string[] = [];
  for (let module = 0; module < 25; module++) {
    const moduleName = 'm' + String(module).padStart(2, '0');
    modules.push('pub mod ' + moduleName + ';');
    write({ root: rust, relative: 'crates/' + name + '/src/' + moduleName + '/mod.rs', text: 'pub fn value() -> u32 { ' + module + ' }\n' });
    directories++;
  }
  // The displayed file imports from another file; renaming the import there makes an unresolved import here.
  modules.push('use crate::m00::value as first;', 'pub fn uses() -> u32 { first() }');
  write({ root: rust, relative: 'crates/' + name + '/src/lib.rs', text: modules.join('\n') + '\n' });
}
// A package with both Cargo.toml and package.json keeps its JavaScript dependencies beside its sources.
for (let pkg = 0; pkg < 1000; pkg++) folder({ root: rust, relative: 'crates/c000/node_modules/pkg-' + String(pkg).padStart(4, '0') + '/lib' });
write({ root: rust, relative: 'Cargo.toml', text: ['[workspace]', 'resolver = "3"', 'members = [' + crates.map(name => '"crates/' + name + '"').join(', ') + ']', ''].join('\n') });
// Confined, cargo cannot write a lockfile into the read-only project, so one is committed as real projects have.
write({ root: rust, relative: 'Cargo.lock', text: ['# This file is automatically @generated by Cargo.', '# It is not intended for manual editing.', 'version = 4', '', ...crates.flatMap(name => ['[[package]]', 'name = "' + name + '"', 'version = "0.1.0"', ''])].join('\n') });
filler(rust);

// TypeScript: forty source directories of twenty-five files that import from generated packages.
const ts = join(base, 'ts-project');
write({ root: ts, relative: 'package.json', text: JSON.stringify({ name: 'watch-ts', private: true, type: 'module' }, null, 2) + '\n' });
write({ root: ts, relative: 'tsconfig.json', text: JSON.stringify({ compilerOptions: { target: 'es2022', module: 'esnext', moduleResolution: 'bundler', strict: true, noEmit: true }, include: ['src'] }, null, 2) + '\n' });
for (let pkg = 0; pkg < 3000; pkg++) {
  const name = 'pkg-' + String(pkg).padStart(4, '0');
  write({ root: ts, relative: 'node_modules/' + name + '/package.json', text: JSON.stringify({ name, version: '1.0.0', types: 'index.d.ts' }) + '\n' });
  write({ root: ts, relative: 'node_modules/' + name + '/index.d.ts', text: 'export declare const value: number;\n' });
  folder({ root: ts, relative: 'node_modules/' + name + '/lib' });
}
for (let group = 0; group < 40; group++) {
  for (let file = 0; file < 25; file++) {
    const pkg = 'pkg-' + String(group * 25 + file).padStart(4, '0');
    write({ root: ts, relative: 'src/g' + String(group).padStart(2, '0') + '/f' + String(file).padStart(2, '0') + '.ts', text: 'import { value } from "' + pkg + '";\nexport const doubled: number = value * 2;\n' });
  }
  directories++;
}
// The displayed file imports from another file; removing the export there makes an error here.
write({ root: ts, relative: 'src/g00/dep.ts', text: 'export const local: number = 1;\n' });
write({ root: ts, relative: 'src/g00/f00.ts', text: 'import { value } from "pkg-0000";\nimport { local } from "./dep";\nexport const doubled: number = value * 2 + local;\n' });
cpSync(join(store, 'typescript@7.0.2', 'node_modules', 'typescript'), join(ts, 'node_modules', 'typescript'), { recursive: true });
cpSync(join(store, '@typescript+typescript-linux-x64@7.0.2', 'node_modules', '@typescript', 'typescript-linux-x64'), join(ts, 'node_modules', '@typescript', 'typescript-linux-x64'), { recursive: true });
filler(ts);
const countDirectories = (root: string): number => {
  let count = 0;
  const walk = (directory: string): void => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory() && !entry.isSymbolicLink()) {
        count++;
        walk(join(directory, entry.name));
      }
    }
  };
  walk(root);
  return count;
};

// Every process below `root` (by parent links in /proc/<pid>/stat), with its inotify watch count.
interface WatchRow { pid: number; comm: string; instances: number; watches: number }
const descendants = (root: number | undefined): WatchRow[] => {
  // A process that did not start has no descendants.
  if (root === undefined) return [];
  const parents = new Map<number, number>();
  for (const pid of readdirSync('/proc').filter(name => /^\d+$/.test(name))) {
    try {
      const stat = readFileSync('/proc/' + pid + '/stat', 'utf8');
      parents.set(Number(pid), Number(stat.slice(stat.lastIndexOf(')') + 2).split(' ')[1]));
    } catch { /* exited */ }
  }
  const found = new Set<number>([root]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const [pid, parent] of parents) {
      if (found.has(parent) && !found.has(pid)) {
        found.add(pid);
        grew = true;
      }
    }
  }
  const rows: WatchRow[] = [];
  for (const pid of found) {
    let comm: string;
    let fds: string[];
    try {
      comm = readFileSync('/proc/' + pid + '/comm', 'utf8').trim();
      fds = readdirSync('/proc/' + pid + '/fd');
    } catch { continue; }
    let instances = 0;
    let watches = 0;
    for (const fd of fds) {
      let target: string;
      try { target = readlinkSync('/proc/' + pid + '/fd/' + fd); } catch { continue; }
      if (target !== 'anon_inode:inotify') continue;
      instances++;
      try { watches += (readFileSync('/proc/' + pid + '/fdinfo/' + fd, 'utf8').match(/^inotify wd:/gm) ?? []).length; } catch { /* closed */ }
    }
    rows.push({ pid, comm, instances, watches });
  }
  return rows;
};

const env: NodeJS.ProcessEnv = { ...process.env, XDG_CACHE_HOME: join(base, 'app-cache') };
delete env.CARGO_TARGET_DIR;
delete env.CARGO_BUILD_BUILD_DIR;
const selected = process.env.SERVER_WATCH_CASES ? new Set(process.env.SERVER_WATCH_CASES.split(',')) : undefined;
// Each change replaces the imported file's content; the next case writes the original back first.
// rust-analyzer's own analysis reports a call with the wrong number of arguments (mismatched-arg-count);
// `cargo check` runs only on save, so it is turned off here and cannot report the change by coincidence.
const rustChange = { file: 'crates/c000/src/m00/mod.rs', original: 'pub fn value() -> u32 { 0 }\n', changed: 'pub fn value(extra: u32) -> u32 { extra }\n' };
const rustNativeOnly = '[language-server.rust-analyzer.config]\ncheckOnSave = false\n';
const tsChange = { file: 'src/g00/dep.ts', original: 'export const local: number = 1;\n', changed: 'export const other: number = 1;\n' };
interface WatchCase {
  name: string;
  project: string;
  file: string;
  change?: { file: string; original: string; changed: string };
  forward?: boolean;
  extra_languages?: string;
  binary?: string;
}
const allCases: WatchCase[] = [
  { name: 'rust-production', project: rust, file: 'crates/c000/src/lib.rs', change: rustChange, forward: true, extra_languages: rustNativeOnly },
  { name: 'rust-no-forwarding', project: rust, file: 'crates/c000/src/lib.rs', change: rustChange, forward: false, extra_languages: rustNativeOnly },
  { name: 'typescript-production', project: ts, file: 'src/g00/f00.ts', change: tsChange, forward: true },
  { name: 'typescript-no-forwarding', project: ts, file: 'src/g00/f00.ts', change: tsChange, forward: false },
  ...(process.env.SERVER_WATCH_BEFORE_BINARY ? [
    { name: 'rust-before', project: rust, file: 'crates/c000/src/lib.rs', binary: process.env.SERVER_WATCH_BEFORE_BINARY },
    { name: 'typescript-before', project: ts, file: 'src/g00/f00.ts', binary: process.env.SERVER_WATCH_BEFORE_BINARY },
  ] : []),
];
// SERVER_WATCH_ROUNDS repeats the selected cases in order, so two cases alternate and runs of one case can be compared.
const rounds = Number(process.env.SERVER_WATCH_ROUNDS ?? '1');
const chosen = allCases.filter(item => !selected || selected.has(item.name));
const cases = Array.from({ length: rounds }, (_, round) => chosen.map(item => ({ ...item, name: rounds > 1 ? item.name + '-' + (round + 1) : item.name }))).flat();
// Directory inodes of each project, classified, so watched inodes can be named.
const classify = (root: string): Map<number, string> => {
  const kinds = new Map<number, string>();
  const walk = ({ directory, kind }: { directory: string; kind: string }): void => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (!entry.isDirectory() || entry.isSymbolicLink()) continue;
      const child = join(directory, entry.name);
      const childKind = kind !== 'source' ? kind : entry.name === 'target' ? 'target' : entry.name === '.git' ? 'git' : entry.name === 'node_modules' ? 'node_modules' : 'source';
      kinds.set(statSync(child).ino, childKind);
      walk({ directory: child, kind: childKind });
    }
  };
  kinds.set(statSync(root).ino, 'source');
  walk({ directory: root, kind: 'source' });
  return kinds;
};
const inodes = new Map<string, Map<number, string>>([[rust, classify(rust)], [ts, classify(ts)]]);
// Watched inodes of one process, read from its inotify descriptors.
const watchedInodes = (pid: number): number[] => {
  const found: number[] = [];
  let fds: string[] = [];
  try { fds = readdirSync('/proc/' + pid + '/fd'); } catch { return found; }
  for (const fd of fds) {
    let target: string;
    try { target = readlinkSync('/proc/' + pid + '/fd/' + fd); } catch { continue; }
    if (target !== 'anon_inode:inotify') continue;
    let info = '';
    try { info = readFileSync('/proc/' + pid + '/fdinfo/' + fd, 'utf8'); } catch { continue; }
    for (const match of info.matchAll(/^inotify wd:\S+ ino:([0-9a-f]+)/gm)) found.push(parseInt(match[1] ?? '', 16));
  }
  return found;
};
const summary: { base: string; directories: Record<string, number>; sample_seconds: number; cases: unknown[] } = {
  base,
  directories: { rust: countDirectories(rust), typescript: countDirectories(ts) },
  sample_seconds: sampleSeconds,
  cases: [],
};
// The records of the inspection binary's event stream, one JSON object per line; only the fields read here are named.
interface ServerEvent { step?: number; ms: number; result?: { ready?: boolean; within?: boolean; folders?: unknown } }
interface PlanStep { label: string; do: string; [field: string]: unknown }
for (const item of cases) {
  // The changed file starts each case with its original content, outside any watch the case can see.
  if (item.change) writeFileSync(join(item.project, item.change.file), item.change.original);
  const steps: PlanStep[] = item.change
    ? [
      { label: 'open', do: 'open', file: item.file },
      { label: 'ready', do: 'ready', seconds: 120 },
      // A server registers its watchers once it has loaded the project; the IDE then scans and watches.
      ...(item.forward ? [{ label: 'folders', do: 'folders', minimum: 1, seconds: 150 }] : []),
      { label: 'baseline', do: 'diagnostics', minimum: 0, maximum: 0, seconds: 120 },
      { label: 'load', do: 'sleep', milliseconds: sampleSeconds * 1000 },
      { label: 'change', do: 'write', file: item.change.file, text: item.change.changed },
      { label: 'updated', do: 'diagnostics', minimum: 1, maximum: 100, seconds: 60 },
      { label: 'close', do: 'close' },
    ]
    : [
      { label: 'open', do: 'open', file: item.file },
      { label: 'ready', do: 'ready', seconds: 120 },
      { label: 'load', do: 'sleep', milliseconds: beforeSeconds * 1000 },
      { label: 'close', do: 'close' },
    ];
  const plan = { project: item.project, ...(item.extra_languages ? { extra_languages: item.extra_languages } : {}), ...(item.change ? { forward_file_changes: item.forward } : {}), steps };
  const planPath = join(results, item.name + '.plan.json');
  writeFileSync(planPath, JSON.stringify(plan, null, 2));
  const run = spawn(item.binary ?? binary, [planPath], { env, stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = '';
  let stderr = '';
  run.stdout.on('data', chunk => { stdout += chunk; });
  run.stderr.on('data', chunk => { stderr += chunk; });
  const done = once(run, 'exit');
  const peaks = new Map<string, { comm: string; instances: number; watches: number }>();
  const sampleStart = performance.now();
  let firstWatchSeconds: number | null = null;
  const kinds: Record<string, Record<string, number>> = {};
  let finished = false;
  done.then(() => { finished = true; });
  while (!finished) {
    for (const row of descendants(run.pid)) {
      const key = row.comm;
      const previous = peaks.get(key) ?? { comm: row.comm, instances: 0, watches: 0 };
      peaks.set(key, { comm: row.comm, instances: Math.max(previous.instances, row.instances), watches: Math.max(previous.watches, row.watches) });
      if (row.watches > 0 && firstWatchSeconds === null) firstWatchSeconds = Math.round((performance.now() - sampleStart) / 1000);
      if (row.watches > 0 && row.watches >= previous.watches) {
        const counted: Record<string, number> = {};
        for (const inode of watchedInodes(row.pid)) {
          const kind = inodes.get(item.project)?.get(inode) ?? 'outside the project';
          counted[kind] = (counted[kind] ?? 0) + 1;
        }
        kinds[row.comm] = counted;
      }
    }
    await wait(500);
  }
  const [status] = await done;
  writeFileSync(join(results, item.name + '.events.jsonl'), stdout);
  writeFileSync(join(results, item.name + '.stderr.txt'), stderr);
  const events = stdout.split('\n').filter(Boolean).map((line): ServerEvent => JSON.parse(line));
  const ready = events.find(event => event.step === 1)?.result?.ready === true;
  // What each labelled step returned: `within` for waits, `folders` for the folder count.
  const stepEvent = (label: string): ServerEvent | undefined => events.find(event => event.step === steps.findIndex(step => step.label === label));
  const outcome = (label: string): ServerEvent['result'] => stepEvent(label)?.result;
  // The change step records when the file was written; the updated step records when the wait ended.
  const updated = stepEvent('updated');
  const changed = stepEvent('change');
  const updatedAfter = updated?.result?.within && changed ? updated.ms - changed.ms : null;
  const processes = [...peaks.values()].sort((left, right) => right.watches - left.watches);
  const totalWatches = processes.reduce((sum, row) => sum + row.watches, 0);
  const record = { name: item.name, exit: status, ready, total_watches: totalWatches, processes, watched_kinds: kinds, first_watch_seconds: firstWatchSeconds, folders: outcome('folders'), baseline: outcome('baseline'), updated: outcome('updated'), updated_after_ms: updatedAfter };
  summary.cases.push(record);
  console.log(JSON.stringify({ name: record.name, exit: status, ready, total_watches: totalWatches, processes: processes.filter(row => row.instances > 0).map(row => row.comm + '=' + row.watches + ' watches/' + row.instances + ' instances').join(', '), watched_kinds: kinds, folders: record.folders, baseline_within: record.baseline?.within ?? null, updated_within: record.updated?.within ?? null, updated_after_ms: updatedAfter }));
}
writeFileSync(join(results, 'results.json'), JSON.stringify(summary, null, 2));
console.log('directories: ' + JSON.stringify(summary.directories));
console.log('Server watch results: ' + join(results, 'results.json'));
