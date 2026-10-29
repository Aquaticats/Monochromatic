/**
 Tests for recounting a group of the sixteenth shape from its own ballots: the
 usable count, each place's rendering, the winner, each receiving place's carry
 and what became of each place. Each case breaks one valid group in one way and
 pins the whole refusal.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ConsistencyWinner, } from '@monochromatic-dev/translation-repair-consistency';
import {
  ARTIFACT_SCHEMA_VERSION_V18,
  type ArtifactConsistencyBallot,
  type ArtifactConsistencyCarry,
  type ArtifactConsistencyChoiceBallot,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencyOccurrence,
  type ArtifactConsistencyRunOff,
  assertSpanGroupCoherent,
  type TwoLaneArtifactGeneration,
} from '../../dist/final/node/index.mjs';
import { refusalTextOf, } from './consistency-refusal.test-fixture.ts';

/**
 Where the group sits in the section, for the refusals.
 */
const GROUP = 'cat.consistency.groups[0]';

/**
 The refusal of a splice candidate for the second place of a generation seventeen group, which code could not have moved.
 */
const NO_SPLICE_FOR_OCCURRENCE_TWO = `ArtifactParseError: artifact parse failed at ${GROUP}.carries: `
  + 'expected no splice candidate for OCCURRENCE 2, whose words code cannot move: the frame or the chosen frame has no '
  + 'slot to move them by, a single mark beside the words differs between the two places, the quotation marks '
  + 'inside the words differ, the two hold a different number of block breaks (a blank line, or a line break the '
  + 'Markdown parse reads as a new block), or a single mark stands beside the words and the chosen words end on that '
  + 'side in another class of character than the receiving words, a letter or a digit against anything else.';

/**
 The group the cases read: a repeated sentence in two places whose three ballots choose the first.
 */
type AskedGroup = ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };

/**
 One ballot, choosing the first place and quoting both with the closing full stop as a judge may.

 @param seat - seat that answered

 @param over - fields this case replaces

 @returns Ballot

 @example
 ```ts
 const ballot = ballotOf({ seat: 'hyper-cat', },);
 ```
 */
