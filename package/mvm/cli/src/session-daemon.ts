/**
 Starts libvirt's session daemon in a user unit of its own before mvm's first
 `virsh` call, so that the daemon and every guest it starts leave the cgroup of
 the command that ran `mvm`.

 `virsh` starts `virtqemud` on demand, and the daemon it starts inherits the
 cgroup of the command that ran `virsh` (libvirt 12.9,
 `virNetSocketForkDaemon` in `src/rpc/virnetsocket.c`); that command's memory
 cap, time limit or cleanup then kills the daemon and every guest of every
 session. libvirt starts a daemon of its own only when `LIBVIRT_AUTOSTART` is
 unset or not `0` (`src/libvirt.c`, `virConnectOpen`; `remoteGetURIDaemonInfo`
 in `src/remote/remote_sockets.c`), so mvm asks with that variable set to `0`,
 which connects without starting anything, and starts the daemon itself when
 nothing answers.

 A daemon that is already running is used as it is, whatever cgroup it is in.

 The unit runs with `ExitType=cgroup`, so systemd keeps it and its cgroup while
 any guest process runs after the daemon exits: a guest outlived a daemon that
 started it on `main`, and this keeps that. When mvm cannot use a unit at all
 (no `systemd-run`, or a user manager it cannot reach), it lets `virsh` start the
 daemon the way it did before and warns, naming the cgroup defect that leaves.

 The question asked before any start lives in `session-daemon-question.ts`, the
 fallback that lets `virsh` start the daemon when no unit can be used in
 `session-daemon-fallback.ts`, and the memory cap in `session-daemon-memory.ts`.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  LIBVIRT_URI,
  SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,
  SESSION_DAEMON_UNIT,
} from './config.ts';
import {
  SessionDaemonCommandError,
  SessionDaemonStartError,
} from './libvirt-errors.ts';
import {
  libvirtTools,
  parseToolCommand,
  type ToolCommand,
  toolNotFoundRemedy,
  VIRTQEMUD_COMMAND_ENV,
  VIRSH_COMMAND_ENV,
} from './libvirt-tools.ts';
import {
  takeFallback,
  type SessionDaemonFallbackReason,
} from './session-daemon-fallback.ts';
import { sessionDaemonMemoryMax, } from './session-daemon-memory.ts';
import {
  ANSWER_POLL_INTERVAL_MS,
  answerDeadlineMs,
  daemonAnswers,
  PROBE_DEADLINE_MS,
} from './session-daemon-question.ts';
import {
  CommandFailedError,
  ExecutableNotFoundError,
  renderCommandLine,
} from './spawn-errors.ts';
import { spawn, } from './spawn.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Environment

/**
 Executable name of systemd's transient unit runner.
 */
const SYSTEMD_RUN = 'systemd-run';

/**
 Argument with which `systemd-run` prints its version instead of starting a unit.
 */
const VERSION_ARGUMENT = '--version';

/**
 Environment of the `systemd-run` calls whose answers mvm classifies: the texts
 it matches are systemd's own, so they are asked for in one language.
 */
const SYSTEMD_RUN_ENV = {
  LC_ALL: 'C',
} as const;

/**
 Milliseconds `systemd-run` may take to start the unit.

 Measured here on 2026-11-21, `systemd-run --user` returned in 8 to 29 ms; a
 user manager that has not answered in thirty seconds is stuck, and waiting
 longer would hide that.
 */
const SYSTEMD_RUN_DEADLINE_MS = 30_000;

/**
 Name of the tool mvm asks libvirt through, spelled as it appears in a command.
 */
const VIRSH_NAME = 'virsh';

/**
 Name of the daemon that tool becomes.
 */
const DAEMON_NAME = 'virtqemud';

/**
 Example value shown when {@link VIRTQEMUD_COMMAND_ENV} cannot be read.
 */
const VIRTQEMUD_EXAMPLE = `["${DAEMON_NAME}"]`;

