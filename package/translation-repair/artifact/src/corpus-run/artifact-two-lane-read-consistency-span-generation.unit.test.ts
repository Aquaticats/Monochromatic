/**
 Tests for the generation that says how a recount reads containment and inline code: a group written under generation eighteen or
 earlier is recounted as the pass that wrote it counted, with a joint of a longer word (`cat's`) beside a quote no bar to
 containment and with line endings that pad inline code left as shown separators, and a group written under generation nineteen
 is recounted as the locator reads them now. Each case stands a group the old reading accepts beside the same group stamped with
 the new generation, which refuses it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  ARTIFACT_SCHEMA_VERSION_V18,
  ARTIFACT_SCHEMA_VERSION_V19,
  type ArtifactConsistencyBallot,
  type ArtifactConsistencyGroup,
  type ArtifactConsistencyOccurrence,
  assertSpanGroupCoherent,
  type TwoLaneArtifactGeneration,
} from '../../dist/final/node/index.mjs';

/**
 Where the group sits in the section, for the refusals.
 */
const GROUP = 'cat.consistency.groups[0]';

/**
 The group the cases read: a repeated wording or frame in two places whose three ballots choose the first.
 */
type AskedGroup = ArtifactConsistencyGroup & { readonly verdict: { readonly kind: 'asked'; }; };

/**
 The refusal of a located rendering that is not words a majority of the quotes contain.
 */
const WORDS_REFUSAL = `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering: `
  + 'expected words that the quotes of more than half of all the usable ballots contain, as the place\'s English holds them.';

/**
 The refusal of a slot that is not the one the ballots give inside the located words.
 */
const SLOT_REFUSAL = `ArtifactParseError: artifact parse failed at ${GROUP}.occurrences[0].rendering: `
  + 'expected the slot a strict majority of the usable ballots gave inside those words, and none where they gave none.';

/**
 The frame the slot cases repeat.
 */
const FRAME = { kind: 'refrain', prefix: '所以这只猫一定是', suffix: '的小宝贝', } as const;

/**
 Inline code padded by line endings beside a word: CommonMark shows `kittens` where the padded code holds `tens`.
 */
const PADDED_CODE = 'The kit`\ntens\n` ate';

/**
 A group whose three ballots all choose the first place, with the quotes of each place and the rendering each records.

 @param quotes - what each ballot quoted for the first and the second place, and the slot it gave for the first

 @param first - first place's recorded rendering

 @param second - second place's recorded rendering

 @param repeat - repeated wording or frame

 @returns Group, second place consistent with the first and no carry

 @example
 ```ts
 const group = groupOf({ quotes: [], first, second, repeat: FRAME, },);
 ```
 */
function groupOf(
  {
    quotes,
    first,
    second,
    repeat,
  }: {
    readonly quotes: readonly {
      readonly first: string;
      readonly second: string;
      readonly slot?: string;
    }[];
    readonly first: ArtifactConsistencyOccurrence['rendering'];
    readonly second: ArtifactConsistencyOccurrence['rendering'];
    readonly repeat: AskedGroup['repeat'];
  },
): AskedGroup {
  return {
    repeat,
    occurrences: [
      {
        label: 'OCCURRENCE 1',
        sliceIndex: 0,
        startOffset: 6,
        endOffset: 18,
        line: 3,
        rendering: first,
        outcome: 'chosen',
      },
      {
        label: 'OCCURRENCE 2',
        sliceIndex: 1,
        startOffset: 30,
        endOffset: 42,
        line: 5,
        rendering: second,
        outcome: 'consistent',
      },
    ],
    unplaced: [],
    verdict: {
      kind: 'asked',
      ballots: quotes.map(function ballotFor(
        given,
        at,
      ): ArtifactConsistencyBallot {
        return {
          seat: `seat-${String(at,)}`,
          reason: 'The first place words it best.',
          quotes: [
            { label: 'OCCURRENCE 1', quote: given.first, ...((given.slot === undefined) ? {} : { slot: given.slot, }), },
            { label: 'OCCURRENCE 2', quote: given.second, ...((given.slot === undefined) ? {} : { slot: given.slot, }), },
          ],
          choice: 'OCCURRENCE 1',
          keep: [],
        };
      },),
      usable: quotes.length,
      quorum: 2,
      winner: { kind: 'chosen', label: 'OCCURRENCE 1', },
    },
    carries: [],
  };
}

