/**
 The checks the passing-token-rate-limit proof makes about both pi runs:
 what each run's feed says it did, and what a reader can act on.

 @module
 */

import {
  assistantMessagesOf,
  assistantText,
  retryWaitsOf,
  type PiRun
} from './verify-token-rate-limit-retried-pi-run.ts';

//region Constants

/**
 Start of the error text this package writes over the provider's own answer.
 */
const RETRIED_ERROR_MESSAGE_START = '429: token rate limit reached';

/**
 Words the provider's own answer carries, which the unchanged run must still
 end on and the stored session must keep.
 */
const PROVIDER_ANSWER_WORDS = 'insufficient_quota';

/**
 Diagnostic type this package writes onto the message it retries,
 which a reader of a stored session finds the provider's answer under.
 */
const DIAGNOSTIC_TYPE = 'qwen-token-plan-individual.passing-token-rate-limit';

/**
 Reply the stand-in gives every chat completion after the run's first.
 */
const REPLY_TEXT = 'pong';

//endregion Constants

//region Types

/**
 One check the proof made: what it measured, and whether it held.
 */
export type ProofCheck = {
  /**
   Whether the check held.
   */
  readonly ok: boolean;
  /**
   What the check measured, in words a reader acts on.
   */
  readonly line: string;
};

/**
 What reading both runs needs.
 */
export type ProofRuns = {
  /**
   The run loading this checkout's bundle.
   */
  readonly retried: PiRun;
  /**
   The run loading the unchanged package's bundle.
   */
  readonly unchanged: PiRun;
  /**
   How many chat completions the stand-in answered over both runs.
   */
  readonly chatRequests: number;
  /**
   Texts of the sessions the retried run stored.
   */
  readonly storedSessions: readonly string[];
};

//endregion Types

//region The checks

/**
 The assistant messages one run ended on, by stop reason.

 @param run - run to read

 @param stopReason - reason the messages must have ended for

 @returns the text of each matching message, in order

 @example
 ```typescript
 const replies = textsOf({ run, stopReason: 'stop' });
 ```
 */
function textsOf({
  run,
  stopReason,
}: {
  readonly run: PiRun;
  readonly stopReason: string;
},): string[] {
  return assistantMessagesOf({ run, },)
    .filter(function endedForReason(message,) {
      return message.stopReason === stopReason;
    },)
    .map(function textOf(message,) {
      return assistantText({ message, },);
    },);
}

/**
 The error texts one run's assistant messages carry.

 @param run - run to read

 @returns each error text the run reported, in order

 @example
 ```typescript
 const errors = errorTextsOf({ run });
 ```
 */
function errorTextsOf({
  run,
}: {
  readonly run: PiRun;
},): string[] {
  return assistantMessagesOf({ run, },)
    .flatMap(function errorOf(message,) {
      /**
       Error text the message carries, which it may hold as anything.
       */
      const {errorMessage} = message;
      return (typeof errorMessage) === 'string' ? [errorMessage,] : [];
    },);
}

/**
 How many diagnostics each of one run's assistant messages carries.

 @param run - run to read

 @returns one count per message,
 which is where the provider's own answer stays

 @example
 ```typescript
 const counts = diagnosticCountsOf({ run });
 ```
 */
function diagnosticCountsOf({
  run,
}: {
  readonly run: PiRun;
},): number[] {
  return assistantMessagesOf({ run, },)
    .flatMap(function diagnosticsOf(message,) {
      /**
       Diagnostics the message carries, which it may hold as anything.
       */
      const {diagnostics} = message;
      return Array.isArray(diagnostics,) ? [diagnostics.length,] : [];
    },);
}

/**
 Build every check the proof makes about both runs.

 @param runs - what both runs left behind

 @returns the checks, in the order they are reported

 @example
 ```typescript
 const checks = checksOf({ runs: { retried, unchanged, chatRequests } });
 ```
 */
export function checksOf({
  runs,
}: {
  readonly runs: ProofRuns;
},): ProofCheck[] {
  /**
   Waits pi announced before its own attempts in the retried run.
   */
  const retriedWaits = retryWaitsOf({ run: runs.retried, },);
  /**
   Error texts the retried run reported.
   */
  const retriedErrors = errorTextsOf({ run: runs.retried, },);
  /**
   Replies the retried run ended with.
   */
  const retriedReplies = textsOf({
    run: runs.retried,
    stopReason: 'stop',
  },);
  /**
   Assistant messages the unchanged run reported.
   */
  const unchangedMessages = assistantMessagesOf({ run: runs.unchanged, },);
  /**
   Error texts the unchanged run reported.
   */
  const unchangedErrors = errorTextsOf({ run: runs.unchanged, },);
  return [
    // The retried run: pi announced its own retry, and the run ended on the
    // reply.
    {
      ok: runs.storedSessions
        .some(function keepsTheProviderAnswer(session,) {
          return session.includes(DIAGNOSTIC_TYPE,)
            && session.includes(PROVIDER_ANSWER_WORDS,);
        },),
      line: `the retried run stored ${String(runs.storedSessions
        .length,)} session file(s), and ${runs.storedSessions
          .some(function keepsTheProviderAnswer(session,) {
        return session.includes(DIAGNOSTIC_TYPE,);
      },) ? 'one carries' : 'none carries'} this package's diagnostic with the provider's own answer`,
    },
    {
      ok: (runs.retried
        .exitCode
        === 0) && (!runs.retried
          .timedOut),
      line: `the retried run exited ${String(runs.retried
        .exitCode,)} (timed out: ${String(runs.retried
          .timedOut,)})`,
    },
    {
      ok: retriedWaits.length > 0,
      line: `the retried run's feed carries ${String(retriedWaits.length,)} auto_retry_start line(s), waits ${retriedWaits.map(String,)
        .join(', ')
        || 'none'}`,
    },
    {
      ok: retriedErrors.some(function retriedText(text,) {
        return text.startsWith(RETRIED_ERROR_MESSAGE_START,);
      },),
      line: `the retried run's failed answer carries this package's retried text, with ${JSON.stringify(diagnosticCountsOf({ run: runs.retried, },),)} diagnostic(s): ${JSON.stringify(retriedErrors,)}`,
    },
    {
      ok: retriedReplies.includes(REPLY_TEXT,),
      line: `the retried run ended with the stand-in's reply: ${JSON.stringify(retriedReplies,)}`,
    },
    // The unchanged run: pi made no attempt, and the run ended on the 429.
    {
      ok: retryWaitsOf({ run: runs.unchanged, },)
        .length
        === 0,
      line: 'the unchanged run\'s feed carries no auto_retry_start line',
    },
    // A feed that already holds every other answer is what a run that never
    // ends also prints, so the run has to have ended by itself as well.
    {
      ok: (!runs.unchanged
        .timedOut) && (runs.unchanged
          .exitCode
          !== undefined),
      line: `the unchanged run ended by itself: exited ${String(runs.unchanged
        .exitCode,)} (timed out: ${String(runs.unchanged
          .timedOut,)})`,
    },
    {
      ok: unchangedMessages.length === 1,
      line: `the unchanged run settled on ${String(unchangedMessages.length,)} assistant message(s)`,
    },
    {
      ok: unchangedErrors.some(function providerWords(text,) {
        return text.includes(PROVIDER_ANSWER_WORDS,);
      },),
      line: `the unchanged run ended on the provider's own body: ${JSON.stringify(unchangedErrors,)}`,
    },
    {
      ok: runs.chatRequests >= 2,
      line: `the stand-in answered ${String(runs.chatRequests,)} chat completion(s)`,
    },
  ];
}

//endregion The checks