/**
 Milliseconds between two start attempts while a unit of the same name is still
 loaded, as it is while the daemon a previous caller started is exiting.

 Basis: measured here on 2026-11-21, a unit name was refused for as long as its
 unit ran, and a probe asking for the same name every 25 ms was accepted again
 2.054 s after the start, 54 ms after that unit's two-second main process was due
 to exit. A quarter of a second therefore finds a freed name promptly without a
 busy loop.
 */
const START_RETRY_INTERVAL_MS = 250;

//endregion Environment

//region The daemon command

/**
 Builds the command that runs `virtqemud` from the command that runs `virsh`, or
 takes it from its own variable.

 Unset, {@link VIRTQEMUD_COMMAND_ENV} leaves the derivation: `virsh` in each
 argument becomes `virtqemud`, which covers a bare `virsh`, a path such as
 `/usr/bin/virsh`, and the Flatpak's `--command=virsh`.

 @param override - Text of {@link VIRTQEMUD_COMMAND_ENV}, `''` when unset

 @param virsh - Command mvm runs `virsh` with

 @returns Executable that runs `virtqemud`, then its leading arguments

 @throws {@link ToolCommandConfigError} when the override is set and unreadable

 @throws {@link SessionDaemonCommandError} when no argument names `virsh`

 @example
 ```ts
 sessionDaemonCommand({ override: '', virsh: { argv: ['virsh'], origin: 'path' } }); // => ['virtqemud']
 ```
 */
export function sessionDaemonCommand({
  override,
  virsh,
}: {
  readonly override: string;
  readonly virsh: ToolCommand;
},): readonly [
  string,
  ...string[],
] {
  if (override !== '') {
    return parseToolCommand({
      example: VIRTQEMUD_EXAMPLE,
      value: override,
      variable: VIRTQEMUD_COMMAND_ENV,
    },);
  }
  /**
   Whether any argument carries the tool's name, which is what the derivation replaces.
   */
  const namesVirsh = virsh.argv
    .some(function namesTool(argument,) {
      return argument.includes(VIRSH_NAME,);
    },);
  if (!namesVirsh) {
    throw new SessionDaemonCommandError({
      variable: VIRTQEMUD_COMMAND_ENV,
      virsh,
    },);
  }
  /**
   The executable, which carries the tool's name.
   */
  const [first, ...rest] = virsh.argv;
  return [
    first.replaceAll(
      VIRSH_NAME,
      DAEMON_NAME,
    ),
    ...rest.map(function toDaemon(argument,) {
      return argument.replaceAll(
        VIRSH_NAME,
        DAEMON_NAME,
      );
    },),
  ];
}

/**
 Renders the arguments `systemd-run` gets: the unit name with its settings, then
 the daemon command, then the daemon's idle timeout.

 @param daemon - Command that runs `virtqemud`, then its leading arguments

 @param memoryMax - Cap on the memory of every guest together, as the machine's
 half is read at start time

 @returns Arguments after the executable name

 @example
 ```ts
 startArguments({ daemon: ['virtqemud'], memoryMax: '31576M' }); // => ['--user', '--unit=mvm-libvirt-session', ..., 'virtqemud', '--timeout=120']
 ```
 */
function startArguments({
  daemon,
  memoryMax,
}: {
  readonly daemon: readonly string[];
  readonly memoryMax: string;
},): readonly string[] {
  return [
    '--user',
    `--unit=${SESSION_DAEMON_UNIT}`,
    // Removes the unit when the daemon exits, so the fixed name is free for the next start.
    '--collect',
    `--property=MemoryMax=${memoryMax}`,
    // `continue`: a guest the kernel kills for the cap must not stop the daemon and every other guest.
    '--property=OOMPolicy=continue',
    // `cgroup`: the unit stays up while any guest process lives after the daemon exits, as it did before the unit
    // existed. systemd.service(5) on systemd 262: "If set to cgroup, the service will be considered running as long as
    // at least one process in the cgroup has not exited".
    '--property=ExitType=cgroup',
    ...daemon,
    `--timeout=${String(SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,)}`,
  ];
}

//endregion The daemon command

//region Starting the unit

