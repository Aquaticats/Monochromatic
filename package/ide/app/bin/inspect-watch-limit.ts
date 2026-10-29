#!/usr/bin/env node
// Positive control for the inotify watch limit: run the real IDE in the nested compositor inside a disposable
// user namespace whose own watch limit (user.max_inotify_watches) is lowered, so only a few shown folders
// can be watched. The host's limit is never touched; the namespace and its limit end with the session.
// The script expands every folder with real keys, then checks that the IDE logged the limit once, made only a
// few failing watch calls (counted by strace) instead of one per folder per sweep, still listed a file created
// in an unwatched folder through the timers and the safety sweep, retried at once when the tree was scrolled
// (Home moves the selection from the last row to the first), and logged once when the limit was raised.
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { closeSync, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, openSync, readFileSync, realpathSync, rmSync, statfsSync, statSync, writeFileSync } from 'node:fs';
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
const positive = ({ name, fallback }: { name: string; fallback: string }): number => {
  const value = Number(process.env['usage_' + name] ?? fallback);
  if (!Number.isInteger(value) || value < 1) throw new Error(name + ' must be a positive integer');
  return value;
};
const folders = positive({ name: 'folders', fallback: '12' });
const limit = positive({ name: 'limit', fallback: '4' });
const seconds = positive({ name: 'seconds', fallback: '60' });
if (limit >= folders + 1) throw new Error('limit must be below the number of shown directories (folders + 1)');

// The IDE logs from the UI thread; a live log on a busy disk would stall it, so it goes to memory-backed storage.
const TMPFS_MAGIC = 0x01021994;
const liveRoot = process.env.XDG_RUNTIME_DIR;
if (!liveRoot || (statSync(liveRoot).mode & 0o077) !== 0 || statfsSync(liveRoot).type !== TMPFS_MAGIC) throw new Error('XDG_RUNTIME_DIR must name a private memory-backed directory for the live session log');
for (const tool of ['unshare', 'nsenter', 'strace']) {
  if (spawnSync(tool, ['--version'], { stdio: 'ignore' }).status !== 0) throw new Error('This control needs ' + tool);
}

