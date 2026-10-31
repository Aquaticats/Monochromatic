import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  ProviderName,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { EveryProviderDryError, } from './budget-routing.ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ChatTextReply,
  ChatTextRequest,
} from './chat-contract.ts';
import {
  readJsonOutcome,
  responseFormatSpreadOf,
} from './chat-json-outcome.ts';
import {
  alternateModelsFor,
  describeLeftOut,
} from './reask-alternate-models.ts';
import { measuresModelOf, } from './reask-mode.ts';
import {
  takenFor,
  takeFor,
} from './reask-claims.ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  contextRoot,
  rendersAsNothing,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Nudged re-ask elsewhere
// LEDGER P9, the owner's ruling of 2026-09-28 ("Enable with the nudge"). A
// reply that could not be used is asked again, once, of the SAME MODEL ON
// ANOTHER PROVIDER, since the providers extract structure by different
// mechanisms (a forced tool on one, a `response_format` on the others) and the
// same weights can conform on one stack and not another. The router's re-ask
// had never run in a pass: `promptUniqueClient` buys every JSON reply through
// `chatText` and reads it itself, so the router's `chatJson` was never called.
//
// A REASONING RUNAWAY IS THE EXCEPTION (ledger B1072, the lead's decision of
// 2026-10-29, confirmed by the owner on 2026-10-30, "Everywhere (Recommended)").
// A reply that ended in a length stop with no content spent its whole
// completion cap thinking, which is the model and not the stack: 6 of the 7
// direct DeepSeek calls that ran to the cap in the runs of 2026-10-22 to
// 2026-10-24 were the re-ask of a NeuralWatt runaway of the same model, and each
// ran away again. Such a reply is asked again of ANOTHER MODEL
// (`reask-alternate-models.ts`: by the lowest share of runaways, without a model
// its role is held out of and without a model on the bench), each in turn until
// one has a provider with budget. Where none has, the model that ran away is the
// only one left to ask, and the re-ask is what it was before, nudged, on another
// provider, said on a warning. Every other reason a reply could not be used, a
// schema mismatch that holds content, a cut answer, a refusal, is re-asked as
// before.
//
// A DIFFERENT MODEL GETS THE PLAIN QUESTION ("Plain question (Recommended)",
// 2026-10-30). The nudge says "Your previous reply could not be used", which the
// other model never wrote, and a new model name already makes a new prompt for
// the uniqueness wrapper. WHAT THAT DOES TO THE DIGESTS the payload store keys
// on: the ask of model X is now `modelPromptDigest` of X and the original
// messages, which is the digest of X's own first ask of that question. A stored
// or claimed payload of that ask is replayed and X is not sampled again, so a
// model asked the same question for two seats of one gather answers both with
// one payload (`askedAlready`, and the bench, keep the stage from choosing it
// twice for one seat). The nudged digest of the same-model fall back is
// unchanged.
//
// NUDGED, ON THE FALL BACK TO THE SAME MODEL, SO THAT RE-ASK IS A NEW PROMPT.
// The uniqueness wrapper samples one model and prompt once; the same bytes asked
// again would be the same sample by that rule, and would come back from its
// cache in no time.
//
// A RUNAWAY'S SEAT IS NAMED, OR NO OTHER MODEL IS ASKED (ledger B1079). The request's `reaskSeat` carries the role
// whose holds decide which models may take the re-ask, the bench, and the record of the models already asked for the
// seats of the question (`reask-claims.ts`), so that two seats that ran away together are never re-asked of one model.
// A request that names no seat is re-asked of its own model, on a warning that says so.
//
// A MEASUREMENT SEAT IS RE-ASKED OF ITS OWN MODEL, NOT WARNED OF (ledger B1141, the lead's ruling of 2026-10-31:
// "measurement calls keep the same-model re-ask"). A probe, a bench or a calibration scores the model it asks, so a
// rival's answer recorded as that model's would be scored as its own. `ReaskSeat.reask` names the mode, and
// `reask-mode.ts` reads it for this re-ask, the stage's recovery round and the picture reader alike.
//
// A WORDING OF ITS OWN, not either of the stage recovery round's
// (`recovery-nudge.ts`). That round later re-asks every seat still unreadable
// with its nudge on whatever provider the policy picks; sharing a wording
// would make its prompt this re-ask's digest, answered from the claims with
// the reply that already failed, and the round would go silent for exactly
// the seats it exists for.
//
// ONCE, AND NEVER FROM A REPLY NO ROUTER TAGGED: a payload stored before the
// tag existed, or a client that routes nowhere, cannot name a provider to
// avoid, and the recovery round is its second chance.

