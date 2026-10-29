/**
 Guards class sixty-one (XingZ611, 2026-09-19): a source-only passage the
 roster found absent, standing inside a container the archive carries with
 fewer blocks than the original writes there, is admitted on that deficit.
 The whole-page budget refuses it on a page whose translated part runs long,
 and the disclosure block's third paragraph shipped as a recorded gap while
 the archive's block had one paragraph fewer than the original's. A container
 whose archive half has as many blocks as the original's admits nothing, and
 a split with an anchored claim is not an absent verdict. Cat-themed
 invention throughout; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  admitContainerDeficit,
  CONTAINER_DEFICIT_UNREAD_FINDING,
  type InsertionCoverageRow,
} from '../../dist/final/node/index.mjs';
import { insertion, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/insertion-only-slice.test-fixture.ts';
import { pair, } from '@monochromatic-dev/translation-repair-page-pass/ts/corpus-run/title-reference.test-fixture.ts';

/**
 Opening half of a disclosure block: the tag and its summary.
 */
const OPEN_SOURCE = '<details>\n\n<summary>猫的故事</summary>';

/**
 Archive's rendering of the opening half, the tag written its own way.
 */
const OPEN_TARGET = '<details style="margin-top: 0.5rem;">\n<summary>The cat\'s story</summary>';

/**
 First paragraph inside the block, translated.
 */
const FIRST_SOURCE = '橘猫在窗台上睡了整个下午。';

/**
 Archive's rendering of the first paragraph.
 */
const FIRST_TARGET = 'The orange cat slept on the windowsill all afternoon.';

/**
 Paragraph the archive never rendered.
 */
const MISSING_SOURCE = '猫在夜里回家了，谁也没有听见门响。';

/**
 Closing half: the last paragraph and the closing tag.
 */
const CLOSE_SOURCE = '猫不喜欢洗澡。\n\n</details>';

/**
 Archive's rendering of the closing half.
 */
const CLOSE_TARGET = 'The cat does not like baths.\n\n</details>';

/**
 Unresolved row for the missing paragraph at slice 2.

 @param verdict - what the roster said about it

 @returns Row the whole-page budget refused

 @example
 ```ts
 const row = missingRow({ verdict: 'absent', },);
 ```
 */
function missingRow(
  { verdict, }: { readonly verdict: 'absent' | 'anchored-split'; },
): InsertionCoverageRow {
  return {
    position: 2,
    sliceIndex: 2,
    sourceText: MISSING_SOURCE,
    frontMatter: false,
    verdictKind: (verdict === 'absent') ? 'absent' : 'split',
    anchoredFull: 0,
    anchoredPartial: (verdict === 'absent') ? 0 : 1,
    absentCount: (verdict === 'absent') ? 2 : 1,
    heard: 2,
    asked: 2,
    missingDestinationCount: 0,
    coverageFinding: 'insertion-coverage (slice 2)',
    stageFindings: [],
    coverageEvidence: [],
    destinationFindings: [],
  };
}

/**
 Whether a finding names the slice-2 container deficit admission, the check
 every admission case here runs over `deficit.findings`.

 @param finding - finding from a deficit's findings list

 @returns Whether this finding is the slice-2 admission

 @example
 ```ts
 expect(deficit.findings.some(admittedDeficit,),).toBe(true,);
 ```
 */
function admittedDeficit(finding: string,): boolean {
  return finding.startsWith('insertion-container-deficit-admitted (slice 2 inside details of slices 0 to 3',);
}

/**
 Archive's closing half rendering the paragraph the original's slice 2 writes,
 so the archive's block has as many blocks as the original's.
 */
const WHOLE_CLOSE_TARGET = `The cat came home at night.\n\n${CLOSE_TARGET}`;

/**
 Four slices: the opening half, one paired paragraph, the missing paragraph
 and the closing half, the archive's block one paragraph short unless a half
 says otherwise.

 @param openSource - original text of the opening half

 @param openTarget - archive text of the opening half

 @param firstSource - original text of the paired paragraph

 @param firstTarget - archive text of the paired paragraph

 @param missingSource - original text of the passage the archive never rendered

 @param closeSource - original text of the closing half

 @param closeTarget - archive text of the closing half

 @returns Slices in document order

 @example
 ```ts
 const slices = containerSlices({ closeTarget: CLOSE_TARGET, },);
 ```
 */
function containerSlices(
  {
    openSource = OPEN_SOURCE,
    openTarget = OPEN_TARGET,
    firstSource = FIRST_SOURCE,
    firstTarget = FIRST_TARGET,
    missingSource = MISSING_SOURCE,
    closeSource = CLOSE_SOURCE,
    closeTarget = CLOSE_TARGET,
  }: {
    readonly openSource?: string;
    readonly openTarget?: string;
    readonly firstSource?: string;
    readonly firstTarget?: string;
    readonly missingSource?: string;
    readonly closeSource?: string;
    readonly closeTarget?: string;
  },
): readonly ChunkPair[] {
  return [
    pair({
      sliceIndex: 0,
      source: openSource,
      target: openTarget,
    },),
    pair({
      sliceIndex: 1,
      source: firstSource,
      target: firstTarget,
    },),
    insertion({
      sliceIndex: 2,
      source: missingSource,
    },),
    pair({
      sliceIndex: 3,
      source: closeSource,
      target: closeTarget,
    },),
  ];
}