/**
 The text of the refusal the recount raises for a group, or the word `accepted` where it raises none.

 @param group - group, asked of the bench

 @param generation - generation the section is read as written under

 @returns Refusal as text, or `accepted`

 @example
 ```ts
 const verdict = recountOf({ group, generation: ARTIFACT_SCHEMA_VERSION_V18, });
 ```
 */
function recountOf(
  {
    group,
    generation,
  }: {
    readonly group: AskedGroup;
    readonly generation: TwoLaneArtifactGeneration;
  },
): string {
  try {
    assertSpanGroupCoherent({
      group,
      path: GROUP,
      generation,
    },);
    return 'accepted';
  }
  catch (error) {
    if (!(error instanceof ArtifactParseError))
      throw error;
    return String(error,);
  }
}

/**
 The group in which two judges quoted "cat's bowl" and one quoted "cat" for both places, and both are recorded as located by "cat".
 */
const JOINT_GROUP = groupOf({
  quotes: [
    { first: 'cat\'s bowl', second: 'cat\'s bowl', },
    { first: 'cat\'s bowl', second: 'cat\'s bowl', },
    { first: 'cat', second: 'cat', },
  ],
  first: { kind: 'located', quote: 'cat', start: 0, },
  second: { kind: 'located', quote: 'cat', start: 0, },
  repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
});

/**
 The group in which every judge quoted inline code padded by line endings with the slot "kit", and both places record that slot.
 */
const PADDED_SLOT_GROUP = groupOf({
  quotes: [
    { first: PADDED_CODE, second: PADDED_CODE, slot: 'kit', },
    { first: PADDED_CODE, second: PADDED_CODE, slot: 'kit', },
    { first: PADDED_CODE, second: PADDED_CODE, slot: 'kit', },
  ],
  first: { kind: 'located', quote: PADDED_CODE, start: 0, slot: 'kit', },
  second: { kind: 'located', quote: PADDED_CODE, start: 0, slot: 'kit', },
  repeat: FRAME,
});

