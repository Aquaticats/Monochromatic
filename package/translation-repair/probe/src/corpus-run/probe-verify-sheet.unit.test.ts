/**
 Tests for the blind sheet that decides whether the probe finds damage or
 invents it.

 THE SHEET AND ITS MANIFEST ARE WRITTEN BY TWO FUNCTIONS THAT EACH SORT, and
 everything the measurement produces rests on those two sorts agreeing. The
 sheet numbers its items and the manifest numbers its rows, and a scorer reads
 a grade off position N of one and attributes it to position N of the other.
 If the orders ever diverged, every grade would land on the wrong item and the
 result would still look like a clean measurement. The case asserting the two
 orders agree is the one that matters most in this file.

 THE BLINDNESS IS THE OTHER HALF. `kind` says which partition an item came
 from, and a grader who can see it is answering a different question. It rides
 in the manifest and must never reach the sheet, so one case searches the whole
 rendered sheet for both partition names.

 ORDER IS BY DIGEST OF IDENTITY, WHICH IS NOT A DETAIL. Insertion order would
 leak the partition whenever the caller built one partition before the other,
 which is how both callers build them.

 FIXTURES ARE INVENTED AND CAT-THEMED. The real sheet quotes unlicensed corpus
 text and is written outside the repository; nothing resembling it belongs in a
 committed test.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ClaimAdmissibility, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  formatVerifyManifest,
  formatVerifySheet,
  orderBlind,
  type VerifyItem,
} from '../../dist/final/node/index.mjs';

//region Probe verify sheet tests

/**
 Partition an item came from when a reader had already flagged it.

 NOT SPELLED THE WAY PRODUCTION SPELLS IT. `probe-verify.ts` passes `damaged`
 and `control`, and the sheet's own instructions ask a grader to judge
 "whether the EDIT damaged the translation". Searching a rendered sheet for
 the production label therefore matches ordinary English and reports a leak
 that is not there, which is exactly what the first draft of this file did.
 These labels are tokens nothing else can produce, so a match is the field.
 */
const FIRST_PARTITION = 'partitionalpha';

/**
 Partition an item came from when nobody had flagged it.
 */
const SECOND_PARTITION = 'partitionbeta';

/**
 Prober whose claims these fixtures carry.
 */
const CLAIMANT = 'cat-house/tabbyscribe-2';

/**
 Wording an edit added, which a claim of added damage quotes.
 */
const ADDED_WORDING = 'the cat sat regally upon the mat';

/**
 Wording an edit dropped, which a claim of lost content quotes.
 */
const DROPPED_WORDING = 'and the kitten watched from the stairs';

/**
 Builds one item for the sheet.

 @param entryId - entry the region belongs to

 @param envelopeId - region inside that entry

 @param kind - partition this item came from

 @param claims - what the probe said, empty when it said nothing

 @returns Item shaped as the formatters take one

 @example
 ```ts
 const item = verifyItem({ entryId: 'whiskers', envelopeId: 'e1', kind: FIRST_PARTITION, claims: [], },);
 ```
 */
function verifyItem(
  {
    entryId,
    envelopeId,
    kind,
    claims,
  }: {
    readonly entryId: string;
    readonly envelopeId: string;
    readonly kind: string;
    readonly claims: readonly {
      readonly evidence: string;
      readonly omittedText: string;
      readonly admissibility: ClaimAdmissibility;
    }[];
  },
): VerifyItem {
  return {
    kind,
    relabelCase: {
      entryId,
      positions: [1,],
      region: {
        envelopeId,
        issueIds: [],
        before: `before text of ${envelopeId}`,
        editorAfter: `after text of ${envelopeId}`,
      },
      issues: [],
      sourceText: `source text of ${envelopeId}`,
      baselineText: `baseline text of ${envelopeId}`,
      recorded: 'silent',
    },
    claims: claims.map(function toClaim(
      {
        evidence,
        omittedText,
        admissibility,
      },
    ) {
      return {
        modelId: CLAIMANT,
        category: 'meaning-changed',
        severity: 'major',
        evidence,
        omittedText,
        reason: 'the original says nothing of the sort',
        admissibility,
      };
    },),
  };
}

