/**
 Tests for what a settled translate document says about slices a GUARD
 refused, which is three different guards wearing one count.

 WHY THIS FILE EXISTS. `assembleTranslation` reports `refusedSliceCount` as
 the size of a filter naming three dispositions, and `alignmentRefusals`
 writes one finding per refusal in three shapes. Until this file existed, every
 case reaching either of them drove ONE of the three, `refused-alignment`, so dropping
 either of the other two from the filter would have cost the artifact a
 refusal and failed nothing. The two other kinds were pinned at the slice that
 produces them and nowhere downstream.

 A FOURTH SLICE IS SETTLED NORMALLY so the count cannot pass by counting every
 slice, and so the finding list cannot pass by naming every slice.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  assembleTranslation,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

//region Fixtures

/**
 Original, four sections, one of them quoting a line.
 */
const SOURCE_TEXT = `## 甲

猫猫在窗台上睡觉。

## 乙

猫猫记得那句话：

> 碗里永远有猫粮。

## 丙

胡须酱追着蝴蝶跑。

## 丁

猫猫又睡着了。
`;

/**
 Archive translation of it, which every guard here keeps.
 */
const TARGET_TEXT = `## Alpha

The cat sleeps on the windowsill.

## Beta

The cat remembers the promise:

> The bowl is always full.

## Gamma

Whiskers chases butterflies.

## Delta

The cat is asleep again.
`;

/**
 Declared name the archive carries and one replacement drops.
 */
const DECLARED_NAME = 'Whiskers';

/**
 What each slice's stage produced and how its slice was disposed of, in
 document order.

 ONE SLICE PER REFUSAL KIND, then one settled slice, so the three counts a
 reader compares (slices, refusals, changes) are three different numbers.
 */
const PLANNED = [
  {
    disposition: 'refused-alignment',
    stageText: 'Short.',
    dropped: [],
  },
  {
    disposition: 'refused-quote-loss',
    stageText: 'The cat remembers the promise: the bowl is always full.',
    dropped: [],
    // What the guard compared, which the record stores: the finding names these
    // rather than recounting the record's texts (ledger B42).
    quotedPassages: {
      archive: 1,
      replacement: 0,
    },
  },
  {
    disposition: 'refused-declared-name',
    stageText: 'The cat chases butterflies.',
    dropped: [DECLARED_NAME,],
  },
  {
    disposition: 'stage-result',
    stageText: '',
    dropped: [],
  },
] as const;

/**
 Builds one settled record shaped as the driver settles one.

 CAST because `TranslateStageResult` carries a producer, a tally and two
 chosen indices this test never reaches: assembly reads `disposition`,
 `sliceIndex`, `outputText`, `changed`, `alignment`, `droppedDeclaredNames`,
 `quotedPassages` and `stageResult.text`, and those are the fields spelled
 out here. The
 sibling `translate-lane-wordings.unit.test.ts` builds its records the same
 way and for the same reason.

 @param sliceIndex - slice this record settles

 @param incumbentText - archive wording at that slice, which every refusal
 here keeps

 @param at - position in `PLANNED`, which decides the disposition

 @returns Record the assembly can read

 @example
 ```ts
 const record = recordFor({ sliceIndex: 0, incumbentText: 'The cat naps.', at: 0, },);
 ```
 */
function recordFor(
  {
    sliceIndex,
    incumbentText,
    at,
  }: {
    readonly sliceIndex: number;
    readonly incumbentText: string;
    readonly at: number;
  },
): TranslateSliceRecord {
  /**
   This slice's plan, which the fixture guarantees exists.
   */
  const planned = PLANNED[at];
  return {
    kind: 'translate-slice',
    schemaVersion: 1,
    sliceIndex,
    outputText: incumbentText,
    // FALSE ON ALL FOUR: a refused slice keeps the archive, and the settled one
    // here is a slice the judges left alone. Nothing is replaced, so the
    // footnote guard has nothing to withdraw and the document that comes back
    // must be the archive byte for byte.
    changed: false,
    disposition: planned?.disposition,
    findings: [],
    droppedDeclaredNames: planned?.dropped,
    ...((planned !== undefined) && ('quotedPassages' in planned)
      ? { quotedPassages: planned.quotedPassages, }
      : {}),
    alignment: {
      kind: 'incumbent-dominates-source',
      sourceCodePoints: 11,
      incumbentCodePoints: 33,
      minProtectedIncumbent: 20,
      maxRatio: 2,
    },
    stageResult: {
      text: (planned?.stageText === '')
        ? incumbentText
        : planned?.stageText,
      origin: 'fresh',
      decision: 'judged',
      voteWeight: 1,
      ballots: [],
      heardTranslators: 2,
      candidateCount: 2,
      slate: [],
      perCandidate: [],
      findings: [],
    },
  } as unknown as TranslateSliceRecord;
}

