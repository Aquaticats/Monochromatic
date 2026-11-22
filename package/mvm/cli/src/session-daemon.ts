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

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  LIBVIRT_URI,
  SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,
  SESSION_DAEMON_MEMORY_MAX,
  SESSION_DAEMON_UNIT,
} from './config.ts';
import {
  CANNOT_CONNECT_TEXT,
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
  CommandFailedError,
  CommandTimedOutError,
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
 Environment every `virsh` call and the daemon command get.

 `LC_ALL=C` keeps state names and error texts in one language.
 `LIBVIRT_AUTOSTART=0` stops libvirt from starting a daemon on demand, which
 would put the daemon in this command's cgroup; the same setting makes
 {@link daemonAnswers} a question rather than a start.
 */
export const LIBVIRT_ENV = {
  LC_ALL: 'C',
  LIBVIRT_AUTOSTART: '0',
} as const;

/**
 Milliseconds mvm waits for a started daemon to answer before it reports the
 start as failed.

 Basis: measured here on 2026-11-21, a daemon started in a user unit answered
 203 ms after `systemd-run` returned. Thirty seconds leaves room for a loaded
 host while still reporting a daemon that died at startup.
 */
export const SESSION_DAEMON_ANSWER_DEADLINE_MS = 30_000;

/**
 Environment variable that overrides {@link SESSION_DAEMON_ANSWER_DEADLINE_MS},
 in milliseconds, for a host where the daemon needs longer.

 A value that is not a positive whole number is ignored, because it only changes
 how patiently mvm waits.
 */
export const SESSION_DAEMON_ANSWER_DEADLINE_ENV = 'MVM_SESSION_DAEMON_START_TIMEOUT_MS';

/**
 Milliseconds between two questions while mvm waits for a started daemon.

 The daemon answered 203 ms after `systemd-run` returned when measured on
 2026-11-21, so a question every tenth of a second finds it promptly without a
 busy loop.
 */
const ANSWER_POLL_INTERVAL_MS = 100;

/**
 Milliseconds one question to libvirt may take.

 Measured here on 2026-11-21, a daemon that answers did so in 37 ms; a socket
 with no daemon behind it is refused at once. Ten seconds is therefore a daemon
 that is there and stuck, which {@link daemonAnswers} reports as not answering.
 */
const PROBE_DEADLINE_MS = 10_000;

/**
 Executable name of systemd's transient unit runner.
 */
const SYSTEMD_RUN = 'systemd-run';

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
 Renders the arguments `systemd-run` gets: the unit name with its caps, then the
 daemon command, then the daemon's idle timeout.

 @param daemon - Command that runs `virtqemud`, then its leading arguments

 @returns Arguments after the executable name

 @example
 ```ts
 startArguments(['virtqemud']); // => ['--user', '--unit=mvm-libvirt-session', ..., 'virtqemud', '--timeout=120']
 ```
 */
function startArguments(daemon: readonly string[],): readonly string[] {
  return [
    '--user',
    `--unit=${SESSION_DAEMON_UNIT}`,
    // Removes the unit when the daemon exits, so the fixed name is free for the next start.
    '--collect',
    `--property=MemoryMax=${SESSION_DAEMON_MEMORY_MAX}`,
    // `continue`: a guest the kernel kills for the cap must not stop the daemon and every other guest.
    '--property=OOMPolicy=continue',
    ...daemon,
    `--timeout=${String(SESSION_DAEMON_IDLE_TIMEOUT_SECONDS,)}`,
  ];
}

//endregion The daemon command

//region Asking whether a daemon answers

/**
 Asks libvirt whether a session daemon answers, without letting libvirt start
 one.

 Runs `virsh list --all` with `LIBVIRT_AUTOSTART=0`, the documented setting that
 stops libvirt from starting a daemon on demand: libvirt then connects to the
 session socket and reports the interface's absence instead of forking
 `virtqemud`.

 @param virsh - Command mvm runs `virsh` with

 @returns Whether a daemon answered

 @throws {@link ExecutableNotFoundError} when the command that runs `virsh` does not exist

 @example
 ```ts
 await daemonAnswers({ argv: ['virsh'], origin: 'path' });
 ```
 */
async function daemonAnswers(virsh: ToolCommand,): Promise<boolean> {
  /**
   Logger scoped to the question so its outcome is attributable.
   */
  const rl = tagged({
    tag: daemonAnswers.name,
    l,
  },);
  /**
   Executable and the arguments that precede virsh's own.
   */
  const [command, ...leading] = virsh.argv;
  try {
    await spawn({
      args: [
        ...leading,
        '--connect',
        LIBVIRT_URI,
        'list',
        '--all',
      ],
      command,
      deadlineMs: PROBE_DEADLINE_MS,
      env: LIBVIRT_ENV,
      notFoundRemedy: toolNotFoundRemedy({
        command: virsh,
        tool: VIRSH_NAME,
        variable: VIRSH_COMMAND_ENV,
      },),
    },);
    return true;
  }
  catch (error) {
    if ((error instanceof CommandFailedError)
      && error.stderr
        .includes(CANNOT_CONNECT_TEXT,)) {
      rl.debug('no session daemon answers',);
      return false;
    }
    if (error instanceof CommandTimedOutError) {
      rl.debug(`no session daemon answered within ${String(PROBE_DEADLINE_MS,)} ms`,);
      return false;
    }
    throw error;
  }
}

/**
 Reads how long to wait for a started daemon to answer.

 @returns Milliseconds from {@link SESSION_DAEMON_ANSWER_DEADLINE_ENV}, or
 {@link SESSION_DAEMON_ANSWER_DEADLINE_MS} when it is unset or unusable

 @example
 ```ts
 answerDeadlineMs(); // => 30000 without the variable
 ```
 */
function answerDeadlineMs(): number {
  /**
   Whole number of milliseconds the variable holds, or `NaN` when it holds something else.
   */
  const configured = Math.trunc(Number(
    process.env[SESSION_DAEMON_ANSWER_DEADLINE_ENV]
      ?? '',
  ),);
  return ((Number.isFinite(configured,)) && (configured > 0))
    ? configured
    : SESSION_DAEMON_ANSWER_DEADLINE_MS;
}

//endregion Asking whether a daemon answers

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
 Starts the daemon as a transient user unit named {@link SESSION_DAEMON_UNIT},
 or recognises that another process got there first.

 @param daemon - Command that runs `virtqemud`, then its leading arguments

 @param virsh - Command mvm runs `virsh` with, used for the failure's advice

 @throws {@link SessionDaemonStartError} when the unit could not be started for another reason

 @example
 ```ts
 await startUnit({ daemon: ['virtqemud'], virsh: { argv: ['virsh'], origin: 'path' } });
 ```
 */
async function startUnit({
  daemon,
  virsh,
}: {
  readonly daemon: readonly string[];
  readonly virsh: ToolCommand;
},): Promise<void> {
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
  const args = startArguments(daemon,);
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
      deadlineMs: SYSTEMD_RUN_DEADLINE_MS,
      notFoundRemedy: [
        `mvm starts libvirt's session daemon as a transient user unit, which ${SYSTEMD_RUN} launches.`,
        'Install systemd, whose user manager mvm needs here, or start the daemon by hand once; the error below says how.',
      ].join('\n',),
    },);
  }
  catch (error) {
    if (reportsExistingUnit(error,)) {
      rl.debug(`the unit ${SESSION_DAEMON_UNIT}.service already exists; waiting for its daemon`,);
      return;
    }
    throw new SessionDaemonStartError({
      cause: error,
      startCommandLine: commandLine,
      virsh,
    },);
  }
}