/**
 Checks whether a failed `systemd-run` failed only because the unit name is
 taken, which is how a second starter of the same unit loses.

 systemd 262.2-1 answers `Failed to start transient service unit: Unit
 mvm-libvirt-session.service was already loaded or has a fragment file.`
 (recorded 2026-11-21).

 @param error - Value caught from the `systemd-run` call

 @returns Whether the unit name was already taken

 @example
 ```ts
 reportsExistingUnit(error); // true for the message above
 ```
 */
function reportsExistingUnit(error: unknown,): boolean {
  return (error instanceof CommandFailedError)
    && error.stderr
      .includes(`${SESSION_DAEMON_UNIT}.service`,)
    && error.stderr
      .includes('already',);
}

/**
 This attempt started the unit.
 */
type StartClaimed = { readonly state: 'claimed'; };

/**
 This attempt was refused because a unit of that name is loaded, as it is while a
 daemon a previous caller started is still running or exiting.
 */
type StartAlreadyLoaded = { readonly state: 'already-loaded'; };

/**
 mvm has no user manager to start a unit in, so the daemon has to be started the
 way it was before.
 */
type StartFallback = {
  readonly reason: SessionDaemonFallbackReason;
  readonly state: 'fallback';
};

/**
 What one `systemd-run` attempt did.
 */
type StartOutcome = StartAlreadyLoaded | StartClaimed | StartFallback;

/**
 Checks whether `systemd-run` answered that it cannot reach the user manager, as
 it does when `XDG_RUNTIME_DIR` is not the one the user manager uses.

 Recorded here on 2026-11-21 from a caller with a private runtime directory:
 `Failed to connect to user scope bus via local transport: No such file or
 directory`.

 @param error - Value caught from the `systemd-run` call

 @returns Whether the user manager could not be reached, narrowed to the failed command that said so

 @example
 ```ts
 reportsUnreachableUserManager(error); // true for the message above
 ```
 */
function reportsUnreachableUserManager(error: unknown,): error is CommandFailedError {
  return (error instanceof CommandFailedError)
    && error.stderr
      .includes('Failed to connect to',)
    && error.stderr
      .includes('bus',);
}

/**
 Whether this host has `systemd-run` at all, asked without starting anything and
 without contacting the user manager.

 @returns Whether the executable runs and reports its version

 @example
 ```ts
 await systemdRunExists();
 ```
 */
async function systemdRunExists(): Promise<boolean> {
  try {
    await spawn({
      args: [VERSION_ARGUMENT,],
      command: SYSTEMD_RUN,
      deadlineMs: SYSTEMD_RUN_DEADLINE_MS,
      env: SYSTEMD_RUN_ENV,
    },);
    return true;
  }
  catch (error) {
    // Any failure other than a missing executable still means the program is there.
    return (!(error instanceof ExecutableNotFoundError));
  }
}

/**
 Starts the daemon as a transient user unit named {@link SESSION_DAEMON_UNIT},
 or recognises that another process got there first.

 @param daemon - Command that runs `virtqemud`, then its leading arguments

 @param deadlineMs - Milliseconds this attempt may take, never more than what is left of the caller's deadline

 @param memoryMax - Cap on the memory of every guest together

 @param virsh - Command mvm runs `virsh` with, used for the failure's advice

 @returns What this attempt did

 @throws {@link SessionDaemonStartError} when the unit could not be started for another reason

 @example
 ```ts
 await startUnit({ daemon: ['virtqemud'], deadlineMs: 30_000, memoryMax: '31576M', virsh: { argv: ['virsh'], origin: 'path' } });
 ```
 */
