/**
 Moving a run whose provider's quota is spent to the provider the owner named, on the session it already has.

 Synthetic sets no limit on runs at once, so its 429 means the quota is spent and no later attempt on Synthetic can succeed
 (`.agents/skills/pi-as-subagent/SKILL.md`). The owner's rule of 2026-11-10: "Use neuralwatt/kimi-k3.1-pro-flex whenever Synthetic
 429s."

 The move is a new Pi on the same session file, because a provider is chosen when Pi starts (`--provider` and `--model`) and a
 session keeps its own model otherwise: Pi restores the model a session last used only when the command line names none
 (`dist/core/sdk.js:93` in Pi 1.5.2, `if (!model && hasExistingSession && sessionModel)`), so the words given here win. Pi drops an
 assistant message that ended in an error when it builds the next request (`packages/ai/src/api/transform-messages.ts` in
 `earendil-works/pi`), so the resumed session asks for the work that did not finish rather than for a failure.

 @module
 */

import {
  FEED_PREVIEW_CHARACTERS,
  PROVIDER_WHOSE_QUOTA_MOVES_A_RUN,
  QUOTA_FALLBACK_MODEL,
  QUOTA_FALLBACK_PROVIDER,
} from './driver-constants.ts';
import type {
  ConnectionToPi,
} from './connection-to-pi.ts';
import type {
  ProviderAndModel,
  RecordOfPiRun,
  RunFailure,
} from './driver-types.ts';
import {
  type RunPutAway,
  putTheRunAway,
} from './put-the-run-away.ts';
import { preview, } from './previews-of-pi-messages.ts';
import {
  fieldsOf,
  textOf,
} from './read-pi-fields.ts';
import { kindOfModelError, } from './retries-of-a-model-error.ts';

/**
 The provider and model a quota moves a run to.
 */
export const QUOTA_FALLBACK: ProviderAndModel = Object.freeze({
  provider: QUOTA_FALLBACK_PROVIDER,
  model: QUOTA_FALLBACK_MODEL,
},);

/**
 Tells whether this failure leaves the run a provider it can go on with.

 A quota is spent, so another attempt on the same provider reaches the same answer, and only a provider whose quota the owner
 named a fallback for has somewhere to go. Every other failure is answered where it happened, or ends the run.

 @param provider - the provider this Pi runs on

 @param failure - the error Pi settled on, when nothing after it recovered

 @returns whether the run moves to the fallback provider

 @example
 ```ts
 const move = switchWarranted({ provider: 'synthetic', failure: { failed: true, problem: SPENT_QUOTA_TEXT }, },);
 ```
 */
export function switchWarranted({
  provider,
  failure,
}: {
  readonly provider: string;
  readonly failure: RunFailure
},): boolean {
  if (!failure.failed) {
    return false;
  }
  return (provider === PROVIDER_WHOSE_QUOTA_MOVES_A_RUN)
    && (kindOfModelError(failure.problem,) === 'quota-reached');
}

/**
 The prompt a resumed run is given on the provider its quota moved it to.

 It names the failure the move answers, which provider the session now runs on, and that the work goes on rather than starting
 over.

 @param problem - what the model said when its message ended in an error

 @param from - the provider the run left

 @param to - the provider and model the run resumes on

 @returns the prompt for the first turn on the new provider

 @example
 ```ts
 const prompt = promptAfterAProviderSwitch({ problem: SPENT_QUOTA_TEXT, from: 'synthetic', to: QUOTA_FALLBACK, },);
 ```
 */
export function promptAfterAProviderSwitch({
  problem,
  from,
  to,
}: {
  readonly problem: string;
  readonly from: string;
  readonly to: ProviderAndModel
},): string {
  return [
    `The previous attempt ended before it finished, on a quota of the ${from} provider rather than on anything you did.`,
    `The provider said: ${preview({
      value: problem,
      length: FEED_PREVIEW_CHARACTERS.modelError,
    },)}`,
    `This session continues on ${to.provider}/${to.model}.`,
    'Continue the task from where you left off, and do not repeat work that already succeeded.',
  ].join('\n\n',);
}

/**
 What moving a run off a spent quota left: the session it resumes on, or the teardown of the Pi that ended.
 */
export type ProviderMove =
  | {
    readonly moved: true;

    /**
     The provider and model the run resumes on.
     */
    readonly to: ProviderAndModel;

    /**
     The session file the new Pi opens, which is the one the old Pi reported.
     */
    readonly sessionFile: string;

    /**
     How the Pi that ended was put away, which the caller keeps for the summary.
     */
    readonly putAway: RunPutAway
  }
    | {
      readonly moved: false;

    /**
     How the Pi that ended was put away, which the caller keeps for the summary.
     */
      readonly putAway: RunPutAway
    };

/**
 Moves a run whose provider's quota is spent to the fallback provider, on the session it already has.

 The Pi that ended is asked for its statistics, which name the session file, and is then put away within the bounds every teardown
 has. A Pi that cannot be confirmed gone ends the move rather than leaving two Pis on one worktree, and so does one that reports
 no session file: a run with nowhere to resume is the run that ended, not a run that moved.

 @param connection - the Pi that ended on the failure

 @param record - the run's record, which the move is noted on

 @param provider - the provider that Pi ran on

 @param model - the model that Pi ran on

 @returns what the move left

 @example
 ```ts
 const move = await switchProviderOnSpentQuota({ connection, record, provider: 'synthetic', model: 'kimi', },);
 ```
 */
export async function switchProviderOnSpentQuota({
  connection,
  record,
  provider,
  model,
}: {
  readonly connection: ConnectionToPi;
  readonly record: RecordOfPiRun;
  readonly provider: string;
  readonly model: string
},): Promise<ProviderMove> {
  /**
   How the Pi that ended was put away, whose statistics are what name the session.
   */
  const putAway = await putTheRunAway({
    connection,
    record,
  },);
  if (!putAway.stopped) {
    record.note({
      kind: 'problem',
      problem: `the run could not move to ${QUOTA_FALLBACK.provider}/${QUOTA_FALLBACK.model}, because the Pi that ended could not be confirmed gone`,
    },);
    return {
      moved: false,
      putAway,
    };
  }
  /**
   The session file Pi reported, which a new Pi reopens with `--session`.
   */
  const sessionFile = textOf(fieldsOf(putAway.sessionStats,)
    .sessionFile,);
  if (sessionFile === '') {
    record.note({
      kind: 'problem',
      problem: `the run could not move to ${QUOTA_FALLBACK.provider}/${QUOTA_FALLBACK.model}, because the Pi that ended reported no session file to resume`,
    },);
    return {
      moved: false,
      putAway,
    };
  }
  record.note({
    kind: 'provider-switch',
    from: {
      provider,
      model,
    },
    to: QUOTA_FALLBACK,
    sessionFile,
    why: `a spent quota on ${provider}`,
  },);
  return {
    moved: true,
    to: QUOTA_FALLBACK,
    sessionFile,
    putAway,
  };
}
