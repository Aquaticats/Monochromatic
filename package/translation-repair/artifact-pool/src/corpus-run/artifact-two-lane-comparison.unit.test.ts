/**
 Tests for version 2's own comparison rules.

 WHAT THESE PIN is what version 2 MEANS, which is a different question from
 what the pipeline currently computes. The rules are duplicated from the live
 comparator on purpose: the vocabulary froze the words a row may use and this
 freezes how a row is decided, so a later change to the pipeline's verdicts
 cannot reinterpret artifacts already on disk under an unchanged version.

 They also pin the pair the rules exist to keep apart, which equal text cannot:
 a blank slice the archive does translate and a passage it never translated
 both carry the empty string.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactComparisonRow,
  ArtifactComparisonError,
  type ArtifactDeliveryRow,
  assertDerivationsAgree,
  compareLanes,
  comparisonRowDifferences,
  decisionsEqual,
  deliveriesEqual,
  outcomesEqual,
} from '../../dist/final/node/index.mjs';

/**
 Archive wording of the slice both lanes work on.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Original of that slice.
 */
const SOURCE_NAP = '猫猫在窗台上睡觉。';

/**
 Builds one delivery row, defaulting everything a case does not care about.

 @param sliceIndex - slice this row is for

 @param incumbentKind - whether the archive holds wording here

 @param incumbentText - archive's own wording

 @param shippedText - what this lane's document carries

 @param outcome - what this lane did

 @param delivery - how the document came to carry what it carries

 @returns Row shaped as version 2 records one

 @example
 ```ts
 const row = row({ sliceIndex: 0, shippedText: ARCHIVE_NAP, },);
 ```
 */
function deliveryRow(
  {
    sliceIndex,
    incumbentKind,
    incumbentText,
    shippedText,
    outcome,
    delivery,
  }: {
    readonly sliceIndex: number;
    readonly incumbentKind: 'present' | 'absent';
    readonly incumbentText: string;
    readonly shippedText: string;
    readonly outcome: ArtifactDeliveryRow['outcome'];
    readonly delivery: ArtifactDeliveryRow['delivery'];
  },
): ArtifactDeliveryRow {
  return {
    sliceIndex,
    sourceText: SOURCE_NAP,
    incumbentKind,
    incumbentText,
    outcome,
    shippedText,
    delivery,
  };
}

/**
 One row where the lane kept the archive's wording after examining it.

 @param sliceIndex - slice this row is for

 @returns Row carrying a decision that matches the archive

 @example
 ```ts
 const kept = keptArchive({ sliceIndex: 0, },);
 ```
 */
function keptArchive({ sliceIndex, }: { readonly sliceIndex: number; },): ArtifactDeliveryRow {
  return deliveryRow({
    sliceIndex,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    shippedText: ARCHIVE_NAP,
    outcome: {
      kind: 'decided',
      acceptedText: ARCHIVE_NAP,
    },
    delivery: { kind: 'incumbent-retained', },
  },);
}

/**
 One row where the lane shipped a replacement.

 @param sliceIndex - slice this row is for

 @param text - wording it shipped

 @returns Row carrying a shipped replacement

 @example
 ```ts
 const shipped = shippedText({ sliceIndex: 0, text: 'The cat naps.', },);
 ```
 */
