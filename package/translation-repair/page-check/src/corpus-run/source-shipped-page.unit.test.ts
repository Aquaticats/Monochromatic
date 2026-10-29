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
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  assertPageParses,
  guardPageAssembly,
  pageAgreement,
  publishFixedPage,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { liveConstructsOf, } from './source-live-constructs.test-fixture.ts';
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
 Rows of the XingZ6020 shape: the archive holds a paragraph with five footnote references and stops, while the
 original goes on to define them in two slices, one of them also defining a sixth note nothing references.
 */
const FIVE_REFERENCES: readonly SourceShippedRow[] = [
  {
    incumbentText: 'A tabby[^1] kept the shop[^2] company[^4], napped[^5] and purred[^6].\n',
    sourceText: '一只虎斑猫[^1]陪着书店[^2]，打盹[^5]、咕噜[^6]。[^4]\n',
  },
  {
    incumbentText: '',
    sourceText: '\n[^1]: 一。\n[^2]: 二。\n[^3]: 三。\n',
  },
  {
    incumbentText: '',
    sourceText: '\n[^4]: 四。\n[^5]: 五。\n[^6]: 六。\n',
  },
];

/**
 Publishes the rows' entry and reads the page back.

 @param rows - slices, anchors among them

 @param artifact - artifact to publish, the rows' own unless a case supplies another

 @param sourceText - whole source page the destinations are read from, the rows' own unless a case supplies another

 @param slices - slices to publish, the rows' own unless a case supplies the ones a preparation cut

 @param archiveText - archive the slices index, the rows' own unless a case supplies another

 @returns Page, the artifact it was published from and the defects it shipped with

 @example
 ```ts
 const { page, } = await publishRows({ rows: ONE_RUN, },);
 ```
 */
