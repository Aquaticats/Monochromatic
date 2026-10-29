import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type DecisionAnswer,
  type DecisionQuestion,
  type DecisionState,
  isDecisionStateOverContext,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  decisionsCardOf,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import { endsTheRun, } from './stage-run-end.ts';
// TYPE-ONLY AND DELIBERATELY CIRCULAR: `stage-call.ts` calls into this file
// for a decision seat and this file names its voice type. Erased before
// anything runs; the alternative is a third file holding one type alias.
import type { StageVoice, } from './stage-call.ts';

//region Stage decision call
// The typed half of one stage exchange. A stage that can phrase its ballot
// as a question (a choice among candidates, a yes-or-no, a score) hands
// this the state and the questions beside its chat sheet, and a decision
// seat on the bench is asked the question where a chat seat is asked the
// sheet. Both answer into the same `StageVoice`, so the quorum, the tally
// and the ledger never learn which kind of seat spoke.

/**
 One stage's ballot as a typed question, beside its chat sheet.

 @example
 ```ts
 const decision: StageDecision = { state: { candidates: { '1': 'The cat naps.', }, }, questions: { best: { type: 'choice', instructions: 'Which is best?', criteria: { '0': 'none', '1': 'candidate 1', }, }, }, read: function read(answers,) { return { best: '1', reason: 'typed', }; }, };
 ```
 */
export type StageDecision = {
  /**
   What the questions are about, the same evidence the chat sheet carries.
   */
  readonly state: DecisionState;

  /**
   Questions by name.
   */
  readonly questions: Readonly<Record<string, DecisionQuestion>>;

  /**
   Reads the answers into the shape the stage's own guard admits; a reading
   the guard refuses is a lost voice, as a chat reply the guard refuses is.
   */
  readonly read: (answers: Readonly<Record<string, DecisionAnswer>>,) => unknown;
};

/**
 Runs one typed-decision exchange for a pipeline stage.

 @param client - injected model client, asked through its `decide`

 @param modelId - decision seat to ask

 @param decision - stage's question, absent when the stage has none

 @param signal - caller abort honored by the exchange

 @param exchangeTimeoutMs - deadline for the exchange

 @param validate - stage's own guard over the read answer

 @param stage - stage label for logging

 @param l - logger of the calling stage

 @returns Voice as data; a lost voice never throws unless the caller aborted
 or every provider read dry

 @throws The call's error, once the caller aborted

 @throws `EveryProviderDryError` when the client's decision call meets the router refusing every provider, which ends
 the run at the owner's instruction. THE SHIPPED DECISIONS ROUTE DOES NOT RAISE IT (`provider-router-decide.ts` raises
 `NoProviderForModelError` when OpenRouter reads dry, which stays a lost, unreachable seat), so this holds for a
 decision call that does, such as one behind the pass guard; whether a dry decisions endpoint should end the run is
 undecided.

 @example
 ```ts
 const voice = await attemptDecisionCall({ client, modelId, decision, signal, exchangeTimeoutMs, validate, stage, l, },);
 ```
 */
export async function attemptDecisionCall<ValueT,>(
  {
    client,
    modelId,
    decision,
    signal,
    exchangeTimeoutMs,
    validate,
    stage,
    l,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelId: RosterModelId;
    readonly decision?: StageDecision;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly validate: (value: unknown,) => value is ValueT;
    readonly stage: string;
    readonly l: Logger;
  }>,
): Promise<StageVoice<ValueT>> {
  /**
   Typed exchange the client offers, absent on a client with no decisions
   transport.
   */
  const { decide, } = client;
  // A SEAT NO ROUND CAN ASK IS OUT OF REACH (ledger P13, 2026-09-28), as a
  // seat the router refuses is: marked reachable, it sized the quorum as a
  // voice that could come and was re-asked every retry round, to the same
  // loss. Both cases are latent (no run log holds either line), since only
  // the select benches seat a decision seat and every select stage threads
  // its question.
  if (decision === undefined) {
    l.warn(`${stage} ${modelId}: a decision seat asked a stage with no typed question, seat out of reach`,);
    return {
      heard: false,
      answered: false,
      unreachable: true,
    };
  }
  if (decide === undefined) {
    l.warn(`${stage} ${modelId}: a decision seat with no decisions client, seat out of reach`,);
    return {
      heard: false,
      answered: false,
      unreachable: true,
    };
  }
  try {
    /**
     Answers as the endpoint gave them.
     */
    const reply = await decide({
      modelId,
      state: decision.state,
      questions: decision.questions,
      signal,
      exchangeTimeoutMs,
    },);

    /**
     Answers read into the stage's own shape.
     */
    const value = decision.read(reply.answers,);
    if (!validate(value,)) {
      l.warn(`${stage} ${modelId}: typed answer failed the stage guard, voice lost`,);
      return {
        heard: false,
        answered: true,
        unreachable: false,
        // A typed answer is never cut at a length limit: the endpoint
        // answered every question, and the stage's guard refused the reading.
        unreadable: 'off-shape',
      };
    }
    return {
      heard: true,
      value,
    };
  }
  catch (error) {
    // Aborts must always win so user steering can stop a fan-out.
    if (signal.aborted)
      throw error;

    // Ends the run at the owner's instruction, so it is not this seat's lost voice (`attemptStageCall` says why).
    if (endsTheRun({ error, },))
      throw error;

    if (isDecisionStateOverContext({ error, },)) {
      // THE BALLOT, NOT THE SEAT, IS PAST REACH (ledger P13): the endpoint
      // refuses a state longer than the seat's context, so this gather sizes
      // its quorum without the seat and never re-asks it, and the next
      // ballot asks it again.
      /**
       State as the wire carried it, measured for the log line.
       */
      const stateJson = JSON.stringify(decision.state,);

      /**
       Seat's context in tokens, as its card records it.
       */
      const { contextLength, } = decisionsCardOf({ modelId, },);
      l.warn(
        `${stage} ${modelId}: the endpoint refused a state of ${String(stateJson.length,)} ${
          wordForCount({
            count: stateJson.length,
            one: 'character',
            many: 'characters',
          },)
        } as past `
          + `the seat's ${String(contextLength,)}-token context, seat out of reach for this ballot`,
      );
      return {
        heard: false,
        answered: false,
        unreachable: true,
      };
    }

    /**
     Whether the router refused the call because the decisions endpoint
     reads dry, which is a fact about the bench rather than the exchange.
     */
    const unreachable = error instanceof NoProviderForModelError;
    l.warn(`${stage} ${modelId}: ${exchangeFailureLogText({ error, },)}, ${unreachable ? 'seat unreachable' : 'voice lost'}`,);
    return {
      heard: false,
      answered: false,
      unreachable,
    };
  }
}

//endregion Stage decision call
