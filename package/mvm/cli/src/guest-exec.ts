/**
 Runs one shell command in a guest through the QEMU guest agent and returns
 the result that provably belongs to it.

 The agent keeps a finished command's result until someone reads it and finds
 it by process ID alone, so a process ID the guest reuses can first return an
 older command's result. Every command therefore prints a fresh marker line
 first, and only a result carrying that marker is returned. A status request
 that gets no answer while the guest is busy is asked again; it says nothing
 about the command.

 @module
 */

import { MS_PER_SECOND, } from '@monochromatic-dev/module-const/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  agentCommand,
  GuestAgentProtocolError,
  GuestAgentReplyError,
  GuestAgentUnreachableError,
} from './agent-command.ts';
import {
  createExecMarker,
  type ExecMarker,
  markedExecArgs,
  NOT_MARKED,
  outputAfterMarker,
} from './exec-shell.ts';
import {
  type DiscardedGuestResult,
  GuestExecAttributionError,
  GuestExecLaunchError,
  GuestExecStatusUnavailableError,
} from './guest-exec-errors.ts';
import {
  type Pacing,
  statusReads,
} from './guest-exec-status.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Public types and limits

/**
 Result of executing a command inside a VM via guest agent.
 */
export type ExecResult = {
  stdout: string;
  stderr: string;
  exitCode: number;
};

/**
 Time limits of one guest command run.

 @example
 ```ts
 const patient: GuestExecLimits = { ...DEFAULT_GUEST_EXEC_LIMITS, unresponsiveLimitMs: 900_000 };
 ```
 */
export type GuestExecLimits = {
  /**
   Seconds libvirt waits for the agent to answer one request.
   */
  readonly agentCallTimeoutSeconds: number;
  /**
   Milliseconds between status requests while the command runs.
   */
  readonly pollIntervalMs: number;
  /**
   Milliseconds of status requests failing in a row after which the run gives up.
   */
  readonly unresponsiveLimitMs: number;
};

/**
 Default limits: a busy guest gets a minute per request and five minutes of
 unanswered requests before the run gives up. libvirt's own default of five
 seconds per request was observed to lapse while a guest extracted an archive.
 */
export const DEFAULT_GUEST_EXEC_LIMITS: GuestExecLimits = {
  agentCallTimeoutSeconds: 60,
  pollIntervalMs: 250,
  unresponsiveLimitMs: 300_000,
};

//endregion Public types and limits

//region Launch

/**
 Starts the command in the guest and returns its process ID.

 @param arg - Arguments of the program

 @param domain - Prefixed libvirt domain name

 @param path - Program to run in the guest

 @param timeoutSeconds - Seconds libvirt waits for the agent to confirm the launch

 @returns Guest process ID

 @throws {@link GuestExecLaunchError} when the agent refused the launch or did not confirm it

 @example
 ```ts
 const pid = await launch({ arg: ['-c', 'true'], domain: 'mvm-dev', path: '/bin/sh', timeoutSeconds: 60 });
 ```
 */
async function launch({
  arg,
  domain,
  path,
  timeoutSeconds,
}: {
  readonly arg: readonly string[];
  readonly domain: string;
  readonly path: string;
  readonly timeoutSeconds: number;
},): Promise<number> {
  /**
   Logger scoped to the launch so a refused or unconfirmed start is attributable.
   */
  const rl = tagged({
    tag: launch.name,
    l,
  },);
  /**
   Agent answer to the launch; the protocol reference gives `{ pid }`.
   */
  const answer = await (async function requestLaunch(): Promise<unknown> {
    try {
      return await agentCommand({
        domain,
        execute: 'guest-exec',
        parameters: {
          arg,
          'capture-output': true,
          path,
        },
        timeoutSeconds,
      },);
    }
    catch (error) {
      if (error instanceof GuestAgentReplyError) {
        rl.debug(`launch refused in ${domain}`,);
        throw new GuestExecLaunchError({
          cause: error,
          domain,
          mayBeRunning: false,
        },);
      }
      if (error instanceof GuestAgentUnreachableError) {
        rl.debug(`launch not confirmed in ${domain}`,);
        throw new GuestExecLaunchError({
          cause: error,
          domain,
          mayBeRunning: true,
        },);
      }
      throw error;
    }
  })();
  if (((typeof answer) === 'object')
    && (answer !== null)
    && ('pid' in answer)
    && ((typeof answer.pid) === 'number'))
    {
    return answer.pid;
  }
  throw new GuestAgentProtocolError({
    commandName: 'guest-exec',
    received: JSON.stringify(answer,),
  },);
}

//endregion Launch

//region Attributed result

/**
 Sentinel for "the latest status request was answered".
 A unique symbol models "no outage" without a nullish union.
 */
const ANSWERING: unique symbol = Symbol('set while status requests are being answered',);

/**
 Waits for the finished result that carries `marker` and returns it without the marker line.

 @param domain - Prefixed libvirt domain name

 @param limits - Time limits of the run

 @param marker - Marker the command prints first

 @param pid - Guest process ID the command was started as

 @returns The command's own result

 @throws {@link GuestExecAttributionError} when the agent forgets the process ID before reporting a marked result

 @throws {@link GuestExecStatusUnavailableError} when the domain is gone or the agent stays silent past the limit

 @throws {@link GuestExecOutputTooLargeError} when the finished result is larger than libvirt hands over

 @example
 ```ts
 const result = await attributedResult({ domain: 'mvm-dev', limits, marker, pid: 42 });
 ```
 */
