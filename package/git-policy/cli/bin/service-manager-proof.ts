#!/usr/bin/env node
/**
 What:
  The proof that a Git command never makes the user service manager of an account do work whose cost grows with the units
  it holds, and that the desktop session's own service survives everything a commit does.

 Why:
  On 2026-11-12T01:24:54-05:00 the desktop session ended because one commit asked the shared user manager for a reload:
  the manager re-read every unit it held for 139,770 ms, a reply to the window manager's unit expired while it did, and
  the manager read the expiry as a lost bus name and stopped the window manager with the whole session
  (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).

 How the proof works:
  Every manager this file exercises lives in a container of its own, started here and removed here. A private session
  bus (`dbus-daemon`) and a `systemd --user` run inside it, an unrelated `Type=dbus` service is bound to a target that
  stands in for the desktop, and the manager then holds thousands of transient units, so a reload of it is slow, as it was
  on this host. Two releases are used in turn from two worktrees, several callers commit at once, and the unrelated
  service's state is read after every step.

  Two arms run the same steps, each in a container of its own: one with the wrapper's release as it is installed from
  `main` (the code the reload comes from), and one with the release built from this worktree. The arm from `main` must
  end the unrelated service and its target; the arm from this worktree must leave them alive, and the manager must never
  be asked for an operation whose cost grows with the units it holds.

 Usage:
  `node bin/service-manager-proof.ts --main-wrapper <path> [--image <tag>] [--units <count>] [--probe-deadline <value>]`
  The main release's executable and the release built from this worktree are copied into each container, so both arms run
  the same machine.
 */
import { execFileSync, } from 'node:child_process';
import {
  mkdtempSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir, } from 'node:os';
import { join, resolve, } from 'node:path';

//region The bounds this file runs under
/**
 The most a single `podman` or container command may take here. Every wait in this file is bounded by one of these, so a
 broken fixture ends instead of hanging the gate.
 */
const commandCeilingMilliseconds = 900_000;

/**
 The deadline the unrelated desktop service is given for its own start, as the incident's fixture gave one.
 A reload holds the manager's event loop past it, which is what makes the manager read a queued answer as a lost bus
 name; the incident's own value was a controlled input too.
 */
const defaultProbeDeadline = '200ms';

/**
 How many transient units the manager holds before the steps begin, so a reload of it costs what it costs on a host full
 of agents. Measured: 2,011 loaded socket units took a disposable manager 1.1 to 1.3 seconds to reload.
 */
const defaultUnitCount = 4_000;

/**
 The real Git inside the container, which every wrapper runs and the commit service runs.
 */
const containerGit = '/usr/bin/git';
//endregion

//region The commands this file runs
/**
 Options of this run.
 */
type Options = {
  readonly image: string;
  readonly mainWrapper: string;
  readonly units: number;
  readonly probeDeadline: string;
};

/**
 Runs one bounded command and returns its trimmed standard output.

 @param args - program and arguments

 @returns the command's standard output, trimmed

 @example
 ```ts
 run({ args: ['podman', '--version'] });
 ```
 */
