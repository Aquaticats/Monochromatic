/**
 The disposable machine a reload proof runs on: a container with a system manager as its init, a private session bus, a
 user manager, and thousands of transient units.

 Every manager here lives inside the container this file starts, so a wrapper started in it can reach nothing of the
 account's own manager. The container is removed by whoever holds one, through `Symbol.asyncDispose`, whatever it found.
 */
import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as pause, } from 'node:timers/promises';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 The most a single `podman` command may take here, so a broken fixture ends instead of hanging the gate.
 */
const commandCeilingMilliseconds = 900_000;

/**
 The account's runtime directory inside every container, which is the only manager this file can reach.
 */
export const containerRuntimeDirectory: string = '/run/user/0';

/**
 The log the fixture's user manager writes its own reload records to, because it is started with one of its own.
 */
export const managerLogPath: string = `${containerRuntimeDirectory}/user-manager.log`;

/**
 The real Git inside the container, which every wrapper runs and which the commit service runs.
 */
export const containerGit: string = '/usr/bin/git';

/**
 Where the wrapper under test and the release from `main` are put inside a container.
 */
export const releasePaths: {
  readonly first: string;
  readonly second: string;
  readonly seed: string;
} = {
  /**
   The release under test, at the first of its two paths.
   */
  first: '/opt/release-a',
  /**
   The same release at a path of its own: the "release and a build from a worktree" the old template named in full.
   */
  second: '/opt/release-b',
  /**
   The release from `main`, which seeds a machine before an arm runs.
   */
  seed: '/opt/release-seed',
} as const;

/**
 The worktrees a machine's scratch repository has.
 */
export const worktreePaths: {
  readonly first: string;
  readonly second: string;
} = {
  /**
   The first worktree of the scratch repository.
   */
  first: '/work/one',
  /**
   The second worktree, which is a second commit service instance.
   */
  second: '/work/two',
} as const;

/**
 How many times the container's system manager is asked whether it came up.
 */
const readinessAttempts = 240;

/**
 How long one wait for the container's system manager lasts.
 */
const readinessPauseMilliseconds = 500;

/**
 The memory this machine's container may use, which is more than the gate's own containers because it holds a service
 manager with thousands of units and runs an unoptimised build of the wrapper.
 */
const memoryLimit = '8g';

/**
 How many times one script inside a container is started before the container runtime's refusal is reported.
 */
const execAttempts = 4;

/**
 How long each retry of a refused session waits, in milliseconds.
 */
const execRetryPauseMilliseconds = 1_000;

/**
 The milliseconds in one second, named so the container's name reads as a derivation and not as a number.
 */
const millisecondsPerSecond = 1_000;

/**
 Scratch lives inside the package's gitignored `node_modules/.monochromatic`, never the system temporary directory, whose
 backing device is unknowable.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy',
);

/**
 Every machine this run started and has not removed yet, taken by `removeKeptMachines` at the end of the run.
 */
const kept: {
  readonly machine: Machine;
  readonly identity: string
}[] = [];

/**
 What:
  Removes every machine this run started.

 Why:
  A container whose init is a system manager does not end when the `podman run` that started it is signalled, and one left
  behind keeps a manager and its units running on this host.

 @example
 ```ts
 await removeKeptMachines();
 ```
 */
export async function removeKeptMachines(): Promise<void> {
  for (const held of kept) {
    if (process.env
      .GPC_KEEP_CONTAINER
      !== undefined) {
      console.log(`machine ${held.machine
        .container} was kept for diagnosis`);
      continue;
    }
    await stopMachine(held,);
  }
  kept.length = 0;
}

/**
 One disposable machine: the container's name and how many units it holds.
 */
export type Machine = {
  /**
   The container's name, which every command of this proof names.
   */
  readonly container: string;
  /**
   How many transient units the machine was asked to hold.
   */
  readonly units: number;
};

/**
 The started machine and the removal that ends it, so a holder never forgets the container it owns.

 A run that throws still removes its container, because `await using` disposes this value, and a container whose system
 manager is its init does not end when the `podman run` that started it is signalled.
 */