/**
 Logger root for the re-ask.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 What the re-ask adds to the prompt.

 NEUTRAL ON WHY the reply could not be used, since it serves a reply that did
 not parse or match its shape and one that read as a refusal alike, and it
 must not press a model that declined.
 */
export const CROSS_PROVIDER_NUDGE: ChatMessage = {
  role: 'user',
  content: 'Your previous reply could not be used: it was not the JSON object the question asks for. '
    + 'Answer the same question again, replying with ONLY that JSON object, nothing before or after it.',
};

/**
 The first reply and what it turned out to be.
 */
type FirstAnswer<ValueT,> = {
  /**
   Reply as it arrived, tagged with its provider when a router served it.
   */
  readonly reply: ChatTextReply;

  /**
   What reading it produced.
   */
  readonly outcome: ChatJsonOutcome<ValueT>;
};

/**
 Stands for no other roster model having taken the re-ask of a reasoning runaway, which is when the model that ran
 away is the only one left to ask.
 */
const NO_OTHER_MODEL_TOOK_IT: unique symbol = Symbol('every roster model other than the reasoning runaway lacked a provider with budget for its re-ask',);

/**
 What the re-ask got back and from which model it was asked.
 */
type ReaskAnswer = {
  /**
   Model the nudged exchange was addressed to.
   */
  readonly modelId: RosterModelId;

  /**
   Reply to it.
   */
  readonly reply: ChatTextReply;
};

/**
 Whether a reply could not be used because the model spent its whole completion cap reasoning.

 A REASONING RUNAWAY IS A LENGTH STOP WITH NO CONTENT: the provider reported its token ceiling, which the reading names
 `truncated-completion`, and the answer channel holds nothing a reader sees. A cut answer that holds content is a
 different fault, one the same model can finish elsewhere, and is not this.

 @param outcome - what reading the reply produced

 @returns Whether the reply ended at the token ceiling with no content

 @example
 ```ts
 const ranAway = isReasoningRunaway({ outcome, },);
 ```
 */
export function isReasoningRunaway<ValueT,>(
  { outcome, }: { readonly outcome: ChatJsonOutcome<ValueT>; },
): boolean {
  return (outcome.kind === 'schema-mismatch')
    && (outcome.reason === 'truncated-completion')
    && rendersAsNothing({ text: outcome.rawText, },);
}

/**
 Re-asks a reply that could not be used, nudged, and returns the better of the two outcomes.

 A REPLY THAT RAN AWAY REASONING is asked of another roster model; any other reply is asked of the same model on
 another provider.

 @param request - exchange the first reply answered

 @param first - first reply and its outcome

 @param ask - performs the nudged exchange, through whatever claims and
 stores the caller keeps

 @returns Re-ask's outcome when it is usable, else the first outcome

 @throws the re-ask's error when the caller's own signal aborted it

 @throws {@link EveryProviderDryError} when the re-ask of another model met a router that reads every provider dry,
 which ends the run

 @example
 ```ts
 const outcome = await reaskElsewhereNudged({ request, first: { reply, outcome, }, ask: chatText, },);
 ```
 */