function run({ args, }: { readonly args: readonly string[]; }): string {
  return execFileSync(args[0] as string, args.slice(1), {
    encoding: 'utf8',
    timeout: commandCeilingMilliseconds,
    maxBuffer: 64 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

/**
 The environment every command inside a container runs with, so no wrapper can ever reach this machine's manager.
 */
const containerEnvironment = [
  '--env=XDG_RUNTIME_DIR=/run/user/0',
  '--env=HOME=/root',
  '--env=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/0/bus',
];
//endregion

//region The container, its user manager, and the stand-in for a desktop
/**
 One disposable machine: the container's name, its unit count and the deadline the desktop stand-in has.
 */
type Machine = {
  readonly container: string;
  readonly probeDeadline: string;
  readonly units: number;
};

/**
 Runs a shell script inside the machine under the environment that keeps it away from every other manager.

 @param machine - the container to run in
 @param script - the script, run by `/bin/sh -ec`

 @returns the script's standard output, trimmed

 @example
 ```ts
 inside({ machine, script: 'systemctl --user is-system-running' });
 ```
 */
function inside({ machine, script, }: { readonly machine: Machine; readonly script: string; }): string {
  return run({ args: ['podman', 'exec', ...containerEnvironment, machine.container, '/bin/sh', '-ec', script], });
}

/**
 Copies a host file into the machine.

 @param machine - the container
 @param from - the host path
 @param to - the path inside the container

 @example
 ```ts
 copyIn({ machine, from: '/tmp/machine-id', to: '/etc/machine-id' });
 ```
 */
function copyIn({ machine, from, to, }: { readonly machine: Machine; readonly from: string; readonly to: string; }): void {
  run({ args: ['podman', 'cp', from, `${machine.container}:${to}`,], });
}

/**
 What:
  Starts the container with a system manager as its init, a private session bus, a user manager, and an unrelated
  `Type=dbus` service bound to a target.

 Why:
  The wrapper asks a user manager through a session bus, so the fixture needs both, and the harm the incident showed is
  the death of a program that has nothing to do with Git.
 */
function startMachine({ image, units, }: { readonly image: string; readonly units: number; }): Machine {
  const container = `gpc-reload-proof-${process.pid}-${Math.trunc(Date.now() / 1000)}`;
  const scratch = mkdtempSync(join(tmpdir(), 'gpc-reload-proof-'),);
  // A container that runs systemd as its init needs a machine identity, and the gate's image carries none.
  const machineId = join(scratch, 'machine-id',);
  writeFileSync(machineId, '0123456789abcdef0123456789abcdef\n',);
  run({ args: ['podman', 'run', '--detach', '--name', container, '--user=0', '--cpus=2', '--memory=4g', '--pids-limit=1024',
    '--blkio-weight=10', '--network=none', '--systemd=always', '--security-opt=label=disable',
    `--mount=type=bind,source=${machineId},destination=/etc/machine-id,readonly`, '--entrypoint=/usr/lib/systemd/systemd', image], });
  const machine: Machine = { container, probeDeadline: '', units, };
  const ready = (() => {
    for (let attempt = 0; attempt < 240; attempt += 1) {
      const state = (() => {
        try {
          return inside({ machine, script: 'systemctl is-system-running || true', });
        } catch {
          return '';
        }
      })();
      if (state === 'running' || state === 'degraded') return state;
      execFileSync('sleep', ['0.5',], { timeout: 10_000, });
    }
    throw new Error('the container\'s system manager never came up');
  })();
  // The user manager and the session bus it serves: both inside this container, and named by the account's own runtime
  // directory, so a wrapper started here can reach nothing of this machine's.
  inside({ machine, script: [
    'mkdir --parents /run/user/0 /run/user/0/systemd/user /opt/release-a /opt/release-b /opt/release-seed',
    'chmod 700 /run/user/0',
    'ulimit -n 65536',
    '/usr/sbin/dbus-daemon --session --address=unix:path=/run/user/0/bus --nofork --nopidfile --syslog-only --fork',
    '/usr/lib/systemd/systemd --user >/run/user/0/user-manager.log 2>&1 &',
    'for _ in $(seq 1 200); do [ -S /run/user/0/systemd/private ] && break; sleep 0.1; done',
    'systemctl --user is-system-running --wait 2>/dev/null || true',
  ].join('\n',), });
  // What this machine's manager holds, so a reload of it costs what a reload of a host full of agents costs.
  inside({ machine, script: [
    'i=0',
    `while [ "$i" -lt ${units} ]; do`,
    '  busctl --user call org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager StartTransientUnit \\',
    '    "ssa(sv)a(sa(sv))" "bulk-$i.service" replace 3 Type s oneshot RemainAfterExit b true \\',
    '    ExecStart "a(sasb)" 1 /bin/true 1 /bin/true 0 0 >/dev/null 2>&1 || true',
    '  i=$((i+1))',
    'done',
    'systemctl --user daemon-reload',
    'systemctl --user list-units --all --no-legend --no-pager | wc -l',
  ].join('\n',), });
  console.log(`machine ${container}: ${ready}, holding the units of a loaded host`);
  return machine;
}

/**
 What:
  Starts the stand-in for a desktop: a service that owns a name on this machine's session bus, and a target bound to it
  as the session's own units are bound to the window manager.

 Why:
  The harm the incident showed is this program ending, so it has to exist before the steps that would end it, and it is
  started after any seeding of the machine, so the seeding cannot be mistaken for what an arm did.

 @param input - the machine and the deadline the stand-in has for its own start

 @returns the desktop's state right after it came up

 @example
 ```ts
 startDesktop({ machine, probeDeadline: '300ms' });
 ```
 */
function startDesktop({ machine, probeDeadline, }: { readonly machine: Machine; readonly probeDeadline: string; }): DesktopState {
  const started: Machine = { ...machine, probeDeadline, };
  inside({ machine: started, script: [
    'cat > /run/user/0/systemd/user/desktop-probe.service <<\'EOF\'',
    '[Unit]',
    'Description=Disposable desktop lifetime probe',
    '[Service]',
    'Type=dbus',
    'BusName=org.example.DesktopProbe',
    'ExecStart=/usr/sbin/dbus-test-tool echo --session --name=org.example.DesktopProbe',
    `TimeoutStartSec=${probeDeadline}`,
    'Restart=no',
    'EOF',
    'cat > /run/user/0/systemd/user/desktop-probe.target <<\'EOF\'',
    '[Unit]',
    'Description=Disposable target bound to the desktop probe',
    'Requires=desktop-probe.service',
    'BindsTo=desktop-probe.service',
    'EOF',
    'systemctl --user start desktop-probe.target',
  ].join('\n',), });
  const observed = desktop({ machine: started, },);
  console.log(`desktop stand-in: service=${observed.service} target=${observed.target} owner=${observed.owner}`);
  return observed;
}

/**
 Removes the container and everything it held.

 @param machine - the machine to end

 @example
 ```ts
 stopMachine({ machine });
 ```
 */
function stopMachine({ machine, }: { readonly machine: Machine; }): void {
  try {
    run({ args: ['podman', 'rm', '--force', machine.container,], });
  } catch (error) {
    console.error(`the container ${machine.container} could not be removed: ${String(error)}`);
  }
}

/**
 What:
  The state of the unrelated desktop service, its bound target, and the bus name it owns.

 Why:
  Every step of the scenario reads it, because the harm the incident showed is this service and its target ending, not a
  log line.
 */
type DesktopState = {
  readonly service: string;
  readonly target: string;
  readonly owner: string;
};

/**
 Reads the stand-in desktop's state.

 @param machine - the machine to ask

 @returns the service's state, the target's state, and the bus name's owner or `none`

 @example
 ```ts
 desktop({ machine }); // { service: 'active', target: 'active', owner: ':1.4' }
 ```
 */
function desktop({ machine, }: { readonly machine: Machine; }): DesktopState {
  const answer = inside({ machine, script: [
    'systemctl --user show --property=ActiveState --value desktop-probe.service',
    'systemctl --user show --property=ActiveState --value desktop-probe.target',
    'busctl --user call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus GetNameOwner s org.example.DesktopProbe 2>/dev/null || echo none',
  ].join('\n'), });
  const [service, target, owner] = answer.split('\n',);
  return { service: service ?? '', target: target ?? '', owner: owner ?? '', };
}

/**
 What:
  How many times the user manager was asked to re-read its units, and how long each took.

 Why:
  This is the count the arm from `main` has to show and the arm from this worktree has to show as zero: the number of
  manager-wide operations a commit asked for.
 */
function reloads({ machine, }: { readonly machine: Machine; }): ReadonlyArray<string> {
  // The manager of this fixture is started with its own log file, so its own record of every reload it ran is one read.
  const log = inside({ machine, script: 'cat /run/user/0/user-manager.log 2>/dev/null || true', });
  return log.split('\n',).filter(line => line.includes('Reloading finished in',),);
}
//endregion

//region The scenario every arm runs
/**
 One commit through one release's wrapper, in one worktree.

 @param input - the machine, the release's directory, the worktree and the file to change
 @returns the commit command's exit status and output

 @example
 ```ts
 commit({ machine, release: '/opt/release-a', worktree: '/work/one', change: 'one' });
 ```
 */
function commit({ machine, release, worktree, change, }: { readonly machine: Machine; readonly release: string; readonly worktree: string; readonly change: string; }): { readonly status: number; readonly output: string; } {
  const script = [
    `printf 'base\\n${change}\\n' > ${worktree}/a.txt`,
    `cd ${worktree}`,
    `set +e`,
    `${release}/git add a.txt`,
    `${release}/git commit --message '${change}' -- a.txt`,
    'echo "status=$?"',
  ].join('\n',);
  try {
    const output = inside({ machine, script, },);
    const status = Number(/status=(\d+)/u.exec(output,)?.[1] ?? '1',);
    return { status, output, };
  } catch (error) {
    return { status: 1, output: String(error), };
  }
}

/**
 A worktree of the scratch repository, made by the real Git inside the container.

 @param machine - the machine
 @param name - the worktree's directory name under `/work`

 @example
 ```ts
 worktree({ machine, name: 'two' });
 ```
 */
function worktree({ machine, name, }: { readonly machine: Machine; readonly name: string; }): string {
  inside({ machine, script: [
    'mkdir --parents /work',
    'rm -rf /work/one /work/two',
    'mkdir --parents /work/one',
    `cd /work/one`,
    `${containerGit} init --quiet --initial-branch=main .`,
    `${containerGit} config user.email a@example.invalid`,
    `${containerGit} config user.name 'a person'`,
    'printf "base\\n" > a.txt',
    `${containerGit} add a.txt`,
    `${containerGit} commit --quiet --message base`,
    `mkdir --parents /work/${name}`,
  ].join('\n',), });
  inside({ machine, script: `cd /work/one && ${containerGit} worktree add --quiet --detach /work/${name}`, });
  return `/work/${name}`;
}
//endregion

//region The arms
/**
 What one arm did: the reloads its manager was asked for, the desktop's state after every step, and the commits' statuses.
 */
type ArmResult = {
  readonly name: string;
  readonly reloads: ReadonlyArray<string>;
  readonly steps: ReadonlyArray<string>;
  readonly failures: ReadonlyArray<string>;
  readonly deaths: ReadonlyArray<string>;
};

/**
 What:
  Runs the scenario with one release's wrapper and reads what it did to a manager that holds thousands of units.

 Why:
  The same steps with the two releases are the whole comparison: the release from `main` asks the manager to re-read its
  units, and the release from this worktree must not.
 */
function arm({ name, image, wrapper, probeDeadline, units, seed, }: {
  readonly name: string;
  readonly image: string;
  readonly wrapper: string;
  readonly probeDeadline: string;
  readonly units: number;
  readonly seed?: { readonly wrapper: string; } | undefined;
}): ArmResult {
  const machine = startMachine({ image, units, },);
  const steps: string[] = [];
  const failures: string[] = [];
  const deaths: string[] = [];
  try {
    // The codes this arm runs: the release under test at two paths of its own (a release and a build from a worktree, or a
    // release and the same release elsewhere), which the template named in full before this change.
    run({ args: ['podman', 'cp', wrapper, `${machine.container}:/opt/release-a/git`,], });
    inside({ machine, script: 'cp /opt/release-a/git /opt/release-b/git && chmod +x /opt/release-a/git /opt/release-b/git', });
    const second = worktree({ machine, name: 'two', },);
    const first = '/work/one';
    // What: On the arm from this worktree only, the machine is first seeded by the release from `main`.
    // Why:  A machine that has that release's template installed is the migration this change has to survive, and the seed
    //       has to happen before the stand-in for a desktop starts, so the seeding's own reload cannot be mistaken for
    //       what this arm did.
    if (seed !== undefined) {
      // The earlier release, at a path of its own: it writes its template before this arm's release ever runs.
      run({ args: ['podman', 'cp', seed.wrapper, `${machine.container}:/opt/release-seed/git`,], });
      inside({ machine, script: 'chmod +x /opt/release-seed/git', });
      const seeded = inside({ machine, script: [
        'set +e',
        `printf 'base\\nseed\\n' > ${first}/a.txt`,
        `cd ${first}`,
        '/opt/release-seed/git add a.txt',
        '/opt/release-seed/git commit --message seed -- a.txt 2>&1 | tail -2',
        'echo "seeded=$?"',
      ].join('\n',), });
      steps.push(`the machine was seeded by the release from main, as one that has its template installed: ${seeded.split('\n',).slice(-3,).join(' | ')}`);
      steps.push(`the template the seeding left: ${inside({ machine, script: 'ls /run/user/0/systemd/user 2>/dev/null | tr "\\n" " "; grep --max-count=1 "^ExecStart=" /run/user/0/systemd/user/monochromatic-git-policy-cli@.service 2>/dev/null || echo none', })}`);
    }
    startDesktop({ machine, probeDeadline, },);
    const described = (label: string,): void => {
      const state = desktop({ machine, },);
      steps.push(`${label}: service=${state.service} target=${state.target} owner=${state.owner}`);
      if (state.service !== 'active' || state.target !== 'active' || state.owner === 'none') {
        deaths.push(`${label}: service=${state.service} target=${state.target} owner=${state.owner}`);
      }
    };
    const attempt = (label: string, release: string, where: string, change: string,): void => {
      const result = commit({ machine, release, worktree: where, change, },);
      if (result.status !== 0) failures.push(`${label}: the commit did not land: ${result.output.split('\n',).slice(-6,).join(' | ')}`);
      described(label,);
    };
    const before = reloads({ machine, },).length;
    described('first provisioning and its start',);
    attempt('the first commit of release A', '/opt/release-a', first, 'a1',);
    attempt('an upgrade to release B', '/opt/release-b', first, 'b1',);
    attempt('a downgrade to release A', '/opt/release-a', first, 'a2',);
    attempt('release B again', '/opt/release-b', first, 'b2',);
    attempt('release A in a second worktree', '/opt/release-a', second, 'a3',);
    attempt('release B in the second worktree', '/opt/release-b', second, 'b3',);
    // Several callers at once, in both worktrees and through both releases, as the incident's manager had.
    const parallel: ReadonlyArray<readonly [string, string, string]> = [
      ['/opt/release-a', first, 'p1'],
      ['/opt/release-b', first, 'p2'],
      ['/opt/release-a', second, 'p3'],
      ['/opt/release-b', second, 'p4'],
    ];
    const results = inside({ machine, script: parallel.map(([release, where, change],) => {
      return [`( printf 'base\\n${change}\\n' > ${where}/a.txt; cd ${where}; ${release}/git add a.txt; ${release}/git commit --message '${change}' -- a.txt >/dev/null 2>&1; echo "done ${change} $?" ) &`,];
    },).join('\n',) + '\nwait', },);
    for (const line of results.split('\n',).filter(Boolean,)) {
      if (!line.endsWith(' 0',)) failures.push(`a caller at once: ${line}`);
    }
    described('several callers at once',);
    const asked = reloads({ machine, },);
    const during = asked.length - before;
    steps.push(`the manager was asked to re-read its units ${during} time(s) during this arm's steps (${asked.length} in this machine's whole life)`);
    for (const line of asked.slice(before,)) steps.push(`  ${line.replace(/^.*Reloading finished in /u, 'reload ')}`);
    return { name, reloads: asked.slice(before,), steps, failures, deaths, };
  } finally {
    stopMachine({ machine, },);
  }
}

/**
 Reads the options this run was given.

 @returns the image, the main release's wrapper, the unit count and the deadline

 @example
 ```ts
 options(); // { image: '...', mainWrapper: '/path', units: 2000, probeDeadline: '300ms' }
 ```
 */
function options(): Options {
  const given = new Map<string, string>();
  const arguments_ = process.argv.slice(2,);
  for (let index = 0; index < arguments_.length; index += 2) {
    given.set(arguments_[index] as string, arguments_[index + 1] ?? '',);
  }
  const image = given.get('--image',);
  const mainWrapper = given.get('--main-wrapper',);
  if (image === undefined || mainWrapper === undefined) {
    throw new Error('Usage: node bin/service-manager-proof.ts --image <tag> --main-wrapper <path> [--units <count>] [--probe-deadline <value>]');
  }
  return {
    image,
    mainWrapper,
    units: Number(given.get('--units',) ?? defaultUnitCount,),
    probeDeadline: given.get('--probe-deadline',) ?? defaultProbeDeadline,
  };
}

/**
 What:
  Runs both arms and judges them, printing each arm's steps and the answer it gave.

 Why:
  The arm from `main` shows the defect the issue is about, and the arm from this worktree has to show it gone; a control
  that only counted `Reload` calls in a stand-in manager would prove nothing about a real manager.
 */
function main_(): void {
  const { image, mainWrapper, units, probeDeadline, } = options();
  const candidate = process.env.GPC_CANDIDATE_WRAPPER ?? resolve(import.meta.dirname, '..', 'target', 'debug', 'monochromatic-git-policy-cli');
  // The arm from main runs the same steps with the release that asks for the reload, on a fresh machine.
  const onMain = arm({ name: 'main', image, wrapper: mainWrapper, probeDeadline, units, },);
  // What: The arm from this worktree starts on a machine that already has the release from main installed.
  // Why:  A machine that meets this release for the first time has the earlier release's template on disk, and the change
  //       from that template to this one is the migration the issue asks about. The seeding runs before the stand-in for a
  //       desktop starts, because the reload the old release asks for there is that release's, not this one's.
  const fixed = arm({ name: 'this worktree', image, wrapper: candidate, probeDeadline, units, seed: { wrapper: mainWrapper, }, },);
  for (const result of [onMain, fixed,]) {
    console.log(`\narm ${result.name}:`);
    for (const step of result.steps) console.log(`  ${step}`);
    console.log(`  commits that did not land: ${result.failures.length === 0 ? 'none' : result.failures.join('; ')}`);
    console.log(`  the desktop stand-in ended: ${result.deaths.length === 0 ? 'never' : result.deaths.join('; ')}`);
  }
  console.log(`\nthe arm from main asked for ${onMain.reloads.length} manager-wide reload(s); the arm from this worktree asked for ${fixed.reloads.length}`);
  const problems: string[] = [];
  if (fixed.failures.length > 0) problems.push(`the release from this worktree failed its own steps: ${fixed.failures.join('; ')}`);
  if (fixed.deaths.length > 0) problems.push(`the release from this worktree ended the unrelated desktop: ${fixed.deaths.join('; ')}`);
  if (fixed.reloads.length > 0) problems.push(`the release from this worktree asked the manager to re-read its units ${fixed.reloads.length} time(s)`);
  if (onMain.reloads.length === 0) problems.push('the release from main asked for no reload, so this fixture would prove nothing about it');
  if (onMain.deaths.length === 0) problems.push('the release from main did not end the unrelated desktop, so this fixture does not reproduce the harm');
  if (problems.length > 0) {
    console.error(`\nTHE PROOF FAILED: ${problems.join('; ')}`);
    process.exitCode = 1;
  }
}

main_();
//endregion
