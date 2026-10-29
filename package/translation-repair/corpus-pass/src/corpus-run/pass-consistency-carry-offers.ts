import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  applyConsistencyEdits,
  blockHolding,
  type CarryChosenPlace,
  type CarryReceivingPlace,
  ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { UnhandledMemberInvariantError, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ArtifactConsistencyRepeat, } from '@monochromatic-dev/translation-repair-artifact/ts';
import { askCarryWriters, } from './pass-consistency-carry-ask.ts';
import {
  composeSlice,
  floorRefusalOf,
  writtenParagraphsOf,
} from './pass-consistency-carry-candidates.ts';
import {
  englishOf,
  sliceOfUnit,
  unitLabelsOf,
} from './pass-consistency-carry-units.ts';
import {
  type ConsistencyPlace,
  occurrenceViewOf,
} from './pass-consistency-group.ts';
import type {
  CarryContext,
  CarryUnit,
  LocatedPlace,
} from './pass-consistency-types.ts';
import { RUN_PER_CALL_TIMEOUT_MS, } from '@monochromatic-dev/translation-repair-run/ts';

//region Pass consistency carry offers
// THE TEXTS OFFERED FOR A BLOCK, apart from the current one: the chosen words
// moved by code into the receiving places of the block that code can move them
// into, where the floors pass it, and the blocks the translate stage's writer bench
// writes for it. And what the sheets of both questions show of the places.
//
// A BLOCK CAN BE CARRIED IN PART. The splice is built from the receiving places of
// the block that have a replacement, and a receiving place whose words cannot be
// moved by code (its plan is `unavailable`) keeps its words beside them. Only a
// block with no receiving place that has a replacement is offered no splice, and
// is left to the writers or to the current block. The bench records one choice for
// the block, so the same candidates, ballots and decision stand under each of its
// places, the splice among them included.
//
// THE SPLICE IS NOT HELD TO THE WORDS OF OTHER GROUPS HERE. It replaces the exact
// words of its own receiving places and nothing else, so it can change another
// group's words only where those words overlap its own, and that meeting is
// settled where all the groups' replacements are applied
// (`pass-consistency-apply.ts`): the earlier group wins an overlap with a place
// another group has a replacement over, and a replacement is left out where it
// changes the words of a located place another group does not replace. A
// writer's block, which can rewrite words well away from its receiving places,
// is held to the listed words here.

/**
 What both sheets of a block's carry show of its places.

 @example
 ```ts
 const shown: CarryShown = { chosen, receiving: [], keeping: [], };
 ```
 */
export type CarryShown = {
  /**
   The chosen place with its English paragraph and, for a frame, its English slot.
   */
  readonly chosen: CarryChosenPlace;

  /**
   The receiving places of the block, in page order.
   */
  readonly receiving: readonly CarryReceivingPlace[];

  /**
   English words of the group's other places inside the block that must stay.
   */
  readonly keeping: readonly string[];

  /**
   The original block before the receiving places' block, where the slice has one.
   */
  readonly before?: string;

  /**
   The original block after the receiving places' block, where the slice has one.
   */
  readonly after?: string;
};

/**
 The original blocks before and after the ones that hold the receiving places
 of a unit, in their slice.

 THE PLACES LIE IN NO BODY BLOCK when the original shows the whole slice in
 place of a paragraph (`occurrenceViewOf`), and then there is nothing around it
 to show: each block is absent.

 @param places - receiving places, all of one slice

 @returns Blocks' texts, each absent where the slice has no such block

 @example
 ```ts
 const { before, after, } = neighboursOf({ places, },);
 ```
 */
function neighboursOf(
  { places, }: { readonly places: readonly ConsistencyPlace[]; },
): {
  readonly before?: string;
  readonly after?: string;
} {
  /**
   The slice's body blocks in order, of the first place's slice.
   */
  const blocks = (places[0]
    ?.slice
    .nodes
    ?? [])
    .filter(function isBody(node,): boolean {
      return node.zone === 'body';
    },);
  /**
   Positions of the blocks that hold a place, in order.
   */
  const holding = places.flatMap(function positionOf(place,): readonly number[] {
    return blocks.flatMap(function holdsIt(
      node,
      at,
    ): readonly number[] {
      return ((node.startOffset <= place.startOffset) && (place.endOffset <= node.endOffset)) ? [at,] : [];
    },);
  },);
  if (holding.length === 0)
    return {};
  /**
   The block before the first block that holds a place.
   */
  const before = blocks[Math.min(...holding,) - 1];
  /**
   The block after the last block that holds a place.
   */
  const after = blocks[Math.max(...holding,) + 1];
  return {
    ...((before === undefined) ? {} : { before: before.text, }),
    ...((after === undefined) ? {} : { after: after.text, }),
  };
}