/**
 Runs one assembly over the fixture and returns what it said and returned.

 @returns Assembled result beside every logged line

 @example
 ```ts
 const { result, said, } = await assembledFixture();
 ```
 */
async function assembledFixture(): Promise<{
  readonly result: ReturnType<typeof assembleTranslation>;
  readonly said: readonly string[];
}> {
  /**
   Pair both lanes would run over.
   */
  const prepared = await prepareDocumentPair({
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
  },);

  /**
   Lines the assembly logged.
   */
  const said: string[] = [];
  return {
    result: assembleTranslation({
      prepared,
      settled: prepared.slices
        .map(function toRecord(
          slice,
          at,
        ): TranslateSliceRecord {
          return recordFor({
            sliceIndex: slice.target
              .sliceIndex,
            incumbentText: slice.target
              .text,
            at,
          },);
        },),
      unfilled: [],
      // NOT ZERO, so the summary line's four numbers are four different
      // numbers and a swap between any two of them is visible.
      resumedSliceCount: 2,
      findings: [],
      l: capturingLogger({ messages: said, },),
    },),
    said,
  };
}

//endregion Fixtures

await describe({
  name: assembleTranslation.name,
  children: [
    it({
      name: 'COUNTS a slice refused for quote loss and a slice refused for a dropped declared name '
        + 'alongside one refused on alignment, which is what the artifact reports as `refusedSliceCount`. '
        + 'Until this case was written, every case drove the alignment kind alone, so dropping either '
        + 'of the other two from the filter would have understated the count and failed nothing',
      fn: async () => {
        const { result, } = await assembledFixture();
        expect(result.sliceCount,).toBe(4,);
        expect(result.refusedSliceCount,).toBe(3,);
        // The fourth slice is settled, not refused, so a filter that counted
        // every record would read 4 here.
        expect(result.changedSliceCount,).toBe(0,);
        expect(result.changedSliceIndices,).toEqual([],);
        expect(result.withdrawnSliceIndices,).toEqual([],);
      },
    },),
    it({
      name: 'NAMES each refusal in its own shape, one finding per refused slice and none for the slice '
        + 'nobody refused, so a run whose refusals are all one kind reads differently from a run whose '
        + 'refusals are all another',
      fn: async () => {
        const { result, } = await assembledFixture();

        /**
         Every finding this assembly wrote about a refusal.
         */
        const refusals = result.findings
          .filter(function namesARefusal(finding,): boolean {
            return finding.startsWith('translate-refused-',);
          },);
        expect(refusals,).toHaveLength(3,);
        expect(refusals[0],).toContain('translate-refused-alignment (slice 0:',);
        // The two counts come from OPPOSITE sides of the stored pair, so a swap
        // between them reads as a replacement that gained a quotation.
        expect(refusals[1],).toBe(
          'translate-refused-quote-loss (slice 1: archive carries 1 quoted passage, replacement carries 0 quoted '
            + 'passages)',
        );
        expect(refusals[2],).toBe(
          `translate-refused-declared-name (slice 2: archive text carries "${DECLARED_NAME}" and the `
            + 'replacement does not; keeping the archive text)',
        );
      },
    },),
    it({
      name: 'SAYS a guard refused them rather than naming alignment for all three, which sent a reader '
        + 'looking for an alignment finding two of the three never wrote',
      fn: async () => {
        const { result, said, } = await assembledFixture();
        expect(said,).toContain(
          'translated 4 slices (2 resumed): 0 changed, 3 refused by a guard',
        );
        // Nothing was replaced, so the archive comes back untouched.
        expect(result.translatedText,).toBe(TARGET_TEXT,);
        expect(result.status,).toBe('complete',);
      },
    },),
  ],
},);