await describe({
  name: '',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    describe({
      name: assertSpanGroupCoherent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS A GENERATION EIGHTEEN GROUP WHOSE AGREEMENT HOLDS ONLY UNDER THE EARLIER CONTAINMENT, and refuses the same '
            + 'group stamped nineteen: two judges who quoted "cat\'s bowl" and one who quoted "cat" agreed on "cat" when a joint '
            + 'beside the match was no bar to containment, and a quote of "cat" is contained in one quote only from nineteen',
          fn: async () => {
            expect({
              eighteen: recountOf({ group: JOINT_GROUP, generation: ARTIFACT_SCHEMA_VERSION_V18, },),
              seventeen: recountOf({ group: JOINT_GROUP, generation: 17, },),
              nineteen: recountOf({ group: JOINT_GROUP, generation: ARTIFACT_SCHEMA_VERSION_V19, },),
            },).toEqual({
              eighteen: 'accepted',
              seventeen: 'accepted',
              nineteen: WORDS_REFUSAL,
            },);
          },
        },),
        it({
          name: 'KEEPS THE CONTAINMENT OF THE LOCATOR FOR A GENERATION NINETEEN GROUP: the words "cat\'s bowl" the two judges quoted '
            + 'are accepted in both readings, and so are the words of judges who all quoted "cat"',
          fn: async () => {
            /**
             The group with the first place recorded under the words and the quotes it is read with.
             */
            function groupQuoting({ first, quoted, }: { readonly first: string; readonly quoted: readonly string[]; },): AskedGroup {
              return groupOf({
                quotes: quoted.map(function quoteFor(text,): { readonly first: string; readonly second: string; } {
                  return { first: text, second: text, };
                },),
                first: { kind: 'located', quote: first, start: 0, },
                second: { kind: 'located', quote: first, start: 0, },
                repeat: { kind: 'maximal', wording: '窗台上的阳光让小猫睡着了', },
              },);
            }
            expect([
              recountOf({
                group: groupQuoting({ first: 'cat\'s bowl', quoted: ['cat\'s bowl', 'cat\'s bowl', 'cat',], }),
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },),
              recountOf({
                group: groupQuoting({ first: 'cat', quoted: ['cat', 'cat', 'cat',], }),
                generation: ARTIFACT_SCHEMA_VERSION_V19,
              },),
              recountOf({
                group: groupQuoting({ first: 'cat', quoted: ['cat', 'cat', 'cat',], }),
                generation: ARTIFACT_SCHEMA_VERSION_V18,
              },),
            ],).toEqual(['accepted', 'accepted', 'accepted',],);
          },
        },),
        it({
          name: 'ACCEPTS A GENERATION EIGHTEEN SLOT GIVEN BESIDE A QUOTE THAT HOLDS THE LOCATED WORDS BEFORE A JOINT, and refuses the '
            + 'same group stamped nineteen: the words "big cat" are contained in three of four quotes either way, but the judge who '
            + 'quoted "big cat\'s bowl" with the slot "big" counts for the slot only while the joint bars nothing',
          fn: async () => {
            /**
             The group: four judges, one quoted the longer text with the slot, two quoted the words with it and one without it.
             */
            const group = groupOf({
              quotes: [
                { first: 'big cat\'s bowl', second: 'big cat\'s bowl', slot: 'big', },
                { first: 'big cat', second: 'big cat', slot: 'big', },
                { first: 'big cat', second: 'big cat', slot: 'big', },
                { first: 'big cat', second: 'big cat', },
              ],
              first: { kind: 'located', quote: 'big cat', start: 0, slot: 'big', },
              second: { kind: 'located', quote: 'big cat', start: 0, slot: 'big', },
              repeat: FRAME,
            });
            expect({
              eighteen: recountOf({ group, generation: ARTIFACT_SCHEMA_VERSION_V18, },),
              nineteen: recountOf({ group, generation: ARTIFACT_SCHEMA_VERSION_V19, },),
            },).toEqual({
              eighteen: 'accepted',
              nineteen: SLOT_REFUSAL,
            },);
          },
        },),
        it({
          name: 'ACCEPTS A GENERATION EIGHTEEN GROUP WHOSE SLOT STANDS BESIDE INLINE CODE PADDED BY LINE ENDINGS, and refuses the same '
            + 'group stamped nineteen: the pass that wrote it found "kit" in the span as a word of its own, and from nineteen the '
            + 'padding line endings are not shown, so "kit" is half of "kittens" and the ballots give no slot',
          fn: async () => {
            expect({
              eighteen: recountOf({ group: PADDED_SLOT_GROUP, generation: ARTIFACT_SCHEMA_VERSION_V18, },),
              nineteen: recountOf({ group: PADDED_SLOT_GROUP, generation: ARTIFACT_SCHEMA_VERSION_V19, },),
            },).toEqual({
              eighteen: 'accepted',
              nineteen: SLOT_REFUSAL,
            },);
          },
        },),
        it({
          name: 'ACCEPTS A GENERATION NINETEEN GROUP THAT RECORDS NO SLOT BESIDE THE PADDED CODE, which generation eighteen refuses '
            + 'for the slot it left out',
          fn: async () => {
            /**
             The same group with neither place recording the slot.
             */
            const withoutSlot: AskedGroup = {
              ...PADDED_SLOT_GROUP,
              occurrences: PADDED_SLOT_GROUP.occurrences
                .map(function bare(occurrence,): ArtifactConsistencyOccurrence {
                  return { ...occurrence, rendering: { kind: 'located', quote: PADDED_CODE, start: 0, }, };
                },),
            };
            expect({
              eighteen: recountOf({ group: withoutSlot, generation: ARTIFACT_SCHEMA_VERSION_V18, },),
              nineteen: recountOf({ group: withoutSlot, generation: ARTIFACT_SCHEMA_VERSION_V19, },),
            },).toEqual({
              eighteen: SLOT_REFUSAL,
              nineteen: 'accepted',
            },);
          },
        },),
      ],
    },),
  ],
},);
