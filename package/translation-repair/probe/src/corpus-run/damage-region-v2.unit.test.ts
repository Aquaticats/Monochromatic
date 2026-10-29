/**
 Tests for reading damage regions out of a version 2 delivery ledger.

 The case that matters is WHICH ROWS COUNT. The version 1 draw read a
 repair-lane issue list, and carrying that habit forward would have produced a
 draw covering less than half the regions where this pipeline replaced text,
 while describing itself as a draw over the shipped regions. Measured over the
 six settled entries the split is 32 repair against 37 translate, so the error
 would not have been small and nothing in the output would have shown it.

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
  DamageRegionError,
  regionIdOf,
  regionsOfLane,
} from '../../dist/final/node/index.mjs';

/**
 Builds one delivery row.

 @param sliceIndex - slice index

 @param kind - what the lane did with the slice

 @param incumbentKind - whether the archive had wording there at all

 @returns Row shaped as the ledger carries it

 @example
 ```ts
 const row = rowOf({ sliceIndex: 0, kind: 'replacement-shipped', },);
 ```
 */
function rowOf(
  {
    sliceIndex,
    kind,
    incumbentKind = 'present',
  }: {
    readonly sliceIndex: number;
    readonly kind: 'replacement-shipped' | 'replacement-withdrawn' | 'incumbent-retained';
    readonly incumbentKind?: 'present' | 'absent';
  },
): Parameters<typeof regionsOfLane>[0]['rows'][number] {
  /**
   Delivery, whose withdrawn form carries a reason the others do not.
   */
  const delivery = (kind === 'replacement-withdrawn')
    ? {
      kind,
      reason: 'assembly-integrity' as const,
    }
    : { kind, };

  return {
    sliceIndex,
    sourceText: `窗台上有第${String(sliceIndex,)}只猫。`,
    incumbentKind,
    incumbentText: (incumbentKind === 'present')
      ? `Cat ${String(sliceIndex,)} sits on the sill.`
      : '',
    outcome: { kind: 'decided', },
    shippedText: `Cat ${String(sliceIndex,)} is perched on the windowsill.`,
    delivery,
  } as Parameters<typeof regionsOfLane>[0]['rows'][number];
}

/**
 Selection an artifact carries once a contest has settled it.
 */
const CONTESTED = { kind: 'contested', slices: [], } as unknown as
  Parameters<typeof regionsOfLane>[0]['laneSelection'];

/**
 Selection an artifact carries while nobody has decided it.
 */
const UNDECIDED = { kind: 'pending-human-decision', } as unknown as
  Parameters<typeof regionsOfLane>[0]['laneSelection'];

/**
 How many slice indices the shared readings cover, which is more than any
 case here uses so a new case need not extend it.
 */
const COVERED_SLICES = 16;

/**
 Builds a reading carrying wording for every slice a case might use.

 DEFAULTS TO EACH ROW'S OWN SHIPPED WORDING, so the annotation reads
 `survives` unless a case deliberately says otherwise. `rowOf` generates its
 texts from the index, so the reading can be built without the rows.

 @param text - wording to claim would stand, defaulting to what the lane
 shipped at that index

 @param decidedBy - stage whose decision survived

 @returns Readings by chunk index

 @example
 ```ts
 const readings = readingsOf({ text: 'Something else entirely.', },);
 ```
 */
function readingsOf(
  {
    text,
    decidedBy = 'consolidation',
  }: {
    readonly text?: string;
    readonly decidedBy?: string;
  } = {},
): Parameters<typeof regionsOfLane>[0]['readings'] {
  return new Map(
    Array.from(
      { length: COVERED_SLICES, },
      function perSlice(_unused, sliceIndex,): readonly [
        number,
        unknown,
      ] {
        return [
          sliceIndex,
          {
            kind: 'wording',
            text: text ?? `Cat ${String(sliceIndex,)} is perched on the windowsill.`,
            decidedBy,
          },
        ];
      },
    ),
  ) as Parameters<typeof regionsOfLane>[0]['readings'];
}

