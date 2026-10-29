#!/usr/bin/env node
// Check at the user boundary that a reader that stops reading the IDE's log does not freeze the window,
// and observe the defect it replaced as the guard control.
//
// Two builds come from one disposable package copy: the sources as they are, and the same sources with the
// log written synchronously from the logging thread (the writer swapped for plain standard output, where the
// log goes). Each runs in the repository's nested compositor with every record of the application on
// (`RUST_LOG=ide_app=debug,monochromatic_ide=debug`, the fixed level before 2026-10-06); a bare `debug` would
// add every library's records, about twelve times the startup log, without changing what is measured.
// The IDE's standard output goes to a FIFO that this script opens and never reads, so once the kernel's
// pipe buffer is full every write to it blocks. While keys move the caret, the script photographs the
// compositor's screen over and over (`screenshot`, which composites the latest frame the IDE committed, on the GPU)
// and records when the picture changes. A caret that moves, or blinks, redraws the window, and the redraw happens
// on the window's event loop, so a blocked event loop leaves the picture still. The IDE also stamps each key it
// handles in its own log, and the longest pause between two of those stamps is the second account of the same
// stall. After the measurement the FIFO is drained, so the log's gap warning, if any, can be read, and the IDE is quit.
import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { createHash } from 'node:crypto';
import { closeSync, constants, copyFileSync, cpSync, existsSync, mkdirSync, mkdtempSync, openSync, readFileSync, readSync, realpathSync, rmSync, statfsSync, statSync, writeFileSync } from 'node:fs';
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
const disposable = ({ value, what }: { value: string; what: string }): string => {
  const resolved = realpathSync(value);
  if (!resolved.startsWith(realpathSync(privateRoot) + sep) || !statSync(resolved).isDirectory()) throw new Error(what + ' must be a disposable directory below ' + privateRoot);
  return resolved;
};
if (!process.env.usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = disposable({ value: process.env.usage_cache, what: 'Target cache' });
const cargoHome = process.env.usage_cargo ? disposable({ value: process.env.usage_cargo, what: 'Cargo home copy' }) : 'ide-cargo';
const seconds = Number(process.env.usage_seconds ?? 30);
if (!Number.isInteger(seconds) || seconds < 5) throw new Error('seconds must be a whole number of at least 5');
const origin = process.cwd();
// Memory-backed storage for the live session files (see `session`).
const TMPFS_MAGIC = 0x01021994;
const liveRoot = process.env.XDG_RUNTIME_DIR;
if (!liveRoot || (statSync(liveRoot).mode & 0o077) !== 0 || statfsSync(liveRoot).type !== TMPFS_MAGIC) throw new Error('XDG_RUNTIME_DIR must name a private memory-backed directory for the live session files');
demandCurrentCompositor();
const artifact = mkdtempSync(join(privateRoot, 'ide-log-stall-'));
// build.rs embeds target/crate-licenses.json, which the notices task writes; the disposable target directory needs its copy.
if (!existsSync(join(cache, 'crate-licenses.json'))) copyFileSync(join(origin, 'target/crate-licenses.json'), join(cache, 'crate-licenses.json'));
const source = join(artifact, 'package');
cpSync(origin, source, { recursive: true, filter: path => !['target', '.git', 'node_modules'].includes(relative(origin, path).split(sep)[0] ?? '') });
mkdirSync(join(source, 'target'), { recursive: true });
console.log('LOG_STALL_ARTIFACT=' + artifact + ' load=' + readFileSync('/proc/loadavg', 'utf8').trim());

// region builds
const build = (label: string): string => {
  const result = spawnSync('podman', [
    'run', '--rm', '--network=none', ...containerLimits.limitArguments,
    '--security-opt', 'label=disable',
    '--volume', source + ':/work', '--volume', cache + ':/work/target', '--volume', cargoHome + ':/cargo',
    '--workdir', '/work', ...containerLimits.environmentArguments, '--env', 'SLINT_EMIT_DEBUG_INFO=1',
    'localhost/monochromatic/ide', 'cargo', 'build', '--offline', '--bin', 'monochromatic-ide',
  ], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  writeFileSync(join(artifact, 'build-' + label + '.log'), (result.stdout ?? '') + (result.stderr ?? ''));
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error('Build ' + label + ' failed; inspect ' + artifact);
  const binary = join(artifact, 'ide-' + label);
  copyFileSync(join(cache, 'debug', 'monochromatic-ide'), binary);
  return binary;
};
const replaceOne = ({ text, before, after }: { text: string; before: string; after: string }): string => {
  const start = text.indexOf(before);
  if (start < 0) throw new Error('Mutation source anchor is absent: ' + before);
  if (text.indexOf(before, start + before.length) >= 0) throw new Error('Mutation source anchor is ambiguous: ' + before);
  return text.slice(0, start) + after + text.slice(start + before.length);
};
const nativePath = join(source, 'src/native/start.rs');
const native = readFileSync(nativePath, 'utf8');
// One build of the IDE and what the session measured about it; the session fills the measured fields in.
interface Variant {
  name: string;
  binary: string;
  windowShown?: boolean;
  lastStartupError?: string;
  windowAfterMs?: number | null;
  exit?: { code: number | null; signal: NodeJS.Signals | null } | { killed: boolean };
  exitAfterQuitMs?: number;
  leftovers?: string[];
  drainedBytes?: number;
  forcedByCompositor?: boolean;
  keysHandled?: number;
  measured?: { from: string; to: string } | null;
  longestInputPauseMs?: number | null;
  probes?: [number, string][];
  gapWarnings?: string[];
  keys?: number;
  photographs?: number;
  pictureChanges?: number;
  longestStillMs?: number | null;
}
const variants: Variant[] = [{ name: 'background-writer', binary: build('background-writer') }];
try {
  writeFileSync(nativePath, replaceOne({ text: native, before: 'logging::install(logging::filter(""), log_writer, None)?;', after: 'drop(log_writer);\n    logging::install(logging::filter(""), std::io::stdout, None)?;' }));
  variants.push({ name: 'synchronous-writer', binary: build('synchronous-writer') });
} finally { writeFileSync(nativePath, native); }
// endregion

// region session
const project = join(artifact, 'project');
mkdirSync(project);
const file = join(project, 'view.txt');
writeFileSync(file, Array.from({ length: 400 }, (_, index) => 'line ' + index + ' of a file whose caret moves while the log is not read').join('\n') + '\n');
const fontConfig = join(artifact, 'fonts.conf');
writeFileSync(fontConfig, '<fontconfig><dir>/usr/share/fonts</dir><cachedir prefix="xdg">fontconfig</cachedir></fontconfig>');
for (const name of ['config', 'cache', 'data']) mkdirSync(join(artifact, name));
const runtime = [join(origin, 'target/debug/runtime'), join(cache, 'debug/runtime')].find(existsSync);
// The private font cache is built once before the first session; otherwise the first window waits for it
// (38 s once at a load average of 90) and the two variants would start under different conditions.
const fontWarmup = spawnSync('fc-list', [], { env: { ...process.env, FONTCONFIG_FILE: fontConfig, XDG_CACHE_HOME: join(artifact, 'cache') }, stdio: ['ignore', 'ignore', 'pipe'], encoding: 'utf8' });
if (fontWarmup.status !== 0) throw new Error('fc-list failed while building the private font cache: ' + fontWarmup.stderr);
// One control command with a bound: the compositor answers whatever the hosted IDE does, so an answer that does not
// come in 10 s is a failure of the compositor, not a stall of the IDE.
const control = async ({ socketPath, line }: { socketPath: string; line: string }): Promise<string> => {
  const socket = createConnection(socketPath);
  const bound = { signal: AbortSignal.timeout(10000) };
  try {
    await once(socket, 'connect', bound);
    socket.write(line + '\n');
    const [response] = await once(socket, 'data', bound);
    return String(response).trim();
  } finally {
    socket.destroy();
  }
};
// The picture the compositor shows now, as a hash of the PNG it composited. Every call writes the same file.
const pictureHash = async ({ socketPath, path }: { socketPath: string; path: string }): Promise<string> => {
  const response = await control({ socketPath, line: 'screenshot ' + path });
  if (!response.startsWith('ok')) throw new Error('screenshot was refused: ' + response);
  return createHash('sha256').update(readFileSync(path)).digest('hex');
};

// The longest distance between two neighbouring edges, or null when there are fewer than two.
const longestGap = (edges: number[]): number | null => (edges.length > 1 ? Math.max(...edges.slice(1).map((at, index) => at - (edges[index] ?? at))) : null);

const session = async (variant: Variant): Promise<Variant> => {
  const directory = join(artifact, variant.name);
  mkdirSync(directory);
  // Live files go to memory-backed storage: an append to a busy disk can block for many seconds, which
  // would stall the compositor's own logging and blur what is measured. They are copied to the artifact after.
  const live = mkdtempSync(join(liveRoot, 'ide-log-stall-' + variant.name + '-'));
  const fifo = join(live, 'stdout.fifo');
  if (spawnSync('mkfifo', ['--mode=600', fifo]).status !== 0) throw new Error('mkfifo failed for ' + fifo);
  // Opened for reading and writing so the open does not wait for a writer; nothing is read until the drain.
  const unread = openSync(fifo, 'r+');
  const socketPath = join(live, 'control.sock');
  const compositorLogPath = join(live, 'compositor.log');
  const compositorLog = openSync(compositorLogPath, 'w');
  const environment: NodeJS.ProcessEnv = { ...process.env, SLINT_BACKEND: 'winit', FONTCONFIG_FILE: fontConfig, XDG_CONFIG_HOME: join(artifact, 'config'), XDG_CACHE_HOME: join(artifact, 'cache'), XDG_DATA_HOME: join(artifact, 'data'), IDE_STALL_FIFO: fifo, IDE_STALL_RUST_LOG: 'ide_app=debug,monochromatic_ide=debug' };
  if (runtime) environment.HELIX_RUNTIME = runtime;
  delete environment.RUST_LOG;
  // Only the IDE's standard output goes to the FIFO, and only the IDE gets RUST_LOG; the compositor logs to a file.
  // The compositor hands its own standard streams to the program it hosts, so the redirection happens in a shell
  // that then becomes the IDE (`exec`); redirecting the compositor itself would stall the compositor too.
  const launcher = ['/bin/sh', '-c', 'RUST_LOG="$IDE_STALL_RUST_LOG" exec "$0" "$@" >"$IDE_STALL_FIFO"', variant.binary, project, '--file', file];
  const host = spawn(compositor, ['--socket', socketPath, '--size', '1100x660', '--color-scheme', 'dark', '--', ...launcher], { env: environment, stdio: ['ignore', compositorLog, compositorLog] });
  const exited = new Promise<{ code: number | null; signal: NodeJS.Signals | null }>(done => host.on('exit', (code, signal) => done({ code, signal })));
  // [milliseconds into the measurement, picture hash] for each photograph, and the times the picture changed.
  const probes: [number, string][] = [];
  const changes: number[] = [];
  const picturePath = join(live, 'picture.png');
  let keys = 0;
  let measuredFrom: number | undefined;
  let measuredTo: number | undefined;
  const started = performance.now();
  try {
    // The compositor lists the IDE's window once the IDE has committed its first frame. A loop that is blocked
    // before that never maps a window, which counts as a window that never showed, not as a setup failure.
    const ready = performance.now() + 120000;
    variant.windowShown = false;
    while (performance.now() < ready) {
      if (host.exitCode !== null) throw new Error(variant.name + ': the compositor exited early; inspect ' + directory);
      try {
        const windows = await control({ socketPath, line: 'windows' });
        if (windows.startsWith('ok') && !/^ok 0\b/.test(windows)) {
          variant.windowShown = true;
          break;
        }
      } catch (error) {
        // The control socket does not exist yet, or the compositor is not answering; the loop's own deadline decides.
        variant.lastStartupError = (error instanceof Error ? error.message : String(error)).slice(0, 120);
      }
      await wait(200);
    }
    variant.windowAfterMs = variant.windowShown ? Math.round(performance.now() - started) : null;
    // Keys come at a steady pace whether or not the IDE answers; each caret move logs debug records.
    const end = performance.now() + seconds * 1000;
    measuredFrom = Date.now();
    let down = true;
    const typing = (async () => {
      while (variant.windowShown && performance.now() < end) {
        const response = await control({ socketPath, line: 'key ' + (down ? 'down' : 'up') });
        if (response.startsWith('ok')) keys += 1;
        if (keys % 40 === 0) down = !down;
        await wait(50);
      }
    })();
    // The first photograph opens the measurement, so a picture that never changes shows as no change at all.
    let lastHash = variant.windowShown ? await pictureHash({ socketPath, path: picturePath }) : undefined;
    if (lastHash !== undefined) probes.push([0, lastHash.slice(0, 12)]);
    while (variant.windowShown && performance.now() < end) {
      await wait(200);
      const at = Date.now() - measuredFrom;
      const hash = await pictureHash({ socketPath, path: picturePath });
      probes.push([at, hash.slice(0, 12)]);
      if (hash !== lastHash) changes.push(at);
      lastHash = hash;
    }
    await typing;
    measuredTo = Date.now();
  } finally {
    // Drain the FIFO so the IDE can write again, then quit it; the drained text holds any gap warning.
    // A non-blocking descriptor is read in a timer: a blocking read of a FIFO that this script also holds open
    // for writing would never return and keep the script alive after its work.
    const drained: Buffer[] = [];
    const drainFd = openSync(fifo, constants.O_RDONLY | constants.O_NONBLOCK);
    const drainOnce = () => {
      const buffer = Buffer.alloc(65536);
      for (;;) {
        let count: number;
        try { count = readSync(drainFd, buffer, 0, buffer.length, null); } catch (error) { if (error instanceof Error && 'code' in error && error.code === 'EAGAIN') return; throw error; }
        if (count === 0) return;
        drained.push(Buffer.from(buffer.subarray(0, count)));
      }
    };
    const draining = setInterval(drainOnce, 20);
    await wait(500);
    const quitAt = performance.now();
    await control({ socketPath, line: 'quit' }).catch(() => host.kill('SIGTERM'));
    // The compositor gets time to stop its private bus; a forced kill leaves that bus behind.
    const outcome = await Promise.race([exited, wait(30000).then(() => undefined)]);
    if (outcome === undefined) {
      host.kill('SIGTERM');
      variant.exit = await Promise.race([exited, wait(10000).then(() => { host.kill('SIGKILL'); return { killed: true }; })]);
    } else variant.exit = outcome;
    variant.exitAfterQuitMs = Math.round(performance.now() - quitAt);
    clearInterval(draining);
    drainOnce();
    closeSync(drainFd);
    closeSync(unread);
    closeSync(compositorLog);
    // Processes the compositor started for this session (its private bus) must be gone with it.
    variant.leftovers = spawnSync('pgrep', ['--full', 'monochromatic-nested-wayland-session-' + host.pid + '-'], { encoding: 'utf8' }).stdout.trim().split('\n').filter(Boolean);
    for (const pid of variant.leftovers) spawnSync('kill', [pid]);
    const text = Buffer.concat(drained).toString().replaceAll(/\u001b\[[0-9;]*m/g, '');
    writeFileSync(join(directory, 'ide-log-drained.log'), text);
    copyFileSync(compositorLogPath, join(directory, 'compositor.log'));
    rmSync(live, { recursive: true, force: true });
    variant.drainedBytes = Buffer.concat(drained).length;
    variant.forcedByCompositor = readFileSync(join(directory, 'compositor.log'), 'utf8').includes('forcing shutdown');
    // The IDE stamps each key it handles on the window's thread; the longest pause between two of them inside
    // the measured interval is the window's own account of how long it could not handle input.
    const handled = text.split('\n').filter(line => line.includes('monochromatic_ide::native::input:')).map(line => Date.parse(line.slice(0, 27))).filter(at => at >= (measuredFrom ?? Infinity) && at <= (measuredTo ?? -Infinity));
    variant.keysHandled = handled.length;
    variant.measured = measuredFrom === undefined || measuredTo === undefined ? null : { from: new Date(measuredFrom).toISOString(), to: new Date(measuredTo).toISOString() };
    // The interval's start and end count as edges, so a window that stops handling keys for good shows that pause too.
    const edges = measuredFrom === undefined || measuredTo === undefined ? [] : [measuredFrom, ...handled, measuredTo];
    variant.longestInputPauseMs = longestGap(edges);
    variant.probes = probes;
    variant.gapWarnings = text.split('\n').filter(line => line.includes('were not written because the log output did not accept them')).map(line => line.slice(0, 220));
  }
  // The longest time the picture stood still, counted from the start of the measurement and to its end, so a picture
  // that stops changing for good shows that stillness too. The resolution is one photograph, about 200 ms.
  const stillEdges = measuredFrom === undefined || measuredTo === undefined ? [] : [0, ...changes, measuredTo - measuredFrom];
  Object.assign(variant, {
    keys,
    photographs: probes.length,
    pictureChanges: changes.length,
    longestStillMs: longestGap(stillEdges),
  });
  return variant;
};
// endregion

const results: Variant[] = [];
for (const variant of variants) {
  results.push(await session(variant));
  writeFileSync(join(artifact, 'results.json'), JSON.stringify(results, null, 2));
  console.log(JSON.stringify(results.at(-1)));
}
const [current, synchronous] = results;
if (current === undefined || synchronous === undefined) throw new Error('The check measures two builds; a build is missing from the results');
// Two accounts of the window's thread: the longest pause between two keys it handled (its own records,
// stamped when handled), and the longest time the compositor's picture of the window stood still while the caret
// moved. A pause or a still picture of 5 s counts as a stall. A busy host can delay single redraws by seconds,
// so a stall needs 5 s, and slower answers are recorded in results.json but do not decide the check.
const describe = (item: Variant): string => item.keysHandled + ' keys handled, longest pause ' + item.longestInputPauseMs + ' ms; window shown: ' + item.windowShown;
const describePicture = (item: Variant): string => !item.windowShown ? 'no window was mapped in 120 s, so no photograph was taken' : item.pictureChanges + ' picture changes in ' + item.photographs + ' photographs, longest still ' + item.longestStillMs + ' ms';
const stalled = (item: Variant): boolean => !item.windowShown || item.keysHandled === 0 || (item.longestInputPauseMs ?? 0) >= 5000;
const stillPicture = (item: Variant): boolean => !item.windowShown || item.longestStillMs === null || (item.longestStillMs ?? 0) >= 5000;
const checks: [string, boolean][] = [
  ['with the background writer, the window kept handling input (' + describe(current) + ')', !stalled(current)],
  ['with the background writer, the picture kept changing (' + describePicture(current) + ')', !stillPicture(current)],
  ['with the synchronous writer, the window stopped handling input (' + describe(synchronous) + ')', stalled(synchronous)],
  ['with the synchronous writer, the picture stood still (' + describePicture(synchronous) + ')', stillPicture(synchronous)],
  ['no process the compositor started outlived it', results.every(item => item.leftovers?.length === 0)],
];
for (const [check, passed] of checks) console.log((passed ? 'PASS ' : 'FAIL ') + check);
writeFileSync(join(artifact, 'results.json'), JSON.stringify({ load: readFileSync('/proc/loadavg', 'utf8').trim(), results, checks }, null, 2));
console.log('Log stall check ' + (checks.every(([, passed]) => passed) ? 'passed' : 'FAILED') + ': ' + join(artifact, 'results.json'));
if (!checks.every(([, passed]) => passed)) process.exitCode = 1;
