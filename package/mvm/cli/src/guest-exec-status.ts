/**
 Status requests for one guest process started through the QEMU guest agent.

 Each request is classified: the process still runs, it finished with a
 result, the agent no longer knows it, the request got no answer, or the
 domain is gone. The series of requests is strictly one at a time.

 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  agentCommand,
  GuestAgentAnswerTooLargeError,
  GuestAgentProtocolError,
  GuestAgentReplyError,
  GuestAgentUnreachableError,
} from './agent-command.ts';
import { decodeBase64, } from './exec-shell.ts';
import {
  type DiscardedGuestResult,
  GuestExecOutputTooLargeError,
} from './guest-exec-errors.ts';
import type { GuestExecLimits, } from './guest-exec.ts';
import { virsh, } from './virsh.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Shapes

/**
 What one status request found.

 @example
 ```ts
 const read: StatusRead = { kind: 'running' };
 ```
 */
export type StatusRead =
  | {
    readonly kind: 'exited';
    readonly result: DiscardedGuestResult;
    readonly truncated: boolean;
  }
    | {
      readonly kind: 'forgotten';
      readonly error: GuestAgentReplyError;
    }
    | {
      readonly kind: 'gone';
      readonly error: GuestAgentUnreachableError;
      readonly state: string;
    }
    | { readonly kind: 'running'; }
    | {
      readonly kind: 'unanswered';
      readonly error: GuestAgentUnreachableError;
    };

/**
 Pause control shared by the status loop and the request it paces.

 @example
 ```ts
 const pacing: Pacing = { pauseBeforeNextRead: false };
 ```
 */
export type Pacing = {
  /**
   Whether the next status request waits one poll interval first.
   */
  pauseBeforeNextRead: boolean;
};

//endregion Shapes

//region Interpreting an answer

/**
 Exit status reported for a command a signal ended: the shell convention of 128 plus the signal number.
 */
const SIGNAL_EXIT_BASE = 128;

/**
 Reads one optional property of an agent answer.

 @param key - Property name from the protocol reference

 @param of - Agent answer object

 @returns Property value, or `undefined` when the answer has no such property

 @example
 ```ts
 property({ key: 'exitcode', of: { exited: true, exitcode: 3 } }); // => 3
 ```
 */
function property({
  key,
  of,
}: {
  readonly key: string;
  readonly of: object;
},): unknown {
  return Reflect.get(
    of,
    key,
  );
}

/**
 Decodes one optional base64 output stream of a status answer.

 @param key - `out-data` or `err-data`

 @param of - Status answer object

 @returns Decoded text, `''` when the command wrote nothing to that stream

 @example
 ```ts
 stream({ key: 'out-data', of: { 'out-data': 'aGk=' } }); // => 'hi'
 ```
 */
function stream({
  key,
  of,
}: {
  readonly key: string;
  readonly of: object;
},): string {
  /**
   Encoded stream; the agent leaves the property out when nothing was captured.
   */
  const encoded = property({
    key,
    of,
  },);
  return ((typeof encoded) === 'string') ? decodeBase64(encoded,) : '';
}

/**
 Resolves the exit status of a finished process from its status answer.

 @param answer - Status answer object of a finished process

 @returns Exit status, 128 plus the signal number for a signalled process, or 0 when the agent sent neither

 @example
 ```ts
 exitCodeOf({ exited: true, signal: 9 }); // => 137
 ```
 */
function exitCodeOf(answer: object,): number {
  /**
   Exit status; absent when a signal ended the process.
   */
  const exitcode = property({
    key: 'exitcode',
    of: answer,
  },);
  if ((typeof exitcode) === 'number')
    return exitcode;
  /**
   Signal that ended the process; absent on a normal exit.
   */
  const signal = property({
    key: 'signal',
    of: answer,
  },);
  if ((typeof signal) === 'number')
    return SIGNAL_EXIT_BASE + signal;
  return 0;
}

/**
 Interprets a `guest-exec-status` answer.

 @param answer - Value the agent returned

 @returns Whether the process still runs, or its finished result

 @throws {@link GuestAgentProtocolError} when the answer has no boolean `exited`

 @example
 ```ts
 interpretStatus({ exited: true, exitcode: 0 }); // => { kind: 'exited', ... }
 ```
 */
function interpretStatus(answer: unknown,): Extract<StatusRead, { kind: 'exited' | 'running'; }> {
  if (((typeof answer) !== 'object') || (answer === null)) {
    throw new GuestAgentProtocolError({
      commandName: 'guest-exec-status',
      received: JSON.stringify(answer,),
    },);
  }
  /**
   Whether the process finished; the only property the protocol always sends.
   */
  const exited = property({
    key: 'exited',
    of: answer,
  },);
  if ((typeof exited) !== 'boolean') {
    throw new GuestAgentProtocolError({
      commandName: 'guest-exec-status',
      received: JSON.stringify(answer,),
    },);
  }
  if (!exited) {
    return { kind: 'running', };
  }
  /**
   Whether the agent dropped part of standard output for exceeding its capture limit.
   */
  const outTruncated = property({
    key: 'out-truncated',
    of: answer,
  },);
  /**
   Whether the agent dropped part of standard error for exceeding its capture limit.
   */
  const errTruncated = property({
    key: 'err-truncated',
    of: answer,
  },);
  return {
    kind: 'exited',
    result: {
      exitCode: exitCodeOf(answer,),
      stderr: stream({
        key: 'err-data',
        of: answer,
      },),
      stdout: stream({
        key: 'out-data',
        of: answer,
      },),
    },
    truncated: (outTruncated === true) || (errTruncated === true),
  };
}