export type RunningMachine = {
  /**
   The container this value owns, which every step of a proof names.
   */
  readonly machine: Machine;
  /**
   Removes the container and everything it held, called by `await using` when the holder's block ends.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 What:
  Runs one command and returns its captured standard output, trimmed.

 Why:
  Every command here is bounded and captured, so a fixture that stops answering ends the run with the command that stopped
  instead of hanging the gate.

 @param args - program and arguments

 @returns the command's exit status and its trimmed standard output

 @example
 ```ts
 await run({ args: ['podman', '--version'] });
 ```
 */
export async function run({ args, }: { readonly args: readonly string[]; }): Promise<{
  readonly status: number;
  readonly stdout: string;
}> {
  /**
   The program to run, which the arguments' first entry always is.
   */
  const [command] = args;
  if (command === undefined) throw new NativeVerificationError('a command needs a program to run');
  /**
   What the command answered, captured so a passing step stays quiet.
   */
  const outcome = await runCommand({
    command,
    args: args.slice(1,),
    capture: true,
    deadlineMilliseconds: commandCeilingMilliseconds,
  });
  return {
    status: outcome.status ?? 1,
    stdout: outcome.stdout
      .trim(),
  };
}

/**
 What:
  Runs a shell script inside the machine under the environment that keeps it away from every other manager.

 Why:
  The wrapper this proof exercises reads `XDG_RUNTIME_DIR` and the session bus address, so setting them here is what makes a
  container of its own the only manager it can reach.

 @param machine - the container to run in

 @param script - the script, run by `/bin/sh -ec`

 @returns the script's standard output, trimmed

 @example
 ```ts
 await inside({ machine, script: 'systemctl --user is-system-running' });
 ```
 */
export function inside({
  machine,
  script,
}: {
  readonly machine: Machine;
  readonly script: string;
}): Promise<string> {
  /**
   The environment the script runs with, which names this container's manager and no other.
   */
  const environment = [
    `--env=XDG_RUNTIME_DIR=${containerRuntimeDirectory}`,
    '--env=HOME=/root',
    `--env=DBUS_SESSION_BUS_ADDRESS=unix:path=${containerRuntimeDirectory}/bus`,
  ];
  /**
   The environment the script runs with, which names this container's manager and no other; the attempt below retries a
   session the container runtime refused to create, and never a script that ran and failed.
   */
  return runInsideAttempt({
    machine,
    script,
    environment,
    attempt: 0,
  });
}

/**
 What:
  Runs one script inside the machine, once per attempt, retrying only the container runtime's own refusals.

 Why:
  Podman refuses a session while it is writing its own state, and a fixture that reported that as a failing step would report
  the container runtime instead of the wrapper. The script never ran then, so asking again never repeats a step the wrapper
  performed.

 @param machine - the container to run in

 @param script - the script, run by `/bin/sh -ec`

 @param environment - the environment arguments that name this container's manager

 @param attempt - how many sessions have been asked for already

 @returns what the script printed, trimmed

 @example
 ```ts
 await runInsideAttempt({ machine, script: 'true', environment: [], attempt: 0 });
 ```
 */
async function runInsideAttempt({
  machine,
  script,
  environment,
  attempt,
}: {
  readonly machine: Machine;
  readonly script: string;
  readonly environment: readonly string[];
  readonly attempt: number;
}): Promise<string> {
  try {
    /**
     What this attempt printed.
     */
    const answered = await run({
      args: [
        'podman',
        'exec',
        ...environment,
        machine.container,
        '/bin/sh',
        '-ec',
        script,
      ],
    });
    return answered.stdout;
  } catch (error) {
    if ((!(error instanceof NativeVerificationError)) || (!isTransportFailure({ message: error.message, }))) throw error;
    if ((attempt + 1) >= execAttempts) throw error;
    await pause(execRetryPauseMilliseconds,);
    return runInsideAttempt({
      machine,
      script,
      environment,
      attempt: attempt + 1,
    });
  }
}

/**
 Whether a `podman exec` failure says nothing about the wrapper: the runtime would not create the session, or the machine
 would not start the fixture's own desktop stand-in within the deadline that stand-in is given.

 @param message - the error's message

 @returns true for the messages podman answers when it will not create a session

 @example
 ```ts
 isTransportFailure({ message: 'can only create exec sessions on running containers' }); // true
 ```
 */
export function isTransportFailure({ message, }: { readonly message: string; }): boolean {
  /**
   The phrases podman answers when it cannot create a session, which say nothing about the script.
   */
  const phrases = [
    'can only create exec sessions',
    'timed out waiting for file',
    'container state improper',
    'no container with name or ID',
    // What: The fixture's own `Type=dbus` stand-in did not claim its name within the deadline it is given.
    // Why:  On a host whose load is in the hundreds the echo program that claims the name can be scheduled late, and the
    //       arm has then proved nothing about the wrapper: no step of it ran. The arm starts again on a fresh machine, and a
    //       host that cannot bring the stand-in up at all ends the run as a run that could not complete.
    'A dependency job for desktop-probe.target failed',
  ];
  for (const phrase of phrases) {
    if (message.includes(phrase,)) return true;
  }
  return false;
}

/**
 What:
  Runs a script and answers nothing when the machine cannot be asked.

 Why:
  A container whose manager is still starting answers a question with a failure, which is a state to wait out rather than an
  error to report.

 @param machine - the machine to ask

 @param script - the script to run

 @returns what the script printed, or an empty text when it could not run

 @example
 ```ts
 await askedOrNothing({ machine, script: 'systemctl is-system-running' });
 ```
 */
export async function askedOrNothing({
  machine,
  script,
}: {
  readonly machine: Machine;
  readonly script: string;
}): Promise<string> {
  try {
    return await inside({
      machine,
      script,
    });
  } catch (error) {
    if (error instanceof NativeVerificationError) return '';
    throw error;
  }
}

/**
 What:
  Starts the container with a system manager as its init, a private session bus, a user manager, and thousands of units.

 Why:
  The wrapper asks a user manager through a session bus, so the fixture needs both; and the cost of a reload is what the
  manager holds, so the machine holds what a host full of agents holds.

 @param image - the gate's image, which carries systemd, Git and dbus

 @param units - how many transient units the manager may hold

 @returns the started machine and the removal that ends it

 @example
 ```ts
 await using machine = await startMachine({ image: 'localhost/x', units: 4000 });
 ```
 */
export async function startMachine({
  image,
  units,
}: {
  readonly image: string;
  readonly units: number;
}): Promise<RunningMachine> {
  await mkdir(
    scratchBase,
    { recursive: true, },
  );
  /**
   The container's name, unique to this run so two runs never share a manager.
   */
  const container = `gpc-reload-proof-${String(process.pid)}-${String(Math.trunc(Date.now() / millisecondsPerSecond))}`;
  /**
   The file the container reads as `/etc/machine-id`, which a manager as PID 1 needs and the gate's image does not carry.
   It lives as long as the container does, because the container reads it through a bind mount for its whole life.
   */
  const identity = join(
    scratchBase,
    `machine-id-${container}`,
  );
  await writeFile(
    identity,
    '0123456789abcdef0123456789abcdef\n',
  );
  await run({
    args: [
      'podman',
      'run',
      '--detach',
      '--name',
      container,
      '--user=0',
      '--cpus=2',
      `--memory=${memoryLimit}`,
      '--pids-limit=1024',
      '--blkio-weight=10',
      '--network=none',
      '--systemd=always',
      '--security-opt=label=disable',
      `--mount=type=bind,source=${identity},destination=/etc/machine-id,readonly`,
      '--entrypoint=/usr/lib/systemd/systemd',
      image,
    ],
  });
  /**
   The machine this function hands back, which every later step names.
   */
  const machine: Machine = {
    container,
    units,
  };
  await waitForSystemManager({
    machine,
    attempt: 0,
  });
  await inside({
    machine,
    script: [
      `mkdir --parents ${containerRuntimeDirectory}/systemd/user ${releasePaths.first} ${releasePaths.second} ${releasePaths.seed}`,
      `chmod 700 ${containerRuntimeDirectory}`,
      'ulimit -n 65536',
      `/usr/sbin/dbus-daemon --session --address=unix:path=${containerRuntimeDirectory}/bus --nofork --nopidfile --syslog-only --fork`,
      `DBUS_SESSION_BUS_ADDRESS=unix:path=${containerRuntimeDirectory}/bus /usr/lib/systemd/systemd --user >${managerLogPath} 2>&1 &`,
      `for _ in $(seq 1 200); do [ -S ${containerRuntimeDirectory}/systemd/private ] && break; sleep 0.1; done`,
      'systemctl --user is-system-running --wait 2>/dev/null || true',
    ].join('\n',),
  });
  /**
   How many units the manager holds after the fixture loaded them, printed so a run says what it proved against.
   */
  const loaded = await inside({
    machine,
    script: [
      'i=0',
      `while [ "$i" -lt ${String(units)} ]; do`,
      '  busctl --user call org.freedesktop.systemd1 /org/freedesktop/systemd1 org.freedesktop.systemd1.Manager \\',
      '    StartTransientUnit "ssa(sv)a(sa(sv))" "bulk-$i.service" replace 3 Type s oneshot RemainAfterExit b true \\',
      '    ExecStart "a(sasb)" 1 /bin/true 1 /bin/true 0 0 >/dev/null 2>&1 || true',
      '  i=$((i+1))',
      'done',
      'systemctl --user daemon-reload',
      'systemctl --user list-units --all --no-legend --no-pager | wc -l',
    ].join('\n',),
  });
  console.log(`machine ${container}: holding ${loaded.trim()} units, so a reload of it costs what a loaded host's costs`);
  kept.push({
    machine,
    identity,
  });
  return Object.freeze({
    machine,
    // What: The removal this machine's holder calls when its block ends.
    // Why:  `await using` disposes the value even when the steps threw, and a container whose init is a system manager does
    //       not end when the `podman run` that started it is signalled.
    // What: The removal this machine's holder calls when its block ends hands the container to the run's own cleanup.
    // Why:  `await using` disposes the value even when the steps threw, so nothing is left running; the removal itself waits
    //       for the end of the run, because removing a container whose init is a system manager keeps this host's container
    //       storage busy long enough that the next machine's sessions are refused.
    async [Symbol.asyncDispose](): Promise<void> {
      await Promise.resolve();
    },
  });
}

/**
 What:
  Waits until the container's system manager has finished coming up.

 Why:
  A manager that is still starting cannot start a user manager, and every step here needs one; the wait is bounded so a
  container that never comes up fails the run instead of hanging it.

 @param machine - the machine to wait for

 @param attempt - how many waits this step has already made

 @example
 ```ts
 await waitForSystemManager({ machine, attempt: 0 });
 ```
 */
async function waitForSystemManager({
  machine,
  attempt,
}: {
  readonly machine: Machine;
  readonly attempt: number;
}): Promise<void> {
  if (attempt >= readinessAttempts) {
    throw new NativeVerificationError(`the container ${machine.container} never brought its system manager up`);
  }
  /**
   What the container's system manager says about itself, or nothing when it cannot be asked yet.
   */
  const state = await askedOrNothing({
    machine,
    script: 'systemctl is-system-running || true',
  });
  if ((state === 'running') || (state === 'degraded')) return;
  await pause(readinessPauseMilliseconds,);
  return waitForSystemManager({
    machine,
    attempt: attempt + 1,
  });
}

/**
 What:
  Removes a machine's container and everything it held.

 Why:
  A container whose init is a system manager does not end when the `podman run` that started it is signalled, so it is
  ended by the name it was given.

 @param machine - the machine to end

 @example
 ```ts
 await stopMachine({ machine });
 ```
 */
export async function stopMachine({
  machine,
  identity,
}: {
  readonly machine: Machine;
  /**
   This container's machine identity file, which the bind mount read and which goes with the container.
   */
  readonly identity: string;
}): Promise<void> {
  /**
   What `podman rm` answered; a container already gone is not an error to report.
   */
  const ended = await runCommand({
    command: 'podman',
    args: [
      'rm',
      '--force',
      machine.container,
    ],
    capture: true,
    allowFailure: true,
    deadlineMilliseconds: commandCeilingMilliseconds,
  });
  /**
   Whether the removal failed, which is worth a line on standard error and never a failed proof.
   */
  const failed = (ended.status ?? 1) !== 0;
  if (failed) console.error(`the container ${machine.container} could not be removed: ${ended.stderr
    .trim()}`);
  await rm(
    identity,
    { force: true, },
  );
}
