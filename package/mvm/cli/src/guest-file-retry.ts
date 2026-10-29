/**
 Repeats a guest agent file command that got no answer.

 A guest under load can leave the agent silent for minutes: on 2026-10-05 a
 Windows guest stopped answering during a 64 MiB push for at least 2 minutes
 and 52 seconds, and the push failed because it had asked only three times in
 25 seconds. A command is therefore repeated until the agent answers, the
 domain is gone, or the silence has lasted the limit.

 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import {
  MS_PER_SECOND,
} from '@monochromatic-dev/module-const/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { GuestAgentUnreachableError, } from './agent-command.ts';
import { domainPresence, } from './guest-exec-status.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Limits

/**
 Time limits of a file transfer through the guest agent.

 @example
 ```ts
 const limits: GuestFileTransferLimits = { ...DEFAULT_GUEST_FILE_TRANSFER_LIMITS, unresponsiveLimitMs: 60_000 };
 ```
 */
export type GuestFileTransferLimits = {
  /**
   Milliseconds between two tries of a command that got no answer.
   */
  readonly retryPauseMs: number;

  /**
   Milliseconds the agent may stay silent before the transfer gives up.
   */
  readonly unresponsiveLimitMs: number;
};

/**
 Limits used unless a caller passes others: five minutes of silence, the same
 as a running guest command gets, and one second between tries.
 */
export const DEFAULT_GUEST_FILE_TRANSFER_LIMITS: GuestFileTransferLimits = {
  retryPauseMs: 1_000,
  unresponsiveLimitMs: 300_000,
};

/**
 Where and how long a transfer keeps asking.

 @example
 ```ts
 const patience: Patience = { domain: 'mvm-dev', limits: DEFAULT_GUEST_FILE_TRANSFER_LIMITS };
 ```
 */
export type Patience = {
  /**
   Prefixed libvirt domain name, asked for its state after an unanswered command.
   */
  readonly domain: string;

  /**
   Time limits of the transfer.
   */
  readonly limits: GuestFileTransferLimits;
};

//endregion Limits

//region Error

/**
 The guest agent stopped answering file commands for good:
 the domain is gone, or the silence lasted the limit.

 @example
 ```ts
 try {
   await answered({ patience, run: () => file.read(1024) });
 }
 catch (error) {
   if (error instanceof GuestAgentSilentError) console.error(error.message);
 }
 ```
 */
export class GuestAgentSilentError extends Error {
  /**
   @param cause - Last command that got no answer, kept for its text

   @param reason - What was observed, in a sentence ending without punctuation
   */
  constructor({
    cause,
    reason,
  }: {
    readonly cause: GuestAgentUnreachableError;
    readonly reason: string;
  },) {
    super(
      `${reason}. Last command without an answer: ${cause.message}`,
      { cause, },
    );
    this.name = 'GuestAgentSilentError';
  }
}

//endregion Error

//region Attempts

/**
 One try that got no answer from the guest agent, with what libvirt says about the domain afterwards.
 */
type Unanswered = {
  /**
   Whether the domain can no longer answer at all.
   */
  readonly gone: boolean;

  /**
   Domain state libvirt reported after the try.
   */
  readonly state: string;

  /**
   Failure of the try.
   */
  readonly unanswered: GuestAgentUnreachableError;
};

/**
 Runs one try and turns "no answer from the agent" into a value, so the caller can ask again.
 Any other failure propagates.

 @param domain - Prefixed libvirt domain name, asked for its state after an unanswered try

 @param run - Try to make

 @returns The try's result, or the unanswered marker with the domain's state
 */
async function attempt<const Result,>({
  domain,
  run,
}: {
  readonly domain: string;
  readonly run: () => Promise<Result>;
},): Promise<Result | Unanswered> {
  /**
   Logger scoped to the try so a repeated command is attributable.
   */
  const rl = tagged({
    tag: attempt.name,
    l,
  },);
  try {
    return await run();
  }
  catch (error) {
    if (!(error instanceof GuestAgentUnreachableError))
      throw error;

    rl.info(`a file command got no answer from the guest agent: ${error.message}`,);
    return {
      ...(await domainPresence(domain,)),
      unanswered: error,
    };
  }
}

