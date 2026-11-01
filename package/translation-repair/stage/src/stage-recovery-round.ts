import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  alternateModelsFor,
  describeLeftOut,
  measuresModelOf,
  type ReaskSeat,
  releaseFor,
  rulePastReading,
  takenFor,
  takeFor,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  RECOVERY_NUDGES,
  UNREADABLE_CAUSES,
  type UnreadableCause,
} from './recovery-nudge.ts';
import {
  type RoundOutcome,
  runGatherRound,
} from './stage-round.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Stage recovery round
// ONE RECOVERY ROUND, OUTSIDE THE QUORUM LOOP AND AFTER IT (`stage-quorum.ts`
// runs it once its quorum rounds are done).
//
// The quorum loop stops the moment quorum stands, which is correct for what
// it is for and is why nothing it re-asks has ever been re-asked: measured
// over 109 rounds of a ten-model roster on 2026-08-25, the first fan-out
// met quorum every time, 1054 voices of 1090 were heard, 31 rounds lost at
// least one, and zero retry rounds ran.
//
// Thirteen of those 36 losses were the model ANSWERING in a shape nothing
// could read, spread over 7 distinct slices of 15 with at most 2 on any
// one, so no input reliably breaks a model. The other 23 were silence,
// which `doc/audit/where-a-round-spends-its-wall-clock.md` measures to be a
// model still thinking. Re-asking both would spend the expensive half to
// recover the cheap half, so only the answered half is re-asked here.
//
// ONE ROUND, NEVER A LADDER. A model that formats badly twice is telling us
// something about itself rather than about the weather, and the calibration
// is what answers that.
//
// ITS OWN MODULE since ledger B109, when counting its log lines' nouns took
// `stage-quorum.ts` past the line budget; the round was already the one
// self-contained step of the gather.

/**
 Everything a gather round takes except the seats it asks, the voices it
 waits for and its reserve: the part every round of one gather shares, which
 the recovery round sends unchanged apart from the nudge it appends.

 @example
 ```ts
 const shared: SharedRoundRequest<MeowReply> = { client, messages, signal, exchangeTimeoutMs, responseFormat, validate, stage, l, graceMs, };
 ```
 */
export type SharedRoundRequest<ValueT,> = Omit<
  Parameters<typeof runGatherRound<ValueT>>[0],
  'heardNeeded' | 'modelIds' | 'reserve' | 'reaskSeat'
  > & NamedSeat;

/**
 The seat of a gather, required of the part of a round request the recovery round reads, so it never asks a runaway
 seat of a model on a guess about its role (ledger B1079).
 */
type NamedSeat = {
  /**
   Role, bench and shared record of the gather.
   */
  readonly reaskSeat: ReaskSeat;
};

// A SEAT WHOSE ANSWER RAN AWAY IS ASKED OF ANOTHER MODEL (ledger B1073, B1074, the
// owner's answer of 2026-10-30, "Everywhere (Recommended)"). A reply that ended in a
// length stop with no content is a model that spent its whole completion cap
// thinking, and this round used to ask the seat's own model a third time: in
// `XingZ6020` slice 92 the round's ask was the third runaway of one model for one
// question, and the next round's re-ask the fourth. The round now asks the models
// `alternateModelsFor` lists, in the order of their share of runaways, with the
// plain question ("Plain question (Recommended)") and records the answer as the
// seat's voice. It leaves out the models the seat's role is held out of, the
// models seated on the question (the lead's adoption of 2026-10-30) and the model
// an earlier re-ask for the seat already went to, whose claimed reply the
// uniqueness wrapper would otherwise replay as a second voice. It also leaves out
// the models asked for the other seats of the gather, which the seats share one
// record of and take from before they ask (`ReaskSeat.claims`, ledger B1080): the
// seats of this round run together, so two that ran away would each take the first
// model left. Only where no model is left is the seat's own model asked, under the
// `cut-short` nudge as before, on a warning.

/**
 Asks a seat whose answer ran away of another model, once, and records the answer as the seat's voice.

 @param roundRequest - what every round of the gather sends, prompt and bench included

 @param seat - seat whose answer ran away

 @param askedInstead - model an earlier re-ask for the seat already went to, when it went to another model

 @param l - logger of the recovery round

 @returns Outcome for the seat: the first reachable model's answer, or the seat's own model's under the cut-short nudge
 where no other model could take the re-ask

 @throws Whatever the round of the model asked throws, which is the caller's own abort, a request the route refused as
 malformed, or the router's every-provider-dry refusal

 @example
 ```ts
 const outcome = await recoverRunaway({ roundRequest, seat, l, },);
 ```
 */