/**
 Waits until the started daemon answers, asking again at
 {@link ANSWER_POLL_INTERVAL_MS} until {@link answerDeadlineMs} runs out.

 @param daemon - Command that runs `virtqemud`, then its leading arguments

 @param virsh - Command mvm runs `virsh` with

 @throws {@link SessionDaemonStartError} when the daemon does not answer in time

 @example
 ```ts
 await waitForAnswer({ daemon: ['virtqemud'], virsh: { argv: ['virsh'], origin: 'path' } });
 ```
 */
async function waitForAnswer({
  daemon,
  virsh,
}: {
  readonly daemon: readonly string[];
  readonly virsh: ToolCommand;
},): Promise<void> {
  /**
   Logger scoped to the wait so its outcome is attributable.
   */
  const rl = tagged({
    tag: waitForAnswer.name,
    l,
  },);
  /**
   Milliseconds after which mvm reports the start as failed.
   */
  const deadline = Date.now() + answerDeadlineMs();
  while (Date.now() < deadline) {
    // oxlint-disable-next-line no-await-in-loop -- each question depends on the answer to the one before it
    if (await daemonAnswers(virsh,)) {
      rl.debug(`the session daemon in ${SESSION_DAEMON_UNIT}.service answers`,);
      return;
    }
    // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling delay
    await wait(ANSWER_POLL_INTERVAL_MS,);
  }
  throw new SessionDaemonStartError({
    startCommandLine: renderCommandLine({
      args: startArguments(daemon,),
      command: SYSTEMD_RUN,
    },),
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
  if (await daemonAnswers(virsh,)) {
    rl.debug('a session daemon answers; mvm uses it as it is',);
    return;
  }
  /**
   Command that runs the daemon this host's libvirt needs.
   */
  const daemon = sessionDaemonCommand({
    override: process.env[VIRTQEMUD_COMMAND_ENV]
      ?? '',
    virsh,
  },);
  await startUnit({
    daemon,
    virsh,
  },);
  await waitForAnswer({
    daemon,
    virsh,
  },);
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
