import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  blockStandingOf,
  carryOffersOf,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  exchangeFailureLogText,
  exchangeFailureText,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ArtifactConsistencyRepeat, } from '@monochromatic-dev/translation-repair-artifact/ts';
import {
  carryWithoutChoice,
  choose,
} from './pass-consistency-carry-choose.ts';
import {
  splicedParagraphOf,
  writtenOffers,
} from './pass-consistency-carry-offers.ts';
import {
  englishOf,
  englishOfUnit,
  type HeldReceiver,
  type KeptWords,
  sliceOfUnit,
  unitLabelsOf,
  unitsOf,
} from './pass-consistency-carry-units.ts';
import type {
  CarryContext,
  CarryReceiver,
  CarryUnit,
  LocatedPlace,
  UnitCarried,
} from './pass-consistency-types.ts';

//region Pass consistency carry
// CARRYING THE CHOSEN RENDERING TO THE RECEIVING PLACES OF A BLOCK BY WRITING,
// NOT BY SPLICING (the lead's decision of 2026-10-12, open to the owner's veto).
// Receiving places are grouped by the block that holds them
// (`pass-consistency-carry-units.ts`). The writers of the translate stage are
// given the chosen place's English and its words, the receiving places' original
// paragraphs with their neighbours and the current English paragraph, the house
// rules, and the tense block of an entry that records a death; they return the
// paragraph with the repeated wording rendered as the chosen place renders it in
// every receiving place. The bench then chooses, reason before choice, by strict
// majority, among the current paragraph, the code splice (where the chosen words
// can be moved into every receiving place by code and the floors pass it) and each
// writer's paragraph that passes the floors
// (`pass-consistency-carry-choose.ts`). The winner's edit meets the other groups'
// edits and the floors again in `pass-consistency-apply.ts`; a block whose carry
// fails keeps its English with a finding. NO MODEL OUTPUT REACHES THE PAGE
// WITHOUT THE BENCH AND THE FLOORS.

/**
 The record of a carry that failed, for each receiving place it was for.

 @param labels - labels of the receiving places

 @param error - what went wrong

 @returns Failed carry, which changes nothing

 @example
 ```ts
 const carried = failedCarryOf({ labels: ['OCCURRENCE 2',], error: new Error('no',), },);
 ```
 */
function failedCarryOf(
  {
    labels,
    error,
  }: {
    readonly labels: readonly string[];
    readonly error: unknown;
  },
): UnitCarried {
  return {
    carries: carryWithoutChoice({
      labels,
      decision: {
        kind: 'failed',
        detail: exchangeFailureText({ error, },),
      },
      candidates: [],
    },),
    edits: [],
    findings: [
      `consistency-carry-failed (${labels.join(' and ',)}): ${
        (labels.length === 1) ? 'the place keeps its English' : 'the places keep their English'
      }`,
    ],
  };
}

/**
 Carries the chosen rendering to the receiving places of one block, without
 containing trouble.

 @param context - what the carry reads

 @param repeat - repeated wording or frame

 @param chosen - chosen place and its located rendering

 @param unit - receiving places of one block

 @returns Carry's record for each place, replacements it makes and what it reported

 @throws Whatever the asks and the readers throw, which the caller contains

 @example
 ```ts
 const carried = await carryUncontained({ context, repeat, chosen, unit, },);
 ```
 */
async function carryUncontained(
  {
    context,
    repeat,
    chosen,
    unit,
  }: {
    readonly context: CarryContext;
    readonly repeat: ArtifactConsistencyRepeat;
    readonly chosen: LocatedPlace;
    readonly unit: CarryUnit;
  },
): Promise<UnitCarried> {
  /**
   The receiving places' slice English.
   */
  const english = englishOfUnit({ unit, },);
  /**
   The block with the chosen words moved in by code, if that is possible.
   */
  const {
    spliced,
    findings: spliceFindings,
  } = splicedParagraphOf({
    unit,
    english,
    context,
  },);
  /**
   The blocks the writers wrote that the floors pass.
   */
  const writing = await writtenOffers({
    context,
    repeat,
    chosen,
    unit,
    english,
  },);
  return await choose({
    context,
    repeat,
    chosen,
    unit,
    offers: carryOffersOf({
      current: unit.block
        .text,
      splice: spliced,
      written: writing.written,
      lineStructured: sliceOfUnit({ unit, },)
        .lineStructured,
    },),
    findings: [
      ...spliceFindings,
      ...writing.findings,
    ],
  },);
}

/**
 Carries the chosen rendering to the receiving places of one block, containing
 whatever goes wrong: the places then keep their English, with a finding.

 @param context - what the carry reads

 @param repeat - repeated wording or frame

 @param chosen - chosen place and its located rendering

 @param unit - receiving places of one block

 @returns Carry's record for each place, replacements it makes and what it reported

 @throws The abort reason when the caller stopped the work

 @example
 ```ts
 const carried = await carryToUnit({ context, repeat, chosen, unit, },);
 ```
 */