async function startUnit({
  daemon,
  deadlineMs,
  memoryMax,
  virsh,
}: {
  readonly daemon: readonly string[];
  readonly deadlineMs: number;
  readonly memoryMax: string;
  readonly virsh: ToolCommand;
},): Promise<StartOutcome> {
  /**
   Logger scoped to the start so its outcome is attributable.
   */
  const rl = tagged({
    tag: startUnit.name,
    l,
  },);
  /**
   Complete `systemd-run` arguments.
   */
  const args = startArguments({
    daemon,
    memoryMax,
  },);
  /**
   The start as a reader can repeat it.
   */
  const commandLine = renderCommandLine({
    args,
    command: SYSTEMD_RUN,
  },);
  rl.debug(`starting the session daemon: ${commandLine}`,);
  try {
    await spawn({
      args,
      command: SYSTEMD_RUN,
      deadlineMs,
      env: SYSTEMD_RUN_ENV,
      notFoundRemedy: [
        `mvm starts libvirt's session daemon as a transient user unit, which ${SYSTEMD_RUN} launches.`,
        'Install systemd, whose user manager mvm needs here, or start the daemon by hand once; the error below says how.',
      ].join('\n',),
    },);
    return { state: 'claimed', };
  }
  catch (error) {
    if (reportsExistingUnit(error,)) {
      rl.debug(`the unit ${SESSION_DAEMON_UNIT}.service is still loaded; retrying the start while it stays loaded`,);
      return { state: 'already-loaded', };
    }
    if (error instanceof ExecutableNotFoundError) {
      rl.debug(`${SYSTEMD_RUN} does not exist`,);
      return {
        reason: 'no-systemd-run',
        state: 'fallback',
      };
    }
    if (reportsUnreachableUserManager(error,)) {
      /**
       What the user manager answered, on one line.
       */
      const answer = error.stderr
        .trim();
      rl.debug(`systemd's user manager cannot be reached: ${answer}`,);
      return {
        reason: 'user-manager-unreachable',
        state: 'fallback',
      };
    }
    throw new SessionDaemonStartError({
      cause: error,
      startCommandLine: commandLine,
      virsh,
    },);
  }
}

/**
 Starts the unit, and starts it again whenever systemd says a unit of that name
 is still loaded, until its daemon answers or {@link answerDeadlineMs} runs out.

 A unit whose daemon is exiting still holds its name for a moment, and the
 daemon's socket is already gone, so a single refused start cannot be told apart
 from a daemon that is merely slow; retrying the start until the deadline covers
 both, and is what makes a caller that arrives while the previous daemon exits
 wait instead of failing.

 @param daemon - Command that runs `virtqemud`, then its leading arguments

 @param virsh - Command mvm runs `virsh` with

 @throws {@link SessionDaemonStartError} when the daemon does not answer in time

 @throws {@link SessionDaemonMemoryError} when the machine's total memory cannot be read

 @example
 ```ts
 await startAndWait({ daemon: ['virtqemud'], virsh: { argv: ['virsh'], origin: 'path' } });
 ```
 */
