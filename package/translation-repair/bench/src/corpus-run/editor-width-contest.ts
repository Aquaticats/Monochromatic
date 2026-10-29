import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { PatchOutcome, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  selectChunkPatch,
  type ShippedProducer,
} from '@monochromatic-dev/translation-repair-repair/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ArmOutcome, } from './editor-width-arm.ts';
import type { WidthProbeInput, } from './editor-width-input.ts';
import {
  type HeadToHeadVerdict,
  readHeadToHead,
  type WidthArm,
} from './editor-width-model.ts';
import { RUN_PER_CALL_TIMEOUT_MS, } from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Editor width contest
// Two shipped repairs put on one slate, so the panel says which it prefers.
//
// RUN TWICE, once in each seating order. `selectChunkPatch` shows judges the
// candidates in the order it is handed them and does not shuffle, so the seat
// each repair occupies is the probe's to choose and therefore the probe's to
// cancel. A preference that survives the swap is about the text; one that does
// not is about the seat, and reporting the first order alone would launder the
// second into a quality result.

/**
 Seat a contest settled on.

 NAMED BY SEAT RATHER THAN BY TEXT. An arm that declined to repair offers the
 untouched translation, which is byte-identical to the fallback a panel that
 will not separate the pair falls back to. Reading the winner by comparing
 shipped text therefore credited indecision to whichever arm had declined, and
 the wide arm declines more often because it splits one selection minimum
 across twice the candidates. That turned "the panel could not choose" into
 "the wide arm won" exactly where the draw is most sensitive.
 */
type ContestSeat = 'first' | 'second' | 'none';

/**
 What one seating order decided.
 */
type ContestRound = {
  /**
   Seat the panel shipped, which is neither when it would not separate them.
   */
  readonly winner: ContestSeat;

  /**
   Ballots that named a candidate at all.
   */
  readonly usableBallots: number;
};

/**
 What both seating orders decided together.
 */
export type ContestedPair = {
  /**
   Reading of the two orders, which agree or the position decided it.
   */
  readonly verdict: HeadToHeadVerdict;

  /**
   Usable ballots across both orders.
   */
  readonly usableBallots: number;
};

/**
 Puts two shipped repairs on one slate and asks the panel which it prefers.

 @param client - injected model client

 @param input - slice both repairs belong to

 @param first - repair seated first

 @param second - repair seated second

 @param judgeModelIds - panel, the same one both arms faced

 @param signal - cancellation

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Text the panel preferred and how many ballots named anything

 @example
 ```ts
 const round = await contest({ client, input, first, second, judgeModelIds, signal, l, stageSettings, },);
 ```
 */