async function recoverRunaway<ValueT,>(
  {
    roundRequest,
    seat,
    askedInstead,
    l,
  }: ForeignBorrowed<{
    readonly roundRequest: SharedRoundRequest<ValueT>;
    readonly seat: RosterModelId;
    readonly askedInstead?: RosterModelId;
    readonly l: Logger;
  }>,
): Promise<RoundOutcome<ValueT>> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: recoverRunaway.name,
    l,
  },);

  /**
   The seat the gather named: its role, its bench and the record its seats share.
   */
  const named: ReaskSeat = roundRequest.reaskSeat;

  /**
   Role and bench of the gather.
   */
  const {
    role,
    bench,
  } = named;

  /**
   Asks the seat's own model once more, under the cut-short nudge, which is the one ask a measurement makes and the ask
   a run falls back to where no other model could take the re-ask.

   @returns One outcome that round returned
   */
  async function askSeatItself(): Promise<RoundOutcome<ValueT>> {
    /**
     The seat's own model's round, nudged as the cut-short wording has it.
     */
    const ownRound = await runGatherRound<ValueT>({
      ...roundRequest,
      l: rl,
      messages: [
        ...roundRequest.messages,
        RECOVERY_NUDGES.runaway,
      ],
      modelIds: [seat,],
      heardNeeded: 0,
    },);

    /**
     The one outcome that round returned.
     */
    const own = ownRound.at(0,);
    if (own === undefined)
      throw new Error(`unreachable: the round that asked ${seat} returned no outcome`,);
    return own;
  }
  // A MEASUREMENT SCORES THE MODEL IT ASKS (ledger B1141): a rival's answer recorded as the seat's would be scored as the
  // seat's own, so the seat is asked again, nudged, and that is no fault to warn of.
  if (measuresModelOf({ seat: named, })) {
    rl.info(
      `${roundRequest.stage}: ${seat}: reasoning runaway (a length stop with no content); this is a measurement, `
        + `so no other model is asked and ${seat} is re-asked, nudged`,
    );
    return await askSeatItself();
  }

  /**
   The bench with the seat on it, which an ask of another model carries so that model's own runaway is not re-asked of
   the seat that began this one.
   */
  const benchWithSeat: readonly RosterModelId[] = bench.includes(seat,)
    ? bench
    : [
      ...bench,
      seat,
    ];

  /**
   Models to ask in the order of their share of runaways, and the ones left out of the seat's role, the bench or an
   earlier ask.
   */
  const {
    alternates,
    leftOut,
  } = alternateModelsFor({
    modelId: seat,
    role,
    bench,
    askedAlready: (askedInstead === undefined) ? [] : [askedInstead,],
    askedForOtherSeats: takenFor({ seat: named, },),
    // A MODEL THE RUN WITHHOLDS FROM THE SEAT'S ROLE IN THE READING IS NOT AN ALTERNATE (ledger B1330): the client's
    // dryness view says where Hyper would serve a model, so Qwen3.8-27B is left out of an editor's alternates then and
    // not where Synthetic serves it; a client with no view leaves out every model with such a hold.
    routeTimeHolds: await rulePastReading({
      readDry: roundRequest.client.providerDryness,
      signal: roundRequest.signal,
      l: rl,
    },),
  },);
  // EACH MODEL LEFT OUT AND WHY IS WRITTEN ONCE, BEFORE THE FIRST ASK, and also where the list is empty, so the lines of
  // the asks that follow, which a model without a provider repeats, never carry it again (ledger B1100).
  rl.info(
    `${roundRequest.stage}: ${seat}: reasoning runaway (a length stop with no content); `
      + `left out: ${describeLeftOut({ leftOut, },)}; to ask in turn: ${alternates.length === 0 ? 'none' : alternates.join(', ',)}`,
  );
  for (const alternate of alternates) {
    // TAKEN BEFORE THE ASK, in the same step as the check (`reask-claims.ts` in the provider): another runaway seat of
    // this gather whose function ran first may have taken this model since the list was made (ledger B1080).
    if (!takeFor({
      seat: named,
      modelId: alternate,
    },)) {
      rl.info(`${roundRequest.stage}: ${seat}: ${alternate} was asked for another seat of this question since the list was made`,);
      continue;
    }
    rl.info(`${roundRequest.stage}: ${seat}: re-asking ${alternate} instead, plain question`,);

    /* oxlint-disable no-await-in-loop -- one model at a time: the next is asked only when this one had no provider with budget */
    /**
     The other model's round, asked the question as it stood.
     */
    const askedRound = await runGatherRound<ValueT>({
      ...roundRequest,
      l: rl,
      modelIds: [alternate,],
      heardNeeded: 0,
      reaskSeat: {
        ...named,
        bench: benchWithSeat,
      },
    },);
    /* oxlint-enable no-await-in-loop */

    /**
     The one outcome that round returned.
     */
    const asked = askedRound.at(0,);
    if (asked === undefined)
      throw new Error(`unreachable: the round that asked ${alternate} returned no outcome`,);

    /**
     What came back from the other model.
     */
    const { voice, } = asked;
    if ((!voice.heard) && voice.unreachable) {
      // REFUSED BEFORE ANY EXCHANGE: the model was not asked, so its claim goes back for another seat (ledger B1201).
      // One the router refused after a call was cut (B1230) was asked, and stays claimed.
      if (voice.afterExchange !== true) {
        releaseFor({
          seat: named,
          modelId: alternate,
        },);
      }
      rl.debug(`${roundRequest.stage}: ${seat}: ${alternate} cannot take the re-ask, no provider serves it with budget`,);
      continue;
    }
    // A TRANSPORT FAILURE IS THAT MODEL'S FAILURE (ledger B1201), as it is in the provider's own re-ask: its exchange
    // never delivered an answer, the next model of the order is asked, and the seat is lost only when every model of the
    // order has failed. The caller's abort and the refusal of every provider dry never reach here as a lost voice; both
    // are thrown out of the round. The model stays claimed, since it was asked. An answer that arrived and could not be
    // read ends the order, as it does there.
    if ((!voice.heard) && (!voice.answered)) {
      rl.warn(`${roundRequest.stage}: ${seat}: re-asking ${alternate} failed (no answer arrived); trying the next model`,);
      continue;
    }
    // THE OUTCOME STANDS UNDER THE SEAT, AND THE VOICE NAMES THE MODEL THAT WROTE IT (ledger B1142): the model asked, or
    // the one it was itself re-asked of where its own answer ran away, so no weight, tally or record reads the seat.
    return {
      modelId: seat,
      voice: voice.heard ? {
        ...voice,
        answeredBy: voice.answeredBy ?? alternate,
      } : voice,
    };
  }
  rl.warn(
    `${roundRequest.stage}: ${seat}: no other model could take the re-ask of a reasoning runaway `
      + `(${String(alternates.length,)} tried); re-asking ${seat}, nudged`,
  );
  return await askSeatItself();
}

