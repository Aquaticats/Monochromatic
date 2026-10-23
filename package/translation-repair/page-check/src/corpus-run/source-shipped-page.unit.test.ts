/**
 Tests for the page an entry ships when its archive stops before the original does.

 THE OWNER'S RULING OF 2026-10-23, asked what a shipped page carries for a slice with no archive text and
 no backed wording: "Source text + note". The page carries the original text of the passage, preceded by a
 note saying it has not been translated yet and the original follows, footnote definitions included; a later
 run replaces it once a wording is backed. The corpus pass XingZ6020 shipped 88 of 118 slices and left the
 other 30 off the page with nothing to say so.

 DRIVEN THROUGH THE REAL ASSEMBLER AND THE REAL PUBLISHER, over a fixture whose archive ends at its second
 paragraph while the original runs on for more.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  pageAgreement,
  publishFixedPage,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 The note the owner's ruling puts before a run of untranslated slices.
 */
const NOTE = '> This passage has not been translated yet. The original text follows.';

/**
 The one paragraph the archive covers.
 */
const OPENING_ROW: SourceShippedRow = {
  incumbentText: '## Description\n\nA tabby who kept the bookshop company.\n',
  sourceText: '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n',
};

/**
 A second paragraph the archive covers, between two runs of untranslated slices.
 */
const MIDDLE_ROW: SourceShippedRow = {
  incumbentText: '\nShe slept on the counter.\n',
  sourceText: '\n她睡在柜台上。\n',
};

/**
 Source text of the first untranslated slice: a heading, a link, a footnote reference.
 */
const FIRST_SOURCE = '\n## 后记\n\n去看 [她的主页](https://example.net/tabby)[^1]。\n';

/**
 Source text of the second untranslated slice: the footnote definition.
 */
const SECOND_SOURCE = '\n[^1]: 她的主页现在已经关闭。\n';

/**
 Rows of an archive that stops after the opening, with two source slices past its end.
 */
const ONE_RUN: readonly SourceShippedRow[] = [
  OPENING_ROW,
  {
    incumbentText: '',
    sourceText: FIRST_SOURCE,
  },
  {
    incumbentText: '',
    sourceText: SECOND_SOURCE,
  },
];

/**
 Rows with an untranslated slice before and one after a paragraph the archive covers.
 */
const TWO_RUNS: readonly SourceShippedRow[] = [
  OPENING_ROW,
  {
    incumbentText: '',
    sourceText: FIRST_SOURCE,
  },
  MIDDLE_ROW,
  {
    incumbentText: '',
    sourceText: SECOND_SOURCE,
  },
];

/**
 Publishes the rows' entry and reads the page back.

 @param rows - the slices, anchors among them

 @param artifact - artifact to publish, the rows' own unless a case supplies another

 @returns The page, the artifact it was published from and the defects it shipped with

 @example
 ```ts
 const { page, } = await publishRows({ rows: ONE_RUN, },);
 ```
 */
async function publishRows(
  {
    rows,
    artifact = sourceShippedArtifact({ rows, },),
  }: {
    readonly rows: readonly SourceShippedRow[];
    readonly artifact?: WouldShipSource;
  },
): Promise<{
  readonly page: string;
  readonly artifact: WouldShipSource;
  readonly checks: readonly string[];
}> {
  await using tree = await scratchDir({ prefix: 'source-shipped-', },);
  /**
   The publication.
   */
  const published = await publishFixedPage({
    artifact,
    slices: sourceShippedSlices({ rows, },),
    archiveText: sourceShippedArchive({ rows, },),
    sourceText: rows
      .map(function sourceOf({ sourceText, },): string {
        return sourceText;
      },)
      .join('',),
    entryId: 'BookshopCat',
    publishDir: tree.path,
    l: tagged({ tag: 'source-shipped-test', },),
  },);
  return {
    page: await readFile(
      published.path,
      'utf8',
    ),
    artifact,
    checks: published.defects.map(function checkOf({ check, },): string {
      return check;
    },),
  };
}