export async function reaskElsewhereNudged<ValueT,>(
  {
    request,
    first,
    ask,
  }: {
    readonly request: ForeignBorrowed<ChatJsonRequest<ValueT>>;
    readonly first: FirstAnswer<ValueT>;
    readonly ask: (nudged: ForeignBorrowed<ChatTextRequest>,) => Promise<ChatTextReply>;
  },
): Promise<ChatJsonOutcome<ValueT>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: reaskElsewhereNudged.name,
    l,
  },);
  if (first.outcome
    .kind
    === 'ok')
    return first.outcome;

  /**
   Provider the first reply came from, which the re-ask must avoid.
   */
  const { servedBy, } = first.reply;
  if (servedBy === undefined) {
    rl.info(`${request.modelId}: ${first.outcome
      .kind} from an untagged reply; no re-ask elsewhere`,);
    return first.outcome;
  }

  /**
   Provider to avoid, typed apart from the tagged reply so that `askOtherModels` and `askSameModelElsewhere`, which
   narrowing does not reach, read it.
   */
  const avoid: ProviderName = servedBy;

  /**
   The conversation the same model's re-ask carries: the question, then the nudge.
   */
  const nudgedMessages = [
    ...request.messages,
    CROSS_PROVIDER_NUDGE,
  ];

  /**
   Asks the roster's other models in turn, with the question as it stood, the next only when the one before had no
   provider with budget.

   @returns Model that took it and its reply, or the sentinel where no other model could

   @throws whatever the exchange threw when the caller aborted, and the refusal of every provider dry; any other failure
   of one model moves the order on to the next
   */
  async function askOtherModels(): Promise<ReaskAnswer | typeof NO_OTHER_MODEL_TOOK_IT> {
    /**
     Logger pre-tagged with this function's name, beneath the re-ask's.
     */
    const ol = tagged({
      tag: askOtherModels.name,
      l: rl,
    },);

    // A REQUEST THAT NAMES NO SEAT HAS NO ROLE TO READ THE HOLDS OF (ledger B1079), so no other model is asked on a
    // guess: the roster-wide role, which honours the fewest holds, once asked models their cards drop from the seat.
    /**
     Seat the request names, which carries the role, the bench and the record of the question.
     */
    const { reaskSeat, } = request;
    if (reaskSeat === undefined) {
      ol.warn(
        `${request.modelId}: the request names no seat, so no role says which model may take the re-ask of a `
          + `reasoning runaway; re-asking ${request.modelId} on another provider, as before`,
      );
      return NO_OTHER_MODEL_TOOK_IT;
    }
    // A MEASUREMENT SCORES THE MODEL IT ASKS (ledger B1141): a rival's answer recorded as that model's would be scored
    // as its own, so the runaway is re-asked of the model itself, on another provider, nudged, and that is no fault to warn of.
    if (measuresModelOf({ seat: reaskSeat, })) {
      ol.info(
        `${request.modelId}: reasoning runaway on ${avoid} (a length stop with no content); this is a measurement, `
          + `so no other model is asked and ${request.modelId} is re-asked on another provider, nudged`,
      );
      return NO_OTHER_MODEL_TOOK_IT;
    }

    /**
     Models to try in the order of their share of runaways, and the ones left out of the seat's role, bench, earlier
     asks for this question or holds.
     */
    const {
      alternates,
      leftOut,
    } = alternateModelsFor({
      modelId: request.modelId,
      role: reaskSeat.role,
      bench: reaskSeat.bench,
      askedForOtherSeats: takenFor({ seat: reaskSeat, },),
    },);
    // EACH MODEL LEFT OUT AND WHY IS WRITTEN ONCE, BEFORE THE FIRST ASK, and also where the list is empty, so the lines
    // of the asks that follow, which a model without a provider repeats, never carry it again (ledger B1100).
    ol.info(
      `${request.modelId}: reasoning runaway on ${avoid} (a length stop with no content); `
        + `left out: ${describeLeftOut({ leftOut, },)}; to ask in turn: ${alternates.length === 0 ? 'none' : alternates.join(', ',)}`,
    );
    for (const modelId of alternates) {
      // TAKEN BEFORE THE ASK, in the same step as the check (`reask-claims.ts`): another seat of the question that
      // ran away at the same time may have taken this model since the list was made.
      if (!takeFor({
        seat: reaskSeat,
        modelId,
      },)) {
        ol.info(`${request.modelId}: ${modelId} was asked for another seat of this question since the list was made`,);
        continue;
      }
      ol.info(`${request.modelId}: re-asking ${modelId} instead, plain question`,);
      try {
        return {
          modelId,
          // oxlint-disable-next-line no-await-in-loop -- one model at a time: the next is asked only when this one had no provider with budget
          reply: await ask({
            ...request,
            modelId,
            // A SEAT WHOSE QUESTION SPEAKS TO ITS ASKER hands the other model its own conversation (ledger B1100).
            ...((reaskSeat.otherModelMessages === undefined) ? {} : { messages: reaskSeat.otherModelMessages, }),
          },),
        };
      }
      catch (error) {
        // THE CALLER'S ABORT AND EVERY PROVIDER DRY END THE RE-ASK, not the one model: the first stops all work, and the
        // second ends the run at the owner's instruction (below).
        if (request.signal.aborted || (error instanceof EveryProviderDryError))
          throw error;
        if (error instanceof NoProviderForModelError) {
          ol.debug(`${request.modelId}: ${modelId} cannot take the re-ask (${exchangeFailureLogText({ error, },)})`,);
          continue;
        }
        // A TRANSPORT ERROR FROM ONE MODEL IS THAT MODEL'S FAILURE (ledger B1162): the next model of the order is asked,
        // and the seat is lost only when every model of the order has failed. Rethrown from here, it reached the catch
        // below, which kept the runaway and never tried a later model.
        ol.warn(
          `${request.modelId}: re-asking ${modelId} failed (${exchangeFailureLogText({ error, },)}); trying the next model`,
        );
      }
    }
    ol.warn(
      `${request.modelId}: no other model could take the re-ask of a reasoning runaway (${String(alternates.length,)} tried); `
        + `re-asking ${request.modelId} on another provider, as before`,
    );
    return NO_OTHER_MODEL_TOOK_IT;
  }

  /**
   Asks the same model on a provider other than the one that served the first reply.

   @returns Model and its reply

   @throws whatever the exchange threw
   */
  async function askSameModelElsewhere(): Promise<ReaskAnswer> {
    return {
      modelId: request.modelId,
      reply: await ask({
        ...request,
        messages: nudgedMessages,
        otherThan: avoid,
      },),
    };
  }

  try {
    /**
     The nudged exchange: of another model where the first reply ran away reasoning and one can take it, else of the
     same model elsewhere.
     */
    const other = isReasoningRunaway({ outcome: first.outcome, },)
      ? await askOtherModels()
      : NO_OTHER_MODEL_TOOK_IT;
    if (other === NO_OTHER_MODEL_TOOK_IT) {
      rl.info(`${request.modelId}: ${first.outcome
        .kind} on ${avoid}; re-asking another provider, nudged`,);
    }

    /**
     The model asked and its reply.
     */
    const answered = (other === NO_OTHER_MODEL_TOOK_IT) ? await askSameModelElsewhere() : other;

    /**
     The nudged exchange's reply.
     */
    const { reply, } = answered;

    /**
     What the other stack's answer turned out to be.
     */
    const second = readJsonOutcome({
      modelId: answered.modelId,
      reply,
      validate: request.validate,
      ...responseFormatSpreadOf({ request, },),
    },);
    if (second.kind === 'ok') {
      rl.info(`${answered.modelId}: ${reply.servedBy ?? 'another provider'} answered usably on the re-ask`,);
      // The model is named on the answer only where it was not the one asked.
      return (answered.modelId === request.modelId) ? second : {
        ...second,
        answeredBy: answered.modelId,
      };
    }
    // THE FIRST ANSWER IS RETURNED WHEN BOTH FAIL: it came from the provider
    // the policy preferred, and the caller's own handling is written against
    // it. Both are logged, so a reader can see the re-ask happened, and the
    // second failure rides on the returned outcome, so the line that records
    // the lost voice names both failures and not only the first's (the lead's
    // ruling of 2026-10-15 on the run of TianqiChen66625).
    rl.info(`${answered.modelId}: ${reply.servedBy ?? 'another provider'} answered ${second.kind} too`,);

    /**
     Why the re-ask failed, naming the model asked where it was not the one that ran away, so a lost voice does not read
     as that model answering twice.
     */
    const whatFailed = (second.kind === 'schema-mismatch') ? second.detail : second.marker;
    return {
      ...first.outcome,
      reaskFailure: {
        servedBy: reply.servedBy ?? 'another provider',
        kind: second.kind,
        rawText: second.rawText,
        cause: (answered.modelId === request.modelId) ? whatFailed : `${whatFailed}; asked of ${answered.modelId}`,
        // Conditional spread keeps the model absent where the re-ask went to the one that ran away.
        ...((answered.modelId === request.modelId) ? {} : { askedModelId: answered.modelId, }),
      },
    };
  }
  catch (error) {
    if (request.signal
      .aborted)
      throw error;
    // EVERY PROVIDER DRY ENDS THE RUN, at the owner's instruction, and the re-ask of another model is the one re-ask
    // that can meet it: it is routed like any first ask (`routedText`), which raises that refusal. The re-ask hinted
    // away from one provider (`otherThan`) reads the budgets once (`secondOpinionsFrom`) and raises only
    // `NoProviderForModelError`, so it cannot.
    if (error instanceof EveryProviderDryError)
      throw error;
    // THE FIRST ANSWER STANDS, since it is an answer and the re-ask was only a
    // second chance: no other provider could take it, one refused it on
    // budget, or its exchange failed. Its prompt was released by the caller's
    // claims, so a later ask of the same question may still buy it.
    rl.warn(`${request.modelId}: no re-ask elsewhere (${exchangeFailureLogText({ error, },)}); keeping the first answer`,);
    return first.outcome;
  }
}

//endregion Nudged re-ask elsewhere