/**
 Re-asks, once, every seat whose answer arrived and could not be read, each
 under the nudge naming what happened to it, except a seat whose answer ran
 away, which is asked of another model with the plain question.

 @param roundRequest - what every round of the gather sends, prompt included

 @param unreadable - seats still unreadable after the quorum rounds, in roster
 order

 @param causeOf - why each of those seats' latest answer could not be read,
 which picks its nudge (ledger P10)

 @param askedInstead - for a seat whose answer ran away and was already
 re-asked of another model, that model, which the round leaves out; absent
 where no seat was

 @returns One outcome per seat re-asked, grouped by cause in
 `UNREADABLE_CAUSES` order

 @example
 ```ts
 const recovered = await runRecoveryRound({ roundRequest, unreadable, causeOf: unreadableSeats, },);
 ```
 */
export async function runRecoveryRound<ValueT,>(
  {
    roundRequest,
    unreadable,
    causeOf,
    askedInstead,
  }: ForeignBorrowed<{
    readonly roundRequest: SharedRoundRequest<ValueT>;
    readonly unreadable: readonly RosterModelId[];
    readonly causeOf: ReadonlyMap<RosterModelId, UnreadableCause>;
    readonly askedInstead?: ReadonlyMap<RosterModelId, RosterModelId>;
  }>,
): Promise<readonly RoundOutcome<ValueT>[]> {
  /**
   Stage label and prompt the gather's rounds share.
   */
  const {
    stage,
    messages,
  } = roundRequest;

  /**
   Logger tagged with this round.
   */
  const rl = tagged({
    tag: runRecoveryRound.name,
    l: roundRequest.l,
  },);

  /**
   The unreadable seats grouped by what happened to them, in
   `UNREADABLE_CAUSES` order; a cause no seat has asks nobody.
   */
  const byCause = UNREADABLE_CAUSES
    .map(function seatsWith(cause,): {
      readonly cause: UnreadableCause;
      readonly modelIds: readonly RosterModelId[];
    } {
      return {
        cause,
        modelIds: unreadable.filter(function hasCause(modelId,): boolean {
          return causeOf.get(modelId,) === cause;
        },),
      };
    },)
    .filter(function asksSomeone(group,): boolean {
      return group.modelIds
        .length
        > 0;
    },);
  rl.warn(
    `${stage}: recovery round for ${String(unreadable.length,)} unreadable ${
      wordForCount({
        count: unreadable.length,
        one: 'answer',
        many: 'answers',
      },)
    } (${
      byCause
        .map(function describe(group,): string {
          return `${String(group.modelIds
            .length,)} ${group.cause}`;
        },)
        .join(', ',)
    })`,
  );

  /**
   Second reading of the voices that finished but could not be read.

   NEEDING NONE OF THEM IS THE BOUND. Quorum usually stands by now, and
   where the quorum rounds ran out short the recovered voices still count
   toward it; either way this round is entitled to no more than a
   straggler window: `heardNeeded: 0` leaves
   `runGatherRound` with nothing to wait for, which opens the grace window
   at once and abandons whatever has not arrived when it closes. Asking
   for all of them instead would let one re-ask that hangs hold the whole
   gather for a full exchange deadline, which is six minutes in a run and
   the opposite of what a recovery is for.

   ONE ROUND PER WORDING, RUN TOGETHER (ledger P10): a round carries one
   prompt, and each group's prompt names what happened to it; both open
   their windows at once, so the pair costs one window, not two.

   A voice that comes back promptly is still collected: the window
   resolves as soon as every ask settles.
   */
  const recovered = (await Promise.all(byCause.map(async function recoverGroup(group,) {
    // A RUNAWAY IS ASKED OF ANOTHER MODEL, one seat at a time within the group's window, each seat's own models in turn.
    if (group.cause === 'runaway') {
      return await allInInputOrder({
        members: group.modelIds
          .map(async function recoverSeat(seat,) {
            /**
             Model an earlier re-ask for this seat already went to, if one did.
             */
            const askedAlready = askedInstead?.get(seat,);
            return await recoverRunaway<ValueT>({
              roundRequest,
              seat,
              // Conditional spread keeps the model absent where no earlier re-ask went to another.
              ...((askedAlready === undefined) ? {} : { askedInstead: askedAlready, }),
              l: rl,
            },);
          },),
      },);
    }
    return await runGatherRound<ValueT>({
      ...roundRequest,
      l: rl,
      // A DIFFERENT PROMPT, OR THE ROUND BUYS NOTHING. `promptUniqueClient`
      // serves a second call for the same model and prompt from its cache,
      // schema mismatch included, so re-sending the same bytes came back
      // with the same unreadable answer in 0 to 1 ms every time it was
      // measured (five recovery rounds over two passes on
      // 2026-09-02). The nudge tells the model what happened and makes the
      // digest new.
      messages: [
        ...messages,
        RECOVERY_NUDGES[group.cause],
      ],
      modelIds: group.modelIds,
      heardNeeded: 0,
    },);
  },),))
    .flat();

  /**
   Re-asked voices that came back readable, counted on their own line so
   the round's value can be read off a run log without pairing gather
   lines by hand (the owner kept the round on 2026-09-03, and
   this is what says whether it earns its call).
   */
  const recoveredHeard = recovered.filter(function heard(outcome,): boolean {
    return outcome.voice
      .heard;
  },);
  rl.info(
    `${stage}: recovery round heard ${String(recoveredHeard.length,)} of `
      + `${String(unreadable.length,)} re-asked ${
        wordForCount({
          count: unreadable.length,
          one: 'voice',
          many: 'voices',
        },)
      }`,
  );
  return recovered;
}

//endregion Stage recovery round
