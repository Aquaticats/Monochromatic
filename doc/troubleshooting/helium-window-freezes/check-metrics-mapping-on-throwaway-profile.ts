// Starts Helium on a throwaway profile in its own systemd user unit and reports whether
// the browser main process maps a disk-backed `BrowserMetrics/*.pma` file.
//
// Usage: node check-metrics-mapping-on-throwaway-profile.ts <scratch-dir> <arm> [restart]
//
// Arms:
// - `control`: plain `/usr/bin/helium-browser`; the mapping is expected (positive control).
// - `launcher`: through the installed `~/.local/lib/helium/launcher.ts`.
// - `desktop`: the first `Exec=` line of `~/.local/share/applications/helium.desktop`.
// - `sentinel`: plain Helium on a profile whose `BrowserMetrics` folder was made
//   read-only, with a file inside, before the start.
// - `prepared-mid-session`: plain Helium; once its spare file exists, the folder is made
//   read-only with a file inside while it runs, then it relaunches (implies `restart`).
// `restart` additionally opens `chrome://restart` over DevTools and checks the relaunched process.
// The log ends with any Helium core dumps recorded since the run started.
//
// Probe pitfalls this script handles:
// - Chromium moves its browser main process out of the launching unit into its own
//   `app-org.chromium.Chromium-<pid>.scope`, so the unit's cgroup never lists it,
//   and stopping the unit does not stop the browser.
// - Chromium rewrites its process title into one space-joined string in `/proc/<pid>/cmdline`.
// - Helium deletes the `BrowserMetrics` folder and the active `.pma` file soon after startup;
//   the mapping stays live as `(deleted)`, so only `/proc/<pid>/maps` shows it.
import { execFileSync } from 'node:child_process';
import {
  appendFileSync,
  chmodSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

//region Arguments and paths
const [scratchArgument, arm, restartArgument] = process.argv.slice(2);
if (!scratchArgument || !arm) {
  throw new Error('Usage: node check-metrics-mapping-on-throwaway-profile.ts <scratch-dir> <arm> [restart]');
}
const scratch = resolve(scratchArgument);
const wantRestart = restartArgument === 'restart';
const profile = join(scratch, `profile-${arm}`);
const unit = `helium-metrics-check-${arm}`;
const heliumBinary = '/opt/helium-browser-bin/helium';
mkdirSync(scratch, { recursive: true });
const logFile = join(scratch, `${arm}${wantRestart ? '-restart' : ''}.log`);
//endregion

function log(line: string) {
  const stamped = `${new Date().toISOString()} ${line}`;
  console.log(stamped);
  appendFileSync(logFile, `${stamped}\n`);
}

//region Command for each arm
const browserArguments = [
  `--user-data-dir=${profile}`,
  '--ozone-platform=headless',
  '--remote-debugging-port=0',
  '--no-first-run',
  '--no-default-browser-check',
  '--password-store=basic',
  'about:blank',
];

function desktopMainExec(): string[] {
  const text = readFileSync(join(homedir(), '.local', 'share', 'applications', 'helium.desktop'), 'utf8');
  const execLine = text.split('\n').find(line => line.startsWith('Exec='));
  if (!execLine) {
    throw new Error('The local helium.desktop has no Exec= line.');
  }
  return execLine.slice('Exec='.length).split(' ').filter(token => token !== '%U');
}

// Same steps as the launcher and as the manual preparation of a running profile.
function prepareReadOnlyMetricsFolder() {
  const metrics = join(profile, 'BrowserMetrics');
  mkdirSync(metrics, { recursive: true, mode: 0o700 });
  chmodSync(metrics, 0o700);
  writeFileSync(join(metrics, 'read-only-on-purpose.txt'), 'sentinel\n');
  chmodSync(metrics, 0o500);
}

if (arm === 'sentinel') {
  prepareReadOnlyMetricsFolder();
}

const commandsByArm = new Map<string, () => string[]>([
  ['control', () => ['/usr/bin/helium-browser']],
  ['sentinel', () => ['/usr/bin/helium-browser']],
  ['prepared-mid-session', () => ['/usr/bin/helium-browser']],
  ['launcher', () => ['/usr/bin/node', join(homedir(), '.local', 'lib', 'helium', 'launcher.ts')]],
  ['desktop', desktopMainExec],
]);
const commandForArm = commandsByArm.get(arm);
if (!commandForArm) {
  throw new Error(`Unknown arm ${arm}; expected one of ${[...commandsByArm.keys()].join(', ')}.`);
}
const command = commandForArm();
//endregion

//region Process discovery and reporting
function isExpectedProcfsError(error: unknown): boolean {
  const code = (error as NodeJS.ErrnoException).code;
  return code === 'ENOENT' || code === 'EACCES' || code === 'ESRCH';
}

function browserPids(): number[] {
  const pids = readdirSync('/proc').filter(entry => /^\d+$/.test(entry)).map(Number);
  return pids.filter(pid => {
    try {
      if (readlinkSync(`/proc/${pid}/exe`) !== heliumBinary) {
        return false;
      }
      const argv = readFileSync(`/proc/${pid}/cmdline`, 'utf8').split(/[\0 ]/);
      return argv.includes(`--user-data-dir=${profile}`) && !argv.some(argument => argument.startsWith('--type='));
    } catch (error) {
      // Other users' processes and processes that exit mid-scan are not this arm's browser.
      if (isExpectedProcfsError(error)) {
        return false;
      }
      throw error;
    }
  });
}

function scopeOf(pid: number): string | undefined {
  return readFileSync(`/proc/${pid}/cgroup`, 'utf8').match(/(app-org\.chromium\.Chromium-\d+\.scope)/)?.[1];
}

function report(label: string) {
  const metricsDir = join(profile, 'BrowserMetrics');
  const dirState = existsSync(metricsDir)
    ? `mode ${(statSync(metricsDir).mode & 0o777).toString(8)} entries [${readdirSync(metricsDir).join(', ')}]`
    : 'absent';
  log(`${label}: BrowserMetrics dir ${dirState}`);
  const pids = browserPids();
  log(`${label}: browser main pids ${JSON.stringify(pids)}`);
  for (const pid of pids) {
    log(`${label}: pid ${pid} scope ${scopeOf(pid) ?? 'none'}`);
    const maps = readFileSync(`/proc/${pid}/maps`, 'utf8').split('\n').filter(line => /BrowserMetrics|\.pma/.test(line));
    log(`${label}: pid ${pid} metrics mappings ${maps.length}${maps.length ? `\n  ${maps.join('\n  ')}` : ''}`);
  }
}
//endregion

//region DevTools helpers
async function waitForDevToolsPort(): Promise<string | undefined> {
  const portFile = join(profile, 'DevToolsActivePort');
  for (let attempt = 0; attempt < 40; attempt++) {
    if (existsSync(portFile)) {
      return readFileSync(portFile, 'utf8').split('\n')[0];
    }
    await sleep(500);
  }
  return undefined;
}

async function requestRestart(port: string) {
  const response = await fetch(`http://127.0.0.1:${port}/json/version`);
  const version = await response.json() as { webSocketDebuggerUrl: string };
  const socket = new WebSocket(version.webSocketDebuggerUrl);
  await new Promise<void>((resolveOpen, rejectOpen) => {
    socket.addEventListener('open', () => resolveOpen(), { once: true });
    socket.addEventListener('error', () => rejectOpen(new Error('The DevTools socket failed to open.')), { once: true });
  });
  socket.addEventListener('message', event => log(`DevTools reply ${String(event.data).slice(0, 300)}`));
  socket.send(JSON.stringify({ id: 1, method: 'Target.createTarget', params: { url: 'chrome://restart' } }));
  await sleep(1000);
  socket.close();
}
//endregion

//region Run, observe, stop
const runStart = new Date().toISOString().replace('T', ' ').replace(/\.\d+Z$/, ' UTC');
log(`arm ${arm}: ${[...command, ...browserArguments].join(' ')}`);
execFileSync('systemd-run', [
  '--user', `--unit=${unit}`, '--collect',
  '--property=ExitType=cgroup', '--property=MemoryMax=4G', '--property=CPUQuota=400%',
  '--property=TasksMax=1024', '--property=IOWeight=1',
  ...command, ...browserArguments,
], { stdio: 'inherit' });

try {
  const port = await waitForDevToolsPort();
  log(`DevToolsActivePort ${port ?? 'never appeared within 20 s'}`);
  await sleep(3000);
  report('after initialization');
  await sleep(30000);
  report('30 s later');
  if (arm === 'prepared-mid-session') {
    // Mirror a profile prepared while Helium runs: wait for the spare file Chromium
    // writes on a delayed worker task, then make the folder read-only with a file inside.
    const spare = join(profile, 'BrowserMetrics-spare.pma');
    for (let attempt = 0; attempt < 360 && !existsSync(spare); attempt++) {
      await sleep(500);
    }
    log(`spare file ${existsSync(spare) ? `present, ${statSync(spare).size} bytes` : 'absent after 180 s'}`);
    prepareReadOnlyMetricsFolder();
    report('after preparing the folder mid-session');
  }
  if ((wantRestart || arm === 'prepared-mid-session') && port) {
    const before = browserPids();
    await requestRestart(port);
    for (let attempt = 0; attempt < 60; attempt++) {
      const now = browserPids();
      if (now.length && !now.some(pid => before.includes(pid))) {
        break;
      }
      await sleep(500);
    }
    await sleep(8000);
    report('after chrome://restart');
    await sleep(20000);
    report('20 s after restart');
  }
} finally {
  const scopes = browserPids().map(scopeOf).filter(scope => scope !== undefined);
  execFileSync('systemctl', ['--user', 'stop', unit, ...scopes], { stdio: 'inherit' });
  log(`stopped ${unit} ${scopes.join(' ')}`);
  await sleep(2000);
  log(`browser main pids left for this profile: ${JSON.stringify(browserPids())}`);
  try {
    const dumps = execFileSync('coredumpctl', ['list', '--no-pager', `--since=${runStart}`, heliumBinary], { encoding: 'utf8' });
    log(`Helium core dumps since ${runStart}:\n${dumps}`);
  } catch (error) {
    // coredumpctl exits non-zero when it finds no matching dump.
    log(`coredumpctl found no Helium core dump since ${runStart}: ${String((error as { stderr?: string }).stderr ?? error).trim()}`);
  }
}
//endregion