function shipped(
  {
    sliceIndex,
    text,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
  },
): ArtifactDeliveryRow {
  return deliveryRow({
    sliceIndex,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    shippedText: text,
    outcome: {
      kind: 'decided',
      acceptedText: text,
    },
    delivery: { kind: 'replacement-shipped', },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: compareLanes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'names the four ways two documents can differ on a slice, which is the reading a later analysis '
              + 'joins on: neither moved, one moved, both moved to the same words, both moved to different ones',
          fn: async () => {
            /**
             Wording the repair lane shipped.
             */
            const mended = 'The cat is asleep on the windowsill.';

            /**
             Wording the translate lane shipped where it differed.
             */
            const fresh = 'The cat naps on the windowsill.';

            /**
             Four slices, one per verdict the rules can reach with an archive
             present.
             */
            const rows = compareLanes({
              repair: [
                keptArchive({ sliceIndex: 0, },),
                shipped({ sliceIndex: 1, text: mended, },),
                shipped({ sliceIndex: 2, text: fresh, },),
                shipped({ sliceIndex: 3, text: mended, },),
              ],
              translate: [
                keptArchive({ sliceIndex: 0, },),
                keptArchive({ sliceIndex: 1, },),
                shipped({ sliceIndex: 2, text: fresh, },),
                shipped({ sliceIndex: 3, text: fresh, },),
              ],
            },);
            expect(rows.map(function toVerdict(one,): string {
              return one.laneRelation;
            },),).toEqual([
              'archive-stands',
              'repair-only',
              'both-agree',
              'both-differ',
            ],);
          },
        },),
        it({
          name:
            'separates a passage the archive NEVER translated from its wording standing, which equal text '
              + 'cannot: a blank slice and an untranslated one both carry the empty string, and reporting the '
              + 'second as the archive standing tells a reader a translation is being kept where none exists',
          fn: async () => {
            /**
             One anchor neither lane filled, and one blank slice the archive does
             hold.
             */
            const rows = compareLanes({
              repair: [
                deliveryRow({
                  sliceIndex: 0,
                  incumbentKind: 'absent',
                  incumbentText: '',
                  shippedText: '',
                  outcome: { kind: 'not-applicable', },
                  delivery: { kind: 'gap-remains', },
                },),
                deliveryRow({
                  sliceIndex: 1,
                  incumbentKind: 'present',
                  incumbentText: '',
                  shippedText: '',
                  outcome: {
                    kind: 'decided',
                    acceptedText: '',
                  },
                  delivery: { kind: 'incumbent-retained', },
                },),
              ],
              translate: [
                deliveryRow({
                  sliceIndex: 0,
                  incumbentKind: 'absent',
                  incumbentText: '',
                  shippedText: '',
                  outcome: { kind: 'unfilled', },
                  delivery: { kind: 'gap-remains', },
                },),
                deliveryRow({
                  sliceIndex: 1,
                  incumbentKind: 'present',
                  incumbentText: '',
                  shippedText: '',
                  outcome: {
                    kind: 'decided',
                    acceptedText: '',
                  },
                  delivery: { kind: 'incumbent-retained', },
                },),
              ],
            },);
            expect(rows.map(function toVerdict(one,): string {
              return one.laneRelation;
            },),).toEqual([
              'gap-remains',
              'archive-stands',
            ],);
          },
        },),
        it({
          name:
            'reports the two lanes` DECISIONS apart from what their documents carry, so a slice one lane '
              + 'never decided reads as not comparable rather than as the two having chosen differently',
          fn: async () => {
            /**
             One slice the repair lane heard nobody about.
             */
            const rows = compareLanes({
              repair: [
                deliveryRow({
                  sliceIndex: 0,
                  incumbentKind: 'present',
                  incumbentText: ARCHIVE_NAP,
                  shippedText: ARCHIVE_NAP,
                  outcome: { kind: 'incumbent-fallback', },
                  delivery: { kind: 'incumbent-retained', },
                },),
              ],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            expect(rows[0]?.decisionComparison,).toEqual({
              kind: 'not-comparable',
              undecidedLanes: ['repair',],
            },);

            // And the document verdict still says both carry the archive, which is
            // a different fact from either lane having chosen it.
            expect(rows[0]?.laneRelation,).toBe('archive-stands',);
          },
        },),
        it({
          name:
            'REFUSES two ledgers of different lengths, and two that name different slices at one position: '
              + 'a comparison joined by index alone would accept rows in the wrong order, which is one of the '
              + 'things reading by position exists to catch',
          fn: async () => {
            /**
             What lengthsDiffer raised, read for its class as well as its wording.
             */
            const refusalOfLengthsDiffer = caught(function lengthsDiffer() {
              compareLanes({
                repair: [
                  keptArchive({ sliceIndex: 0, },),
                  keptArchive({ sliceIndex: 1, },),
                ],
                translate: [keptArchive({ sliceIndex: 0, },),],
              },);
            },);

            expect(refusalOfLengthsDiffer,).toBeInstanceOf(ArtifactComparisonError,);
            expect((refusalOfLengthsDiffer as Error).message,)
              .toBe('the repair ledger covers 2 slices and the translate ledger 1, so they describe different '
                + 'preparations',);

            /**
             What positionsDisagree raised, read for its class as well as its wording.
             */
            const refusalOfPositionsDisagree = caught(function positionsDisagree() {
              compareLanes({
                repair: [
                  keptArchive({ sliceIndex: 0, },),
                  keptArchive({ sliceIndex: 1, },),
                ],
                translate: [
                  keptArchive({ sliceIndex: 1, },),
                  keptArchive({ sliceIndex: 0, },),
                ],
              },);
            },);

            expect(refusalOfPositionsDisagree,).toBeInstanceOf(ArtifactComparisonError,);
            expect((refusalOfPositionsDisagree as Error).message,)
              .toBe('position 0 names slice 0 in the repair ledger and slice 1 in the translate ledger',);
          },
        },),
        it({
          name:
            'REFUSES two ledgers that disagree about the ORIGINAL at one position, which is how a pair built '
              + 'over different slicings shows up: the slice numbers can still line up while the two lanes were '
              + 'reading different sentences',
          fn: async () => {
            /**
             What sourcesDisagree raised, read for its class as well as its wording.
             */
            const refusalOfSourcesDisagree = caught(function sourcesDisagree() {
              compareLanes({
                repair: [keptArchive({ sliceIndex: 0, },),],
                translate: [
                  {
                    ...keptArchive({ sliceIndex: 0, },),
                    sourceText: '猫猫在门口等着。',
                  },
                ],
              },);
            },);

            expect(refusalOfSourcesDisagree,).toBeInstanceOf(ArtifactComparisonError,);
            expect((refusalOfSourcesDisagree as Error).message,)
              .toBe('slice 0 carries a different original in each ledger, so the two ledgers were built over '
                + 'different slicings',);
          },
        },),
        it({
          name:
            'REFUSES two ledgers that disagree about the ARCHIVE`S WORDING at one slice while agreeing on its '
              + 'original, which is two preparations of one slicing against different archive states',
          fn: async () => {
            /**
             What wordingsDisagree raised, read for its class as well as its wording.
             */
            const refusalOfWordingsDisagree = caught(function wordingsDisagree() {
              compareLanes({
                repair: [keptArchive({ sliceIndex: 0, },),],
                translate: [
                  {
                    ...keptArchive({ sliceIndex: 0, },),
                    incumbentText: 'The cat sleeps on the step.',
                  },
                ],
              },);
            },);

            expect(refusalOfWordingsDisagree,).toBeInstanceOf(ArtifactComparisonError,);
            expect((refusalOfWordingsDisagree as Error).message,)
              .toBe('slice 0 carries a different archive wording in each ledger, so the two ledgers were built '
                + 'over different preparations',);
          },
        },),
        it({
          name:
            'REFUSES two ledgers that disagree about whether the archive translates a slice, even where both '
              + 'carry the same text, since that disagreement is exactly the pair equal text hides',
          fn: async () => {
            /**
             What kindsDisagree raised, read for its class as well as its wording.
             */
            const refusalOfKindsDisagree = caught(function kindsDisagree() {
              compareLanes({
                repair: [
                  deliveryRow({
                    sliceIndex: 0,
                    incumbentKind: 'present',
                    incumbentText: '',
                    shippedText: '',
                    outcome: {
                      kind: 'decided',
                      acceptedText: '',
                    },
                    delivery: { kind: 'incumbent-retained', },
                  },),
                ],
                translate: [
                  deliveryRow({
                    sliceIndex: 0,
                    incumbentKind: 'absent',
                    incumbentText: '',
                    shippedText: '',
                    outcome: { kind: 'unfilled', },
                    delivery: { kind: 'gap-remains', },
                  },),
                ],
              },);
            },);

            expect(refusalOfKindsDisagree,).toBeInstanceOf(ArtifactComparisonError,);
            expect((refusalOfKindsDisagree as Error).message,)
              .toBe('the archive\'s wording at slice 0 is present in the repair ledger and absent in the translate '
                + 'ledger',);
          },
        },),
      ],
    },),

    describe({
      name: assertDerivationsAgree.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ACCEPTS two derivations that match row for row, which is what makes either one answerable for '
              + 'the other while the pipeline`s rules and version 2`s rules still say the same thing',
          fn: async () => {
            /**
             One comparison, derived once.
             */
            const rows = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            assertDerivationsAgree({
              frozen: rows,
              live: rows,
            },);
          },
        },),
        it({
          name:
            'REFUSES a disagreement, because an artifact written while the two derivations differ would mean '
              + 'something the version number does not say: a stopped pass is the cheap outcome, since whoever '
              + 'changed the rules then decides whether version 2 changed with them',
          fn: async () => {
            /**
             What version 2's rules say about one kept slice.
             */
            const frozen = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);

            /**
             The same rows with one lane relation changed, standing in for a pipeline
             whose rules have moved.
             */
            const live = frozen.map(function retitle(row,) {
              return {
                ...row,
                laneRelation: 'both-agree' as const,
              };
            },);
            /**
             What derivationsDiffer raised, read for its class as well as its wording.
             */
            const refusalOfDerivationsDiffer = caught(function derivationsDiffer() {
              assertDerivationsAgree({
                frozen,
                live,
              },);
            },);

            expect(refusalOfDerivationsDiffer,).toBeInstanceOf(ArtifactComparisonError,);
            // The field is named; the rows, which carry slice text, are not quoted.
            expect((refusalOfDerivationsDiffer as Error).message,)
              .toBe('version 2 and the pipeline disagree about slice 0 on laneRelation; one of them changed, and '
                + 'which artifacts mean what depends on which',);
          },
        },),
        it({
          name:
            'ACCEPTS two derivations whose rows carry the same values in a different KEY ORDER, which is what '
              + 'the reader will hold: a row parsed out of a file is ordered however the file wrote it, and '
              + 'calling that a changed rule would stop a pass over a difference no reader can see',
          fn: async () => {
            /**
             One comparison, derived once.
             */
            const frozen = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);

            /**
             The same rows with every key written in the opposite order, standing
             in for rows read back off disk.
             */
            const live = frozen.map(function reorderKeys(row,): ArtifactComparisonRow {
              return Object.fromEntries(
                Object.entries(row,)
                  .toReversed(),
              ) as ArtifactComparisonRow;
            },);

            // POSITIVE CONTROL for the case itself: unless the reordering actually
            // changed the serialized bytes, this case would pass against the
            // stringify comparison it exists to keep from coming back.
            expect(JSON.stringify(live[0],),).not
              .toBe(JSON.stringify(frozen[0],),);
            assertDerivationsAgree({
              frozen,
              live,
            },);
          },
        },),
        it({
          name:
            'REFUSES derivations of different lengths, so a comparator that dropped or added a row is caught '
              + 'before the row-by-row reading starts and reports the counts rather than a field',
          fn: async () => {
            /**
             One row, against nothing.
             */
            const frozen = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            /**
             What lengthsDiffer raised, read for its class as well as its wording.
             */
            const refusalOfLengthsDiffer = caught(function lengthsDiffer() {
              assertDerivationsAgree({
                frozen,
                live: [],
              },);
            },);

            expect(refusalOfLengthsDiffer,).toBeInstanceOf(ArtifactComparisonError,);
            expect((refusalOfLengthsDiffer as Error).message,)
              .toBe('version 2 derives 1 comparison row where the pipeline derives 0, so the two no longer '
                + 'describe one comparison',);
          },
        },),
      ],
    },),

    describe({
      name: 'comparison row equality',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'reads the UNION members apart from their names: two outcomes both decided on different wording '
              + 'are different, while two carrying the same name and nothing else are the same',
          fn: async () => {
            expect(outcomesEqual({
              left: {
                kind: 'decided',
                acceptedText: ARCHIVE_NAP,
              },
              right: {
                kind: 'decided',
                acceptedText: 'The cat sleeps on the windowsill.',
              },
            },),).toBe(false,);
            expect(outcomesEqual({
              left: { kind: 'incumbent-fallback', },
              right: { kind: 'incumbent-fallback', },
            },),).toBe(true,);
            expect(outcomesEqual({
              left: { kind: 'unfilled', },
              right: { kind: 'not-evaluated', },
            },),).toBe(false,);
          },
        },),
        it({
          name:
            'reads a withdrawal`s REASON, so a replacement pulled for assembly integrity is not the same '
              + 'delivery as one pulled because the document was blocked as untranslated',
          fn: async () => {
            expect(deliveriesEqual({
              left: {
                kind: 'replacement-withdrawn',
                reason: 'assembly-integrity',
              },
              right: {
                kind: 'replacement-withdrawn',
                reason: 'blocked-non-translation',
              },
            },),).toBe(false,);
            expect(deliveriesEqual({
              left: { kind: 'incumbent-retained', },
              right: { kind: 'incumbent-retained', },
            },),).toBe(true,);
          },
        },),
        it({
          name:
            'reads `undecidedLanes` IN ORDER and by length, because the field is stated as lane order: a '
              + 'reversed pair names a different lane first, and a longer list names a lane the other does not',
          fn: async () => {
            expect(decisionsEqual({
              left: {
                kind: 'not-comparable',
                undecidedLanes: [
                  'repair',
                  'translate',
                ],
              },
              right: {
                kind: 'not-comparable',
                undecidedLanes: [
                  'translate',
                  'repair',
                ],
              },
            },),).toBe(false,);
            expect(decisionsEqual({
              left: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
              right: {
                kind: 'not-comparable',
                undecidedLanes: [
                  'repair',
                  'translate',
                ],
              },
            },),).toBe(false,);
            expect(decisionsEqual({
              left: {
                kind: 'comparable',
                verdict: 'same',
              },
              right: {
                kind: 'comparable',
                verdict: 'different',
              },
            },),).toBe(false,);
          },
        },),
        it({
          name:
            'ACCEPTS two readings saying the same thing, of either kind, and REFUSES a comparable reading beside '
              + 'one that is not, from either side, since whether the two lanes could be compared is part of what '
              + 'the row says',
          fn: async () => {
            expect(decisionsEqual({
              left: {
                kind: 'comparable',
                verdict: 'same',
              },
              right: {
                kind: 'comparable',
                verdict: 'same',
              },
            },),).toBe(true,);
            expect(decisionsEqual({
              left: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
              right: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
            },),).toBe(true,);
            expect(decisionsEqual({
              left: {
                kind: 'comparable',
                verdict: 'same',
              },
              right: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
            },),).toBe(false,);
            expect(decisionsEqual({
              left: {
                kind: 'not-comparable',
                undecidedLanes: ['repair',],
              },
              right: {
                kind: 'comparable',
                verdict: 'same',
              },
            },),).toBe(false,);
          },
        },),
        it({
          name:
            'answers over EVERY field version 2 owns, so a row differing in exactly one of them names that one '
              + 'whichever one it is: a check reading only some fields would pass artifacts it should stop',
          fn: async () => {
            /**
             One row every entry of `variants` changes exactly one field of.
             */
            const [row,] = compareLanes({
              repair: [shipped({ sliceIndex: 0, text: 'The cat naps.', },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            if (row === undefined)
              throw new Error('the comparison produced no rows to vary',);
            expect(comparisonRowDifferences({
              left: row,
              right: row,
            },),).toStrictEqual([],);

            /**
             One altered row per field, each differing from `row` in that field
             alone.
             */
            const variants: readonly ArtifactComparisonRow[] = [
              {
                ...row,
                sliceIndex: 1,
              },
              {
                ...row,
                incumbentKind: 'absent',
              },
              {
                ...row,
                incumbentText: 'The cat dozes.',
              },
              {
                ...row,
                repairText: 'The cat dozes.',
              },
              {
                ...row,
                translateText: 'The cat dozes.',
              },
              {
                ...row,
                laneRelation: 'both-differ',
              },
              {
                ...row,
                repairOutcome: { kind: 'unfilled', },
              },
              {
                ...row,
                translateOutcome: { kind: 'unfilled', },
              },
              {
                ...row,
                decisionComparison: {
                  kind: 'not-comparable',
                  undecidedLanes: ['repair',],
                },
              },
              {
                ...row,
                repairDelivery: { kind: 'gap-remains', },
              },
              {
                ...row,
                translateDelivery: { kind: 'gap-remains', },
              },
            ];
            expect(variants.map(function differingCount(variant,): number {
              return comparisonRowDifferences({
                left: row,
                right: variant,
              },)
                .length;
            },),).toEqual(variants.map(function exactlyOne(): number {
              return 1;
            },),);
          },
        },),
      ],
    },),

    describe({
      name: comparisonRowDifferences.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES no field for a row compared with itself',
          fn: async () => {
            /**
             Rows derived once from two agreeing ledgers.
             */
            const rows = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            expect(rows.length,).toBe(1,);
            /**
             The one row.
             */
            const row = rows[0] as ArtifactComparisonRow;
            expect(comparisonRowDifferences({
              left: row,
              right: row,
            },),).toStrictEqual([],);
          },
        },),
        it({
          name: 'NAMES the one field a retitled row differs on, and nothing else',
          fn: async () => {
            /**
             Rows derived once from two agreeing ledgers.
             */
            const rows = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            /**
             The one row.
             */
            const row = rows[0] as ArtifactComparisonRow;
            expect(comparisonRowDifferences({
              left: row,
              right: {
                ...row,
                laneRelation: 'both-agree',
              },
            },),).toStrictEqual(['laneRelation',],);
          },
        },),
        it({
          name: 'NAMES differing fields in row order, text before relation, and never a value',
          fn: async () => {
            /**
             Rows derived once from two agreeing ledgers.
             */
            const rows = compareLanes({
              repair: [keptArchive({ sliceIndex: 0, },),],
              translate: [keptArchive({ sliceIndex: 0, },),],
            },);
            /**
             The one row.
             */
            const row = rows[0] as ArtifactComparisonRow;
            /**
             Names of the fields that differ.
             */
            const differing = comparisonRowDifferences({
              left: row,
              right: {
                ...row,
                repairText: 'Whiskers stayed on the sill.',
                laneRelation: 'both-agree',
              },
            },);
            expect(differing,).toStrictEqual(['repairText', 'laneRelation',],);
            expect(differing.join(' ',),).not.toContain('Whiskers',);
          },
        },),
      ],
    },),
  ],
},);