async function startAndWait({
  daemon,
  virsh,
}: {
  readonly daemon: readonly string[];
  readonly virsh: ToolCommand;
},): Promise<void> {
  /**
   Logger scoped to the start and the wait so its outcome is attributable.
   */
  const rl = tagged({
    tag: startAndWait.name,
    l,
  },);
  /**
   Cap on the memory of every guest together, read from this machine now.
   */
  const memoryMax = await sessionDaemonMemoryMax();
  /**
   Milliseconds after which mvm reports the start as failed.
   */
  const deadline = Date.now() + answerDeadlineMs();
  /**
   Progress of the loop, in one `const` container so no root-level `let` widens the scope of the statements after it.

   `claimed`: this call already started the unit, so no further start is needed.
   `lastStartRefused`: the last attempt was refused because a unit of that name is loaded.
   `nextStartAt`: when the next start attempt may run; zero so the first one runs at once.
   */
  const progress = {
    claimed: false,
    lastStartRefused: false,
    nextStartAt: 0,
  };
  while (Date.now() < deadline) {
    /**
     Milliseconds left of the caller's deadline, which no single step may outlast.
     */
    const remainingMs = Math.max(
      1,
      deadline - Date.now(),
    );
    if ((!progress.claimed) && (Date.now() >= progress.nextStartAt)) {
      /**
       What this attempt did.
       */
      // oxlint-disable-next-line no-await-in-loop -- each attempt reports on the one before it
      const outcome = await startUnit({
        daemon,
        deadlineMs: Math.min(
          SYSTEMD_RUN_DEADLINE_MS,
          remainingMs,
        ),
        memoryMax,
        virsh,
      },);
      if (outcome.state === 'fallback') {
        takeFallback(outcome.reason,);
        return;
      }
      progress.claimed = outcome.state === 'claimed';
      progress.lastStartRefused = outcome.state === 'already-loaded';
      progress.nextStartAt = Date.now() + START_RETRY_INTERVAL_MS;
    }
    // oxlint-disable-next-line no-await-in-loop -- each question depends on the answer to the one before it
    if (await daemonAnswers({
      deadlineMs: Math.min(
        PROBE_DEADLINE_MS,
        remainingMs,
      ),
      virsh,
    },)) {
      rl.debug(`the session daemon in ${SESSION_DAEMON_UNIT}.service answers`,);
      return;
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling delay
    await wait(ANSWER_POLL_INTERVAL_MS,);
  }
  throw new SessionDaemonStartError({
    startCommandLine: renderCommandLine({
      args: startArguments({
        daemon,
        memoryMax,
      },),
      command: SYSTEMD_RUN,
    },),
    unitStillLoaded: progress.lastStartRefused,
    virsh,
  },);
}

/**
 Uses the daemon that answers, or starts one in its own user unit and waits for
 it to answer.

 @throws {@link SessionDaemonStartError} when no daemon answers and none could be started

 @example
 ```ts
 await checkAndStart();
 ```
 */
async function checkAndStart(): Promise<void> {
  /**
   Logger scoped to the check so its outcome is attributable.
   */
  const rl = tagged({
    tag: checkAndStart.name,
    l,
  },);
  /**
   Command that runs virsh on this host.
   */
  const { virsh, } = await libvirtTools();
  if (await daemonAnswers({
    deadlineMs: PROBE_DEADLINE_MS,
    virsh,
  },)) {
    rl.debug('a session daemon answers; mvm uses it as it is',);
    return;
  }
  try {
    /**
     Command that runs the daemon this host's libvirt needs.
     */
    const daemon = sessionDaemonCommand({
      override: process.env[VIRTQEMUD_COMMAND_ENV]
        ?? '',
      virsh,
    },);
    await startAndWait({
      daemon,
      virsh,
    },);
  }
  catch (error) {
    if (!(error instanceof SessionDaemonCommandError))
      throw error;

    // A unit needs a command to run in it; the on-demand start needs none. Without systemd-run there is no unit to
    // start, so the command that runs the daemon is not the caller's problem here.
    if (await systemdRunExists())
      throw error;

    rl.debug('no systemd-run, and nothing here can name the daemon command; letting virsh start the daemon instead',);
    takeFallback('no-systemd-run',);
  }
}

//endregion Starting the unit

//region Entry point

/**
 Not yet settled first check, so a long-lived process such as the MCP server
 asks once; `pending` is absent again after a failed check, so a later call asks
 again.
 */
const checks: { pending?: Promise<void>; } = {};

/**
 Makes sure libvirt's session daemon answers, starting it in its own user unit
 when none does. The first call in a process does the work; later calls await
 the same result.

 @param force - Whether to ask and start again even when this process already checked

 @throws {@link SessionDaemonStartError} when no daemon answers and none could be started

 @throws {@link SessionDaemonCommandError} when the daemon command cannot be derived

 @throws {@link ExecutableNotFoundError} when the command that runs `virsh` does not exist

 @example
 ```ts
 await ensureSessionDaemon();
 ```
 */
export async function ensureSessionDaemon(
  { force = false, }: { readonly force?: boolean; } = {},
): Promise<void> {
  if (force) {
    delete checks.pending;
  }
  /**
   Check this process already started, when it did.
   */
  const started = checks.pending;
  if (started !== undefined) {
    await started;
    return;
  }
  /**
   Check this call starts; recorded before the first `await` so a concurrent caller waits on it.
   */
  const pending = checkAndStart();
  checks.pending = pending;
  try {
    await pending;
  }
  catch (error) {
    if (checks.pending === pending) {
      delete checks.pending;
    }
    throw error;
  }
}

//endregion Entry point