async function contest(
  {
    client,
    input,
    first,
    second,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly input: WidthProbeInput;
    readonly first: ArmOutcome;
    readonly second: ArmOutcome;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ContestRound> {
  /**
   The two repairs as a slate, each still carrying the producers that wrote
   it so a judge voting for its own work is discounted as in production.
   */
  const candidates: readonly Candidate<PatchOutcome>[] = [
    first,
    second,
  ]
    .map(function toCandidate(arm,): Candidate<PatchOutcome> {
      return {
        producer: {
          kind: 'composite',
          contributors: arm.producers,
        },
        value: arm.patch,
        rendered: arm.patch
          .patchedText,
      };
    },);

  /**
   Untouched translation, which is what a rejected slate falls back to.
   */
  const unchanged: PatchOutcome = {
    patchedText: input.targetText,
    applied: [],
    rejected: [],
  };

  /**
   Panel decision over the pair.
   */
  const selection = await selectChunkPatch({
    client,
    candidates,
    judgeModelIds,
    sourceText: input.sourceText,
    // A PAIR THE PANEL WILL NOT SEPARATE FALLS BACK TO NEITHER. Handing the
    // first seat as the fallback would turn every indecision into a win for
    // whichever arm happened to sit there, which is the position bias this
    // whole two-order design exists to cancel.
    //
    // MARKED `incumbent` SO IT CANNOT BE MISTAKEN FOR AN ARM. The stage reports
    // this producer verbatim as `shippedProducer` when it declines, and both
    // arms are seated as composites, so the kind alone separates a real win
    // from an indecision. A composite with no contributors would not: that is
    // exactly what an arm whose own producer went unattributed carries.
    indecisionFallback: {
      producer: {
        kind: 'incumbent',
        matched: [],
      },
      value: unchanged,
      rendered: unchanged.patchedText,
    },
    rejectionFallback: unchanged,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  /**
   Who the stage says wrote what it shipped.

   Both arms are seated as composites and both fallbacks are not, so this
   separates a decided round from a declined one without looking at text.
   */
  const { shippedProducer, } = selection;

  /**
   Text that shipped, read once.
   */
  const shipped = selection
    .patch
    .patchedText;

  return {
    winner: seatThatWon({
      shippedProducer,
      shipped,
      first,
      second,
    },),
    usableBallots: selection
      .rounds
      .flatMap(function ballotsOf(round,) {
        return round.ballots;
      },)
      .filter(function named(ballot,) {
        return ballot.weight > 0;
      },)
      .length,
  };
}

/**
 Names which seat a round settled on.

 @param shippedProducer - who the stage says wrote what shipped

 @param shipped - text that shipped

 @param first - candidate seated first

 @param second - candidate seated second

 Exported so the collision it exists to prevent can be pinned by a test rather
 than argued about: a declining arm and the indecision fallback ship the same
 bytes, so a reader that went by text alone credited indecision to whichever
 arm had declined.

 @internal

 @returns Seat that won, or none when no candidate did

 @throws Error when the stage shipped a composite whose text is neither seat's

 @example
 ```ts
 const winner = seatThatWon({ shippedProducer, shipped, first, second, },);
 ```
 */
export function seatThatWon(
  {
    shippedProducer,
    shipped,
    first,
    second,
  }: {
    readonly shippedProducer: ShippedProducer;
    readonly shipped: string;
    readonly first: ArmOutcome;
    readonly second: ArmOutcome;
  },
): ContestSeat {
  // Neither fallback is a composite, and both arms are, so a non-composite
  // means the panel never separated the pair. Checking this BEFORE the text
  // settles the case the text cannot: a declining arm and the fallback ship the
  // same bytes.
  if (shippedProducer.kind !== 'composite')
    return 'none';

  /**
   What the first seat offered.
   */
  const { patchedText: firstText, } = first.patch;

  /**
   What the second seat offered.
   */
  const { patchedText: secondText, } = second.patch;

  if (shipped === firstText)
    return 'first';

  if (shipped === secondText)
    return 'second';

  // Each seat is a composite carrying its own patch as the candidate's value,
  // and a stage that reports a composite ships one candidate's value, so text
  // matching neither seat means the slate held something else.
  throw new Error('unreachable: the stage shipped a composite whose text is neither seat\'s, and the two seats are the only composites on the slate',);
}

/**
 Reads a seat as the arm that sat in it.

 @param seat - seat the round settled on

 @param firstArm - arm seated first in that round

 Exported alongside {@link seatThatWon} so the two orders can be shown to map
 their seats to opposite arms, which is the whole mechanism that cancels
 position bias.

 @internal

 @returns Arm that won, or none

 @example
 ```ts
 const winner = armInSeat({ seat, firstArm: 'narrow', },);
 ```
 */
export function armInSeat(
  {
    seat,
    firstArm,
  }: {
    readonly seat: ContestSeat;
    readonly firstArm: WidthArm;
  },
): WidthArm | 'none' {
  if (seat === 'none')
    return 'none';

  /**
   Arm that sat second, which is whichever one did not sit first.
   */
  const secondArm: WidthArm = (firstArm === 'narrow') ? 'wide' : 'narrow';

  return (seat === 'first') ? firstArm : secondArm;
}

/**
 Judges the pair in both seating orders and reads the two together.

 @param client - injected model client

 @param input - slice both repairs belong to

 @param narrow - narrow arm's repair

 @param wide - wide arm's repair

 @param judgeModelIds - panel

 @param signal - cancellation

 @param l - logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Verdict and how many ballots carried it

 @example
 ```ts
 const contested = await bothOrders({ client, input, narrow, wide, judgeModelIds, signal, l, stageSettings, },);
 ```
 */
export async function bothOrders(
  {
    client,
    input,
    narrow,
    wide,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly input: WidthProbeInput;
    readonly narrow: ArmOutcome;
    readonly wide: ArmOutcome;
    readonly judgeModelIds: readonly RosterModelId[];
    readonly signal: AbortSignal;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ContestedPair> {
  /**
   Narrow seated first.
   */
  const narrowFirst = await contest({
    client,
    input,
    first: narrow,
    second: wide,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  },);

  /**
   Wide seated first, which is the same question with the seats swapped.
   */
  const wideFirst = await contest({
    client,
    input,
    first: wide,
    second: narrow,
    judgeModelIds,
    signal,
    l,
    stageSettings,
  },);

  return {
    verdict: readHeadToHead({
      firstOrderWinner: armInSeat({
        seat: narrowFirst.winner,
        firstArm: 'narrow',
      },),
      secondOrderWinner: armInSeat({
        seat: wideFirst.winner,
        firstArm: 'wide',
      },),
    },),
    usableBallots: narrowFirst.usableBallots + wideFirst.usableBallots,
  };
}

//endregion Editor width contest
