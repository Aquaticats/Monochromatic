import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { DeepReadonlyData, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import {
  BedrockModelNotServedError,
  type ChatJsonOutcome,
  ClaudeModelNotServedError,
  ClaudePrefillError,
  ClaudeSchemaKeywordError,
  DeepSeekModelNotServedError,
  EmptyConversationError,
  type JsonSchemaResponseFormat,
  type ModelClient,
  ModelNotServedError,
  NeuralWattModelNotServedError,
  OpenRouterModelNotServedError,
  SyntheticModelNotServedError,
  UnnameableToolError,
  XiaomiModelNotServedError,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  isDecisionSeat,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import { endsTheRun, } from './stage-run-end.ts';
import {
  type UnreadableCause,
  unreadableCauseOf,
} from './recovery-nudge.ts';
import {
  attemptDecisionCall,
  type StageDecision,
} from './stage-decision-call.ts';

//region Stage call
// One guarded model exchange for pipeline stages: non-ok outcomes and
// transport failures become an absent voice (the ensemble tolerates missing
// panelists and critics), while caller aborts always propagate so user
// steering wins, and so does the router's every-provider-dry refusal, which
// the owner ruled ends the run (`EveryProviderDryError`). The client's
// transport-level retries have already run by the time this helper sees a
// failure.

/**
 One stage voice as data:
 either the model's validated reply, or the recorded loss of that voice.

 @example
 ```ts
 const voice: StageVoice<CriticReportWire> = { heard: false, answered: false, };
 ```
 */
export type StageVoice<ValueT,> =
  | {
    /**
     Reply arrived and validated.
     */
    readonly heard: true;

    /**
     Validated reply value.
     */
    readonly value: ValueT;
  }
    | {
    /**
     Voice lost to a transport failure, a grace abandonment, an unfilled
     roster position or a refused seat: no answer arrived.
     */
      readonly heard: false;

    /**
     No answer arrived, so the recovery round does not re-ask it; see the
     answered variant for why the two losses are kept apart.
     */
      readonly answered: false;

    /**
     Whether no wet provider served the seat at all.

     THE ONE LOSS THAT SAYS SOMETHING ABOUT THE BENCH RATHER THAN THE MODEL.
     `judgeSeatsFor` keeps a seat on the bench when every provider serving
     it reads dry, and the router refuses the call in the same millisecond as
     `NoProviderForModelError`. The owner's decision of 2026-09-09
     (`translation-repair-short-bench-share.md`) sizes the selection
     minimum by the seats a provider could serve, so the stage has to tell
     this refusal from a transport that failed on a wet provider: the second
     is a lost voice on a bench that stands, the first a bench that is
     smaller than it looks.
     */
      readonly unreachable: boolean;
    }
    | {
    /**
     Voice lost to an answer that arrived and nothing could read.
     */
      readonly heard: false;

    /**
     An answer arrived.

     SEPARATES THE TWO LOSSES THAT LOOK ALIKE AND ARE NOT. `ChatJsonOutcome`
     has three kinds and only `ok` is usable, so a non-`ok` outcome carries
     `rawText` and means an answer arrived, whole or cut at the length limit.
     A thrown failure, a grace abandonment and an unfilled roster position all
     mean none did.

     `stage-quorum.ts` re-asks on exactly this: an answer that arrived is
     worth one fresh call, nudged by what happened to it, and a model still
     thinking is worth a second timeout on the critical path to nobody.
     */
      readonly answered: true;

    /**
     A seat that answered was reached.
     */
      readonly unreachable: false;

    /**
     Why nothing could read it, which picks the recovery round's wording
     (ledger P10): a reply the length limit cut is told it ran out of room,
     not that its shape was wrong.
     */
      readonly unreadable: UnreadableCause;
    };

/**
 Longest model text a lost-voice warning carries, counted in grapheme
 clusters so the bound cannot cut a character in half.

 A parse failure is almost always diagnosable from the OPENING characters:
 the Kimi-K3 outage was a two-character channel marker, and 507 mismatches in
 one pass were explained by it. Its 2026-08-13 recurrence was explained the
 same way, from `p|>` and `ep|>` recorded here.
 */
const RAW_PREVIEW_CHARS = 120;

/**
 Bounded opening of a reply's text, on one line, for a warning that names why a reply was lost.

 @param rawText - verbatim model text

 @returns The first grapheme clusters of the text with its line breaks replaced by spaces

 @example
 ```ts
 const opening = openingOf({ rawText: '{"purr":\n"loud"}', },);
 ```
 */
function openingOf({ rawText, }: { readonly rawText: string; },): string {
  /**
   Model text on one line.

   Both line terminators are replaced, not just the newline: a reply using
   carriage returns would otherwise break the one-line guarantee this exists
   to keep.
   */
  const flattened = rawText
    .replaceAll(
    '\r\n',
    ' ',
    )
    .replaceAll(
    '\n',
    ' ',
    )
    .replaceAll(
    '\r',
    ' ',
    );

  /**
   Grapheme clusters of the flattened text, in order.

   Segmented rather than sliced because the bound falls in the middle of real
   model output. `slice` counts UTF-16 code units and can cut a surrogate pair
   in half; iterating code points fixes that and still splits an emoji or a
   combining mark from its base. The preview is the whole diagnostic here, so
   it has to survive the cut intact.

   Marked foreign because this is where host-owned mutable state enters: the
   segmenter hands back its own records, whose fields the host declares
   writable, and nothing here owns or mutates them.
   */
  const clusters: readonly DeepReadonlyData<Pick<Intl.SegmentData, 'segment'>>[] = [
    ...new Intl.Segmenter(
      undefined,
      { granularity: 'grapheme', },
    )
      .segment(flattened,),
  ];

  /**
   Bounded opening of the model text, since a warning is one line.
   */
  const opening = clusters
    .slice(
    0,
    RAW_PREVIEW_CHARS,
    )
    .map(function toText(cluster,): string {
      return cluster.segment;
    },)
    .join('',);
  return opening;
}

/**
 Names WHY a voice was lost, not merely that one was.

 `schema-mismatch` covers three different faults needing three different
 fixes, truncated thinking, unparseable content and a rejected guard, and the
 client already distinguishes them in `detail`. That distinction was logged at
 DEBUG and discarded here, so a run recorded `schema-mismatch, voice lost`
 hundreds of times while saying nothing about which fault it was.

 WHEN THE RE-ASK ON ANOTHER PROVIDER FAILED TOO, its failure is named beside the
 first answer's (the lead's ruling of 2026-10-15 on the run of TianqiChen66625,
 where only the first was logged and a label written in a form no guard accepted
 read as one failed reply).

 @param outcome - non-ok exchange outcome

 @returns Cause with its sub-kind, plus a bounded opening of the model text, and the re-ask's own when it failed too

 @example
 ```ts
 lostVoiceCause({ outcome, },);
 ```
 */
function lostVoiceCause(
  {
    outcome,
  }: {
    readonly outcome: Exclude<ChatJsonOutcome<unknown>, { readonly kind: 'ok'; }>;
  },
): string {
  /**
   The first answer's failure.
   */
  const first = (outcome.kind === 'schema-mismatch')
    ? `schema-mismatch (${outcome.detail}) raw=${JSON.stringify(openingOf({ rawText: outcome.rawText, },),)}`
    : `refusal-shaped (${outcome.marker}) raw=${JSON.stringify(openingOf({ rawText: outcome.rawText, },),)}`;
  /**
   How the re-ask failed too, as a list of zero or one.
   */
  const [reask,] = (outcome.reaskFailure === undefined) ? [] : [outcome.reaskFailure,];
  return (reask === undefined)
    ? first
    : `${first}; the re-ask on ${reask.servedBy} failed too: ${reask.kind} (${reask.cause}) raw=${
      JSON.stringify(openingOf({ rawText: reask.rawText, },),)
    }`;
}

/**
 The classes of error that are a fault in the request our own code built, each thrown from local state before any
 exchange is made.

 Four are the Claude route refusing the request: a schema keyword it cannot send, a conversation that ends on an
 assistant turn, a conversation that holds no turn to answer, and a seat that is no Claude card. Eight more are
 thrown by the same kind of local lookup or build: the seven clients other than Claude's each refuse a seat their
 roster lookup does not serve (`servedIdFor`, or `syntheticServes` for Synthetic) before the request body is built,
 and `UnnameableToolError` refuses the name the stage's own schema carries. None of them reads a provider's answer,
 and each recurs on every call of the stage until the code or the roster wiring is changed.
 */
const REQUEST_FAULT_CLASSES: readonly (new (...parameters: never) => Error)[] = [
  ClaudeSchemaKeywordError,
  ClaudePrefillError,
  EmptyConversationError,
  ClaudeModelNotServedError,
  BedrockModelNotServedError,
  DeepSeekModelNotServedError,
  ModelNotServedError,
  NeuralWattModelNotServedError,
  OpenRouterModelNotServedError,
  SyntheticModelNotServedError,
  XiaomiModelNotServedError,
  UnnameableToolError,
];

/**
 Whether an error is a request our own code built wrong, refused before any exchange was made.

 The twelve classes are listed with `REQUEST_FAULT_CLASSES`. Each was classified by its throw site, and the rule is
 that a class thrown only from local state is a fault of the request, while a class that can come from a live answer,
 or from the data one document carries, is a lost voice.

 What stays a lost voice, and why. `ProviderHttpError` and every other transport failure are a provider's answer or
 a connection that dropped; a deadline, an idle stream and a runaway or degenerate stream are a model or a network
 being slow or wrong; `NoProviderForModelError` is a fact about which providers the bench holds wet, and the call
 reports it as an unreachable seat. `MalformedImageUriError` is thrown by the request builder from the data URI of
 one picture. `encodeImageAsset` refuses a zero-byte file with its own error before a URI exists, so the URIs the
 stages build from a document's bytes always carry a payload; a URI that is malformed in another way came from a
 caller's own content part, and ending the round would stop the seats that carry it correctly. It recurs for that
 caller and not for the stage, so the seat's voice is lost for the call and the warning names it.

 @param error - caught value, of unknown type by construction

 @returns Whether the error is one of the twelve classes

 @example
 ```ts
 if (isRequestFault({ error, },))
   throw error;
 ```
 */
export function isRequestFault({ error, }: { readonly error: unknown; },): boolean {
  return REQUEST_FAULT_CLASSES.some(function isInstance(faultClass,): boolean {
    return error instanceof faultClass;
  },);
}

/**
 Runs one schema-validated exchange for a pipeline stage.

 @param client - injected model client

 @param modelId - model to call

 @param messages - prompt for the exchange

 @param signal - caller abort honored by the exchange

 @param exchangeTimeoutMs - deadline armed inside the per-model slot

 @param maxAnswerChars - bound on one answer, when the caller knows how
 large its own input was

 @param responseFormat - structured-output constraint

 @param validate - client-side schema guard

 @param stage - stage label for logging

 @param l - logger of the calling stage

 @param decision - the stage's ballot as a typed question, for a decision
 seat on the bench; absent on a stage that has none, which loses that
 seat's voice rather than asking it for prose

 @returns Voice as data; a lost voice never throws unless the caller aborted, the route refused the request as
 malformed, or every provider read dry

 @throws Whatever the caller's abort carried, once the caller aborted

 @throws `EveryProviderDryError` when the router refused the call because no provider has budget left, which ends the
 run at the owner's instruction and so is never recorded as one seat's lost voice

 @throws {@link ClaudeSchemaKeywordError} when the response format carries a keyword the Claude route cannot send

 @throws {@link ClaudePrefillError} when the conversation ends on an assistant turn

 @throws {@link EmptyConversationError} when the conversation holds no turn to answer

 @throws {@link ClaudeModelNotServedError} when the seat is no Claude card

 @throws {@link BedrockModelNotServedError}, {@link DeepSeekModelNotServedError}, {@link ModelNotServedError},
 {@link NeuralWattModelNotServedError}, {@link OpenRouterModelNotServedError}, {@link SyntheticModelNotServedError}
 or {@link XiaomiModelNotServedError} when the seat is handed to a client that does not serve it

 @throws {@link UnnameableToolError} when the schema's own name is one the protocol rejects

 @example
 ```ts
 const voice = await attemptStageCall({ ..., stage: 'critic', l, },);
 ```
 */
export async function attemptStageCall<ValueT,>(
  {
    client,
    modelId,
    messages,
    signal,
    exchangeTimeoutMs,
    maxAnswerChars,
    responseFormat,
    validate,
    stage,
    l,
    decision,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelId: RosterModelId;
    readonly messages: readonly ChatMessage[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly maxAnswerChars?: number;
    readonly responseFormat: JsonSchemaResponseFormat;
    readonly validate: (value: unknown,) => value is ValueT;
    readonly stage: string;
    readonly l: Logger;
    readonly decision?: StageDecision;
  }>,
): Promise<StageVoice<ValueT>> {
  // A DECISION SEAT NEVER SEES THE SHEET: it is asked the stage's typed
  // question, or loses its voice where the stage has none (2026-09-18).
  if (isDecisionSeat({ modelId, },)) {
    return await attemptDecisionCall({
      client,
      modelId,
      // Conditional spread keeps the question absent instead of undefined.
      ...((decision === undefined) ? {} : { decision, }),
      signal,
      exchangeTimeoutMs,
      validate,
      stage,
      l,
    },);
  }
  try {
    /**
     Outcome of the exchange.
     */
    const outcome = await client.chatJson({
      modelId,
      messages,
      signal,
      exchangeTimeoutMs,
      // Conditional spread keeps the knob absent instead of undefined.
      ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
      responseFormat,
      validate,
    },);
    if (outcome.kind !== 'ok') {
      l.warn(`${stage} ${modelId}: ${lostVoiceCause({ outcome, },)}, voice lost`,);

      // ANSWERED. Every non-`ok` kind carries `rawText`, so an answer arrived:
      // whole, with only the shape defeating the guard, or cut at the length
      // limit before it was complete, which the cause keeps apart (ledger
      // P10). The kinds are `refusal-shaped` and `schema-mismatch`, and nothing
      // else reaches here: a stream cut by the idle, runaway or degeneration
      // guards throws and lands in this function's `catch` as not answered, and a
      // straggler the round abandons is classified in `stage-round.ts`, never
      // here. CONFIRMED LIVE on the first recovery rounds (two calibration arms
      // of 2026-08-26): all four re-asked voices were `schema-mismatch`, three
      // came back readable, and none of the thirteen grace cuts was re-asked.
      return {
        heard: false,
        answered: true,
        unreachable: false,
        unreadable: unreadableCauseOf({ outcome, },),
      };
    }
    return {
      heard: true,
      value: outcome.value,
    };
  }
  catch (error) {
    // Aborts must always win so user steering can stop a fan-out.
    if (signal.aborted)
      throw error;

    // A REQUEST THE ROUTE REFUSED AS MALFORMED IS NOT A LOST VOICE. It is a
    // fault in our own stage code that every later call of the stage meets
    // again, so counting it as a seat that failed would carry the pass on
    // without the seat, with one warn line for each call, in place of the
    // refusal at build time that exists to stop it (F5 of the re-check of the
    // second fix round).
    if (isRequestFault({ error, },))
      throw error;

    // EVERY PROVIDER DRY ENDS THE RUN, at the owner's instruction, so it is not one seat's lost voice. XingZ6018
    // (2026-10-22) recorded it as one for 77 s of rounds until the bench floor stopped the run with status=ERROR.
    if (endsTheRun({ error, },))
      throw error;
    /**
     Whether the router refused the call because no wet provider serves the
     seat, which is a fact about the bench rather than about this exchange.
     */
    const unreachable = error instanceof NoProviderForModelError;
    l.warn(`${stage} ${modelId}: ${exchangeFailureLogText({ error, },)}, ${unreachable ? 'seat unreachable' : 'voice lost'}`,);

    // NOT ANSWERED. A thrown failure is a transport that never delivered one,
    // and the client's own ladder has already spent its retries on it.
    return {
      heard: false,
      answered: false,
      unreachable,
    };
  }
}

//endregion Stage call