async function publishRows(
  {
    rows,
    artifact = sourceShippedArtifact({ rows, },),
    sourceText = rows
      .map(function sourceOf({ sourceText: recorded, },): string {
        return recorded;
      },)
      .join('',),
    slices = sourceShippedSlices({ rows, },),
    archiveText = sourceShippedArchive({ rows, },),
  }: {
    readonly rows: readonly SourceShippedRow[];
    readonly artifact?: WouldShipSource;
    readonly sourceText?: string;
    readonly slices?: readonly ChunkPair[];
    readonly archiveText?: string;
  },
): Promise<{
  readonly page: string;
  readonly artifact: WouldShipSource;
  readonly checks: readonly string[];
  readonly footnoteDefects: readonly string[];
  readonly warnings: readonly string[];
}> {
  /**
   Warnings the publication wrote.
   */
  const warnings: string[] = [];
  /**
   Logger keeping its warnings.
   */
  const l: Logger = {
    ...tagged({ tag: 'source-shipped-test', },),
    warn: function keep(message: string,): void {
      warnings.push(message,);
    },
  };
  await using tree = await scratchDir({ prefix: 'source-shipped-', },);
  /**
   The publication.
   */
  const published = await publishFixedPage({
    artifact,
    slices,
    archiveText,
    sourceText,
    entryId: 'BookshopCat',
    publishDir: tree.path,
    l,
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
    footnoteDefects: published.defects
      .filter(function footnoteCheck({ check, },): boolean {
        return check === 'page-footnotes';
      },)
      .map(function messageOf({ message, },): string {
        return message;
      },),
    warnings,
  };
}

/**
 Warn lines the composition of the page wrote, apart from the other lines a publication writes (the source page that
 does not parse as MDX is reported there too, as it always was).

 @param warnings - every warn line the publication wrote

 @returns Lines of the composition

 @example
 ```ts
 const lines = compositionLines({ warnings, },);
 ```
 */
function compositionLines({ warnings, }: { readonly warnings: readonly string[]; },): readonly string[] {
  return warnings.filter(function ofComposition(line,): boolean {
    return line.includes('[composeSourcePage]',);
  },);
}

/**
 Rows with one untranslated slice after the archive's opening.

 @param source - slice's text as the artifact records it

 @returns Rows

 @example
 ```ts
 const rows = openingThen({ source: '{ 她说\n', },);
 ```
 */
function openingThen({ source, }: { readonly source: string; },): readonly SourceShippedRow[] {
  return [
    OPENING_ROW,
    {
      incumbentText: '',
      sourceText: source,
    },
  ];
}

/**
 Runs the page-assembly guard over the rows' entry.

 @param rows - slices, anchors among them

 @returns What the guard recorded: the slices it trimmed or withdrew and its findings

 @example
 ```ts
 const assembly = assemblyOf({ rows: ONE_RUN, },);
 ```
 */
function assemblyOf(
  { rows, }: { readonly rows: readonly SourceShippedRow[]; },
): ReturnType<typeof guardPageAssembly> {
  return guardPageAssembly({
    artifact: sourceShippedArtifact({ rows, },),
    slices: sourceShippedSlices({ rows, },),
    sourceText: rows
      .map(function sourceOf({ sourceText, },): string {
        return sourceText;
      },)
      .join('',),
    targetText: sourceShippedArchive({ rows, },),
  },);
}

/**
 One source text the strict MDX grammar reads as syntax, with how the page must carry it once the ladder has run.

 @example
 ```ts
 const hostile: HostileSource = { name: 'a bare brace', source: '{ 她说\n', carried: String.raw`\{ 她说`, };
 ```
 */
type HostileSource = {
  /**
   What the text is, for the case name.
   */
  readonly name: string;

  /**
   The slice's text as the artifact records it.
   */
  readonly source: string;

  /**
   What the page holds after the note once the text is encoded as literal text.
   */
  readonly carried: string;
};

/**
 Sources the strict grammar reads as syntax, each of which stopped the entry when the page carried it as written.
 */
const HOSTILE_SOURCES: readonly HostileSource[] = [
  {
    name: 'a bare opening brace in a sentence',
    source: '她说 { 然后走了。\n',
    carried: String.raw`她说 \{ 然后走了。`,
  },
  {
    name: 'an unclosed brace opening the slice',
    source: '{ 她说\n',
    carried: String.raw`\{ 她说`,
  },
  {
    name: 'an unclosed details tag',
    source: '<details>\n\n她说\n',
    carried: '\\<details\\>\n\n她说',
  },
  {
    name: 'a lone closing details tag',
    source: '她说\n\n</details>\n',
    carried: '她说\n\n\\</details\\>',
  },
  {
    name: 'an indented line holding an expression and an unclosed tag',
    source: '她说\n\n  {x} <y>\n',
    carried: '她说\n\n  \\{x\\} \\<y>',
  },
  {
    name: 'a line starting with export that is not valid JavaScript, which the grammar reads as an ES module',
    source: 'export to PDF 的步骤如下。\n',
    carried: '&#101;xport to PDF 的步骤如下。',
  },
  {
    name: 'a line starting with import that is not valid JavaScript',
    source: '她说\n\nimport data from the old system\n',
    carried: '她说\n\n&#105;mport data from the old system',
  },
  {
    name: 'a fence in a list item followed by a bare brace',
    source: '- ```\n  {\n  ```\n\n然后 { 走了\n',
    carried: '- ```\n  {\n  ```\n\n然后 \\{ 走了',
  },
  {
    name: 'an angle autolink',
    source: '见 <https://example.net/a>\n',
    carried: '见 [https://example.net/a](https://example.net/a)',
  },
  {
    name: 'a definition with a brace and a heading of an open bracket and a bare address, which the parser starts at '
      + 'the definition',
    source: '[a {b]: u\n[ https://example.net/a{b\n---\n',
    carried: '[a \\{b]: u\n[ https://example.net/a\\{b\n---',
  },
];

/**
 Two runs of which the first leaves a fence or display math open at its end and a line of the second would close it,
 with the slices the first step writes as literal text, and the defects the page ships with: a comment that never closes
 inside a display math is taken out of it (`source-ships-opener-in-code.ts`), so the page shows it as the source does and
 ships no defect.
 */
const OPEN_FLOW_PAIRS: readonly {
  readonly name: string;
  readonly first: string;
  readonly second: string;
  readonly slices: string;
  readonly checks: readonly string[];
}[] = [
  {
    name: 'a display math the first run opens, a comment that never closes beside it, and one the second run opens',
    first: '$$\n{x\n\n<!--\n',
    second: '$$\nx < {y}\n',
    slices: 'slices [1, 3]',
    checks: [],
  },
  {
    name: 'a display math the first run opens and a second run holding a comment that never closes',
    first: '$$\n{y\n',
    second: '$$\nz < {w}\n<!--\n',
    slices: 'slices [1, 3]',
    checks: [],
  },
  {
    name: 'a fence after a byte order mark in the first run, which the page reads as text, and a fence in the second',
    first: '\uFEFF```\n{\n```\n',
    second: '`a\n{b` {c\n\uFEFF```\n{\n```\n<Foo />\n',
    slices: 'slices [1, 3]',
    checks: [],
  },
];

/**
 Levels of block quote the plain reading refuses (its bound is 256), which no way of writing makes the grammar accept.
 */
const NESTING_PAST_BOUND = 300;

/**
 The source page of the entries whose rows no writing makes parse, read for destinations as the plain page it is (the
 preparation of a real source refuses a text nested past the bound before a page is composed, and the page itself
 never holds a fence that its rows split).
 */
const PLAIN_SOURCE_PAGE = '一只虎斑猫。\n\n她说\n\n她睡在柜台上。\n\n她又说\n';

/**
 Lone paragraph that parses as written, the control for the sources the grammar refuses.
 */
const PARSING_SOURCE = '\n## 后记\n\n行内 `a < b {c}` 与 <b>粗体</b> 和 {props}。\n';

await describe({
  name: 'the page an entry ships when its archive stops before the original',
  children: [
    describe({
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
          name: 'SHIPS THE ORIGINAL UNDER ITS NOTE FOR AN ARCHIVE WITH NO BODY, the slices a preparation cuts from it, '
            + 'since a page with nothing on it is the one outcome the ruling of 2026-10-23 rules out',
          fn: async () => {
            /**
             Original with two sections, whose archive is empty.
             */
            const original = '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n\n## 后记\n\n她睡在柜台上。\n';
            /**
             Slices the preparation cuts from that pair.
             */
            const { slices, } = prepareDocumentPair({
              sourceText: original,
              targetText: '',
            },);
            /**
             Rows of those slices, which the artifact records the lanes' agreement over.
             */
            const rows = slices.map(function toRow(slice,): SourceShippedRow {
              return {
                incumbentText: '',
                sourceText: slice.source.text,
              };
            },);

            const { page, } = await publishRows({
              rows,
              sourceText: original,
              slices,
              archiveText: '',
            },);
            expect(page,).toContain(NOTE,);
            expect(page,).toContain('## 简介\n\n一只在书店陪伴大家的虎斑猫。',);
            expect(page,).toContain('## 后记\n\n她睡在柜台上。',);
          },
        },),

        it({
          name: 'PUTS ONE NOTE BEFORE EACH CONTIGUOUS RUN, not one per slice: two runs with a translated paragraph '
            + 'between them carry two notes, and the paragraph between them is untouched',
          fn: async () => {
            const { page, } = await publishRows({ rows: TWO_RUNS, },);
            expect(page.split(NOTE,).length - 1,).toBe(2,);
            expect(page,).toContain('She slept on the counter.',);
            /**
             Where each part of the page starts.
             */
            const first = page.indexOf(FIRST_SOURCE.trim(),);
            const middle = page.indexOf('She slept on the counter.',);
            const second = page.indexOf(SECOND_SOURCE.trim(),);
            expect(first,).toBeLessThan(middle,);
            expect(middle,).toBeLessThan(second,);
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
            + 'for the case of a page that keeps it',
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
          name: 'KEEPS EACH SLICE\'S TEXT AS THE ARTIFACT RECORDS IT whatever it holds: angle brackets, braces, '
            + 'backticks, a footnote definition, a link, a heading, a run of blank lines and straight quotes all '
            + 'arrive, trimmed at the ends of the slice only',
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
          name: 'JOINS THE SLICES OF A RUN WITH ONE BLANK LINE, whatever separated them in the original: two slices '
            + 'that the original sets on adjacent lines (a tight list) arrive as separate paragraphs, so the page '
            + 'carries each slice\'s text and not the original\'s line structure between them',
          fn: async () => {
            /**
             Rows whose two untranslated slices were adjacent lines of the original, a newline apart.
             */
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: '- 甲\n',
              },
              {
                incumbentText: '',
                sourceText: '- 乙\n',
              },
            ];
            const { page, } = await publishRows({ rows, },);
            expect(page,).toBe(`${OPENING_ROW.incumbentText}\n${NOTE}\n\n- 甲\n\n- 乙\n`,);
          },
        },),

        it({
          name: 'KEEPS THE ORIGINAL OUT OF THE PAGE-ASSEMBLY PASSES AND THE FOOTNOTE GUARD, which rewrite English and '
            + 'withdraw what breaks the archive\'s footnote graph: an original that holds English-looking words, '
            + 'straight quotes and a footnote definition no archive reference answers (which the guard trims from a '
            + 'slice of definitions alone) comes back untouched, and the guard records nothing against any slice',
          fn: async () => {
            /**
             Original built to be rewritten by the Canadian-spelling and quote passes, or withdrawn by the guard.
             */
            const english = '\n[^9]: The colour of the day was "grey", a neighbour\'s note.\n';
            /**
             Rows with that original as the one untranslated slice.
             */
            const rows = openingThen({ source: english, },);
            expect(assemblyOf({ rows, },),).toEqual({
              trimmed: [],
              withdrawn: [],
              findings: [],
            },);
            const { page, } = await publishRows({ rows, },);
            expect(page,).toContain(`${NOTE}\n\n${english.trim()}`,);
          },
        },),

        it({
          name: 'SHIPS AN ORIGINAL WITH A WINDOWS LINE ENDING AND A US-FORM DATE BYTE FOR BYTE: no page-assembly pass '
            + 'folds the carriage return or rewrites the ordinal, the guard records nothing against the slice, and the '
            + 'published page carries the original as the artifact records it',
          fn: async () => {
            /**
             Original with a Windows line ending and a date the Canadian-form pass rewrites in English wording.
             */
            const english = '\nThe move was on March 5th, 2020.\r\nShe slept on the counter.\r\n';
            /**
             Rows with that original as the one untranslated slice.
             */
            const rows = openingThen({ source: english, },);
            expect(assemblyOf({ rows, },),).toEqual({
              trimmed: [],
              withdrawn: [],
              findings: [],
            },);
            const { page, } = await publishRows({ rows, },);
            expect(page,).toContain(`${NOTE}\n\n${english.trim()}`,);
          },
        },),

        it({
          name: 'RESOLVES THE FIVE FOOTNOTE REFERENCES OF THE XINGZ6020 SHAPE once the original ships, where an '
            + 'artifact that records no original leaves all five unresolved; either way the page-level footnote check '
            + 'reports and the page ships, and with the original the one note nothing references is the only thing '
            + 'named',
          fn: async () => {
            const withSource = await publishRows({ rows: FIVE_REFERENCES, },);
            expect(withSource.footnoteDefects,).toEqual([
              'entry BookshopCat: page footnotes do not resolve; definitions without a reference: 3',
            ],);
            for (const identifier of ['1', '2', '4', '5', '6',])
              expect(withSource.page,).toContain(`[^${identifier}]: `,);
            const { lanes: _lanes, ...recordless } = sourceShippedArtifact({ rows: FIVE_REFERENCES, },) as
              WouldShipSource & {
                readonly lanes: unknown;
              };
            const withoutSource = await publishRows({
              rows: FIVE_REFERENCES,
              artifact: recordless,
            },);
            expect(withoutSource.footnoteDefects,).toEqual([
              'entry BookshopCat: page footnotes do not resolve; references without a definition: 1, 2, 4, 5, 6',
            ],);
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
    },),
    describe({
      name: 'a page whose source text the strict MDX grammar refuses as written',
      children: [
        ...HOSTILE_SOURCES.map(function toCase({ name, source, carried, },) {
          return it({
            name: `ENCODES ${name.toUpperCase()} AS LITERAL TEXT and ships the page, naming step 1 of 3 and the slice `
              + 'in one warn line, with no defect against the page',
            fn: async () => {
              const rows = openingThen({ source, },);
              const { page, artifact, checks, warnings, } = await publishRows({ rows, },);
              expect(function parsesIt(): void {
                assertPageParses({
                  entryId: 'BookshopCat',
                  pageText: page,
                },);
              },).not.toThrow();
              expect(page,).toContain(`${NOTE}\n\n${carried}\n`,);
              expect(checks,).toEqual([],);
              const lines = compositionLines({ warnings, },);
              expect(lines.length,).toBe(1,);
              expect(lines[0],).toContain('step 1 of 3',);
              expect(lines[0],).toContain('slices [1]',);
              // THE PAGE READ BACK AGREES WITH ITS ARTIFACT, the literal text accepted as the original it stands for.
              const { agreement, wording, } = pageAgreement({
                artifact: {
                  ...artifact,
                  preparation: {
                    archiveText: {
                      kind: 'stored',
                      text: sourceShippedArchive({ rows, },),
                    },
                  },
                },
                pageText: page,
              },);
              expect(agreement,).not.toBe('disagreed',);
              expect(wording,).toMatchObject({
                sourceShipped: 1,
                missing: [],
              },);
            },
          },);
        },),

        it({
          name: 'SHIPS A SOURCE THAT PARSES AS WRITTEN with the text of its slice unchanged and no warn line, the '
            + 'control for the cases that encode',
          fn: async () => {
            const { page, checks, warnings, } = await publishRows({
              rows: openingThen({ source: PARSING_SOURCE, },),
            },);
            expect(page,).toContain(`${NOTE}\n\n${PARSING_SOURCE.trim()}\n`,);
            expect(checks,).toEqual([],);
            expect(compositionLines({ warnings, },),).toEqual([],);
          },
        },),

        it({
          name: 'ENCODES ONLY THE RUN THAT DOES NOT PARSE ALONE: a run that parses keeps its text unchanged beside '
            + 'one that is encoded, and the warn line names the encoded run\'s slice only',
          fn: async () => {
            const { page, warnings, } = await publishRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: FIRST_SOURCE,
                },
                MIDDLE_ROW,
                {
                  incumbentText: '',
                  sourceText: '{ 她说\n',
                },
              ],
            },);
            expect(page,).toContain(`${NOTE}\n\n${FIRST_SOURCE.trim()}\n`,);
            expect(page,).toContain(`${NOTE}\n\n\\{ 她说\n`,);
            const lines = compositionLines({ warnings, },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('slices [3]',);
          },
        },),

        it({
          name: 'WRITES THE OPENER OF A COMMENT THAT NEVER CLOSES AS A CHARACTER REFERENCE, so the English after the '
            + 'run stays a paragraph of the page and the package finds no unterminated comment in it',
          fn: async () => {
            const { page, checks, warnings, } = await publishRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: '<!-- 注释\n她说\n',
                },
                MIDDLE_ROW,
              ],
            },);
            expect(page,).toContain(`${NOTE}\n\n&lt;!-- 注释\n她说\n`,);
            expect(page,).toContain('She slept on the counter.',);
            expect(checks,).toEqual([],);
            expect(parseDocument({ text: page, },).parseFindings.map(function kindOf({ kind, },): string {
              return kind;
            },),).toEqual([],);
            const lines = compositionLines({ warnings, },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('step 1 of 3',);
            expect(lines[0],).toContain('slices [1]',);
          },
        },),

        it({
          name: 'SHIPS A RUN NO WRITING MAKES PARSE AS A PAGE WITHOUT IT, keeps the run beside it, names the slice in '
            + 'the defect and the composition line, and the page read back agrees with its artifact and weighs what it '
            + 'should, so a later pass does not publish it again',
          fn: async () => {
            /**
             A run nested past the bound the plain reading keeps, then a
             paragraph the archive covers and a run that parses.
             */
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: `${'> '.repeat(NESTING_PAST_BOUND,)}她说\n`,
              },
              MIDDLE_ROW,
              {
                incumbentText: '',
                sourceText: '她又说\n',
              },
            ];
            const { page, artifact, checks, warnings, } = await publishRows({
              rows,
              sourceText: PLAIN_SOURCE_PAGE,
            },);
            expect(checks,).toEqual(['page-source-unparseable',],);
            expect(page,).not.toContain('她说',);
            expect(page,).toContain('She slept on the counter.',);
            expect(page,).toContain(`${NOTE}\n\n她又说\n`,);
            const lines = compositionLines({ warnings, },);
            expect(lines.at(-1,),).toContain('step 3 of 3',);
            expect(lines.at(-1,),).toContain('slices [1]',);
            const judged = pageAgreement({
              artifact: {
                ...artifact,
                preparation: {
                  archiveText: {
                    kind: 'stored',
                    text: sourceShippedArchive({ rows, },),
                  },
                },
              },
              pageText: page,
            },);
            expect(judged.agreement,).toBe('agreed-weighed',);
            expect(judged.wrongLength,).toBe(false,);
            expect(judged.wording.missing,).toEqual([],);
            expect(judged.sourceDropped,).toEqual([1,],);
          },
        },),

        it({
          name: 'SHIPS BOTH RUNS WITH THE FENCE EACH LEAVES OPEN CLOSED AT ITS END, written as literal text at the '
            + 'first step: each run parses alone and leaves a fence open, the second run\'s opening line would close '
            + 'the first run\'s fence over the English between them and expose the tag after it, and the page read '
            + 'back agrees with its artifact',
          fn: async () => {
            /**
             Two runs that parse alone, the first opening a fence and the second holding a tag inside the fence it
             opens, with a paragraph the archive covers between them.
             */
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: '```\n',
              },
              MIDDLE_ROW,
              {
                incumbentText: '',
                sourceText: '```\n</\n',
              },
            ];
            const { page, artifact, checks, warnings, } = await publishRows({
              rows,
              sourceText: PLAIN_SOURCE_PAGE,
            },);
            expect(checks,).toEqual([],);
            expect(page,).toBe(
              `${OPENING_ROW.incumbentText}\n${NOTE}\n\n\`\`\`\n\`\`\`\n${MIDDLE_ROW.incumbentText}\n${NOTE}\n\n`
                + '```\n</\n```\n',
            );
            expect(liveConstructsOf({ pageText: page, },),).toEqual([],);
            const lines = compositionLines({ warnings, },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('step 1 of 3',);
            expect(lines[0],).toContain('slices [1, 3]',);
            const judged = pageAgreement({
              artifact: {
                ...artifact,
                preparation: {
                  archiveText: {
                    kind: 'stored',
                    text: sourceShippedArchive({ rows, },),
                  },
                },
              },
              pageText: page,
            },);
            expect(judged.agreement,).not.toBe('disagreed',);
            expect(judged.sourceDropped,).toEqual([],);
          },
        },),

        ...OPEN_FLOW_PAIRS.map(function toCase({ name, first, second, slices, checks: expected, },) {
          return it({
            name: `SHIPS NO CODE FROM THE RUNS OF ${name.toUpperCase()}: the flow the first run leaves open is closed `
              + 'at its end, so the opening line of the second run does not close it and expose the text after that '
              + 'line',
            fn: async () => {
              const { page, checks, warnings, } = await publishRows({
                rows: [
                  OPENING_ROW,
                  {
                    incumbentText: '',
                    sourceText: first,
                  },
                  MIDDLE_ROW,
                  {
                    incumbentText: '',
                    sourceText: second,
                  },
                ],
              },);
              expect(liveConstructsOf({ pageText: page, },),).toEqual([],);
              expect(checks,).toEqual(expected,);
              const lines = compositionLines({ warnings, },);
              expect(lines.length,).toBe(1,);
              expect(lines[0],).toContain('step 1 of 3',);
              expect(lines[0],).toContain(slices,);
            },
          },);
        },),

        it({
          name: 'WRITES BOTH RUNS AS LITERAL TEXT AT THE FIRST STEP WHEN EACH HOLDS A FENCE MARK INDENTED FOUR SPACES, '
            + 'which MDX reads as a fence and the plain reading does not: each run parses alone, the one opens a '
            + 'fence the other closes over the English and a brace, and the second step is not needed',
          fn: async () => {
            /**
             Two runs that parse alone, with a paragraph the archive covers between them.
             */
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: '    ```\n',
              },
              MIDDLE_ROW,
              {
                incumbentText: '',
                sourceText: '    ```\n    {\n    ```\n',
              },
            ];
            const { page, checks, warnings, } = await publishRows({ rows, },);
            expect(checks,).toEqual([],);
            expect(page,).toContain(`${NOTE}\n\n    &#96;&#96;&#96;\n`,);
            expect(page,).toContain('She slept on the counter.',);
            const lines = compositionLines({ warnings, },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('step 1 of 3',);
            expect(lines[0],).toContain('slices [1, 3]',);
          },
        },),

        it({
          name: 'WRITES THE LATER RUN AS LITERAL TEXT AT THE FIRST STEP WHEN TWO RUNS IMPORT ONE NAME, so the page '
            + 'does not declare it twice: the first run stays as written, a module, and the second shows its text',
          fn: async () => {
            /**
             Two runs that parse alone and import one name, with a paragraph the archive covers between them.
             */
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: 'import a from "b"\n',
              },
              MIDDLE_ROW,
              {
                incumbentText: '',
                sourceText: 'import a from "b"\n',
              },
            ];
            const { page, checks, warnings, } = await publishRows({ rows, },);
            expect(checks,).toEqual([],);
            expect(page,).toContain(`${NOTE}\n\nimport a from "b"\n`,);
            expect(page,).toContain(`${NOTE}\n\n&#105;mport a from "b"\n`,);
            const lines = compositionLines({ warnings, },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('step 1 of 3',);
            expect(lines[0],).toContain('slices [3]',);
          },
        },),

        it({
          name: 'WRITES THE LATER RUN AS LITERAL TEXT AT THE FIRST STEP WHEN IT IMPORTS A NAME AND HOLDS A COMMENT, '
            + 'which the grammar has no syntax for and the package blanks before it reads the page',
          fn: async () => {
            const { page, checks, warnings, } = await publishRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: 'import a from "b"\n',
                },
                MIDDLE_ROW,
                {
                  incumbentText: '',
                  sourceText: 'import a from "b"\n\n它\n<!-- 注释 -->\n',
                },
              ],
            },);
            expect(checks,).toEqual([],);
            expect(page,).toContain(`${NOTE}\n\n&#105;mport a from "b"\n`,);
            const lines = compositionLines({ warnings, },);
            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('step 1 of 3',);
            expect(lines[0],).toContain('slices [3]',);
          },
        },),

        it({
          name: 'LEAVES TWO RUNS THAT IMPORT DIFFERENT NAMES AS WRITTEN, the control for the repeated import: the '
            + 'page parses with both modules and no step is taken',
          fn: async () => {
            const { page, checks, warnings, } = await publishRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: 'import a from "b"\n',
                },
                MIDDLE_ROW,
                {
                  incumbentText: '',
                  sourceText: 'import c from "b"\n',
                },
              ],
            },);
            expect(checks,).toEqual([],);
            expect(page,).toContain(`${NOTE}\n\nimport c from "b"\n`,);
            expect(compositionLines({ warnings, },),).toEqual([],);
          },
        },),

        it({
          name: 'STILL REPORTS A PAGE THAT NEVER CARRIED ORIGINALS THE GRAMMAR ACCEPTS, the control for the pages '
            + 'shipped without a run: the archive alone, as a page shipped before the originals were carried is, '
            + 'disagrees with an artifact whose runs parse, so a pass publishes it again with them',
          fn: async () => {
            const { artifact, } = await publishRows({ rows: TWO_RUNS, },);
            const judged = pageAgreement({
              artifact: {
                ...artifact,
                preparation: {
                  archiveText: {
                    kind: 'stored',
                    text: sourceShippedArchive({ rows: TWO_RUNS, },),
                  },
                },
              },
              pageText: sourceShippedArchive({ rows: TWO_RUNS, },),
            },);
            expect(judged.agreement,).toBe('disagreed',);
            expect(judged.sourceDropped,).toEqual([],);
          },
        },),

        it({
          name: 'STILL REPORTS TEXT LOST ELSEWHERE ON A PAGE SHIPPED WITHOUT A RUN, the positive control for the page '
            + 'that agrees: the sentence of a paragraph the archive covers is gone, and so the page disagrees',
          fn: async () => {
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: `${'> '.repeat(NESTING_PAST_BOUND,)}她说\n`,
              },
              MIDDLE_ROW,
            ];
            const { page, artifact, } = await publishRows({
              rows,
              sourceText: PLAIN_SOURCE_PAGE,
            },);
            const judged = pageAgreement({
              artifact: {
                ...artifact,
                preparation: {
                  archiveText: {
                    kind: 'stored',
                    text: sourceShippedArchive({ rows, },),
                  },
                },
              },
              pageText: page.replace(
                'She slept on the counter.',
                '',
              ),
            },);
            expect(judged.agreement,).toBe('disagreed',);
            expect(judged.sourceDropped,).toEqual([],);
          },
        },),

        it({
          name: 'READS THE RUN AS ONE UNIT, so a details tag opened in one slice and closed in the next ships as '
            + 'written: neither slice parses alone, and the pair does',
          fn: async () => {
            const { page, warnings, } = await publishRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: '<details>\n\n她说\n',
                },
                {
                  incumbentText: '',
                  sourceText: '\n走了。\n\n</details>\n',
                },
              ],
            },);
            expect(page,).toContain(`${NOTE}\n\n<details>\n\n她说\n\n走了。\n\n</details>\n`,);
            expect(compositionLines({ warnings, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