async function attributedResult({
  domain,
  limits,
  marker,
  pid,
}: {
  readonly domain: string;
  readonly limits: GuestExecLimits;
  readonly marker: ExecMarker;
  readonly pid: number;
},): Promise<ExecResult> {
  /**
   Logger scoped to the wait so discarded results and outages are attributable.
   */
  const rl = tagged({
    tag: attributedResult.name,
    l,
  },);
  /**
   Finished results reported under the process ID without the marker, oldest first.
   */
  const discarded: DiscardedGuestResult[] = [];
  /**
   Pause control for the next status request.
   */
  const pacing: Pacing = { pauseBeforeNextRead: false, };
  /**
   When status requests started failing in a row, or {@link ANSWERING}.
   */
  const outage: { sinceMs: number | typeof ANSWERING; } = { sinceMs: ANSWERING, };
  for await (
    const read of statusReads({
      domain,
      limits,
      pacing,
      pid,
    },)
  ) {
    if (read.kind === 'forgotten') {
      throw new GuestExecAttributionError({
        cause: read.error,
        discarded,
        domain,
        pid,
      },);
    }
    if (read.kind === 'gone') {
      throw new GuestExecStatusUnavailableError({
        cause: read.error,
        domain,
        pid,
        reason: `the domain is now ${read.state}`,
      },);
    }
    if (read.kind === 'unanswered') {
      /**
       Start of the current run of unanswered requests.
       */
      const sinceMs = (outage.sinceMs === ANSWERING) ? Date.now() : outage.sinceMs;
      if ((Date.now() - sinceMs) >= limits.unresponsiveLimitMs) {
        throw new GuestExecStatusUnavailableError({
          cause: read.error,
          domain,
          pid,
          reason: `the guest agent has not answered a status request for ${
            String(Math.round(limits.unresponsiveLimitMs / MS_PER_SECOND,),)
          } seconds`,
        },);
      }
      if (outage.sinceMs === ANSWERING) {
        rl.info(
          `the guest agent in ${domain} did not answer a status request; the command keeps running and is asked about again`,
        );
      }
      outage.sinceMs = sinceMs;
      pacing.pauseBeforeNextRead = true;
      continue;
    }
    outage.sinceMs = ANSWERING;
    if (read.kind === 'running') {
      pacing.pauseBeforeNextRead = true;
      continue;
    }
    /**
     The command's own standard output, or {@link NOT_MARKED} for another command's result.
     */
    const ownStdout = outputAfterMarker({
      marker,
      stdout: read.result
        .stdout,
    },);
    if (ownStdout !== NOT_MARKED) {
      if (read.truncated) {
        rl.warn(`the guest agent captured only part of the output of guest process ${String(pid,)}`,);
      }
      return {
        exitCode: read.result
          .exitCode,
        stderr: read.result
          .stderr,
        stdout: ownStdout,
      };
    }
    rl.warn(
      `discarded a finished result for guest process ${
        String(pid,)
      } in ${domain}: it lacks this command's marker, so it belongs to another command`,
    );
    discarded.push(read.result,);
    // The discarded entry is gone from the agent; the next request finds this command's own entry without a pause.
    pacing.pauseBeforeNextRead = false;
  }
  throw new Error('status requests ended without a result; the series is endless',);
}

//endregion Attributed result

//region Run

/**
 Runs a shell command in a guest and returns its own output and exit status.

 @param command - Shell command to run; passed to the guest's shell unchanged after a marker statement

 @param domain - Prefixed libvirt domain name

 @param limits - Time limits; defaults to {@link DEFAULT_GUEST_EXEC_LIMITS}

 @param osFamily - Guest OS family, which selects the shell's argument form

 @param shell - Shell executable in the guest

 @returns Captured stdout, stderr, and exit code of this command

 @throws {@link GuestExecLaunchError} when the agent refused the launch or did not confirm it

 @throws {@link GuestExecStatusUnavailableError} when the domain is gone or the agent stays silent past the limit

 @throws {@link GuestExecAttributionError} when no reported result carries this command's marker

 @throws {@link GuestExecOutputTooLargeError} when the finished result is larger than libvirt hands over

 @example
 ```ts
 const result = await runGuestCommand({
   command: 'uname -a',
   domain: 'mvm-dev-01',
   osFamily: 'linux',
   shell: '/bin/bash',
 });
 ```
 */
export async function runGuestCommand({
  command,
  domain,
  limits = DEFAULT_GUEST_EXEC_LIMITS,
  osFamily,
  shell,
}: {
  readonly command: string;
  readonly domain: string;
  readonly limits?: GuestExecLimits;
  readonly osFamily: string;
  readonly shell: string;
},): Promise<ExecResult> {
  /**
   Logger scoped to this run so log lines carry the function name.
   */
  const rl = tagged({
    tag: runGuestCommand.name,
    l,
  },);
  /**
   Marker only this launch knows; ties a reported result to this command.
   */
  const marker = createExecMarker();
  /**
   Shell-specific `path` and `arg` array that print the marker, then run the command.
   */
  const {
    arg,
    path,
  } = markedExecArgs({
    command,
    marker,
    osFamily,
    shell,
  },);
  rl.debug(`executing command in ${domain} (${osFamily}, ${path}): ${command}`,);
  /**
   Guest process ID assigned by the QEMU guest agent; reused on every status request.
   */
  const pid = await launch({
    arg,
    domain,
    path,
    timeoutSeconds: limits.agentCallTimeoutSeconds,
  },);
  rl.debug(`guest-exec started with pid ${String(pid,)}`,);
  /**
   Result carrying this launch's marker.
   */
  const result = await attributedResult({
    domain,
    limits,
    marker,
    pid,
  },);
  rl.debug(`command exited with code ${String(result.exitCode,)}`,);
  return result;
}

//endregion Run