//endregion Interpreting an answer

//region Domain state

/**
 Domain states in which a started command cannot still report a result.
 */
const GONE_STATES: ReadonlySet<string> = new Set([
  'shut off',
  'crashed',
],);

/**
 State text used when libvirt cannot report the domain's state at all.
 */
const STATE_UNKNOWN_TO_LIBVIRT = 'unknown to libvirt';

/**
 Asks libvirt for the domain's state after a status request got no answer.

 @param domain - Prefixed libvirt domain name

 @returns State name, or {@link STATE_UNKNOWN_TO_LIBVIRT} when libvirt cannot report it

 @example
 ```ts
 await domainState('mvm-dev'); // => 'running'
 ```
 */
async function domainState(domain: string,): Promise<string> {
  /**
   Logger scoped to the state query so its failure is attributable.
   */
  const rl = tagged({
    tag: domainState.name,
    l,
  },);
  try {
    return await virsh({
      args: [
        'domstate',
        domain,
      ],
    },);
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.debug(`libvirt cannot report the state of ${domain}: ${error.message}`,);
    return STATE_UNKNOWN_TO_LIBVIRT;
  }
}

/**
 Says whether a domain whose guest agent gave no answer can still answer later.
 A domain that is shut off, crashed, or unknown to libvirt cannot.

 @param domain - Prefixed libvirt domain name

 @returns Domain state, and whether that state rules out any later answer

 @example
 ```ts
 const { gone, state } = await domainPresence('mvm-dev'); // => { gone: false, state: 'running' }
 ```
 */
export async function domainPresence(domain: string,): Promise<{
  readonly gone: boolean;
  readonly state: string;
}> {
  /**
   State libvirt reports for the domain now.
   */
  const state = await domainState(domain,);
  return {
    gone: GONE_STATES.has(state,) || (state === STATE_UNKNOWN_TO_LIBVIRT),
    state,
  };
}

//endregion Domain state

//region Requests

/**
 Makes one status request for the process, after the pause the loop asked for.

 @param domain - Prefixed libvirt domain name

 @param limits - Time limits of the run

 @param pacing - Pause control; read here, set by the status loop

 @param pid - Guest process ID to ask about

 @returns What the request found

 @example
 ```ts
 const read = await readStatus({ domain: 'mvm-dev', limits, pacing: { pauseBeforeNextRead: false }, pid: 42 });
 ```
 */
async function readStatus({
  domain,
  limits,
  pacing,
  pid,
}: {
  readonly domain: string;
  readonly limits: GuestExecLimits;
  readonly pacing: Readonly<Pacing>;
  readonly pid: number;
},): Promise<StatusRead> {
  /**
   Logger scoped to one status request so unanswered requests are attributable.
   */
  const rl = tagged({
    tag: readStatus.name,
    l,
  },);
  if (pacing.pauseBeforeNextRead) {
    await wait(limits.pollIntervalMs,);
  }
  try {
    return interpretStatus(
      await agentCommand({
        domain,
        execute: 'guest-exec-status',
        parameters: { pid, },
        timeoutSeconds: limits.agentCallTimeoutSeconds,
      },),
    );
  }
  catch (error) {
    if (error instanceof GuestAgentReplyError) {
      rl.debug(`the agent in ${domain} no longer knows guest process ${String(pid,)}`,);
      return {
        error,
        kind: 'forgotten',
      };
    }
    if (error instanceof GuestAgentAnswerTooLargeError) {
      // The agent has answered and thereby dropped the result; asking again would only find the process ID unknown.
      rl.debug(`the result of guest process ${String(pid,)} in ${domain} is too large for libvirt to hand over`,);
      throw new GuestExecOutputTooLargeError({
        cause: error,
        domain,
        pid,
      },);
    }
    if (!(error instanceof GuestAgentUnreachableError))
      throw error;

    rl.debug(`status request for guest process ${String(pid,)} in ${domain} got no answer`,);
    /**
     Domain state after the unanswered request; a domain that is gone ends the wait at once.
     */
    const {
      gone,
      state,
    } = await domainPresence(domain,);
    if (gone) {
      return {
        error,
        kind: 'gone',
        state,
      };
    }
    return {
      error,
      kind: 'unanswered',
    };
  }
}

/**
 Endless series of status requests for one process, one at a time:
 each request starts only after the previous one was consumed.

 @param domain - Prefixed libvirt domain name

 @param limits - Time limits of the run

 @param pacing - Pause control the consumer sets between requests

 @param pid - Guest process ID to ask about

 @returns Status reads in order

 @example
 ```ts
 for await (const read of statusReads({ domain, limits, pacing, pid })) {
   if (read.kind === 'exited') break;
 }
 ```
 */
export async function* statusReads({
  domain,
  limits,
  pacing,
  pid,
}: {
  readonly domain: string;
  readonly limits: GuestExecLimits;
  readonly pacing: Readonly<Pacing>;
  readonly pid: number;
},): AsyncGenerator<StatusRead> {
  // Every request is consumed by a loop that returns or throws; the consumer bounds the series.
  for (;;) {
    yield readStatus({
      domain,
      limits,
      pacing,
      pid,
    },);
  }
}

//endregion Requests