function ballotOf(
  {
    seat,
    over = {},
  }: {
    readonly seat: string;
    readonly over?: {
      readonly quotes?: ArtifactConsistencyBallot['quotes'];
      readonly choice?: string;
      readonly keep?: ArtifactConsistencyBallot['keep'];
    };
  },
): ArtifactConsistencyBallot {
  return {
    seat,
    reason: 'The first place words it best.',
    quotes: [
      { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
      { label: 'OCCURRENCE 2', quote: 'the kitten naps', },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
    ...over,
  };
}

/**
 The carry of the second place: the bench chose the chosen words moved in by code.
 */
const CARRY: ArtifactConsistencyCarry = {
  label: 'OCCURRENCE 2',
  candidates: [
    { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'Later the kitten naps in the sun.', },
    { label: 'CANDIDATE 2', kind: 'splice', writers: [], text: 'Later the cat naps in the sun.', },
  ],
  ballots: [
    { seat: 'hyper-cat', reason: 'It words the original best.', choice: 'CANDIDATE 2', },
    { seat: 'bedrock-cat', reason: 'It words the original best.', choice: 'CANDIDATE 2', },
  ],
  usable: 2,
  quorum: 2,
  decision: { kind: 'chosen', label: 'CANDIDATE 2', },
};

/**
 A place of the sentence, with the fields a case replaces.

 @param position - position of the place in the group

 @param over - fields this case replaces

 @returns Place

 @example
 ```ts
 const place = placeOf({ position: 1, over: { outcome: 'consistent', }, },);
 ```
 */
function placeOf(
  {
    position,
    over = {},
  }: {
    readonly position: number;
    readonly over?: {
      readonly rendering?: ArtifactConsistencyOccurrence['rendering'];
      readonly outcome?: ArtifactConsistencyOccurrence['outcome'];
    };
  },
): ArtifactConsistencyOccurrence {
  return (position === 0)
    ? {
      label: 'OCCURRENCE 1',
      sliceIndex: 0,
      startOffset: 6,
      endOffset: 18,
      line: 3,
      rendering: { kind: 'located', quote: 'The cat naps', start: 0, },
      outcome: 'chosen',
      ...over,
    }
    : {
      label: 'OCCURRENCE 2',
      sliceIndex: 1,
      startOffset: 30,
      endOffset: 42,
      line: 5,
      rendering: { kind: 'located', quote: 'the kitten naps', start: 6, },
      outcome: 'replaced',
      ...over,
    };
}

/**
 What the bench was asked and answered, with the fields a case replaces: three
 ballots that choose the first place, a quorum of two and the first place the winner.

 @param ballots - ballots heard

 @param usable - how many of them can be counted

 @param quorum - voices the gather closed on

 @param winner - how the count ended

 @returns Verdict

 @example
 ```ts
 const verdict = verdictWith({ usable: 2, },);
 ```
 */
function verdictWith(
  {
    ballots = [ballotOf({ seat: 'hyper-cat', },), ballotOf({ seat: 'bedrock-cat', },), ballotOf({ seat: 'synthetic-cat', },),],
    usable = 3,
    quorum = 2,
    winner = { kind: 'chosen', label: 'OCCURRENCE 1', },
  }: {
    readonly ballots?: readonly ArtifactConsistencyBallot[];
    readonly usable?: number;
    readonly quorum?: number;
    readonly winner?: ConsistencyWinner;
  },
): AskedGroup['verdict'] {
  return {
    kind: 'asked',
    ballots,
    usable,
    quorum,
    winner,
  };
}

/**
 The three ballots of a group that quote its first place as `The cat naps.` and its second as the text given for the seat.

 @param quotes - what each seat quotes of the second place, in seat order

 @returns Ballots

 @example
 ```ts
 const ballots = ballotsQuotingSecond({ quotes: ['the kitten naps', 'kitten naps', 'the kitten naps',], },);
 ```
 */
function ballotsQuotingSecond({ quotes, }: { readonly quotes: readonly string[]; },): readonly ArtifactConsistencyBallot[] {
  return ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotFor(seat, at,): ArtifactConsistencyBallot {
    return ballotOf({
      seat,
      over: {
        quotes: [
          { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
          { label: 'OCCURRENCE 2', quote: quotes[at] ?? '', },
        ],
      },
    },);
  },);
}

/**
 The coherent group, with the fields a case replaces.

 @param over - fields of the group this case replaces

 @returns Group

 @example
 ```ts
 const group = groupOf({ over: { carries: [], }, },);
 ```
 */
function groupOf(
  { over = {}, }: {
    readonly over?: {
      readonly repeat?: AskedGroup['repeat'];
      readonly occurrences?: AskedGroup['occurrences'];
      readonly verdict?: AskedGroup['verdict'];
      readonly carries?: readonly ArtifactConsistencyCarry[];
    };
  } = {},
): AskedGroup {
  return {
    repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
    occurrences: [placeOf({ position: 0, },), placeOf({ position: 1, },),],
    unplaced: [],
    verdict: verdictWith({},),
    carries: [CARRY,],
    ...over,
  };
}

/**
 The group with its verdict's fields replaced.

 @param over - fields of the verdict this case replaces

 @returns Group

 @example
 ```ts
 const group = withVerdict({ over: { usable: 2, }, },);
 ```
 */
function withVerdict(
  { over, }: { readonly over: Parameters<typeof verdictWith>[0]; },
): AskedGroup {
  return groupOf({ over: { verdict: verdictWith(over,), }, },);
}

/**
 The text of the refusal the recount raises for a group.

 @param group - group, asked of the bench

 @param generation - generation the section was written under, which says whether the marks beside the words were recorded

 @returns Refusal as text

 @example
 ```ts
 const text = coherenceRefusal({ group: groupOf(), },);
 ```
 */
function coherenceRefusal(
  {
    group,
    generation = 17,
  }: {
    readonly group: AskedGroup;
    readonly generation?: TwoLaneArtifactGeneration;
  },
): string {
  return refusalTextOf({
    read: function recounts(): unknown {
      return assertSpanGroupCoherent({
        group,
        path: GROUP,
        generation,
      },);
    },
  },);
}

/**
 A group of three places in which the chosen words are carried to two places by one bench record: the second place
 holds an apostrophe the chosen place does not, so code cannot move the words there, and the third holds none.

 @param outcomes - what became of the second and the third place

 @returns Group, the carries one for each of the two places and both the same record

 @example
 ```ts
 const group = partlyCarriedGroup({ outcomes: ['kept-current', 'replaced',], },);
 ```
 */
function partlyCarriedGroup(
  { outcomes, }: { readonly outcomes: readonly [ArtifactConsistencyOccurrence['outcome'], ArtifactConsistencyOccurrence['outcome'],]; },
): AskedGroup {
  return groupOf({
    over: {
      verdict: verdictWith({
        ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotFor(seat,): ArtifactConsistencyBallot {
          return ballotOf({
            seat,
            over: {
              quotes: [
                { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
                { label: 'OCCURRENCE 2', quote: 'the kitten naps', },
                { label: 'OCCURRENCE 3', quote: 'the kitten naps', },
              ],
            },
          },);
        },),
      },),
      occurrences: [
        placeOf({ position: 0, },),
        placeOf({
          position: 1,
          over: {
            rendering: { kind: 'located', quote: 'the kitten naps', start: 6, markAfter: '\'', },
            outcome: outcomes[0],
          },
        },),
        {
          ...placeOf({ position: 1, },),
          label: 'OCCURRENCE 3',
          sliceIndex: 2,
          rendering: { kind: 'located', quote: 'the kitten naps', start: 6, },
          outcome: outcomes[1],
          // A place the splice would move, left as it was, records why.
          ...((outcomes[1] === 'kept-current') ? { keptBecause: 'edge-reading-changes' as const, } : {}),
        },
      ],
      carries: [CARRY, { ...CARRY, label: 'OCCURRENCE 3', },],
    },
  });
}

/**
 The refusal for an outcome the ballots, the renderings and the carries do not give.

 @param place - position of the place

 @param expected - what the outcome should be

 @returns Refusal as text

 @example
 ```ts
 const text = outcomeRefusal({ place: 1, expected: 'consistent', },);
 ```
 */
function outcomeRefusal({ place, expected, }: { readonly place: number; readonly expected: string; },): string {
  return `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[${String(place,)}].outcome: `
    + `expected ${expected}.`;
}

/**
 The generation after seventeen, which reads a first round with a lone leader and no strict majority as one to run off and
 reads a slot beside any quote that contains the located words (the lead's decision of 2026-10-18).
 */
const GENERATION_EIGHTEEN = ARTIFACT_SCHEMA_VERSION_V18;

/**
 The labels of the split group's three places.
 */
const SPLIT_LABELS = ['OCCURRENCE 1', 'OCCURRENCE 2', 'OCCURRENCE 3',] as const;

/**
 A group of three places whose four usable ballots choose two, one and one, with the first place's quotes and slots as given: no
 place has a strict majority, so the group ends with no winner and every place untouched.

 @param first - quote and slot each of the four ballots gave for the first place

 @param repeat - repeated wording or frame

 @param recorded - first place's recorded rendering

 @param runOff - run-off recorded, none by default

 @returns Group

 @example
 ```ts
 const group = splitGroupOf({ first: [], repeat: groupOf().repeat, recorded: placeOf({ position: 0, }).rendering, },);
 ```
 */
function splitGroupOf(
  {
    first,
    repeat,
    recorded,
    runOff,
  }: {
    readonly first: readonly {
      readonly quote: string;
      readonly slot?: string;
    }[];
    readonly repeat: AskedGroup['repeat'];
    readonly recorded: ArtifactConsistencyOccurrence['rendering'];
    readonly runOff?: ArtifactConsistencyRunOff;
  },
): AskedGroup {
  /**
   The labels the four ballots choose, in the order of the seats.
   */
  const choices = [SPLIT_LABELS[0], SPLIT_LABELS[0], SPLIT_LABELS[1], SPLIT_LABELS[2],];
  return {
    repeat,
    occurrences: SPLIT_LABELS.map(function placeAt(
      label,
      position,
    ): ArtifactConsistencyOccurrence {
      return {
        label,
        sliceIndex: position,
        startOffset: 6,
        endOffset: 18,
        line: position + 1,
        rendering: (position === 0) ? recorded : { kind: 'located', quote: 'the kitten naps', start: 6, },
        outcome: 'untouched',
      };
    },),
    unplaced: [],
    verdict: {
      kind: 'asked',
      ballots: choices.map(function ballotFor(
        choice,
        seat,
      ): ArtifactConsistencyBallot {
        const given = first[seat];
        return ballotOf({
          seat: `seat-${String(seat,)}`,
          over: {
            choice,
            quotes: [
              { label: SPLIT_LABELS[0], quote: given?.quote ?? 'The cat naps', ...((given?.slot === undefined) ? {} : { slot: given.slot, }), },
              { label: SPLIT_LABELS[1], quote: 'the kitten naps', },
              { label: SPLIT_LABELS[2], quote: 'the kitten naps', },
            ],
          },
        },);
      },),
      usable: 4,
      quorum: 3,
      winner: { kind: 'none', reason: (runOff === undefined) ? 'no-majority' : 'run-off-tied', },
      ...((runOff === undefined) ? {} : { runOff, }),
    },
    carries: [],
  };
}

/**
 A run-off among the three places in which each of three seats chooses another place, so the second round splits too.
 */
const THREE_WAY_RUN_OFF: ArtifactConsistencyRunOff = {
  finalists: [...SPLIT_LABELS,],
  ballots: SPLIT_LABELS.map(function ballotFor(
    choice,
    seat,
  ): ArtifactConsistencyChoiceBallot {
    return { seat: `seat-${String(seat,)}`, reason: 'It words the original best.', choice, };
  },),
  usable: 3,
  quorum: 2,
};

/**
 The refusal of a run-off missing for a first round of two, one and one.
 */
const RUN_OFF_MISSING = `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
  + 'expected a run-off among OCCURRENCE 1, OCCURRENCE 2, OCCURRENCE 3, the finalists of ballots with no strict majority.';

/**
 The refusal of a run-off recorded for a first round the generation did not run off.
 */
const RUN_OFF_UNEXPECTED = `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.runOff: `
  + 'expected no run-off, since the usable ballots left no finalists.';

/**
 The refusal of a slot that is not the one the ballots give inside the located words.
 */
const SLOT_REFUSAL = `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering: `
  + 'expected the slot a strict majority of the usable ballots gave inside those words, and none where they gave none.';

/**
 The refusal of definitions a generation eighteen group records that move no slot.
 */
const DEFINITIONS_REFUSAL = `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering.definitions: `
  + 'expected the definitions that change the slot the usable ballots give inside those words, and none where they change none.';

/**
 The definition of the reference in the span `a[b]c`, as a page holds it in another slice.
 */
const DEFINITION_OF_B = '[b]: https://x.example/b';

/**
 A frame group whose four ballots all quoted the span `a[b]c` with the slot `b`, and whose first place records the given rendering.

 @param recorded - rendering the first place records

 @param runOff - run-off the generation requires for the split, none by default

 @returns Group

 @example
 ```ts
 const group = referenceGroupOf({ recorded: { kind: 'located', quote: 'a[b]c', start: 0, }, },);
 ```
 */
function referenceGroupOf(
  {
    recorded,
    runOff,
  }: {
    readonly recorded: ArtifactConsistencyOccurrence['rendering'];
    readonly runOff?: ArtifactConsistencyRunOff;
  },
): AskedGroup {
  return splitGroupOf({
    first: [1, 2, 3, 4,].map(function givenSlot(): { readonly quote: string; readonly slot: string; } {
      return { quote: 'a[b]c', slot: 'b', };
    },),
    repeat: FRAME,
    recorded,
    ...((runOff === undefined) ? {} : { runOff, }),
  },);
}

/**
 A frame whose ballots quoted nested spans of the first place: three of four quoted `cat naps in the sun` or a longer span that
 contains it, each with the slot `naps`, and the fourth quoted `cat naps` with the slot `cat`.
 */
const NESTED_SLOT_BALLOTS = [
  { quote: 'the big cat naps in the sun', slot: 'naps', },
  { quote: 'cat naps in the sun', slot: 'naps', },
  { quote: 'cat naps in the sun', slot: 'naps', },
  { quote: 'cat naps', slot: 'cat', },
] as const;

/**
 The frame the nested-slot cases repeat.
 */
const FRAME = { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', } as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertSpanGroupCoherent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A GENERATION SEVENTEEN GROUP OF TWO, ONE AND ONE WITH NO RUN-OFF, since the pass that wrote it ran off only the '
            + 'labels tied for the top count, and refuses a run-off recorded for a lone leader there',
          fn: async () => {
            /**
             The group as a generation seventeen pass wrote it.
             */
            const asWritten = splitGroupOf({
              first: [],
              repeat: groupOf().repeat,
              recorded: { kind: 'located', quote: 'The cat naps', start: 0, },
            },);
            expect([
              assertSpanGroupCoherent({ group: asWritten, path: GROUP, generation: 17, },),
              coherenceRefusal({ group: { ...asWritten, verdict: { ...asWritten.verdict, runOff: THREE_WAY_RUN_OFF, }, }, },),
            ],).toEqual([undefined, RUN_OFF_UNEXPECTED,],);
          },
        },),
        it({
          name: 'REFUSES A GENERATION EIGHTEEN GROUP OF TWO, ONE AND ONE WITH NO RUN-OFF, and accepts one that records the run-off '
            + 'among the three places, whose second round split too',
          fn: async () => {
            /**
             The group without and with the run-off.
             */
            const [bare, run,] = [undefined, THREE_WAY_RUN_OFF,].map(function groupWith(runOff,): AskedGroup {
              return splitGroupOf({
                first: [],
                repeat: groupOf().repeat,
                recorded: { kind: 'located', quote: 'The cat naps', start: 0, },
                ...((runOff === undefined) ? {} : { runOff, }),
              },);
            },);
            expect([
              coherenceRefusal({ group: nonNullishOrThrow(bare,), generation: GENERATION_EIGHTEEN, },),
              assertSpanGroupCoherent({ group: nonNullishOrThrow(run,), path: GROUP, generation: GENERATION_EIGHTEEN, },),
            ],).toEqual([RUN_OFF_MISSING, undefined,],);
          },
        },),
        it({
          name: 'READS A SLOT BESIDE ANY QUOTE THAT CONTAINS THE LOCATED WORDS FROM GENERATION EIGHTEEN, and by the exact quote before: '
            + 'the slot three of four ballots gave beside nested quotes is recorded in the new generation and absent in the old',
          fn: async () => {
            /**
             The group with the first place recorded with the slot or without it.
             */
            const [without, withSlot,] = [undefined, 'naps',].map(function groupWith(slot,): AskedGroup {
              return splitGroupOf({
                first: NESTED_SLOT_BALLOTS,
                repeat: FRAME,
                recorded: {
                  kind: 'located',
                  quote: 'cat naps in the sun',
                  start: 14,
                  ...((slot === undefined) ? {} : { slot, }),
                },
                runOff: THREE_WAY_RUN_OFF,
              },);
            },);
            expect([
              coherenceRefusal({ group: nonNullishOrThrow(without,), generation: GENERATION_EIGHTEEN, },),
              assertSpanGroupCoherent({ group: nonNullishOrThrow(withSlot,), path: GROUP, generation: GENERATION_EIGHTEEN, },),
            ],).toEqual([SLOT_REFUSAL, undefined,],);
            // Generation seventeen recorded no run-off for the split, so the same slots are read there without one.
            /**
             The group as generation seventeen recorded it: no run-off, and the winner of a split with no majority.
             */
            function old(group: AskedGroup,): AskedGroup {
              const { runOff: _dropped, ...verdict } = group.verdict;
              return { ...group, verdict: { ...verdict, winner: { kind: 'none', reason: 'no-majority', }, }, };
            }
            expect([
              assertSpanGroupCoherent({ group: old(nonNullishOrThrow(without,),), path: GROUP, generation: 17, },),
              coherenceRefusal({ group: old(nonNullishOrThrow(withSlot,),), generation: 17, },),
            ],).toEqual([undefined, SLOT_REFUSAL,],);
          },
        },),
        it({
          name: 'PINS THE LIMIT OF THE GENERATION SEVENTEEN SLOT RECOUNT: a span that holds a shortcut reference (`a[b]c`) is read '
            + 'without the definitions of the page, which a generation seventeen file does not record, so a pass that recorded no '
            + 'slot there (with the definition in another slice `b` is no stand) is refused, and the limit is documented, not hidden',
          fn: async () => {
            /**
             The group whose ballots gave the slot `b` beside the span `a[b]c` and whose first place records no slot, as a
             generation seventeen pass wrote a split: with no run-off.
             */
            const group = referenceGroupOf({ recorded: { kind: 'located', quote: 'a[b]c', start: 0, }, },);
            expect(coherenceRefusal({ group, generation: 17, },),).toBe(SLOT_REFUSAL,);
          },
        },),
        it({
          name: 'READS BACK A GENERATION EIGHTEEN GROUP WHOSE SPAN HOLDS A REFERENCE DEFINED IN ANOTHER SLICE, through the '
            + 'definitions the place records, and refuses the same record without them or with the slot the span alone gives',
          fn: async () => {
            /**
             The place as the pass records it: no slot, and the definition that makes `a[b]c` one word.
             */
            const withDefinitions: ArtifactConsistencyOccurrence['rendering'] = {
              kind: 'located',
              quote: 'a[b]c',
              start: 0,
              definitions: DEFINITION_OF_B,
            };
            expect([
              assertSpanGroupCoherent({
                group: referenceGroupOf({ recorded: withDefinitions, runOff: THREE_WAY_RUN_OFF, },),
                path: GROUP,
                generation: GENERATION_EIGHTEEN,
              },),
              coherenceRefusal({
                group: referenceGroupOf({ recorded: { kind: 'located', quote: 'a[b]c', start: 0, }, runOff: THREE_WAY_RUN_OFF, },),
                generation: GENERATION_EIGHTEEN,
              },),
              coherenceRefusal({
                group: referenceGroupOf({ recorded: { ...withDefinitions, slot: 'b', }, runOff: THREE_WAY_RUN_OFF, },),
                generation: GENERATION_EIGHTEEN,
              },),
            ],).toEqual([undefined, SLOT_REFUSAL, SLOT_REFUSAL,],);
          },
        },),
        it({
          name: 'REFUSES DEFINITIONS A GENERATION EIGHTEEN GROUP RECORDS THAT MOVE NO SLOT: the definition of another reference, '
            + 'and definitions beside a place of a group that is no frame',
          fn: async () => {
            expect([
              coherenceRefusal({
                group: referenceGroupOf({
                  recorded: { kind: 'located', quote: 'a[b]c', start: 0, slot: 'b', definitions: '[z]: https://x.example/z', },
                  runOff: THREE_WAY_RUN_OFF,
                },),
                generation: GENERATION_EIGHTEEN,
              },),
              coherenceRefusal({
                group: groupOf({
                  over: {
                    occurrences: [
                      placeOf({
                        position: 0,
                        over: { rendering: { kind: 'located', quote: 'The cat naps', start: 0, definitions: DEFINITION_OF_B, }, },
                      },),
                      placeOf({ position: 1, },),
                    ],
                  },
                }),
                generation: GENERATION_EIGHTEEN,
              },),
            ],).toEqual([DEFINITIONS_REFUSAL, DEFINITIONS_REFUSAL,],);
          },
        },),
        it({
          name: 'ACCEPTS A GROUP WHOSE COUNT, RENDERINGS, WINNER, CARRY AND OUTCOMES ARE WHAT ITS BALLOTS GIVE, the closing full '
            + 'stop of a quote and the words of the place\'s English counting as one span',
          fn: async () => {
            expect(assertSpanGroupCoherent({
              group: groupOf(),
              path: GROUP,
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'RECOUNTS BALLOTS THAT COPIED A MARK BESIDE THEIR WORDS BY THE WORDS ALONE, the same in generation sixteen, which '
            + 'recorded no marks, and in generation seventeen, whether or not the marks the English holds were recorded',
          fn: async () => {
            const copying = [ballotOf({ seat: 'hyper-cat', over: { quotes: [
              { label: 'OCCURRENCE 1', quote: '‘The cat naps’', },
              { label: 'OCCURRENCE 2', quote: '‘the kitten naps’', },
            ], }, },), ballotOf({ seat: 'bedrock-cat', over: { quotes: [
              { label: 'OCCURRENCE 1', quote: '‘The cat naps', },
              { label: 'OCCURRENCE 2', quote: 'the kitten naps’', },
            ], }, },), ballotOf({ seat: 'synthetic-cat', over: { quotes: [
              { label: 'OCCURRENCE 1', quote: 'The cat naps', },
              { label: 'OCCURRENCE 2', quote: 'the kitten naps', },
            ], }, },),];
            const bare = groupOf({ over: { verdict: verdictWith({ ballots: copying, },), }, },);
            const marked = groupOf({
              over: {
                verdict: verdictWith({ ballots: copying, },),
                occurrences: [
                  placeOf({
                    position: 0,
                    over: { rendering: { kind: 'located', quote: 'The cat naps', start: 0, markBefore: '‘', markAfter: '’', }, },
                  },),
                  placeOf({
                    position: 1,
                    over: { rendering: { kind: 'located', quote: 'the kitten naps', start: 6, markBefore: '‘', markAfter: '’', }, },
                  },),
                ],
              },
            },);
            expect([
              assertSpanGroupCoherent({ group: bare, path: GROUP, generation: 16, },),
              assertSpanGroupCoherent({ group: bare, path: GROUP, generation: 17, },),
              assertSpanGroupCoherent({ group: marked, path: GROUP, generation: 17, },),
            ],).toEqual([undefined, undefined, undefined,],);
          },
        },),
        it({
          name: 'READS WORDS THAT HOLD ONE APOSTROPHE AT AN EDGE FROM GENERATION SEVENTEEN, which the ballots\' folded words '
            + 'lack and a recount cannot show proven, refuses what the pass could not have put there, and reads the same '
            + 'words in generation sixteen as before',
          fn: async () => {
            /**
             The group with the first place's recorded words replaced and the second place's worded alike at its edges, so
             the splice between them is one code can make.
             */
            function groupWith({ quote, }: { readonly quote: string; },): AskedGroup {
              return groupOf({
                over: {
                  occurrences: [
                    placeOf({ position: 0, over: { rendering: { kind: 'located', quote, start: 0, }, }, },),
                    placeOf({
                      position: 1,
                      over: {
                        rendering: { kind: 'located', quote: quote.replace('The cat naps', 'the kitten naps',), start: 6, },
                      },
                    },),
                  ],
                },
              },);
            }
            const accepted = ['The cat naps’', '’The cat naps', 'The cat naps\'', 'The cat naps\u{02BC}', '’The cat naps’',];
            const refused = ['‘The cat naps', 'The cat naps‘', 'The cat naps’’', 'The cat naps ’', '’’The cat naps', '’ The cat naps',];
            const reason = 'words that hold at most one apostrophe at an edge, directly beside a letter or a digit and never '
              + 'the opening curly single mark, since the pass puts only a proven apostrophe there';
            expect({
              accepted: accepted.map(function readsIt(quote,): unknown {
                return assertSpanGroupCoherent({ group: groupWith({ quote, },), path: GROUP, generation: 17, },);
              },),
              refused: refused.map(function refusedFor(quote,): string {
                return coherenceRefusal({ group: groupWith({ quote, },), },);
              },),
              sixteen: [...accepted, ...refused,].map(function readsIt(quote,): unknown {
                return assertSpanGroupCoherent({ group: groupWith({ quote, },), path: GROUP, generation: 16, },);
              },),
            },).toEqual({
              accepted: accepted.map(function noRefusal(): undefined {
                return undefined;
              },),
              refused: refused.map(function refusal(): string {
                return `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering: expected ${reason}.`;
              },),
              sixteen: [...accepted, ...refused,].map(function noRefusal(): undefined {
                return undefined;
              },),
            },);
          },
        },),
        it({
          name: 'STILL READS WORDS THAT START OR END WITH A LOW, REVERSED OR ANGLE SINGLE MARK, which a quote may name and the '
            + 'locator spans as loose marks of their own, though the record now stores these marks beside a span: only the '
            + 'opening curly mark and the apostrophe-like marks without a letter inside are refused at an edge',
          fn: async () => {
            /**
             The group whose ballots all quote the words with the marks at their edges, and whose places record them.
             */
            function groupWith({ quote, }: { readonly quote: string; },): AskedGroup {
              /**
               The second place's words, worded alike at its edges.
               */
              const second = quote.replace('The cat naps', 'the kitten naps',);
              return groupOf({
                over: {
                  verdict: verdictWith({
                    ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotQuoting(seat,) {
                      return ballotOf({
                        seat,
                        over: {
                          quotes: [
                            { label: 'OCCURRENCE 1', quote, },
                            { label: 'OCCURRENCE 2', quote: second, },
                          ],
                        },
                      },);
                    },),
                  },),
                  occurrences: [
                    placeOf({ position: 0, over: { rendering: { kind: 'located', quote, start: 0, }, }, },),
                    placeOf({ position: 1, over: { rendering: { kind: 'located', quote: second, start: 6, }, }, },),
                  ],
                },
              },);
            }
            const quotes = [
              '‹The cat naps›',
              '\u{201A}The cat naps\u{201B}',
              '‹The cat naps',
              'The cat naps›',
              '\u{201B}The cat naps',
              'The cat naps\u{201A}',
            ];
            expect(quotes.map(function isAccepted(quote,): boolean {
              return assertSpanGroupCoherent({ group: groupWith({ quote, },), path: GROUP, generation: 17, },) === undefined;
            },),).toEqual(quotes.map(function holdsAccepted(): boolean {
              return true;
            },),);
          },
        },),
        it({
          name: 'REFUSES A SEAT HEARD TWICE AND A USABLE COUNT THAT IS NOT THE BALLOTS\'',
          fn: async () => {
            expect([
              withVerdict({
                over: { ballots: [ballotOf({ seat: 'hyper-cat', },), ballotOf({ seat: 'hyper-cat', },), ballotOf({ seat: 'x', },),], },
              }),
              withVerdict({ over: { usable: 2, }, }),
            ].map(function refusedFor(group,): string {
              return coherenceRefusal({ group, },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.ballots[1].seat: `
                + 'expected a seat heard once, since a gather hears each seat once.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.usable: `
                + 'expected the number of ballots that can be counted against the places.',
            ],);
          },
        },),
        it({
          name: 'REFUSES A LOCATED RENDERING THAT IS NOT WORDS A STRICT MAJORITY OF THE QUOTES CONTAIN, though it is a '
            + 'string some ballot gave, and a reason the ballots contradict',
          fn: async () => {
            expect([
              groupOf({ over: { occurrences: [placeOf({ position: 0, },), placeOf({
                position: 1,
                over: { rendering: { kind: 'located', quote: 'the kitten dozes', start: 6, }, },
              },),], }, }),
              groupOf({ over: { occurrences: [placeOf({ position: 0, },), placeOf({
                position: 1,
                over: { rendering: { kind: 'unlocated', reason: 'no-agreement', }, outcome: 'unlocated', },
              },),], }, }),
            ].map(function refusedFor(group,): string {
              return coherenceRefusal({ group, },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected words that the quotes of more than half of all the usable ballots contain, as the place\'s English holds them.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering.reason: `
                + 'expected a reason the usable ballots do not contradict.',
            ],);
          },
        },),
        it({
          name: 'ALLOWS A REASON THE EARLIER READING OF THE BALLOTS ALLOWED, though the spans the quotes contain contradict it: three '
            + 'quotes of one place that no one text is a strict majority of, but that a shorter quote is inside all of, leave the '
            + 'place unlocated for no agreement in a file written before the spans were located by containment',
          fn: async () => {
            /**
             The group in which the second place is unlocated for the reason, its three ballots quoting it as given.
             */
            function unlocatedFor(
              { reason, quotes, }: { readonly reason: 'no-agreement'; readonly quotes: readonly string[]; },
            ): AskedGroup {
              return groupOf({
                over: {
                  verdict: verdictWith({
                    ballots: ballotsQuotingSecond({ quotes, }),
                  },),
                  occurrences: [
                    placeOf({ position: 0, },),
                    placeOf({ position: 1, over: { rendering: { kind: 'unlocated', reason, }, outcome: 'unlocated', }, },),
                  ],
                  carries: [],
                },
              },);
            }
            expect({
              nestedQuotes: assertSpanGroupCoherent({
                group: unlocatedFor({ reason: 'no-agreement', quotes: ['the kitten naps', 'kitten naps', 'the kitten naps in the sun',], }),
                path: GROUP,
                generation: 17,
              },),
              oneText: coherenceRefusal({
                group: unlocatedFor({ reason: 'no-agreement', quotes: ['the kitten naps', 'the kitten naps', 'the kitten naps',], }),
              },),
            },).toEqual({
              nestedQuotes: undefined,
              oneText: `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering.reason: `
                + 'expected a reason the usable ballots do not contradict.',
            },);
          },
        },),
        it({
          name: 'READS A REASON AGAINST THE FIRST CANDIDATE THAT SHOWS A READER SOMETHING, and a located rendering against the '
            + 'candidates that do: a longer candidate that shows nothing (a lone variation selector twice) is neither the words an '
            + 'empty reason is read against nor words a place can be located by',
          fn: async () => {
            /**
             Three quotes of the second place: two selectors alone, a letter alone, and the letter beside the two selectors.
             The first and the second are each inside two of the three quotes, so both are candidates, and the longer of them
             shows a reader nothing.
             */
            const quotes = ['\u{FE0F}\u{FE0F}', 'x', 'x \u{FE0F}\u{FE0F}',];
            /**
             The group in which the second place has the rendering, its three ballots quoting it as given.
             */
            function renderedAs(
              { rendering, outcome, carries = [], }: {
                readonly rendering: ArtifactConsistencyOccurrence['rendering'];
                readonly outcome: ArtifactConsistencyOccurrence['outcome'];
                readonly carries?: readonly ArtifactConsistencyCarry[];
              },
            ): AskedGroup {
              return groupOf({
                over: {
                  verdict: verdictWith({
                    ballots: ballotsQuotingSecond({ quotes, }),
                  },),
                  occurrences: [placeOf({ position: 0, },), placeOf({ position: 1, over: { rendering, outcome, }, },),],
                  carries,
                },
              },);
            }
            expect({
              emptyReason: coherenceRefusal({
                group: renderedAs({ rendering: { kind: 'unlocated', reason: 'empty', }, outcome: 'unlocated', }),
              },),
              locatedBySelectors: coherenceRefusal({
                group: renderedAs({
                  rendering: { kind: 'located', quote: '\u{FE0F}\u{FE0F}', start: 6, },
                  outcome: 'replaced',
                }),
              },),
              locatedByLetter: assertSpanGroupCoherent({
                group: renderedAs({ rendering: { kind: 'located', quote: 'x', start: 6, }, outcome: 'replaced', carries: [CARRY,], }),
                path: GROUP,
                generation: 17,
              },),
            },).toEqual({
              emptyReason: `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering.reason: `
                + 'expected a reason the usable ballots do not contradict.',
              locatedBySelectors: `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected words that the quotes of more than half of all the usable ballots contain, as the place\'s English holds them.',
              locatedByLetter: undefined,
            },);
          },
        },),
        it({
          name: 'REFUSES A SLOT IN A REPEATED RUN\'S RENDERING AND A WINNER THAT IS NOT THE MAJORITY\'S CHOICE',
          fn: async () => {
            expect([
              groupOf({ over: { occurrences: [placeOf({
                position: 0,
                over: { rendering: { kind: 'located', quote: 'The cat naps', start: 0, slot: 'cat', }, },
              },), placeOf({ position: 1, },),], }, }),
              withVerdict({ over: { winner: { kind: 'chosen', label: 'OCCURRENCE 2', }, }, }),
              withVerdict({ over: { winner: { kind: 'none', reason: 'no-majority', }, }, }),
            ].map(function refusedFor(group,): string {
              return coherenceRefusal({ group, },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering: `
                + 'expected the slot a strict majority of the usable ballots gave inside those words, and none where they gave none.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.winner: `
                + 'expected the winner a strict majority of the usable ballots, reaching the quorum, give, after the run-off when they tied.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.verdict.winner: `
                + 'expected the winner a strict majority of the usable ballots, reaching the quorum, give, after the run-off when they tied.',
            ],);
          },
        },),
        it({
          name: 'REFUSES AN OUTCOME THE BALLOTS, THE RENDERINGS AND THE CARRIES DO NOT GIVE: a chosen place that is not chosen, a '
            + 'place already carrying the words that is not consistent, and a place a majority keeps that is not kept',
          fn: async () => {
            const kept = {
              ...groupOf().verdict,
              ballots: [
                ballotOf({ seat: 'hyper-cat', over: { keep: ['OCCURRENCE 2',], }, },),
                ballotOf({ seat: 'bedrock-cat', over: { keep: ['OCCURRENCE 2',], }, },),
                ballotOf({ seat: 'synthetic-cat', },),
              ],
            };
            expect([
              groupOf({ over: { occurrences: [placeOf({ position: 0, over: { outcome: 'replaced', }, },), placeOf({ position: 1, },),], }, }),
              groupOf({
                over: {
                  occurrences: [
                    placeOf({ position: 0, },),
                    placeOf({ position: 1, over: { rendering: { kind: 'located', quote: 'The cat naps', start: 6, }, }, },),
                  ],
                  carries: [],
                },
              }),
              groupOf({ over: { verdict: kept, carries: [], }, }),
            ].map(function refusedFor(group,): string {
              return coherenceRefusal({ group, },);
            },),).toEqual([
              outcomeRefusal({ place: 0, expected: 'chosen, as the ballots and the renderings give for this place', }),
              `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[1].rendering: `
                + 'expected words that the quotes of more than half of all the usable ballots contain, as the place\'s English holds them.',
              outcomeRefusal({ place: 1, expected: 'kept-context, as the ballots and the renderings give for this place', }),
            ],);
          },
        },),
        it({
          name: 'READS A PLACE A MAJORITY KEEPS AS KEPT-CONTEXT WITH NO CARRY, a place no quote located as unlocated and every '
            + 'place untouched where no place won',
          fn: async () => {
            const kept = verdictWith({
              ballots: [
                ballotOf({ seat: 'hyper-cat', over: { keep: ['OCCURRENCE 2',], }, },),
                ballotOf({ seat: 'bedrock-cat', over: { keep: ['OCCURRENCE 2',], }, },),
                ballotOf({ seat: 'synthetic-cat', },),
              ],
            },);
            const unlocated = verdictWith({
              ballots: [
                ballotOf({ seat: 'hyper-cat', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps.', },], }, },),
                ballotOf({ seat: 'bedrock-cat', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps.', },], }, },),
                ballotOf({ seat: 'synthetic-cat', over: { quotes: [{ label: 'OCCURRENCE 1', quote: 'The cat naps.', },], }, },),
              ],
            },);
            const noWinner = verdictWith({
              usable: 1,
              ballots: [ballotOf({ seat: 'hyper-cat', },),],
              winner: { kind: 'none', reason: 'no-quorum', },
            },);
            expect([
              groupOf({
                over: {
                  verdict: kept,
                  occurrences: [placeOf({ position: 0, },), placeOf({ position: 1, over: { outcome: 'kept-context', }, },),],
                  carries: [],
                },
              }),
              groupOf({
                over: {
                  verdict: unlocated,
                  occurrences: [
                    placeOf({ position: 0, },),
                    placeOf({
                      position: 1,
                      over: { rendering: { kind: 'unlocated', reason: 'no-agreement', }, outcome: 'unlocated', },
                    },),
                  ],
                  carries: [],
                },
              }),
              groupOf({
                over: {
                  verdict: noWinner,
                  occurrences: [
                    placeOf({ position: 0, over: { outcome: 'untouched', }, },),
                    placeOf({ position: 1, over: { outcome: 'untouched', }, },),
                  ],
                  carries: [],
                },
              }),
            ].map(function recounted(group,): unknown {
              return assertSpanGroupCoherent({
                group,
                path: GROUP,
                generation: 17,
              },);
            },),).toEqual([undefined, undefined, undefined,],);
          },
        },),
        it({
          name: 'REFUSES CARRIES THAT ARE NOT ONE FOR EACH RECEIVING PLACE IN ORDER, and a receiving place whose outcome its '
            + 'carry\'s decision does not allow',
          fn: async () => {
            expect([
              groupOf({ over: { carries: [], }, }),
              groupOf({ over: { carries: [CARRY, { ...CARRY, label: 'OCCURRENCE 3', },], }, }),
              groupOf({
                over: {
                  carries: [
                    {
                      ...CARRY,
                      ballots: CARRY.ballots.map(function choosesCurrent(ballot,): ArtifactConsistencyCarry['ballots'][number] {
                        return { ...ballot, choice: 'CANDIDATE 1', };
                      },),
                      decision: { kind: 'chosen', label: 'CANDIDATE 1', },
                    },
                  ],
                },
              }),
              groupOf({ over: { occurrences: [placeOf({ position: 0, },), placeOf({ position: 1, over: { outcome: 'kept-context', }, },),], }, }),
            ].map(function refusedFor(group,): string {
              return coherenceRefusal({ group, },);
            },),).toEqual([
              `ArtifactParseError: artifact parse failed at ${GROUP}.carries: `
                + 'expected a carry for OCCURRENCE 2, which the chosen words are carried to.',
              `ArtifactParseError: artifact parse failed at ${GROUP}.carries: `
                + 'expected one carry for each place the chosen words are carried to, in the order of the places.',
              outcomeRefusal({ place: 1, expected: 'kept-current, for a carry that decided as recorded', }),
              outcomeRefusal({
                place: 1,
                expected: 'replaced, kept-floor, overlapped, kept-current, for a carry that decided as recorded',
              }),
            ],);
          },
        },),
        it({
          name: 'REFUSES A SPLICE CANDIDATE FOR A FRAME\'S PLACE WHOSE SLOT OR WHOSE CHOSEN FRAME\'S SLOT THE BALLOTS DID NOT PIN '
            + 'DOWN, since code cannot move the words without both slots',
          fn: async () => {
            /**
             A refrain whose ballots gave no slot for either place.
             */
            const frame = groupOf({
              over: {
                repeat: { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', },
                occurrences: [
                  { ...placeOf({ position: 0, },), slotStartOffset: 9, slotEndOffset: 11, },
                  { ...placeOf({ position: 1, },), slotStartOffset: 33, slotEndOffset: 35, },
                ],
              },
            });
            expect(coherenceRefusal({ group: frame, },),).toBe(NO_SPLICE_FOR_OCCURRENCE_TWO,);
            expect(coherenceRefusal({ group: frame, generation: 16, },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.carries: `
                + 'expected no splice candidate for OCCURRENCE 2, whose frame or the chosen frame has no slot to move the words by.',
            );
            const alone: ArtifactConsistencyCarry = {
              ...CARRY,
              candidates: [nonNullishOrThrow(CARRY.candidates[0],),],
              ballots: [],
              usable: 0,
              quorum: 0,
              decision: { kind: 'none', reason: 'no-alternative', },
            };
            expect(assertSpanGroupCoherent({
              group: {
                ...frame,
                carries: [alone,],
                occurrences: [
                  { ...placeOf({ position: 0, },), slotStartOffset: 9, slotEndOffset: 11, },
                  { ...placeOf({ position: 1, over: { outcome: 'kept-current', }, },), slotStartOffset: 33, slotEndOffset: 35, },
                ],
              },
              path: GROUP,
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A SPLICE CANDIDATE FOR A PLACE WHOSE MARKS BESIDE ITS WORDS DIFFER FROM THE CHOSEN PLACE\'S in '
            + 'generation seventeen, which recorded them, and accepts one where the marks agree',
          fn: async () => {
            /**
             The group whose second place records the given fields in its rendering.
             */
            function receivingWith({ over, }: { readonly over: Record<string, unknown>; },): AskedGroup {
              return groupOf({
                over: {
                  occurrences: [
                    placeOf({ position: 0, },),
                    placeOf({
                      position: 1,
                      over: { rendering: { kind: 'located', quote: 'the kitten naps', start: 6, ...over, }, },
                    },),
                  ],
                },
              });
            }
            expect([
              coherenceRefusal({ group: receivingWith({ over: { markAfter: '\'', }, }), },),
              coherenceRefusal({ group: receivingWith({ over: { markBefore: '‘', }, }), },),
            ],).toEqual([NO_SPLICE_FOR_OCCURRENCE_TWO, NO_SPLICE_FOR_OCCURRENCE_TWO,],);
            expect(assertSpanGroupCoherent({
              group: groupOf({
                over: {
                  occurrences: [
                    placeOf({
                      position: 0,
                      over: { rendering: { kind: 'located', quote: 'The cat naps', start: 0, markAfter: '’', }, },
                    },),
                    placeOf({
                      position: 1,
                      over: { rendering: { kind: 'located', quote: 'the kitten naps', start: 6, markAfter: '\'', }, },
                    },),
                  ],
                },
              }),
              path: GROUP,
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A SPLICE CANDIDATE FOR A PLACE WHOSE WORDS END IN EMPHASIS BESIDE THE MARK WHILE THE CHOSEN WORDS END IN '
            + 'A LETTER in generation seventeen, which would change which apostrophes a letter attaches, accepts one where '
            + 'both end in a letter, and recounts the same group of generation sixteen as before',
          fn: async () => {
            /**
             The group whose chosen place ends in a letter before an apostrophe and whose second place holds the given words
             before one.
             */
            function groupQuoting({ second, }: { readonly second: string; },): AskedGroup {
              return groupOf({
                over: {
                  verdict: verdictWith({
                    ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotFor(seat,): ArtifactConsistencyBallot {
                      return ballotOf({
                        seat,
                        over: {
                          quotes: [
                            { label: 'OCCURRENCE 1', quote: 'The cats', },
                            { label: 'OCCURRENCE 2', quote: second, },
                          ],
                        },
                      },);
                    },),
                  },),
                  occurrences: [
                    placeOf({
                      position: 0,
                      over: { rendering: { kind: 'located', quote: 'The cats', start: 0, markAfter: '\'', }, },
                    },),
                    placeOf({
                      position: 1,
                      over: { rendering: { kind: 'located', quote: second, start: 6, markAfter: '\'', }, },
                    },),
                  ],
                },
              });
            }
            expect(coherenceRefusal({ group: groupQuoting({ second: 'the **kitten**', }), generation: 17, },),)
              .toBe(NO_SPLICE_FOR_OCCURRENCE_TWO,);
            expect(assertSpanGroupCoherent({
              group: groupQuoting({ second: 'the kittens', }),
              path: GROUP,
              generation: 17,
            },),).toBeUndefined();
            expect(assertSpanGroupCoherent({
              group: groupQuoting({ second: 'the **kitten**', }),
              path: GROUP,
              generation: 16,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A SPLICE CANDIDATE FOR A PLACE WHOSE QUOTATION MARKS INSIDE ITS WORDS DIFFER FROM THE CHOSEN PLACE\'S '
            + 'in generation seventeen, and recounts the same group of generation sixteen as before, which compared none',
          fn: async () => {
            /**
             Three ballots that quote the second place with quotation marks inside its words.
             */
            const quoting = verdictWith({
              ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotFor(seat,): ArtifactConsistencyBallot {
                return ballotOf({
                  seat,
                  over: {
                    quotes: [
                      { label: 'OCCURRENCE 1', quote: 'The cat naps.', },
                      { label: 'OCCURRENCE 2', quote: 'the "kitten" naps', },
                    ],
                  },
                },);
              },),
            },);
            const group = groupOf({
              over: {
                verdict: quoting,
                occurrences: [
                  placeOf({ position: 0, },),
                  placeOf({
                    position: 1,
                    over: { rendering: { kind: 'located', quote: 'the "kitten" naps', start: 6, }, },
                  },),
                ],
              },
            });
            expect(coherenceRefusal({ group, generation: 17, },),).toBe(NO_SPLICE_FOR_OCCURRENCE_TWO,);
            expect(assertSpanGroupCoherent({
              group,
              path: GROUP,
              generation: 16,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'ACCEPTS A PARTLY CARRIED UNIT IN GENERATION SEVENTEEN: a splice chosen for a carry that serves a place code can '
            + 'move and a place code cannot, the first kept-current and the second replaced',
          fn: async () => {
            expect(assertSpanGroupCoherent({
              group: partlyCarriedGroup({ outcomes: ['kept-current', 'replaced',], },),
              path: GROUP,
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES, IN GENERATION SEVENTEEN, A PLACE CODE CANNOT MOVE RECORDED AS REPLACED AFTER A SPLICE WAS CHOSEN, and '
            + 'accepts a place code can move recorded as kept-current after it, since the pass may leave it as it was',
          fn: async () => {
            expect(coherenceRefusal({ group: partlyCarriedGroup({ outcomes: ['replaced', 'replaced',], },), },),).toBe(
              outcomeRefusal({ place: 1, expected: 'kept-current, for a carry that decided as recorded', }),
            );
            expect(assertSpanGroupCoherent({
              group: partlyCarriedGroup({ outcomes: ['kept-current', 'kept-current',], },),
              path: GROUP,
              generation: 17,
            },),).toBeUndefined();
          },
        },),
        it({
          name: 'REFUSES A SPLICE CANDIDATE FOR A PLACE CODE CANNOT MOVE WHEN NO PLACE THAT SHARES ITS RECORD CAN BE SPLICED, '
            + 'even in generation seventeen: the other place of the group was carried under a record of its own',
          fn: async () => {
            /**
             The partly carried unit, with the place code can move recorded under a record that is not the first place's.
             */
            const group = partlyCarriedGroup({ outcomes: ['kept-current', 'replaced',], },);
            expect(coherenceRefusal({
              group: {
                ...group,
                carries: [
                  nonNullishOrThrow(group.carries?.[0],),
                  { ...CARRY, label: 'OCCURRENCE 3', usable: 2, quorum: 1, },
                ],
              },
            },),).toBe(NO_SPLICE_FOR_OCCURRENCE_TWO,);
          },
        },),
        it({
          name: 'ACCEPTS KEPT-CURRENT FOR A PLACE A CHOSEN SPLICE WOULD GIVE AN EDGE APOSTROPHE, and for any place whose plan is '
            + 'replace, since the record holds not the page that decides it; refuses a splice candidate for words that hold '
            + 'another number of block breaks',
          fn: async () => {
            /**
             A group whose chosen place and receiving place are given as the ballots quote them and as their renderings hold them.
             */
            function gainGroup(
              { chosen, receiving, outcome, }: {
                readonly chosen: readonly [string, string];
                readonly receiving: readonly [string, string];
                readonly outcome: ArtifactConsistencyOccurrence['outcome'];
              },
            ): AskedGroup {
              return groupOf({
                over: {
                  verdict: verdictWith({
                    ballots: ['hyper-cat', 'bedrock-cat', 'synthetic-cat',].map(function ballotFor(seat,): ArtifactConsistencyBallot {
                      return ballotOf({
                        seat,
                        over: {
                          quotes: [
                            { label: 'OCCURRENCE 1', quote: chosen[0], },
                            { label: 'OCCURRENCE 2', quote: receiving[0], },
                          ],
                        },
                      },);
                    },),
                  },),
                  occurrences: [
                    placeOf({ position: 0, over: { rendering: { kind: 'located', quote: chosen[1], start: 0, }, }, },),
                    placeOf({
                      position: 1,
                      over: {
                        rendering: { kind: 'located', quote: receiving[1], start: 6, },
                        outcome,
                        // A place left as it was after the splice was chosen records why.
                        ...((outcome === 'kept-current') ? { keptBecause: 'gain-pairs' as const, } : {}),
                      },
                    },),
                  ],
                },
              });
            }
            expect([
              assertSpanGroupCoherent({
                group: gainGroup({
                  chosen: ['the parents', 'the parents’',],
                  receiving: ['the family', 'the family',],
                  outcome: 'kept-current',
                }),
                path: GROUP,
                generation: 17,
              },),
              assertSpanGroupCoherent({
                group: gainGroup({
                  chosen: ['the parents', 'the parents’',],
                  receiving: ['the family', 'the family',],
                  outcome: 'replaced',
                }),
                path: GROUP,
                generation: 17,
              },),
              assertSpanGroupCoherent({
                group: gainGroup({
                  chosen: ['the parents', 'the parents’',],
                  receiving: ['the folks', 'the folks’',],
                  outcome: 'kept-current',
                }),
                path: GROUP,
                generation: 17,
              },),
            ],).toEqual([undefined, undefined, undefined,],);
            expect(coherenceRefusal({
              group: gainGroup({
                chosen: ['the cat\n\nnaps', 'the cat\n\nnaps',],
                receiving: ['the kitten naps', 'the kitten naps',],
                outcome: 'replaced',
              }),
            },),).toBe(NO_SPLICE_FOR_OCCURRENCE_TWO,);
          },
        },),
        it({
          name: 'READS A PARTLY CARRIED UNIT OF GENERATION SIXTEEN BY THE OLD RULE, which compared no marks: the place that '
            + 'only a mark made unavailable is a place the splice replaces, so it cannot be recorded kept-current',
          fn: async () => {
            expect(coherenceRefusal({
              group: partlyCarriedGroup({ outcomes: ['kept-current', 'replaced',], },),
              generation: 16,
            },),).toBe(outcomeRefusal({
              place: 1,
              expected: 'replaced, kept-floor, overlapped, for a carry that decided as recorded',
            },),);
            // Generation sixteen recorded no marks, so a replace plan may not be kept-current there either.
            expect(coherenceRefusal({
              group: partlyCarriedGroup({ outcomes: ['kept-current', 'kept-current',], },),
              generation: 16,
            },),).toBe(outcomeRefusal({
              place: 1,
              expected: 'replaced, kept-floor, overlapped, for a carry that decided as recorded',
            },),);
          },
        },),
        it({
          name: 'REFUSES A CARRY WHOSE OWN BALLOTS DO NOT GIVE ITS DECISION, naming the carry under the group',
          fn: async () => {
            expect(coherenceRefusal({
              group: groupOf({ over: { carries: [{ ...CARRY, usable: 3, },], }, }),
            },),).toBe(
              `ArtifactParseError: artifact parse failed at ${GROUP}.carries[0].usable: `
                + 'expected the number of ballots that can be counted against the candidates.',
            );
          },
        },),
      ],
    },),
  ],
},);