/**
 What both sheets of a unit's carry show of its places.

 THE CHOSEN PLACE'S PARAGRAPH is the block that holds its words, or, where they
 lie in no single block, the whole English of its slice, so the sheets never show
 a bare span as a paragraph.

 @param context - what the carry reads

 @param repeat - repeated wording or frame

 @param chosen - chosen place and its located rendering

 @param unit - unit

 @returns What the sheets show

 @example
 ```ts
 const shown = shownOf({ context, repeat, chosen, unit, },);
 ```
 */
export function shownOf(
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
): CarryShown {
  /**
   The chosen place's slice English.
   */
  const english = englishOf({ place: chosen.place, },);
  /**
   Where the chosen words stand in that English and what they are.
   */
  const { rendering, } = chosen;
  /**
   The chosen words.
   */
  const { core, } = rendering;
  /**
   The block of it that holds the chosen words, as a list of zero or one.
   */
  const [chosenBlock,] = blockHolding({
    english,
    start: rendering.start,
    end: rendering.start + core.length,
  },);
  return {
    chosen: {
      view: occurrenceViewOf({
        place: chosen.place,
        text: context.text,
      },),
      span: core,
      paragraph: chosenBlock?.text ?? english,
      ...(((repeat.kind === 'refrain') && (rendering.slot !== undefined))
        ? { slot: rendering.slot, }
        : {}),
    },
    receiving: unit.receivers
      .map(function receivingOf(receiver,): CarryReceivingPlace {
        return {
          view: occurrenceViewOf({
            place: receiver.place,
            text: context.text,
          },),
          span: receiver.rendering
            .core,
        };
      },),
    keeping: unit.keeping,
    ...neighboursOf({
      places: unit.receivers
        .map(function placeOf({ place, },): ConsistencyPlace {
          return place;
        },),
    },),
  };
}

/**
 The block with the chosen words moved by code into the receiving places that
 have a replacement, when every floor passes it. A receiving place whose words
 cannot be moved by code keeps its words in that block, and the block is still
 offered as long as some other receiving place of it has a replacement; a block
 where none has gets no splice.

 @param unit - unit

 @param english - slice's English

 @param context - what the carry reads

 @returns Block as a list of zero or one, and a finding where a floor refused it

 @throws {@link ConsistencyInvariantError} when two receiving places of a unit have words that overlap, which the counting leaves unlocated

 @example
 ```ts
 const { spliced, findings, } = splicedParagraphOf({ unit, english, context, },);
 ```
 */
export function splicedParagraphOf(
  {
    unit,
    english,
    context,
  }: {
    readonly unit: CarryUnit;
    readonly english: string;
    readonly context: CarryContext;
  },
): {
  readonly spliced: readonly string[];
  readonly findings: readonly string[];
} {
  /**
   The block the receiving places stand in.
   */
  const { block, } = unit;
  /**
   Offset in the English where the block starts, which the places' offsets are counted from.
   */
  const { start: blockStart, } = block;
  /**
   The replacement of each receiving place that has one, with offsets in the block.
   */
  const edits = unit.receivers
    .flatMap(function editOf(receiver,) {
      /**
       How the chosen words are moved into this place.
       */
      const { plan, } = receiver;
      return (plan.kind === 'replace')
        ? [
          {
            id: receiver.label,
            start: plan.start - blockStart,
            end: plan.end - blockStart,
            text: plan.text,
          },
        ]
        : [];
    },);
  // A BLOCK WHOSE EVERY RECEIVING PLACE HAS NO REPLACEMENT HAS NO SPLICE; one with a replacement for some places is spliced
  // for those, and the others keep their words (`choose` in `pass-consistency-carry-choose.ts` records which).
  if (edits.length === 0) {
    return {
      spliced: [],
      findings: [],
    };
  }
  /**
   The block with every replacement made.
   */
  const made = applyConsistencyEdits({
    english: block.text,
    edits,
  },);
  /**
   The replacements left out for overlapping an earlier one.
   */
  const { overlapped, } = made;
  if (overlapped.length > 0) {
    throw new ConsistencyInvariantError({ invariant:
      `the receiving places ${unitLabelsOf({ unit, },)} of one block have words that overlap, and places whose words overlap are left unlocated before a carry`, },);
  }
  /**
   What the floors find against the slice carrying it, none when they pass it.
   */
  const [refusal,] = floorRefusalOf({
    facts: sliceOfUnit({ unit, },),
    currentText: english,
    candidateText: composeSlice({
      english,
      block,
      paragraph: made.text,
    },),
    declared: context.declared,
    declaredNames: context.declaredNames,
    ...((context.declaredSubjectForms === undefined) ? {} : { declaredSubjectForms: context.declaredSubjectForms, }),
  },);
  if (refusal === undefined) {
    return {
      spliced: [made.text,],
      findings: [],
    };
  }
  tagged({
    tag: splicedParagraphOf.name,
    l: context.l,
  },)
    .warn(`withheld the splice for ${unitLabelsOf({ unit, },)} from the bench: the floors refuse its paragraph: ${refusal}`,);
  return {
    spliced: [],
    findings: [`consistency-candidate-refused (splice of ${unitLabelsOf({ unit, },)}): ${refusal}`,],
  };
}