/**
 Endless series of tries at one command, one at a time, with a pause before each repeat.
 The consumer ends the series.

 @param patience - Domain and time limits

 @param run - Try to make; told whether an earlier try went unanswered, so it can restore the file position first

 @returns Outcome of each try in order
 */
async function* attempts<const Result,>({
  patience,
  run,
}: {
  readonly patience: Patience;
  readonly run: (repeated: boolean,) => Promise<Result>;
},): AsyncGenerator<Result | Unanswered> {
  /**
   Makes one try, after the pause when it repeats an unanswered one.

   @param repeated - Whether an earlier try went unanswered, which leaves the file position unknown

   @returns Outcome of the try
   */
  async function tryOnce(repeated: boolean,): Promise<Result | Unanswered> {
    if (repeated) {
      await wait(patience.limits
        .retryPauseMs,);
    }
    return await attempt({
      domain: patience.domain,
      run: function runOnce() {
        return run(repeated,);
      },
    },);
  }
  yield tryOnce(false,);
  // The consumer leaves the loop on an answer, a gone domain, or the silence limit.
  for (;;) {
    yield tryOnce(true,);
  }
}

/**
 Checks whether a try's outcome is the unanswered marker.

 @param outcome - Outcome of one try

 @returns Whether the try got no answer
 */
function isUnanswered(outcome: unknown,): outcome is Unanswered {
  return ((typeof outcome) === 'object') && (outcome !== null)
    && ('unanswered' in outcome);
}

/**
 Sentinel for "the agent has answered every command so far".
 A unique symbol models the empty state without a nullish union.
 */
const NOT_SILENT: unique symbol = Symbol('set before any try went unanswered',);

/**
 Makes a repeatable step until the guest agent answers.

 @param patience - Domain and time limits

 @param run - Step to make; told whether an earlier try went unanswered, so it can restore the file position first

 @returns The step's result

 @throws {@link GuestAgentSilentError} when the domain is gone or the agent stayed silent for the limit

 @example
 ```ts
 const written = await answered({ patience, run: (repeated) => writeAt({ data, file, offset, reposition: repeated }) });
 ```
 */
export async function answered<const Result,>({
  patience,
  run,
}: {
  readonly patience: Patience;
  readonly run: (repeated: boolean,) => Promise<Result>;
},): Promise<Result> {
  /**
   Logger scoped to the step so the wait is visible while it lasts.
   */
  const rl = tagged({
    tag: answered.name,
    l,
  },);
  /**
   Time the current silence began, set at the first try that went unanswered.
   */
  const silence: { since: number | typeof NOT_SILENT; } = { since: NOT_SILENT, };
  for await (
    const outcome of attempts({
      patience,
      run,
    },)
  ) {
    if (!isUnanswered(outcome,)) {
      return outcome;
    }
    if (outcome.gone) {
      throw new GuestAgentSilentError({
        cause: outcome.unanswered,
        reason: `The domain ${patience.domain} is now ${outcome.state}`,
      },);
    }
    /**
     Time of this unanswered try.
     */
    const now = performance.now();
    if (silence.since === NOT_SILENT) {
      silence.since = now;
    }
    /**
     Whole seconds the agent has been silent, counted from the first unanswered try.
     */
    const silentSeconds = Math.round((now - silence.since) / MS_PER_SECOND,);
    if ((now - silence.since)
      >= patience.limits
        .unresponsiveLimitMs) {
      throw new GuestAgentSilentError({
        cause: outcome.unanswered,
        reason: `The guest agent in ${patience.domain} has not answered a file command for ${
          String(silentSeconds,)
        } seconds, while the domain is ${outcome.state}`,
      },);
    }
    rl.info(
      `asking the guest agent in ${patience.domain} again; it has been silent for ${String(silentSeconds,)} of at most ${
        String(Math.round(patience.limits
          .unresponsiveLimitMs
          / MS_PER_SECOND,),)
      } seconds`,
    );
  }
  throw new Error('the series of tries ended without an outcome, which it never does',);
}

//endregion Attempts
