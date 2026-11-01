import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

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
  'heardNeeded' | 'modelIds' | 'reserve'
  >;

/**
 Re-asks, once, every seat whose answer arrived and could not be read, each
 under the nudge naming what happened to it.

 @param roundRequest - what every round of the gather sends, prompt included

 @param unreadable - seats still unreadable after the quorum rounds, in roster
 order

 @param causeOf - why each of those seats' latest answer could not be read,
 which picks its nudge (ledger P10)

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
  }: ForeignBorrowed<{
    readonly roundRequest: SharedRoundRequest<ValueT>;
    readonly unreadable: readonly RosterModelId[];
    readonly causeOf: ReadonlyMap<RosterModelId, UnreadableCause>;
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