export async function carryToUnit(
  {
    context,
    repeat,
    chosen,
    unit,
  }: {
    readonly context: CarryContext;
    readonly repeat: ArtifactConsistencyRepeat;
    readonly chosen: LocatedPlace;
    readonly unit: CarryUnit;
  },
): Promise<UnitCarried> {
  /**
   Logger pre-tagged with this function's name.
   */
  const cl = tagged({
    tag: carryToUnit.name,
    l: context.l,
  },);
  try {
    return await carryUncontained({
      context: {
        ...context,
        l: cl,
      },
      repeat,
      chosen,
      unit,
    },);
  }
  catch (error) {
    if (context.signal
      .aborted)
      throw error;
    cl.error(
      `the carry to ${unitLabelsOf({ unit, },)} could not be made (${exchangeFailureLogText({ error, },)}); the place keeps its English`,
    );
    return failedCarryOf({
      labels: unit.receivers
        .map(function labelOf({ label, },): string {
          return label;
        },),
      error,
    },);
  }
}

/**
 The block of its English that holds a receiving place's words, or the failure
 of reading it.

 @param receiver - receiving place

 @param context - what the carry reads, whose signal and logger the reading uses

 @returns Place with its block, or the failed carry that keeps its English

 @throws The abort reason when the caller stopped the work

 @example
 ```ts
 const holding = holdingOf({ receiver, context, },);
 ```
 */
function holdingOf(
  {
    receiver,
    context,
  }: {
    readonly receiver: CarryReceiver;
    readonly context: CarryContext;
  },
): {
  readonly kind: 'held';
  readonly held: HeldReceiver;
} | {
  readonly kind: 'failed';
  readonly carried: UnitCarried;
} {
  try {
    /**
     Where the receiving place's words stand among the blocks of its English.
     */
    const {
      block,
      spliceOnlyIn,
    } = blockStandingOf({
      english: englishOf({ place: receiver.place, },),
      start: receiver.rendering
        .start,
      end: receiver.rendering
        .start
        + receiver.rendering
          .core
          .length,
    },);
    return {
      kind: 'held',
      held: {
        receiver,
        block,
        spliceOnlyIn,
      },
    };
  }
  catch (error) {
    if (context.signal
      .aborted)
      throw error;
    context.l
      .error(
      `the carry to ${receiver.label} could not be made (${exchangeFailureLogText({ error, },)}); the place keeps its English`,
      );
    return {
      kind: 'failed',
      carried: failedCarryOf({
        labels: [receiver.label,],
        error,
      },),
    };
  }
}

/**
 Carries the chosen rendering to every receiving place of a group: the places
 are grouped by the block that holds them, and each block is carried to once.

 @param context - what the carry reads

 @param repeat - repeated wording or frame

 @param chosen - chosen place and its located rendering

 @param receivers - receiving places, in page order

 @param kept - words of the group's other places that must stay as they are

 @returns One result for each block, and one for each place whose block could not be read

 @throws The abort reason when the caller stopped the work

 @example
 ```ts
 const carried = await carryAll({ context, repeat, chosen, receivers, kept: [], },);
 ```
 */
export async function carryAll(
  {
    context,
    repeat,
    chosen,
    receivers,
    kept,
  }: {
    readonly context: CarryContext;
    readonly repeat: ArtifactConsistencyRepeat;
    readonly chosen: LocatedPlace;
    readonly receivers: readonly CarryReceiver[];
    readonly kept: readonly KeptWords[];
  },
): Promise<readonly UnitCarried[]> {
  /**
   Logger pre-tagged with this function's name.
   */
  const cl = tagged({
    tag: carryAll.name,
    l: context.l,
  },);
  /**
   Each receiving place with its block, or with the failure of reading it.
   */
  const holdings = receivers.map(function reading(receiver,) {
    return holdingOf({
      receiver,
      context: {
        ...context,
        l: cl,
      },
    },);
  },);
  /**
   The carries of the places whose block could not be read.
   */
  const unread = holdings.flatMap(function failedOf(holding,): readonly UnitCarried[] {
    return (holding.kind === 'failed') ? [holding.carried,] : [];
  },);
  /**
   The receiving places grouped by the block that holds them.
   */
  const units = unitsOf({
    held: holdings.flatMap(function heldOf(holding,): readonly HeldReceiver[] {
      return (holding.kind === 'held') ? [holding.held,] : [];
    },),
    kept,
  },);
  return [
    ...unread,
    ...await allInInputOrder({
      members: units.map(async function carryOne(unit,) {
        return await carryToUnit({
          context,
          repeat,
          chosen,
          unit,
        },);
      },),
    },),
  ];
}

//endregion Pass consistency carry
