/**
 Asks libvirt whether a session daemon answers, without letting libvirt start one,
 and says how long that question and the wait after it may take.

 mvm runs `virsh list --all` with `LIBVIRT_AUTOSTART=0`, the documented setting
 that stops libvirt from starting a daemon on demand: libvirt then connects to the
 session socket and reports the interface's absence instead of forking
 `virtqemud`. Asked in a loop, the same question tells a start whether the daemon
 answered.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { LIBVIRT_URI, } from './config.ts';
import { CANNOT_CONNECT_TEXT, } from './libvirt-errors.ts';
import {
  type ToolCommand,
  toolNotFoundRemedy,
  VIRSH_COMMAND_ENV,
} from './libvirt-tools.ts';
import { libvirtEnv, } from './session-daemon-fallback.ts';
import {
  CommandFailedError,
  CommandTimedOutError,
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

//region Deadlines

/**
 Name of the tool mvm asks libvirt through, as the remedy names it.
 */
const VIRSH_NAME = 'virsh';

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
export const ANSWER_POLL_INTERVAL_MS = 100;

/**
 Milliseconds one question to libvirt may take.

 Measured here on 2026-11-21, a daemon that answers did so in 37 ms; a socket
 with no daemon behind it is refused at once. Ten seconds is therefore a daemon
 that is there and stuck, which {@link daemonAnswers} reports as not answering.
 */
export const PROBE_DEADLINE_MS = 10_000;


/**
 Asks libvirt whether a session daemon answers, without letting libvirt start
 one.

 Runs `virsh list --all` with `LIBVIRT_AUTOSTART=0`, the documented setting that
 stops libvirt from starting a daemon on demand: libvirt then connects to the
 session socket and reports the interface's absence instead of forking
 `virtqemud`.

 @param deadlineMs - Milliseconds the question may take, never more than what is left of the caller's deadline

 @param virsh - Command mvm runs `virsh` with

 @returns Whether a daemon answered

 @throws {@link ExecutableNotFoundError} when the command that runs `virsh` does not exist

 @example
 ```ts
 await daemonAnswers({ deadlineMs: 10_000, virsh: { argv: ['virsh'], origin: 'path' } });
 ```
 */
export async function daemonAnswers({
  deadlineMs,
  virsh,
}: {
  readonly deadlineMs: number;
  readonly virsh: ToolCommand;
},): Promise<boolean> {
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
      deadlineMs,
      env: libvirtEnv(),
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
export function answerDeadlineMs(): number {
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

//endregion Deadlines