/**
 Builds a spread of items across both partitions.

 Enough of them that a sort has something to do, and interleaved partitions so
 an order that followed insertion would be visibly grouped.

 @returns Items in insertion order

 @example
 ```ts
 const items = household();
 ```
 */
function household(): readonly VerifyItem[] {
  return [
    'whiskers',
    'mittens',
    'saffron',
    'pepperbox',
    'juniper',
    'clover',
  ].map(function toItem(
    entryId,
    index,
  ): VerifyItem {
    return verifyItem({
      entryId,
      envelopeId: `envelope-${String(index,)}`,
      kind: ((index % 2) === 0) ? FIRST_PARTITION : SECOND_PARTITION,
      claims: [],
    },);
  },);
}

/**
 Reads the entry ids off items, in the order they stand.

 @param items - items to read

 @returns Entry ids in order

 @example
 ```ts
 const order = idsOf({ items, },);
 ```
 */
function idsOf(
  { items, }: { readonly items: readonly VerifyItem[]; },
): readonly string[] {
  return items.map(function toId(item,): string {
    return item.relabelCase.entryId;
  },);
}

/**
 Reads the manifest back as the scorer does.

 @param items - items the manifest was built from

 @returns Rows the manifest carries

 @example
 ```ts
 const rows = manifestRows({ items, },);
 ```
 */