/**
 Readings under which every lane wording survives, which is what the cases
 about population and text want: they are not about the annotation.
 */
const SURVIVING = readingsOf();

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: regionsOfLane.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS ONLY THE ROWS THAT SHIPPED A REPLACEMENT, since a withdrawn one put nothing in '
            + 'the document and a retained one left the archive wording alone: asking whether either '
            + 'damaged the text asks about an edit that never happened',
          fn: async () => {
            /**
             One of each disposition.
             */
            const census = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: SURVIVING,
              lane: 'repair',
              rows: [
                rowOf({
                  sliceIndex: 0,
                  kind: 'replacement-shipped',
                },),
                rowOf({
                  sliceIndex: 1,
                  kind: 'replacement-withdrawn',
                },),
                rowOf({
                  sliceIndex: 2,
                  kind: 'incumbent-retained',
                },),
              ],
            },);

            expect(census.regions.length,).toBe(1,);
            expect(census.regions[0]?.sliceIndex,).toBe(0,);
            expect(census.filledWithoutIncumbent,).toBe(0,);
          },
        },),

        it({
          name: 'SETS ASIDE a slice filled where the archive had no wording, and COUNTS it rather than '
            + 'dropping it: nothing was replaced so no edit could have damaged anything, and a silent '
            + 'drop would make the pool look smaller than the run without saying why',
          fn: async () => {
            const census = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: SURVIVING,
              lane: 'translate',
              rows: [
                rowOf({
                  sliceIndex: 0,
                  kind: 'replacement-shipped',
                },),
                rowOf({
                  sliceIndex: 1,
                  kind: 'replacement-shipped',
                  incumbentKind: 'absent',
                },),
              ],
            },);

            expect(census.regions.length,).toBe(1,);
            expect(census.filledWithoutIncumbent,).toBe(1,);
          },
        },),

        it({
          name: 'CARRIES THE LANE ON EVERY REGION, which is what lets the two be separated afterwards: '
            + 'they answer different questions, and a pooled rate with no label cannot be split back '
            + 'apart once it is written down',
          fn: async () => {
            /**
             The same slice index, shipped by both lanes, which happens.
             */
            const repair = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: SURVIVING,
              lane: 'repair',
              rows: [rowOf({
                sliceIndex: 3,
                kind: 'replacement-shipped',
              },),],
            },);
            const translate = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: SURVIVING,
              lane: 'translate',
              rows: [rowOf({
                sliceIndex: 3,
                kind: 'replacement-shipped',
              },),],
            },);

            expect(repair.regions[0]?.lane,).toBe('repair',);
            expect(translate.regions[0]?.lane,).toBe('translate',);

            // Distinct identities, so one slice shipped by both lanes is two
            // regions rather than one overwriting the other in a keyed collection.
            expect(repair.regions[0]?.regionId,).not.toBe(translate.regions[0]?.regionId,);
          },
        },),

        it({
          name: 'TAKES THE TEXTS FROM THE ROW rather than from anywhere else, so the sheet asks about '
            + 'the wording the judges actually saw',
          fn: async () => {
            const census = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: SURVIVING,
              lane: 'repair',
              rows: [rowOf({
                sliceIndex: 5,
                kind: 'replacement-shipped',
              },),],
            },);

            /**
             Region built from the row.
             */
            const [region,] = census.regions;
            expect(region?.entryId,).toBe('Tabby',);
            expect(region?.sourceText,).toBe('窗台上有第5只猫。',);
            expect(region?.incumbentText,).toBe('Cat 5 sits on the sill.',);
            expect(region?.shippedText,).toBe('Cat 5 is perched on the windowsill.',);
          },
        },),

        it({
          name: 'KEEPS A REGION A LATER STAGE OVERRULED, and NAMES the stage. Rebuilding this population '
            + 'from what would ship would cut it from 378 regions to 30 and empty 33 of 47 artifacts, '
            + 'because on an entry nobody has decided the archive stands and nothing replaces anything. '
            + 'The lane still made the edit, so the damage question is still answerable; what the '
            + 'annotation adds is that no reader of a document would meet the result',
          fn: async () => {
            const census = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: readingsOf({ text: 'The third rendering says something else.', },),
              lane: 'repair',
              rows: [rowOf({
                sliceIndex: 4,
                kind: 'replacement-shipped',
              },),],
            },);

            expect(census.regions.length,).toBe(1,);
            expect(census.regions[0]?.pageRelation,).toEqual({
              kind: 'displaced',
              decidedBy: 'consolidation',
            },);
            expect(census.regions[0]?.shippedText,).toBe('Cat 4 is perched on the windowsill.',);
          },
        },),

        it({
          name: 'ANSWERS UNDECIDED on an entry no stage has decided, where the would-ship text is the '
            + 'ARCHIVE and so differs from every shipped region by construction. Filing those as '
            + 'displaced would report an overrule on all 306 of the undecided regions, when what '
            + 'actually happened is that nobody has decided anything yet',
          fn: async () => {
            const census = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: UNDECIDED,
              readings: readingsOf({
                text: 'Cat 6 sits on the sill.',
                decidedBy: 'archive',
              },),
              lane: 'translate',
              rows: [rowOf({
                sliceIndex: 6,
                kind: 'replacement-shipped',
              },),],
            },);

            expect(census.regions[0]?.pageRelation,).toEqual({ kind: 'undecided', },);
          },
        },),

        it({
          name: 'REFUSES A SHIPPED ROW NAMED BY NO COMPARISON ROW rather than dropping it, since the '
            + 'comparison is derived from the same slicing this ledger delivered: a row missing from it '
            + 'is a contradiction inside one artifact, and skipping it would shrink the draw '
            + 'population for a reason nobody would see',
          fn: async () => {
            /**
             What unnamedSlice raised, read for its class as well as its wording.
             */
            const refusalOfUnnamedSlice = caught(function unnamedSlice() {
              regionsOfLane({
                entryId: 'Tabby',
                laneSelection: CONTESTED,
                readings: new Map() as Parameters<typeof regionsOfLane>[0]['readings'],
                lane: 'repair',
                rows: [rowOf({
                  sliceIndex: 7,
                  kind: 'replacement-shipped',
                },),],
              },);
            },);

            expect(refusalOfUnnamedSlice,).toBeInstanceOf(DamageRegionError,);
            expect((refusalOfUnnamedSlice as Error).message,).toContain('named by no comparison row',);
          },
        },),

        it({
          name: 'REPORTS AN EMPTY LEDGER as no regions rather than failing, since a lane that shipped '
            + 'nothing is an ordinary outcome',
          fn: async () => {
            const census = regionsOfLane({
              entryId: 'Tabby',
              laneSelection: CONTESTED,
              readings: SURVIVING,
              lane: 'repair',
              rows: [],
            },);
            expect(census.regions.length,).toBe(0,);
            expect(census.filledWithoutIncumbent,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: regionIdOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A REGION BY LANE AND INDEX, which is what version 2 addresses a slice by: the '
            + 'envelope ids version 1 used do not exist here, and an index alone would collide '
            + 'whenever both lanes shipped the same slice',
          fn: async () => {
            expect(regionIdOf({
              lane: 'repair',
              sliceIndex: 0,
            },),).toBe('repair#0',);
            expect(regionIdOf({
              lane: 'translate',
              sliceIndex: 12,
            },),).toBe('translate#12',);
            expect(regionIdOf({
              lane: 'repair',
              sliceIndex: 12,
            },),).not.toBe(regionIdOf({
              lane: 'translate',
              sliceIndex: 12,
            },),);
          },
        },),
      ],
    },),
  ],
},);