await describe({
  name: 'a page whose archive stops before the original does',
  children: [
    it({
      name: 'SHIPS THE SOURCE TEXT OF EACH UNTRANSLATED SLICE under one note, after the archive the page keeps',
      fn: async () => {
        const { page, } = await publishRows({ rows: ONE_RUN, },);
        expect(page.startsWith(OPENING_ROW.incumbentText,),).toBe(true,);
        expect(page,).toContain(`\n${NOTE}\n\n## 后记\n\n去看 [她的主页](https://example.net/tabby)[^1]。\n`,);
        expect(page,).toContain('\n[^1]: 她的主页现在已经关闭。\n',);
        expect(page.split(NOTE,).length - 1,).toBe(1,);
      },
    },),

    it({
      name: 'PUTS ONE NOTE BEFORE EACH CONTIGUOUS RUN, not one per slice: two runs with a translated paragraph '
        + 'between them carry two notes, and the paragraph between them is untouched',
      fn: async () => {
        const { page, } = await publishRows({ rows: TWO_RUNS, },);
        expect(page.split(NOTE,).length - 1,).toBe(2,);
        expect(page,).toContain('She slept on the counter.',);
        expect(page.indexOf(FIRST_SOURCE.trim(),),).toBeLessThan(page.indexOf('She slept on the counter.',),);
        expect(page.indexOf('She slept on the counter.',),).toBeLessThan(page.indexOf(SECOND_SOURCE.trim(),),);
      },
    },),

    it({
      name: 'READS EACH SLICE AS SHIPPING ITS SOURCE, naming the note on the slice that opens a run only',
      fn: async () => {
        const slices = wouldShipTextPerSlice({ artifact: sourceShippedArtifact({ rows: ONE_RUN, },), },);
        expect(slices.map(function kindOf({ reading, },): string {
          return reading.kind;
        },),).toEqual(['wording', 'source-ships', 'source-ships',],);
        expect(slices[1]?.reading,).toMatchObject({
          kind: 'source-ships',
          text: FIRST_SOURCE,
          opensRun: true,
        },);
        expect(slices[2]?.reading,).toMatchObject({
          kind: 'source-ships',
          text: SECOND_SOURCE,
          opensRun: false,
        },);
      },
    },),

    it({
      name: 'LEAVES THE PAGE AND ITS ARTIFACT IN AGREEMENT, so no page-agreement defect ships and the check '
        + 'that reads a page back counts the slices that carry the source',
      fn: async () => {
        const { page, artifact, checks, } = await publishRows({ rows: TWO_RUNS, },);
        expect(checks,).not.toContain('page-agreement',);
        const { agreement, wording, } = pageAgreement({
          artifact: {
            ...artifact,
            preparation: {
              archiveText: {
                kind: 'stored',
                text: sourceShippedArchive({ rows: TWO_RUNS, },),
              },
            },
          },
          pageText: page,
        },);
        expect(agreement,).not.toBe('disagreed',);
        expect(wording,).toMatchObject({
          wordings: 2,
          silentSlices: 0,
          sourceShipped: 2,
          missing: [],
        },);
      },
    },),

    it({
      name: 'REPORTS A PAGE THAT LOST THE SOURCE TEXT as disagreeing with its artifact, the positive control '
        + 'for the case above',
      fn: async () => {
        const { page, artifact, } = await publishRows({ rows: ONE_RUN, },);
        const { agreement, wording, } = pageAgreement({
          artifact: {
            ...artifact,
            preparation: {
              archiveText: {
                kind: 'stored',
                text: sourceShippedArchive({ rows: ONE_RUN, },),
              },
            },
          },
          pageText: page.replace(
            '去看',
            '丢了',
          ),
        },);
        expect(agreement,).toBe('disagreed',);
        expect(wording.missing.map(function sliceOf({ sliceIndex, },): number {
          return sliceIndex;
        },),).toEqual([1,],);
      },
    },),

    it({
      name: 'LEAVES THE SOURCE TEXT AS WRITTEN whatever it holds: angle brackets, braces, backticks, a footnote '
        + 'definition, a link, a heading, a run of blank lines and straight quotes all arrive byte for byte',
      fn: async () => {
        /**
         Source text built to be rewritten by anything that treats it as English or as a template.
         */
        const hostile = '\n### 第一节 "引" \'号\'\n\n行内 `a < b {c}` 与 <b>粗体</b> 和 {props} --- 横线[^7]。\n\n\n\n\n'
          + '[^7]: 注 [链接](https://example.net/a?b=1&c=2) \\{转义\\}\n';
        /**
         Rows with that text as the one untranslated slice.
         */
        const rows: readonly SourceShippedRow[] = [
          OPENING_ROW,
          {
            incumbentText: '',
            sourceText: hostile,
          },
        ];
        const { page, } = await publishRows({ rows, },);
        expect(page,).toContain(`${NOTE}\n\n${hostile.trim()}`,);
      },
    },),

    it({
      name: 'SHIPS NOTHING FOR A SOURCE THAT IS BLANK, and keeps the silence of a slice the archive covers with '
        + 'blank text, since neither has a passage to carry',
      fn: async () => {
        const { page, } = await publishRows({
          rows: [
            OPENING_ROW,
            {
              incumbentText: '',
              sourceText: '\n\n',
            },
          ],
        },);
        expect(page,).toBe(OPENING_ROW.incumbentText,);
      },
    },),

    it({
      name: 'KEEPS AN ARTIFACT WITH NO RECORD OF THE ORIGINAL SILENT at an anchor, the reading the consistency '
        + 'pass is handed because it composes the English alone',
      fn: async () => {
        const { lanes: _lanes, ...recordless } = sourceShippedArtifact({ rows: ONE_RUN, },) as WouldShipSource & {
          readonly lanes: unknown;
        };
        const slices = wouldShipTextPerSlice({ artifact: recordless, },);
        expect(slices.map(function kindOf({ reading, },): string {
          return reading.kind;
        },),).toEqual(['wording', 'nothing-ships', 'nothing-ships',],);
      },
    },),
  ],
},);