/**
 Why no paragraph of a unit can be rewritten, in the words of the finding.

 @param unit - unit that is not writable

 @returns Reason the unit records: the kind of block that holds its words, or why no single block does

 @throws {@link ConsistencyInvariantError} when the unit stands in no kind of block and records no reason, since a unit that is not writable has one or the other

 @throws {@link UnhandledMemberInvariantError} when a member of `UnwritableReason` has no branch here, which the compiler rules out

 @example
 ```ts
 const reason = unwritableReasonOf({ unit, },);
 ```
 */
function unwritableReasonOf({ unit, }: { readonly unit: CarryUnit; },): string {
  /**
   The kind of block that holds the words, none where no kind does.
   */
  const [kind,] = unit.spliceOnlyIn;
  if (kind !== undefined)
    return `the words that render the repeated wording stand in a ${kind}, so no paragraph can be rewritten`;
  /**
   Why the unit is not writable, where it stands in no kind of block.
   */
  const { unwritableBecause, } = unit;
  if (unwritableBecause === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      `${unitLabelsOf({ unit, },)} stands in a unit that is not writable and in no kind of block, and such a unit records why it is not writable`, },);
  }
  if (unwritableBecause === 'no-single-block')
    return 'the words that render the repeated wording lie in no single block of the English, so no paragraph can be rewritten';
  if (unwritableBecause === 'overlaps-outside-receiver') {
    return 'the paragraph that holds the words that render the repeated wording overlaps the words of another receiving place '
      + 'of the group that lie outside it, so no paragraph can be rewritten';
  }
  throw new UnhandledMemberInvariantError({
    what: 'reason a unit is not writable',
    member: unwritableBecause,
  },);
}

/**
 Asks the writer bench for the unit's block and keeps the blocks the floors
 pass.

 @param context - what the carry reads

 @param repeat - repeated wording or frame

 @param chosen - chosen place and its located rendering

 @param unit - unit

 @param english - receiving places' slice English

 @returns Surviving blocks and what the asking and the floors reported; none, with a finding saying why, where the writers are not asked

 @example
 ```ts
 const { written, findings, } = await writtenOffers({ context, repeat, chosen, unit, english, },);
 ```
 */
export async function writtenOffers(
  {
    context,
    repeat,
    chosen,
    unit,
    english,
  }: {
    readonly context: CarryContext;
    readonly repeat: ArtifactConsistencyRepeat;
    readonly chosen: LocatedPlace;
    readonly unit: CarryUnit;
    readonly english: string;
  },
): Promise<{
  readonly written: readonly {
    readonly seat: string;
    readonly text: string;
  }[];
  readonly findings: readonly string[];
}> {
  /**
   Whether the writers are asked and which seats, or why they are not.
   */
  const { writers, } = context;
  if (writers.kind === 'not-asked') {
    return {
      written: [],
      findings: [
        `consistency-writers-not-asked (${unitLabelsOf({ unit, },)}): the translators bench was not asked: ${writers.why}`,
      ],
    };
  }
  if (!unit.writable) {
    return {
      written: [],
      findings: [
        `consistency-writers-not-asked (${unitLabelsOf({ unit, },)}): ${unwritableReasonOf({ unit, },)}`,
      ],
    };
  }
  /**
   What both sheets show of the places.
   */
  const shown = shownOf({
    context,
    repeat,
    chosen,
    unit,
  },);
  /**
   The writers' replies.
   */
  const asked = await askCarryWriters({
    client: context.client,
    modelIds: writers.modelIds,
    subject: {
      repeat,
      ...shown,
      crossing: unit.crossing,
      paragraph: unit.block
        .text,
      lineStructured: sliceOfUnit({ unit, },)
        .lineStructured,
      ...((context.identityContext === undefined) ? {} : { identityContext: context.identityContext, }),
      ...((context.referenceContext === undefined) ? {} : { referenceContext: context.referenceContext, }),
      ...((context.lifeEnded === undefined) ? {} : { lifeEnded: context.lifeEnded, }),
      ...((context.subjectNames === undefined) ? {} : { subjectNames: context.subjectNames, }),
    },
    cache: context.cache,
    signal: context.signal,
    exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: context.l,
    stageSettings: context.stageSettings,
  },);
  /**
   The blocks that pass the floors, and a finding per reply withheld.
   */
  const kept = writtenParagraphsOf({
    voices: asked.voices,
    english,
    block: unit.block,
    keeping: unit.keeping,
    crossing: unit.crossing,
    facts: sliceOfUnit({ unit, },),
    declared: context.declared,
    declaredNames: context.declaredNames,
    ...((context.declaredSubjectForms === undefined) ? {} : { declaredSubjectForms: context.declaredSubjectForms, }),
    l: context.l,
  },);
  return {
    written: kept.written,
    findings: [
      ...asked.findings,
      ...kept.findings,
    ],
  };
}

//endregion Pass consistency carry offers
