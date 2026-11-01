#!/usr/bin/env node
// Measure what the idle IDE costs with expanded folders: CPU time, read system calls, directory listings,
// and log output over a fixed interval, for the shipped safety sweep and for a comparison interval.
// Both builds come from one disposable package copy; only the `SAFETY_SWEEP` constant differs.
// The IDE runs in the repository's nested compositor, so a host Wayland session is required.
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { closeSync, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, openSync, readdirSync, readFileSync, readlinkSync, realpathSync, rmSync, statfsSync, statSync, writeFileSync } from 'node:fs';
import { createConnection } from 'node:net';
import { homedir } from 'node:os';
import { join, relative, resolve, sep } from 'node:path';
import { setTimeout as wait } from 'node:timers/promises';
import { compositor, demandCurrentCompositor } from './nested-compositor.ts';
import { containerBounds } from './container-bounds.ts';

// The bound of every container this script starts: the shared computed share of the host, or the caller's IDE_CONTAINER_* values.
const containerLimits = containerBounds();

const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
if (!process.env.usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = realpathSync(process.env.usage_cache);
if (!cache.startsWith(realpathSync(privateRoot) + sep) || !statSync(cache).isDirectory()) throw new Error('Target cache must be a disposable directory below ' + privateRoot);
let cargoHome = 'ide-cargo';
if (process.env.usage_cargo) {
  cargoHome = realpathSync(process.env.usage_cargo);
  if (!cargoHome.startsWith(realpathSync(privateRoot) + sep) || !statSync(cargoHome).isDirectory()) throw new Error('Cargo home copy must be a disposable directory below ' + privateRoot);
}
const profile = process.env.usage_profile ?? 'debug';
if (profile !== 'debug' && profile !== 'release') throw new Error('profile must be debug or release');
const positive = ({ name, fallback }: { name: string; fallback: string }): number => {
  const value = Number(process.env['usage_' + name] ?? fallback);
  if (!Number.isInteger(value) || value < 1) throw new Error(name + ' must be a positive integer');
  return value;
};
const compare = positive({ name: 'compare', fallback: '10' });
const seconds = positive({ name: 'seconds', fallback: '60' });
const runs = positive({ name: 'runs', fallback: '3' });
const folderCounts = (process.env.usage_folders ?? '1,12,100').split(',').map(Number);
if (folderCounts.some(count => !Number.isInteger(count) || count < 1 || count > 200)) throw new Error('folders must be whole numbers from 1 to 200');

// The IDE writes its log from a writer thread, but a log on a busy disk still delays the records this
// measurement times. The live log therefore goes to memory-backed storage and is copied to the artifact after.
const TMPFS_MAGIC = 0x01021994;
const liveRoot = process.env.XDG_RUNTIME_DIR;
if (!liveRoot || (statSync(liveRoot).mode & 0o077) !== 0 || statfsSync(liveRoot).type !== TMPFS_MAGIC) throw new Error('XDG_RUNTIME_DIR must name a private memory-backed directory for the live session logs');

const origin = process.cwd();
demandCurrentCompositor();
const artifact = mkdtempSync(join(privateRoot, 'ide-idle-cost-'));
// build.rs embeds target/crate-licenses.json, which the notices task writes; the disposable target directory needs its copy.
if (!existsSync(join(cache, 'crate-licenses.json'))) copyFileSync(join(origin, 'target/crate-licenses.json'), join(cache, 'crate-licenses.json'));
const source = join(artifact, 'package');
cpSync(origin, source, { recursive: true, filter: path => !['target', '.git', 'node_modules'].includes(relative(origin, path).split(sep)[0] ?? '') });
mkdirSync(join(source, 'target'), { recursive: true });
console.log('IDLE_COST_ARTIFACT=' + artifact + ' profile=' + profile);

// Build the shipped interval first, then the comparison interval from the same copy.
const policyPath = join(source, 'src/refresh_policy.rs');
const policy = readFileSync(policyPath, 'utf8');
const shipped = policy.match(/pub const SAFETY_SWEEP: Duration = Duration::from_secs\((\d+)\);/);
if (!shipped) throw new Error('Cannot find the SAFETY_SWEEP constant in ' + policyPath);
if (seconds < 3 * Math.max(Number(shipped[1]), compare)) throw new Error('seconds must cover at least three safety sweeps of the slower build');
const build = (label: string): string => {
  const result = spawnSync('podman', [
    'run', '--rm', '--network=none', ...containerLimits.limitArguments,
    '--security-opt', 'label=disable',
    '--volume', source + ':/work', '--volume', cache + ':/work/target', '--volume', cargoHome + ':/cargo',
    '--workdir', '/work', ...containerLimits.environmentArguments, '--env', 'SLINT_EMIT_DEBUG_INFO=1',
    'localhost/monochromatic/ide', 'cargo', 'build', '--offline', ...(profile === 'release' ? ['--release'] : []),
  ], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  writeFileSync(join(artifact, 'build-' + label + '.log'), (result.stdout ?? '') + (result.stderr ?? ''));
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error('Build ' + label + ' failed; inspect ' + artifact);
  const binary = join(artifact, 'ide-sweep-' + label);
  copyFileSync(join(cache, profile, 'monochromatic-ide'), binary);
  return binary;
};
// One build of the IDE with the sweep interval it was compiled with.
interface Variant { sweep: number; binary: string }
const shippedText = shipped[1] ?? '';
const shippedVariant: Variant = { sweep: Number(shippedText), binary: build(shippedText + 's') };
const variants: Variant[] = [shippedVariant];
if (compare !== shippedVariant.sweep) {
  writeFileSync(policyPath, policy.replace(shipped[0], 'pub const SAFETY_SWEEP: Duration = Duration::from_secs(' + compare + ');'));
  variants.push({ sweep: compare, binary: build(compare + 's') });
  writeFileSync(policyPath, policy);
}

// One fixture per folder count: sibling folders of eight files each beside the displayed file.
// Folders one level below the root keep path resolution as short as in an ordinary project.
const fixture = (count: number): { project: string; file: string } => {
  const project = join(artifact, 'project-' + count);
  for (let index = 0; index < count; index++) {
    const folder = join(project, 'folder-' + String(index).padStart(3, '0'));
    mkdirSync(folder, { recursive: true });
    for (let entry = 0; entry < 8; entry++) writeFileSync(join(folder, 'file-' + entry + '.txt'), 'idle fixture\n');
  }
  const file = join(project, 'view.txt');
  writeFileSync(file, 'I am a big cat.\n'.repeat(40));
  return { project, file };
};
const fontConfig = join(artifact, 'fonts.conf');
writeFileSync(fontConfig, '<fontconfig><dir>/usr/share/fonts</dir><cachedir prefix="xdg">fontconfig</cachedir></fontconfig>');
for (const name of ['config', 'cache', 'data']) mkdirSync(join(artifact, name));
const runtime = [join(origin, 'target/debug/runtime'), join(cache, 'debug/runtime')].find(existsSync);
// The IDE logs warnings only unless RUST_LOG asks for more; this measurement reads its debug records and the
// compositor's close records, and both programs read RUST_LOG.
const environment: NodeJS.ProcessEnv = { ...process.env, SLINT_BACKEND: 'winit', FONTCONFIG_FILE: fontConfig, XDG_CONFIG_HOME: join(artifact, 'config'), XDG_CACHE_HOME: join(artifact, 'cache'), XDG_DATA_HOME: join(artifact, 'data'), RUST_LOG: 'nested_wayland_session=info,ide_app=debug,monochromatic_ide=debug' };
if (runtime) environment.HELIX_RUNTIME = runtime;

const plain = (text: string): string => text.replace(/\x1b\[[0-9;]*m/g, '');
const cpuTicks = (pid: string): number => {
  const stat = readFileSync('/proc/' + pid + '/stat', 'utf8');
  const fields = stat.slice(stat.lastIndexOf(')') + 2).split(' ');
  return Number(fields[11]) + Number(fields[12]);
};
// The IDE's inotify watches: one `inotify wd:` line per watch in each inotify descriptor's fdinfo.
const inotifyWatches = (pid: string): number => readdirSync('/proc/' + pid + '/fd').filter(fd => {
  try { return readlinkSync('/proc/' + pid + '/fd/' + fd) === 'anon_inode:inotify'; } catch { return false; }
}).reduce((total, fd) => total + (readFileSync('/proc/' + pid + '/fdinfo/' + fd, 'utf8').match(/^inotify wd:/gm) ?? []).length, 0);
// The counters of /proc/<pid>/io, of which this script reads the two named here.
interface ProcessIo { syscr: number; rchar: number }
const io = (pid: string): ProcessIo => Object.fromEntries(readFileSync('/proc/' + pid + '/io', 'utf8').trim().split('\n').map(line => line.split(': ')).map(([key, value]) => [key, Number(value)])) as ProcessIo;
const ticksPerSecond = Number(spawnSync('getconf', ['CLK_TCK'], { encoding: 'utf8' }).stdout.trim());
// Share of recent time in which some task (`some`) or every task (`full`) waited for the resource.
const pressure = (): Record<string, number> => Object.fromEntries(['cpu', 'io'].flatMap(resource => readFileSync('/proc/pressure/' + resource, 'utf8').trim().split('\n')
  .map((line): [string, number] => {
    const average = line.match(/avg10=([\d.]+)/)?.[1];
    if (average === undefined) throw new Error('The pressure line has no avg10 value: ' + line);
    return [resource + '_' + line.split(' ')[0], Number(average)];
  })));
const control = async ({ socketPath, line }: { socketPath: string; line: string }): Promise<string> => {
  const socket = createConnection(socketPath);
  await once(socket, 'connect');
  socket.write(line + '\n');
  const [response] = await once(socket, 'data');
  socket.end();
  return String(response).trim();
};
const round = ({ value, digits }: { value: number; digits: number }): number => Number(value.toFixed(digits));
// The distance from each value to the one before it.
const differences = (values: number[]): number[] => values.slice(1).map((value, index) => value - (values[index] ?? value));
const stampOf = (line: string): number => Date.parse(line.slice(0, 23) + 'Z');

// Expand every folder with real key input through the compositor's seat: Tab moves focus from the source
// to the tree, then Right expands a collapsed folder and Down moves on. Late listings shift rows under the
// focused index, which only makes the walk revisit rows, so each round presses twice the final row count.
const expandAll = async ({ socketPath, logPath, count }: { socketPath: string; logPath: string; count: number }): Promise<void> => {
  const key = async (name: string): Promise<void> => {
    const response = await control({ socketPath, line: 'key ' + name });
    if (!response.startsWith('ok')) throw new Error('Key ' + name + ' was refused: ' + response);
    await wait(8);
  };
  const watching = () => plain(readFileSync(logPath, 'utf8')).split('\n').filter(line => line.includes('watching directory')).length;
  for (let attempt = 0; attempt < 6 && watching() < count + 1; attempt++) {
    // Tab until the tree has focus; while the source has it, these keys only move its caret.
    await key('tab');
    await key('home');
    for (let step = 0; step < 2 * (9 * count + 12) && watching() < count + 1; step++) {
      await key('right');
      await key('down');
    }
    await wait(500);
  }
  if (watching() < count + 1) throw new Error('Only ' + watching() + ' of ' + (count + 1) + ' folders became watched; inspect ' + logPath);
};

// Run the IDE once, wait until every folder is watched and startup has settled, then call `measure(pid, log)`.
// The result also says whether the compositor had to force the IDE closed, which a stalled IDE causes.
const live = mkdtempSync(join(liveRoot, 'ide-idle-cost-'));
const session = async <Value>({ variant, count, tag, measure }: {
  variant: Variant;
  count: number;
  tag: string;
  measure: (pid: string, logPath: string) => Promise<Value>;
}): Promise<{ value: Value; forced_close: boolean }> => {
  const { project, file } = fixture(count);
  const socketPath = join(artifact, 'control-' + tag + '.sock');
  const livePath = join(live, 'ide-' + tag + '.log');
  const logPath = join(artifact, 'ide-' + tag + '.log');
  const log = openSync(livePath, 'w');
  const host = spawn(compositor, ['--socket', socketPath, '--size', '1100x660', '--color-scheme', 'dark', '--', variant.binary, project, '--file', file], { env: environment, stdio: ['ignore', log, log] });
  const exited = once(host, 'exit');
  let value: Value;
  try {
    const deadline = performance.now() + 120000;
    let pid: string | undefined;
    while (true) {
      if (host.exitCode !== null) throw new Error('The compositor exited early; inspect ' + logPath);
      if (performance.now() > deadline) throw new Error('The IDE did not start watching its root in 120 s; inspect ' + logPath);
      // The compositor also starts helpers, and a child that has not replaced itself yet still carries the
      // compositor's own command line, so only a child whose program is exactly the IDE binary counts.
      pid ??= spawnSync('pgrep', ['--parent', String(host.pid)], { encoding: 'utf8' }).stdout.trim().split('\n').filter(Boolean)
        .find(child => {
          try {
            return readFileSync('/proc/' + child + '/cmdline', 'utf8').split('\0')[0] === variant.binary;
          } catch (error) {
            // A helper can exit between the listing and this read; anything else is a real failure.
            if (!(error instanceof Error && 'code' in error && (error.code === 'ENOENT' || error.code === 'ESRCH'))) throw error;
            return false;
          }
        });
      const watching = plain(readFileSync(livePath, 'utf8')).split('\n').filter(line => line.includes('watching directory')).length;
      if (pid && watching >= 1) break;
      await wait(200);
    }
    await wait(1000);
    await expandAll({ socketPath, logPath: livePath, count });
    await wait(5000);
    value = await measure(pid, livePath);
  } finally {
    if (host.exitCode === null) {
      await control({ socketPath, line: 'quit' }).catch(() => host.kill('SIGTERM'));
      await Promise.race([exited, wait(10000).then(() => host.kill('SIGKILL'))]);
    }
    closeSync(log);
    copyFileSync(livePath, logPath);
    rmSync(livePath);
  }
  return { value, forced_close: plain(readFileSync(logPath, 'utf8')).includes('ignored close request') };
};

// One sample of the idle IDE over the interval. `rejected` names why the interval cannot be used:
// the IDE logs every safety reread, so a silence longer than one sweep plus a second means the IDE or
// the whole host stood still for part of the interval, and its counts would understate the cost.
interface IdleSample {
  inotify_watches: number;
  cpu_ms_per_s: number;
  read_calls_per_s: number;
  read_bytes_per_s: number;
  listings_per_s: number;
  directories_listed: number;
  sweeps_started: number;
  sweep_gap_median_s: number | null;
  longest_silence_s: number;
  log_lines_per_s: number;
  log_bytes_per_s: number;
  host_load: number;
  pressure_before: Record<string, number>;
  pressure_after: Record<string, number>;
  rejected?: string;
}
const sample = (sweep: number) => async (pid: string, logPath: string): Promise<IdleSample> => {
  const before = { ticks: cpuTicks(pid), io: io(pid), log: statSync(logPath).size, at: performance.now(), wall: Date.now(), load: readFileSync('/proc/loadavg', 'utf8').split(' ')[0], pressure: pressure() };
  await wait(seconds * 1000);
  const elapsed = (performance.now() - before.at) / 1000;
  const after = { ticks: cpuTicks(pid), io: io(pid), wall: Date.now(), pressure: pressure() };
  const appended = plain(readFileSync(logPath, 'utf8').slice(before.log));
  const lines = appended.split('\n').filter(Boolean);
  const own = lines.filter(line => line.includes('ide_app::') || line.includes('monochromatic_ide::'));
  const stamps = [before.wall, ...own.map(stampOf).filter(Number.isFinite), after.wall];
  const silence = Math.max(...differences(stamps)) / 1000;
  const sweeps = own.filter(line => line.includes('started the safety reread')).map(stampOf);
  const gaps = differences(sweeps).sort((left, right) => left - right);
  const listed = own.filter(line => line.includes('read directory snapshot'));
  return {
    inotify_watches: inotifyWatches(pid),
    cpu_ms_per_s: round({ value: ((after.ticks - before.ticks) * 1000 / ticksPerSecond) / elapsed, digits: 2 }),
    read_calls_per_s: round({ value: (after.io.syscr - before.io.syscr) / elapsed, digits: 1 }),
    read_bytes_per_s: Math.round((after.io.rchar - before.io.rchar) / elapsed),
    listings_per_s: round({ value: listed.length / elapsed, digits: 2 }),
    directories_listed: new Set(listed.map(line => line.match(/path=(\S+)/)?.[1])).size,
    sweeps_started: sweeps.length,
    sweep_gap_median_s: gaps.length === 0 ? null : round({ value: (gaps[Math.floor(gaps.length / 2)] ?? 0) / 1000, digits: 2 }),
    longest_silence_s: round({ value: silence, digits: 2 }),
    log_lines_per_s: round({ value: lines.length / elapsed, digits: 1 }),
    log_bytes_per_s: Math.round(Buffer.byteLength(appended) / elapsed),
    host_load: Number(before.load),
    pressure_before: before.pressure,
    pressure_after: after.pressure,
    ...(silence > sweep + 1 ? { rejected: 'the IDE logged nothing for ' + round({ value: silence, digits: 1 }) + ' s, longer than one ' + sweep + ' s sweep plus 1 s' } : {}),
  };
};

const ATTEMPTS = 4;
// One measured row of the results file: a sample, or the system call counts of the strace run.
interface IdleRow extends Partial<IdleSample> {
  sweep_s: number;
  folders: number;
  run: number | string;
  attempt?: number;
  forced_close?: boolean;
  calls_per_s?: Record<string, number>;
}
const results: IdleRow[] = [];
const save = () => writeFileSync(join(artifact, 'results.json'), JSON.stringify({ profile, seconds, runs, variants: variants.map(item => item.sweep), results }, null, 2));
try {
  for (const count of folderCounts) {
    // Builds alternate within each run, so slow drift of the host load affects both alike.
    for (let run = 1; run <= runs; run++) {
      for (const variant of variants) {
        for (let attempt = 1; attempt <= ATTEMPTS; attempt++) {
          const tag = variant.sweep + 's-' + count + '-run' + run + (attempt === 1 ? '' : '-try' + attempt);
          const { value, forced_close } = await session({ variant, count, tag, measure: sample(variant.sweep) });
          const row: IdleRow = { sweep_s: variant.sweep, folders: count, run, attempt, ...value, forced_close };
          if (forced_close && !row.rejected) row.rejected = 'the IDE did not close when asked, so it had stalled';
          results.push(row);
          save();
          console.log(JSON.stringify(row));
          if (!row.rejected) break;
        }
      }
    }
    for (const variant of variants) {
      // One further run under strace counts every system call; its CPU time is not comparable and not recorded.
      const tag = variant.sweep + 's-' + count + '-strace';
      const summaryPath = join(artifact, 'strace-' + tag + '.txt');
      const { value: calls } = await session({ variant, count, tag, measure: async pid => {
        const tracer = spawn('strace', ['--follow-forks', '--summary-only', '--attach=' + pid, '--output=' + summaryPath], { stdio: 'ignore' });
        const done = once(tracer, 'exit');
        const started = performance.now();
        await wait(seconds * 1000);
        const elapsed = (performance.now() - started) / 1000;
        tracer.kill('SIGINT');
        await done;
        const counted: Record<string, number> = {};
        for (const line of readFileSync(summaryPath, 'utf8').split('\n')) {
          const row = line.trim().split(/\s+/);
          const seconds = row[0] ?? '';
          const calls = row[3] ?? '';
          const name = row.at(-1) ?? '';
          if (row.length >= 5 && /^[\d.]+$/.test(seconds) && /^\d+$/.test(calls)) counted[name] = round({ value: Number(calls) / elapsed, digits: 1 });
        }
        return counted;
      } });
      results.push({ sweep_s: variant.sweep, folders: count, run: 'strace', calls_per_s: calls });
      save();
      const shown = ['openat', 'getdents64', 'read', 'statx', 'newfstatat', 'readlink', 'close', 'write', 'futex', 'total'].map(name => name + '=' + (calls[name] ?? 0)).join(' ');
      console.log(JSON.stringify({ sweep_s: variant.sweep, folders: count, run: 'strace' }) + ' ' + shown);
    }
  }
} finally {
  rmSync(live, { recursive: true, force: true });
}

// Lowest, middle, and highest accepted run per build and folder count; the spread between runs of one
// build is the noise that a difference between builds has to exceed.
const spread = (values: number[]): string => {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted.length === 0 ? 'none' : [sorted[0], sorted[Math.floor(sorted.length / 2)], sorted.at(-1)].join('/');
};
for (const count of folderCounts) {
  for (const variant of variants) {
    const all = results.filter(row => row.folders === count && row.sweep_s === variant.sweep && row.run !== 'strace');
    const kept = all.filter(row => !row.rejected);
    console.log(['SUMMARY', 'folders=' + count, 'sweep=' + variant.sweep + 's', 'accepted=' + kept.length, 'rejected=' + (all.length - kept.length),
      ...(['inotify_watches', 'cpu_ms_per_s', 'read_calls_per_s', 'listings_per_s', 'log_lines_per_s'] as const).map(name => name + '=' + spread(kept.flatMap(row => (row[name] === undefined ? [] : [row[name]]))))].join(' '));
  }
}
console.log('Idle cost results: ' + join(artifact, 'results.json'));