function manifestRows(
  { items, }: { readonly items: readonly VerifyItem[]; },
): readonly {
  readonly position: number;
  readonly entryId: string;
  readonly envelopeId: string;
  readonly kind: string;
  readonly claimants: readonly string[];
}[] {
  return (JSON.parse(formatVerifyManifest({ items, },),) as {
    readonly items: readonly {
      readonly position: number;
      readonly entryId: string;
      readonly envelopeId: string;
      readonly kind: string;
      readonly claimants: readonly string[];
    }[];
  }).items;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: orderBlind.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ORDERS the same items the same way every time',
          fn: async () => {
            /**
             Items as the caller built them.
             */
            const items = household();

            expect(idsOf({ items: orderBlind({ items, },), },),)
              .toEqual(idsOf({ items: orderBlind({ items, },), },),);
          },
        },),
        it({
          name: 'IGNORES the order the caller built them in, which would leak the partition',
          fn: async () => {
            /**
             Items as the caller built them.
             */
            const items = household();

            expect(idsOf({ items: orderBlind({ items, },), },),)
              .toEqual(idsOf({ items: orderBlind({ items: items.toReversed(), },), },),);
          },
        },),
        it({
          name: 'IGNORES the partition itself, so relabelling an item cannot move it',
          fn: async () => {
            /**
             Items as the caller built them.
             */
            const items = household();

            /**
             Same items with every partition label flipped.
             */
            const flipped = items.map(function relabel(item,): VerifyItem {
              return {
                ...item,
                kind: (item.kind === FIRST_PARTITION) ? SECOND_PARTITION : FIRST_PARTITION,
              };
            },);

            expect(idsOf({ items: orderBlind({ items: flipped, },), },),)
              .toEqual(idsOf({ items: orderBlind({ items, },), },),);
          },
        },),
        it({
          name: 'MOVES an item when its identity changes, since identity is the key',
          fn: async () => {
            // A positive control for the "ORDERS the same items the same way every
            // time", "IGNORES the order the caller built them in" and "IGNORES the
            // partition itself" cases: they all assert that
            // something does NOT move the order, and a sort that ignored its input
            // entirely would pass every one of them.
            /**
             Items as the caller built them.
             */
            const items = household();

            /**
             Same items under different entry ids.
             */
            const renamed = items.map(function rename(item,): VerifyItem {
              return {
                ...item,
                relabelCase: {
                  ...item.relabelCase,
                  entryId: `${item.relabelCase.entryId}-ii`,
                },
              };
            },);

            expect(idsOf({ items: orderBlind({ items: renamed, },), },)
              .map(function stripSuffix(id,): string {
                return id.replace(
                  '-ii',
                  '',
                );
              },),)
              .not
              .toEqual(idsOf({ items: orderBlind({ items, },), },),);
          },
        },),
        it({
          name: 'KEEPS every item it was given',
          fn: async () => {
            /**
             Items as the caller built them.
             */
            const items = household();

            expect(idsOf({ items: orderBlind({ items, },), },)
              .toSorted(),)
              .toEqual(idsOf({ items, },)
                .toSorted(),);
          },
        },),
      ],
    },),

    describe({
      name: formatVerifySheet.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NUMBERS the sheet in the order the manifest numbers its rows',
          fn: async () => {
            // THE CASE THIS FILE EXISTS FOR. Both functions sort independently, and
            // a scorer reads a grade off sheet position N as a verdict on manifest
            // row N. Divergence would misattribute every grade and still look clean.
            /**
             Items as the caller built them.
             */
            const items = household();

            /**
             Sheet as a grader receives it.
             */
            const sheet = formatVerifySheet({ items, },);

            for (const row of manifestRows({ items, },)) {
              /**
               Heading the sheet gives that position.
               */
              const heading = `### ${String(row.position,)}. grade: [ ]`;

              /**
               Where that heading stands in the sheet.
               */
              const at = sheet.indexOf(heading,);

              /**
               Text between that heading and whatever follows it.
               */
              const section = sheet.slice(
                at,
                sheet.indexOf(
                  '### ',
                  at + heading.length,
                ),
              );

              expect(at,).not.toBe(-1,);
              expect(section.includes(`source text of ${row.envelopeId}`,),).toBe(true,);
            }
          },
        },),
        it({
          name: 'TELLS a blind grader that the reviewer flagged some items and not others, and never that '
            + 'it flagged each, since the damage sheet mixes both with the claims stripped',
          fn: async () => {
            /**
             The damage sheet's page.
             */
            const sheet = formatVerifySheet({
              items: household(),
              framing: 'blind',
            },);
            expect(sheet.includes('claims each one introduced a defect',),).toBe(false,);
            expect(sheet.includes('flagged some of them as damaging and stayed silent on the others',),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS the reviewer-claims framing by default, which is the verify sheet where every item '
            + 'was flagged and its claims are printed',
          fn: async () => {
            expect(formatVerifySheet({ items: household(), },).includes('claims each one introduced a defect',),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES to name which partition an item came from, anywhere on the page',
          fn: async () => {
            /**
             Sheet as a grader receives it.
             */
            const sheet = formatVerifySheet({ items: household(), },);

            expect(sheet.includes(FIRST_PARTITION,),).toBe(false,);
            expect(sheet.includes(SECOND_PARTITION,),).toBe(false,);
            expect(sheet.includes('kind',),).toBe(false,);
          },
        },),
        it({
          name: 'SHOWS every text a grader needs, on both sides of the edit',
          fn: async () => {
            /**
             Sheet built from one item, so the assertions name one region.
             */
            const sheet = formatVerifySheet({
              items: [
                verifyItem({
                  entryId: 'whiskers',
                  envelopeId: 'lone',
                  kind: FIRST_PARTITION,
                  claims: [],
                },),
              ],
            },);

            expect(sheet.includes('source text of lone',),).toBe(true,);
            expect(sheet.includes('baseline text of lone',),).toBe(true,);
            expect(sheet.includes('before text of lone',),).toBe(true,);
            expect(sheet.includes('after text of lone',),).toBe(true,);
            expect(sheet.includes('Items: 1',),).toBe(true,);
          },
        },),
        it({
          name: 'WITHHOLDS the reviewer\'s claim from an item that carries none',
          fn: async () => {
            /**
             Sheet built from one unclaimed item.
             */
            const sheet = formatVerifySheet({
              items: [
                verifyItem({
                  entryId: 'whiskers',
                  envelopeId: 'lone',
                  kind: SECOND_PARTITION,
                  claims: [],
                },),
              ],
            },);

            expect(sheet.includes('An automated reviewer says',),).toBe(false,);
          },
        },),
        it({
          name: 'NAMES an added-wording claim as added, quoting the after side',
          fn: async () => {
            /**
             Sheet built from one item the probe flagged for adding wording.
             */
            const sheet = formatVerifySheet({
              items: [
                verifyItem({
                  entryId: 'whiskers',
                  envelopeId: 'lone',
                  kind: FIRST_PARTITION,
                  claims: [
                    {
                      evidence: ADDED_WORDING,
                      omittedText: '',
                      admissibility: 'corroborated',
                    },
                  ],
                },),
              ],
            },);

            expect(sheet.includes('wording the edit ADDED or altered',),).toBe(true,);
            expect(sheet.includes(ADDED_WORDING,),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES a dropped-wording claim as dropped, quoting the before side',
          fn: async () => {
            // The two directions read opposite quotes off the same claim, so a
            // formatter that picked the wrong field would show a grader wording
            // that is present in the text it says was removed.
            /**
             Sheet built from one item the probe flagged for dropping wording.
             */
            const sheet = formatVerifySheet({
              items: [
                verifyItem({
                  entryId: 'whiskers',
                  envelopeId: 'lone',
                  kind: FIRST_PARTITION,
                  claims: [
                    {
                      evidence: '',
                      omittedText: DROPPED_WORDING,
                      admissibility: 'removal-corroborated',
                    },
                  ],
                },),
              ],
            },);

            expect(sheet.includes('wording the edit DROPPED',),).toBe(true,);
            expect(sheet.includes(DROPPED_WORDING,),).toBe(true,);
            expect(sheet.includes('wording the edit ADDED or altered',),).toBe(false,);
          },
        },),
        it({
          name: 'NAMES a claim the screen corroborated on its evidence as added and quotes that evidence, '
            + 'where its omitted side is only spaces, and one corroborated on its omitted side as dropped, '
            + 'where its evidence is only spaces (ledger B128)',
          fn: async () => {
            /**
             The reviewer's block on a sheet of one item carrying one claim.

             @param claim - claim's two sides and what the screen made of it

             @returns Lines from the reviewer's opening to the grading rule

             @example
             ```ts
             const block = reviewerBlock({ evidence: 'a', omittedText: ' ', admissibility: 'corroborated', },);
             ```
             */
            function reviewerBlock(
              claim: {
                readonly evidence: string;
                readonly omittedText: string;
                readonly admissibility: ClaimAdmissibility;
              },
            ): string {
              /**
               Sheet built from the one claimed item.
               */
              const sheet = formatVerifySheet({
                items: [
                  verifyItem({
                    entryId: 'whiskers',
                    envelopeId: 'lone',
                    kind: FIRST_PARTITION,
                    claims: [claim,],
                  },),
                ],
              },);
              return sheet.slice(
                sheet.indexOf('An automated reviewer says',),
                sheet.indexOf('Y if the edit lost',),
              );
            }
            expect(reviewerBlock({
              evidence: ADDED_WORDING,
              omittedText: '   ',
              admissibility: 'corroborated',
            },),).toBe([
              'An automated reviewer says the edit introduced a defect:',
              '',
              '-   Says the edit introduced: meaning-changed',
              '-   Quotes this as wording the edit ADDED or altered:',
              '',
              '```text',
              ADDED_WORDING,
              '```',
              '',
              '-   Its reason:',
              '',
              '```text',
              'the original says nothing of the sort',
              '```',
              '',
              '',
            ].join('\n',),);
            expect(reviewerBlock({
              evidence: '   ',
              omittedText: DROPPED_WORDING,
              admissibility: 'removal-corroborated',
            },),).toBe([
              'An automated reviewer says the edit introduced a defect:',
              '',
              '-   Says the edit introduced: meaning-changed',
              '-   Quotes this as wording the edit DROPPED:',
              '',
              '```text',
              DROPPED_WORDING,
              '```',
              '',
              '-   Its reason:',
              '',
              '```text',
              'the original says nothing of the sort',
              '```',
              '',
              '',
            ].join('\n',),);
          },
        },),
        it({
          name: 'REFUSES a claim the screen did not corroborate, since a sheet prints only claims with one '
            + 'anchored side to quote',
          fn: async () => {
            /**
             What formatting a sheet with an unanchored claim throws.
             */
            const refusal = caught(function act(): unknown {
              return formatVerifySheet({
                items: [
                  verifyItem({
                    entryId: 'whiskers',
                    envelopeId: 'lone',
                    kind: FIRST_PARTITION,
                    claims: [
                      {
                        evidence: ADDED_WORDING,
                        omittedText: DROPPED_WORDING,
                        admissibility: 'unanchored',
                      },
                    ],
                  },),
                ],
              },);
            },);
            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe(
              'Error: unreachable: a claim the screen read as unanchored on a sheet, though the probe hands '
                + 'a sheet only claims the screen corroborated, each with one anchored side to quote',
            );
          },
        },),
      ],
    },),

    describe({
      name: formatVerifyManifest.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RECORDS the partition the sheet withholds',
          fn: async () => {
            /**
             Rows the manifest carries.
             */
            const rows = manifestRows({ items: household(), },);

            expect(rows.length,).toBe(household().length,);
            expect(rows.some(function isDamaged(row,): boolean {
              return row.kind === FIRST_PARTITION;
            },),).toBe(true,);
            expect(rows.some(function isControl(row,): boolean {
              return row.kind === SECOND_PARTITION;
            },),).toBe(true,);
          },
        },),
        it({
          name: 'NUMBERS its rows from one, with no gaps',
          fn: async () => {
            expect(manifestRows({ items: household(), },)
              .map(function toPosition(row,): number {
                return row.position;
              },),)
              .toEqual(household()
                .map(function toOneBased(
                  _item,
                  index,
                ): number {
                  return index + 1;
                },),);
          },
        },),
        it({
          name: 'NAMES who claimed what, so a graded sheet can be read per prober',
          fn: async () => {
            /**
             Rows built from one claimed item and one unclaimed one.
             */
            const rows = manifestRows({
              items: [
                verifyItem({
                  entryId: 'whiskers',
                  envelopeId: 'claimed',
                  kind: FIRST_PARTITION,
                  claims: [
                    {
                      evidence: ADDED_WORDING,
                      omittedText: '',
                      admissibility: 'corroborated',
                    },
                  ],
                },),
                verifyItem({
                  entryId: 'mittens',
                  envelopeId: 'unclaimed',
                  kind: SECOND_PARTITION,
                  claims: [],
                },),
              ],
            },);

            /**
             Claimants recorded against the claimed region.
             */
            const claimed = rows
              .filter(function isClaimed(row,): boolean {
                return row.envelopeId === 'claimed';
              },)
              .flatMap(function toClaimants(row,): readonly string[] {
                return row.claimants;
              },);

            expect(claimed,).toEqual([CLAIMANT,],);
            expect(rows
              .filter(function isUnclaimed(row,): boolean {
                return row.envelopeId === 'unclaimed';
              },)
              .flatMap(function toClaimants(row,): readonly string[] {
                return row.claimants;
              },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Probe verify sheet tests