const origin = process.cwd();
demandCurrentCompositor();
const artifact = mkdtempSync(join(privateRoot, 'ide-watch-limit-'));
const source = join(artifact, 'package');
cpSync(origin, source, { recursive: true, filter: path => !['target', '.git', 'node_modules'].includes(relative(origin, path).split(sep)[0] ?? '') });
mkdirSync(join(source, 'target'), { recursive: true });
console.log('WATCH_LIMIT_ARTIFACT=' + artifact);
const built = spawnSync('podman', [
  'run', '--rm', '--network=none', ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', source + ':/work', '--volume', cache + ':/work/target', '--volume', cargoHome + ':/cargo',
  '--workdir', '/work', ...containerLimits.environmentArguments, '--env', 'SLINT_EMIT_DEBUG_INFO=1',
  'localhost/monochromatic/ide', 'cargo', 'build', '--offline', '--bin', 'monochromatic-ide',
], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
writeFileSync(join(artifact, 'build.log'), (built.stdout ?? '') + (built.stderr ?? ''));
if (built.error) throw built.error;
if (built.status !== 0) throw new Error('Build failed; inspect ' + join(artifact, 'build.log'));
const binary = join(artifact, 'monochromatic-ide');
copyFileSync(join(cache, 'debug', 'monochromatic-ide'), binary);

// Sibling folders of eight files each beside the displayed file, as in the idle-cost measurement.
const project = join(artifact, 'project');
for (let index = 0; index < folders; index++) {
  const folder = join(project, 'folder-' + String(index).padStart(3, '0'));
  mkdirSync(folder, { recursive: true });
  for (let entry = 0; entry < 8; entry++) writeFileSync(join(folder, 'file-' + entry + '.txt'), 'limit fixture\n');
}
const displayed = join(project, 'view.txt');
writeFileSync(displayed, 'I am a big cat.\n');
const fontConfig = join(artifact, 'fonts.conf');
writeFileSync(fontConfig, '<fontconfig><dir>/usr/share/fonts</dir><cachedir prefix="xdg">fontconfig</cachedir></fontconfig>');
for (const name of ['config', 'cache', 'data']) mkdirSync(join(artifact, name));
// The measurement reads the IDE's debug lines; a build that logs warnings by default shows more when RUST_LOG asks.
const environment: NodeJS.ProcessEnv = { ...process.env, RUST_LOG: 'ide_app=debug,monochromatic_ide=debug', SLINT_BACKEND: 'winit', FONTCONFIG_FILE: fontConfig, XDG_CONFIG_HOME: join(artifact, 'config'), XDG_CACHE_HOME: join(artifact, 'cache'), XDG_DATA_HOME: join(artifact, 'data') };

const plain = (text: string): string => text.replace(/\x1b\[[0-9;]*m/g, '');
const control = async ({ socketPath, line }: { socketPath: string; line: string }): Promise<string> => {
  const socket = createConnection(socketPath);
  await once(socket, 'connect');
  socket.write(line + '\n');
  const [response] = await once(socket, 'data');
  socket.end();
  return String(response).trim();
};
// Keys are paced: a burst faster than the IDE reads its events overflows the compositor's 4096-byte
// buffer for the client, and libwayland then disconnects the IDE (measured: 'Data too big for buffer').
const key = async ({ socketPath, name }: { socketPath: string; name: string }): Promise<void> => {
  const response = await control({ socketPath, line: 'key ' + name });
  if (!response.startsWith('ok')) throw new Error('Key ' + name + ' was refused: ' + response);
  await wait(8);
};
const live = mkdtempSync(join(liveRoot, 'ide-watch-limit-'));
const livePath = join(live, 'ide.log');
const lines = (): string[] => plain(readFileSync(livePath, 'utf8')).split('\n').filter(Boolean);
const own = (): string[] => lines().filter(line => line.includes('ide_app::') || line.includes('monochromatic_ide::'));
const stampOf = (line: string): number => Date.parse(line.slice(0, 23) + 'Z');
const socketPath = join(artifact, 'control.sock');
const log = openSync(livePath, 'w');
// The namespace's root writes the namespace's own limit, then becomes the IDE; the host limit is not touched.
const inner = 'echo ' + limit + ' > /proc/sys/user/max_inotify_watches && exec "$0" "$@"';
const host = spawn(compositor, ['--socket', socketPath, '--size', '1100x660', '--color-scheme', 'dark', '--',
  'unshare', '--user', '--map-root-user', 'sh', '-c', inner, binary, project, '--file', displayed], { env: environment, stdio: ['ignore', log, log] });
const exited = once(host, 'exit');
// The numbers the control collects; each is set once its measurement has run, and the verdict reads them at the end.
interface WatchLimitResults {
  folders: number;
  limit: number;
  seconds: number;
  observed_seconds?: number;
  limit_warnings?: number;
  directory_failure_warnings?: number;
  warn_lines?: string[];
  watching_lines?: number;
  sweeps_in_window?: number;
  watch_calls_in_window?: number;
  failed_watch_calls_in_window?: number;
  created_file_listed_after_ms?: number | null;
  scroll_retry_lines?: number;
  watch_calls_before_scroll?: number;
  first_watch_call_after_scroll_ms?: number | null;
  raise_status?: number | null;
  raise_error?: string;
  available_again_after_ms?: number | null;
  available_again_lines?: number;
}
const results: WatchLimitResults = { folders, limit, seconds };
const save = () => writeFileSync(join(artifact, 'results.json'), JSON.stringify(results, null, 2));
try {
  // The IDE is the child whose program is the IDE binary after `sh` replaced itself.
  const deadline = performance.now() + 120000;
  let pid: string | undefined;
  while (!pid) {
    if (host.exitCode !== null) throw new Error('The compositor exited early; inspect ' + livePath);
    if (performance.now() > deadline) throw new Error('The IDE did not start in 120 s');
    const children = spawnSync('pgrep', ['--parent', String(host.pid)], { encoding: 'utf8' }).stdout.trim().split('\n').filter(Boolean);
    pid = children.find(child => {
      try { return readFileSync('/proc/' + child + '/cmdline', 'utf8').split('\0')[0] === binary; } catch (error) {
        if (!(error instanceof Error && 'code' in error && (error.code === 'ENOENT' || error.code === 'ESRCH'))) throw error;
        return false;
      }
    });
    await wait(200);
  }
  while (!own().some(line => line.includes('started inotify file-change watching'))) {
    if (performance.now() > deadline) throw new Error('The IDE did not start watching in 120 s');
    await wait(200);
  }
  await wait(1500);
  // Expand every folder with real keys: Tab to the tree, Home, then Right and Down for each row.
  const listed = () => new Set(own().filter(line => line.includes('applied tree directory snapshot') && line.includes('folder-')).map(line => line.match(/path=(\S+)/)?.[1])).size;
  for (let attempt = 0; attempt < 6 && listed() < folders; attempt++) {
    for (const name of ['tab', 'home']) await key({ socketPath, name });
    for (let step = 0; step < 2 * (9 * folders + 2) && listed() < folders; step++) {
      await key({ socketPath, name: 'right' });
      await key({ socketPath, name: 'down' });
    }
    await wait(500);
  }
  if (listed() < folders) throw new Error('Only ' + listed() + ' of ' + folders + ' folders were expanded; inspect ' + livePath);
  await wait(2000);
  const tracePath = join(artifact, 'strace.txt');
  // Every thread of the IDE, so the watch thread's calls are seen; only the watch calls are recorded.
  const tracer = spawn('strace', ['--follow-forks', '--trace=inotify_add_watch', '--output=' + tracePath, '--attach=' + pid], { stdio: 'ignore' });
  const traced = once(tracer, 'exit');
  const observedFrom = Date.now();
  const startLines = own().length;
  // A file created in the last folder, which sorts last and so waits on the limit.
  await wait(3000);
  // The IDE logs canonical paths and the scratch root can be a symlink (the agent scratch lives on another drive),
  // so the log is searched by the canonical folder path.
  const unwatched = join(realpathSync(project), 'folder-' + String(folders - 1).padStart(3, '0'));
  const createdAt = Date.now();
  writeFileSync(join(unwatched, 'created-under-limit.txt'), 'seen through the sweep\n');
  await wait(seconds * 1000 - 3000);
  tracer.kill('SIGINT');
  await traced;
  const window = own().slice(startLines);
  const all = own();
  const listing = all.find(line => line.includes('read directory snapshot') && line.includes(unwatched) && line.includes('entries=9') && stampOf(line) >= createdAt);
  const trace = readFileSync(tracePath, 'utf8').split('\n').filter(line => line.includes('inotify_add_watch('));
  results.observed_seconds = Math.round((Date.now() - observedFrom) / 1000);
  results.limit_warnings = all.filter(line => line.includes(' WARN ') && line.includes('fs.inotify.max_user_watches')).length;
  results.directory_failure_warnings = all.filter(line => line.includes('directory watch failed')).length;
  results.warn_lines = all.filter(line => line.includes(' WARN ')).map(line => line.slice(27, 220));
  results.watching_lines = all.filter(line => line.includes('watching directory')).length;
  results.sweeps_in_window = window.filter(line => line.includes('started the safety reread')).length;
  results.watch_calls_in_window = trace.length;
  results.failed_watch_calls_in_window = trace.filter(line => line.includes('ENOSPC')).length;
  results.created_file_listed_after_ms = listing ? stampOf(listing) - createdAt : null;
  save();
  // Scrolling retries the watches at once: by now the backoff waits at least 16 s, so a call within a second
  // of the key comes from the scroll. The expansion left the selection on the last row, so Home scrolls.
  const scrollTrace = join(artifact, 'strace-scroll.txt');
  const scrollTracer = spawn('strace', ['--follow-forks', '--timestamps=format:unix,precision:ms', '--trace=inotify_add_watch', '--output=' + scrollTrace, '--attach=' + pid], { stdio: 'ignore' });
  const scrollTraced = once(scrollTracer, 'exit');
  await wait(1500);
  const scrolledAt = Date.now();
  await key({ socketPath, name: 'home' });
  await wait(1500);
  scrollTracer.kill('SIGINT');
  await scrollTraced;
  const scrollCalls = readFileSync(scrollTrace, 'utf8').split('\n').filter(line => line.includes('inotify_add_watch(')).map(line => Math.round(Number(line.match(/^(?:\d+\s+)?(\d+\.\d+)/)?.[1]) * 1000));
  results.scroll_retry_lines = own().filter(line => line.includes('the tree scrolled while shown folders lack a watch') && stampOf(line) >= scrolledAt).length;
  results.watch_calls_before_scroll = scrollCalls.filter(at => at < scrolledAt).length;
  results.first_watch_call_after_scroll_ms = scrollCalls.filter(at => at >= scrolledAt).map(at => at - scrolledAt)[0] ?? null;
  save();
  // Raise the namespace's limit from inside the namespace, then wait for a retry to watch everything.
  const raised = spawnSync('nsenter', ['--target', String(pid), '--user', '--preserve-credentials', 'sh', '-c', 'echo 1000 > /proc/sys/user/max_inotify_watches'], { encoding: 'utf8' });
  results.raise_status = raised.status;
  results.raise_error = (raised.stderr ?? '').trim();
  const raisedAt = Date.now();
  // The longest backoff step after one minute under the limit is 32 s; an expansion change would retry at once.
  while (Date.now() - raisedAt < 70000 && !own().some(line => line.includes('inotify watches are available again'))) await wait(500);
  const back = own().find(line => line.includes('inotify watches are available again'));
  results.available_again_after_ms = back ? stampOf(back) - raisedAt : null;
  results.available_again_lines = own().filter(line => line.includes('inotify watches are available again')).length;
  save();
} finally {
  if (host.exitCode === null) {
    await control({ socketPath, line: 'quit' }).catch(() => host.kill('SIGTERM'));
    await Promise.race([exited, wait(10000).then(() => host.kill('SIGKILL'))]);
  }
  closeSync(log);
  copyFileSync(livePath, join(artifact, 'ide.log'));
  rmSync(live, { recursive: true, force: true });
}
console.log(JSON.stringify(results, null, 2));
const failures: string[] = [];
if (results.limit_warnings !== 1) failures.push('expected one limit warning, saw ' + results.limit_warnings);
if (results.directory_failure_warnings !== 0) failures.push('per-directory failure warnings: ' + results.directory_failure_warnings);
if (results.created_file_listed_after_ms === null || (results.created_file_listed_after_ms ?? 0) > 2500) failures.push('the file in an unwatched folder was not listed within 2.5 s');
if ((results.failed_watch_calls_in_window ?? 0) > 10) failures.push('too many failing watch calls: ' + results.failed_watch_calls_in_window);
if ((results.scroll_retry_lines ?? 0) < 1 || results.first_watch_call_after_scroll_ms === null || (results.first_watch_call_after_scroll_ms ?? 0) > 1000) failures.push('scrolling did not retry the watches within a second');
if (results.watch_calls_before_scroll !== 0) failures.push('watch calls in the 1.5 s before the scroll: ' + results.watch_calls_before_scroll);
if (results.available_again_lines !== 1) failures.push('expected one line when watches became available, saw ' + results.available_again_lines);
if (failures.length) throw new Error('Watch-limit control failed: ' + failures.join('; ') + '; inspect ' + artifact);
console.log('Watch-limit control passed: ' + artifact);
