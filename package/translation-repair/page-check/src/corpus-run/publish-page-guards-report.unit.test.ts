/**
 Tests that the two page-level guards ship the page with the defect reported instead of stopping the entry: the owner's
 rules are "no matter what, we must ship" and that a page-agreement check ships with the defect reported.

 THE SCENARIO IS THE PAGE OF XINGZ6023, invented in cats: a content slice keeps the archive, which has no reference to
 note 3, while the translate lane filled the anchor the archive never translated with the definition of note 3. The
 lane's own assembly guard could not clear it (withdrawing the anchor ships its original, which defines note 3 too),
 so the orphan definition reached the page, where a guard that threw stopped the entry with no page written.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type PublishDefect,
  publishFixedPage,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 A content slice the archive keeps, whose original refers to note 3 and whose archive wording does not, and an anchor
 the archive never translated, whose original defines note 3.
 */
const ROWS: readonly SourceShippedRow[] = [
  {
    incumbentText: 'The cat sleeps in warm sunlight.\n\n',
    sourceText: '猫在阳光下睡觉。[^3]\n\n',
  },
  {
    incumbentText: '',
    sourceText: '[^3]: 那是阳光。\n',
  },
];

/**
 What the translate lane wrote at the anchor: the definition alone.
 */
const DEFINITION = '[^3]: That is, a sunbeam.\n';

/**
 Artifact whose translate lane filled the anchor with the definition and whose contest shipped it, the content slice
 keeping the archive as both lanes left it.

 @returns Artifact the publisher reads

 @example
 ```ts
 const artifact = artifactShippingTheDefinition();
 ```
 */
function artifactShippingTheDefinition(): WouldShipSource {
  /**
   The archive-stands artifact the fixture builds, whose anchor row is then filled.
   */
  const base = sourceShippedArtifact({ rows: ROWS, },) as unknown as {
    readonly comparison: readonly Record<string, unknown>[];
  };
  return {
    ...base,
    comparison: [
      base.comparison[0],
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText: DEFINITION,
        laneRelation: 'translate-only',
        repairOutcome: { kind: 'unfilled', },
        translateOutcome: {
          kind: 'decided',
          acceptedText: DEFINITION,
        },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'different',
        },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
    ],
    laneSelection: {
      kind: 'contested',
      slices: [{
        sliceIndex: 1,
        verdict: {
          kind: 'sole-lane',
          lane: 'translate',
        },
        ballots: [],
        usable: 1,
      },],
    },
  } as unknown as WouldShipSource;
}

/**
 Publishes the scenario's entry into a throwaway tree and reads back what landed.

 @param publishDir - tree root to write into

 @param carried - carried passages the page must still carry the evidence of, none by default

 @returns Page text and the defects it shipped with

 @example
 ```ts
 const { text, defects, } = await publishScenario({ publishDir: tree.path, },);
 ```
 */
async function publishScenario(
  {
    publishDir,
    carried = [],
  }: {
    readonly publishDir: string;
    readonly carried?: Parameters<typeof publishFixedPage>[0]['carried'];
  },
): Promise<{ readonly text: string; readonly defects: readonly PublishDefect[]; }> {
  /**
   What the publisher wrote and reported.
   */
  const published = await publishFixedPage({
    artifact: artifactShippingTheDefinition(),
    slices: sourceShippedSlices({ rows: ROWS, },),
    archiveText: sourceShippedArchive({ rows: ROWS, },),
    sourceText: ROWS
      .map(function sourceOf({ sourceText, },): string {
        return sourceText;
      },)
      .join('',),
    entryId: 'BookshopCat',
    publishDir,
    l: tagged({ tag: 'publish-page-guards-test', },),
    ...((carried === undefined) ? {} : { carried, }),
  },);
  return {
    text: await readFile(
      published.path,
      'utf8',
    ),
    defects: published.defects,
  };
}

await describe({
  name: publishFixedPage.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'SHIPS A PAGE CARRYING A DEFINITION NOTHING REFERS TO and reports it as `page-footnote-integrity` beside '
        + 'the page\'s own `page-footnotes` defect, naming `orphan-definition gfm 3`, instead of stopping the entry '
        + 'with no page (XingZ6023, 2026-10-30)',
      fn: async () => {
        await using tree = await scratchDir({ prefix: 'publish-page-guards-', },);
        const {
          text,
          defects,
        } = await publishScenario({ publishDir: tree.path, },);

        expect(text,).toBe(`The cat sleeps in warm sunlight.\n\n${DEFINITION}`,);
        expect(defects.map(function checkOf({ check, },): string {
          return check;
        },),).toEqual(['page-footnotes', 'page-footnote-integrity',],);
        expect(defects[1]?.message ?? '',).toContain('orphan-definition gfm 3',);
        expect(defects[1]?.message ?? '',).toContain('entry BookshopCat',);
      },
    },),
    it({
      name: 'SHIPS A PAGE THAT LOST THE EVIDENCE OF A CARRIED PASSAGE and reports it as `carried-evidence`, naming the '
        + 'slice and the count, instead of stopping the entry with no page',
      fn: async () => {
        await using tree = await scratchDir({ prefix: 'publish-page-guards-', },);
        const {
          text,
          defects,
        } = await publishScenario({
          publishDir: tree.path,
          carried: [{
            position: 1,
            sliceIndex: 3,
            sourceText: '猫在水边等。',
            evidence: ['the cat waits by the water',],
          },],
        },);

        expect(text,).toBe(`The cat sleeps in warm sunlight.\n\n${DEFINITION}`,);
        /**
         The one defect this guard reported.
         */
        const lost = defects.find(function isCarried({ check, },): boolean {
          return check === 'carried-evidence';
        },);
        expect(lost?.message ?? '',).toContain('carried-insertion-evidence-lost (count 1) in slices [3]',);
      },
    },),
    it({
      name: 'REPORTS NEITHER GUARD for a page that keeps the evidence of its carried passage and whose footnotes '
        + 'pair up, which is the control',
      fn: async () => {
        await using tree = await scratchDir({ prefix: 'publish-page-guards-', },);
        const {
          defects,
        } = await publishScenario({
          publishDir: tree.path,
          carried: [{
            position: 1,
            sliceIndex: 3,
            sourceText: '猫在阳光下睡觉。',
            evidence: ['the cat sleeps in warm sunlight',],
          },],
        },);

        expect(defects.some(function isCarried({ check, },): boolean {
          return check === 'carried-evidence';
        },),).toBe(false,);
      },
    },),
  ],
},);