/**
 Three list items written with a blank line between each, one block to the
 parser and three to a blank-line split.
 */
const LOOSE_SOURCE_LIST = '- 窗台\n\n- 阳光\n\n- 午睡';

/**
 The same items written without blank lines.
 */
const TIGHT_SOURCE_LIST = '- 窗台\n- 阳光\n- 午睡';

/**
 The archive's rendering of the list, loose.
 */
const LOOSE_TARGET_LIST = '- The sill\n\n- The sun\n\n- The nap';

/**
 The archive's rendering of the list, tight.
 */
const TIGHT_TARGET_LIST = '- The sill\n- The sun\n- The nap';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'a passage absent inside a container the archive carries one block short (class sixty-one, XingZ611)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS the absent paragraph on the block deficit and names it',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ closeTarget: CLOSE_TARGET, },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([2,],);
            expect(deficit.unresolvedRows,).toEqual([],);
            expect(deficit.findings
              .some(function named(finding,): boolean {
                return admittedDeficit(finding,);
              },),).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES the paragraph unresolved where the archive\'s block has as many blocks as the original\'s',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ closeTarget: WHOLE_CLOSE_TARGET, },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([],);
            expect(deficit.unresolvedRows.length,).toBe(1,);
          },
        },),
        it({
          name: 'COUNTS NOTHING AFTER THE CONTAINER\'S OWN CLOSING TAG, where the closing half goes on to a whole '
            + 'element of the same name whose archive copy is a paragraph short, and still admits where the '
            + 'container itself is short (ledger B67)',
          fn: async () => {
            /**
             Original's closing half going on to a whole element of two paragraphs.
             */
            const closeSource = `${CLOSE_SOURCE}\n\n<details>\n<summary>另一个故事</summary>\n\n第一段。\n\n第二段。\n\n</details>`;

            /**
             The archive's copy of that element, one paragraph short.
             */
            const shortElement = '\n\n<details>\n<summary>Another story</summary>\n\nThe first paragraph.\n\n</details>';
            const whole = admitContainerDeficit({
              slices: containerSlices({
                closeSource,
                closeTarget: `${WHOLE_CLOSE_TARGET}${shortElement}`,
              },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...whole.positions,],).toEqual([],);
            expect(whole.unresolvedRows.length,).toBe(1,);
            const short = admitContainerDeficit({
              slices: containerSlices({
                closeSource,
                closeTarget: `${CLOSE_TARGET}${shortElement}`,
              },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...short.positions,],).toEqual([2,],);
          },
        },),
        it({
          name: 'COUNTS THE WHOLE HALF where the archive\'s opening half does not write the container\'s tag',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ openTarget: '<summary>The cat\'s story</summary>', },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([2,],);
            expect(deficit.findings,).toEqual([
              'insertion-container-deficit-admitted (slice 2 inside details of slices 0 to 3: the original writes '
                + '4 blocks there, the archive 3)',
            ],);
          },
        },),
        it({
          name: 'SPENDS THE DEFICIT IN DOCUMENT ORDER, whatever order the rows come in: one block short, the '
            + 'earlier of two absent paragraphs is admitted and the later stays unresolved',
          fn: async () => {
            /**
             Second paragraph the archive never rendered.
             */
            const secondMissing = '猫睡醒以后又吃了一顿。';
            /**
             Row for the second paragraph, one position after the first.
             */
            const laterRow: InsertionCoverageRow = {
              ...missingRow({ verdict: 'absent', },),
              position: 3,
              sliceIndex: 3,
              sourceText: secondMissing,
            };
            const deficit = admitContainerDeficit({
              // Five blocks in the original's container, four in the archive's.
              slices: [
                pair({
                  sliceIndex: 0,
                  source: OPEN_SOURCE,
                  target: OPEN_TARGET,
                },),
                pair({
                  sliceIndex: 1,
                  source: FIRST_SOURCE,
                  target: FIRST_TARGET,
                },),
                insertion({
                  sliceIndex: 2,
                  source: MISSING_SOURCE,
                },),
                insertion({
                  sliceIndex: 3,
                  source: secondMissing,
                },),
                pair({
                  sliceIndex: 4,
                  source: CLOSE_SOURCE,
                  target: WHOLE_CLOSE_TARGET,
                },),
              ],
              positions: new Set(),
              unresolvedRows: [
                laterRow,
                missingRow({ verdict: 'absent', },),
              ],
            },);
            expect([...deficit.positions,],).toEqual([2,],);
            expect(deficit.unresolvedRows,).toEqual([laterRow,],);
          },
        },),
        it({
          name: 'COUNTS NOTHING BEFORE THE CONTAINER\'S OWN OPENING TAG, where the opening half starts with a whole '
            + 'element of the same name whose archive copy is a paragraph short (ledger B67)',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({
                openSource: `<details>\n<summary>前言</summary>\n\n一段。\n\n二段。\n\n</details>\n\n${OPEN_SOURCE}`,
                openTarget: '<details>\n<summary>Preface</summary>\n\nOne paragraph.\n\n</details>\n\n'
                  + '<details>\n<summary>The cat\'s story</summary>',
                closeTarget: WHOLE_CLOSE_TARGET,
              },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([],);
            expect(deficit.unresolvedRows.length,).toBe(1,);
          },
        },),
        it({
          name: 'ADMITS a split with a minority anchored claim on the deficit and names the split (class sixty-nine, XingZ618)',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ closeTarget: CLOSE_TARGET, },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'anchored-split', },),],
            },);
            expect([...deficit.positions,],).toEqual([2,],);
            expect(deficit.unresolvedRows,).toEqual([],);
            expect(deficit.findings
              .some(function named(finding,): boolean {
                return finding.startsWith('insertion-split-in-container-deficit (slice 2, full 0, partial 1, absent 1 of 2 asked',);
              },),).toBe(true,);
            expect(deficit.findings
              .some(function named(finding,): boolean {
                return admittedDeficit(finding,);
              },),).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES a split with a majority carried verdict out whatever the deficit',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ closeTarget: CLOSE_TARGET, },),
              positions: new Set(),
              unresolvedRows: [{
                ...missingRow({ verdict: 'anchored-split', },),
                verdictKind: 'carried',
                anchoredFull: 2,
                absentCount: 0,
              },],
            },);
            expect([...deficit.positions,],).toEqual([],);
            expect(deficit.unresolvedRows.length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: 'what a container holds, read off the parse (ledger B68)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS NOTHING where only the list is written looser in the original: the container is rendered '
            + 'whole, and a list is one block however its items are spaced',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({
                firstSource: LOOSE_SOURCE_LIST,
                firstTarget: TIGHT_TARGET_LIST,
                closeTarget: WHOLE_CLOSE_TARGET,
              },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([],);
          },
        },),
        it({
          name: 'ADMITS THE MISSING PARAGRAPH where only the list is written looser in the archive, and counts the '
            + 'list once on each side',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({
                firstSource: TIGHT_SOURCE_LIST,
                firstTarget: LOOSE_TARGET_LIST,
              },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([2,],);
            expect(deficit.findings,).toEqual([
              'insertion-container-deficit-admitted (slice 2 inside details of slices 0 to 3: the original writes '
                + '4 blocks there, the archive 3)',
            ],);
          },
        },),
        it({
          name: 'COUNTS A FENCED BLOCK ONCE whatever blank lines it holds, so a container rendered whole admits nothing',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({
                firstSource: '```\n猫\n\n睡觉\n```',
                firstTarget: '```\nCat\nnaps\n```',
                closeTarget: WHOLE_CLOSE_TARGET,
              },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([],);
          },
        },),
        it({
          name: 'COSTS A PASSAGE the blocks the parse reads in it, so a missing loose list spends one block of the '
            + 'deficit, as the container counts it',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ missingSource: LOOSE_SOURCE_LIST, },),
              positions: new Set(),
              unresolvedRows: [{
                ...missingRow({ verdict: 'absent', },),
                sourceText: LOOSE_SOURCE_LIST,
              },],
            },);
            expect([...deficit.positions,],).toEqual([2,],);
          },
        },),
        it({
          name: 'SPENDS NO DEFICIT where a half cannot be read, and says which slice and why, rather than counting '
            + 'what the parser refused',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ firstSource: '猫猫在{窗台上打盹。', },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([],);
            expect(deficit.unresolvedRows.length,).toBe(1,);
            expect(deficit.findings.length,).toBe(1,);
            expect(deficit.findings[0]?.startsWith(
              'insertion-container-deficit-unread (details of slices 0 to 3: the original\'s slice 1 could not be read: ',
            ),).toBe(true,);
          },
        },),
        it({
          name: 'SPENDS NO DEFICIT where a half of the archive cannot be read, and names the archive\'s slice',
          fn: async () => {
            const deficit = admitContainerDeficit({
              slices: containerSlices({ firstTarget: 'The cat {naps on the sill.', },),
              positions: new Set(),
              unresolvedRows: [missingRow({ verdict: 'absent', },),],
            },);
            expect([...deficit.positions,],).toEqual([],);
            expect(deficit.findings.length,).toBe(1,);
            expect(deficit.findings[0]?.startsWith(
              `${CONTAINER_DEFICIT_UNREAD_FINDING} (details of slices 0 to 3: the archive's slice 1 could not be read: `,
            ),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
