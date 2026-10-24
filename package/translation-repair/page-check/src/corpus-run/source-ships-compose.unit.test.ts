/**
 Tests for the steps that make a page carrying the original of untranslated passages parse.

 The page-level cases that drive the real grammar are in `source-shipped-page.unit.test.ts`. A seeded search of forty
 thousand pairs of runs built from a pool of fragments, run with the seeds 31415926 and 27182818, took the first step
 for 32268 and 32142 pairs, and it made the page parse for 27976 and 27683 of them: the refusals found in braces, tags
 and the other syntax of a run, and the runs that parse alone and cannot stand beside another (a fence mark indented
 four spaces or more, a name two runs import). The second step is taken when the page of the first step (the page as
 written, when the first step changed nothing) does not parse and writing every run as literal text gives a page unlike
 the one last tried; the search took it for 383 and 412 pairs, all but 6 and 2 of each a run nested past the bound
 beside a run holding a character the step escapes, and it made the page parse for none. The third step is taken when
 no writing makes the page parse: a run nested past the bound the plain reading keeps, as the module comment of
 `source-ships-encoding.ts` names under LIMITS. The page-level cases for the nested run and for the runs that each
 leave a fence or display math open are in `source-shipped-page.unit.test.ts`, as are the cases that take the first
 step for a fence mark indented four spaces in both runs and for two runs that import one name. The cases here hand the composition a grammar that answers by a rule written in the case, which takes the second
 step and the choices of the third on text the real grammar accepts, so they are cases of the ladder and not of the
 grammar.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  composeSourcePage,
  type ComposedPage,
  type GrammarReader,
  type PageGrammarReading,
  SourcePageCompositionError,
  sourceRunsOf,
  UNTRANSLATED_PASSAGE_NOTE,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import {
  refusingNope,
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 The paragraph the archive covers.
 */
const OPENING_ROW: SourceShippedRow = {
  incumbentText: '## Description\n\nA tabby who kept the bookshop company.\n',
  sourceText: '## 简介\n\n一只在书店陪伴大家的虎斑猫。\n',
};

/**
 A paragraph the archive covers between two runs.
 */
const MIDDLE_ROW: SourceShippedRow = {
  incumbentText: '\nShe slept on the counter.\n',
  sourceText: '\n她睡在柜台上。\n',
};

/**
 Rows with two runs of one slice each, past and around a paragraph the archive covers.
 */
const TWO_RUNS: readonly SourceShippedRow[] = [
  OPENING_ROW,
  {
    incumbentText: '',
    sourceText: '{BAD} 她说\n',
  },
  MIDDLE_ROW,
  {
    incumbentText: '',
    sourceText: '{BAD} 她又说\n',
  },
];

/**
 Composes the page of the rows with a grammar and keeps the warn lines.

 @param rows - slices, anchors among them

 @param readGrammar - what the grammar says of a text

 @returns Composition and the warnings it wrote

 @example
 ```ts
 const { composed, warnings, } = composeRows({ rows: TWO_RUNS, readGrammar: refusingBadBraces, },);
 ```
 */
function composeRows(
  {
    rows,
    readGrammar,
  }: {
    readonly rows: readonly SourceShippedRow[];
    readonly readGrammar: GrammarReader;
  },
): {
  readonly composed: ComposedPage;
  readonly warnings: readonly string[];
} {
  /**
   Warnings written.
   */
  const warnings: string[] = [];
  /**
   Logger keeping its warnings.
   */
  const l: Logger = {
    ...tagged({ tag: 'source-ships-compose-test', },),
    warn: function keep(message: string,): void {
      warnings.push(message,);
    },
  };
  return {
    composed: composeSourcePage({
      artifact: sourceShippedArtifact({ rows, },),
      slices: sourceShippedSlices({ rows, },),
      archiveText: sourceShippedArchive({ rows, },),
      entryId: 'BookshopCat',
      l,
      readGrammar,
    },),
    warnings,
  };
}

/**
 A grammar that refuses a page, and only a page, that holds a brace pair written as it is.

 STANDS FOR A TEXT THAT PARSES ALONE AND NOT IN THE PAGE, which the real grammar has none of for the first step to miss:
 a text of its own is never refused here, since it holds no page heading.

 @param pageText - page or fragment

 @returns Refusal for a page holding the pair, a parse otherwise

 @example
 ```ts
 const reading = refusingBadBraces({ pageText: '## Description\n{BAD}', },);
 ```
 */
function refusingBadBraces({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  return (pageText.includes('## Description',) && pageText.includes('{BAD}',))
    ? {
      kind: 'refused',
      refusal: 'at 1:1 (test/braces)',
    }
    : { kind: 'parses', };
}

/**
 A grammar that refuses every page.

 @param pageText - page or fragment

 @returns Refusal for a page, a parse for a fragment of its own

 @example
 ```ts
 const reading = refusingEveryPage({ pageText: '## Description\n', },);
 ```
 */
function refusingEveryPage({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  return pageText.includes('## Description',)
    ? {
      kind: 'refused',
      refusal: 'at 2:2 (test/always)',
    }
    : { kind: 'parses', };
}

/**
 A grammar that parses everything.

 @returns Parse

 @example
 ```ts
 const reading = parsingEverything();
 ```
 */
function parsingEverything(): PageGrammarReading {
  return { kind: 'parses', };
}

await describe({
  name: 'source ships compose',
  children: [
    describe({
      name: composeSourcePage.name,
      children: [
        it({
          name: 'TAKES NO STEP FOR A PAGE THE GRAMMAR READS AS WRITTEN, whatever it carries: no warn line, no defect',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: TWO_RUNS,
              readGrammar: parsingEverything,
            },);
            expect(composed.step,).toBe('as-written',);
            expect(composed.pageText,).toContain(`${UNTRANSLATED_PASSAGE_NOTE}\n\n{BAD} 她说`,);
            expect(composed.slices,).toEqual([],);
            expect(composed.defects,).toEqual([],);
            expect(warnings,).toEqual([],);
          },
        },),

        it({
          name: 'READS NOTHING AGAINST THE GRAMMAR WHEN NO SLICE SHIPS ITS ORIGINAL, so a page of English alone costs '
            + 'the parse it always cost',
          fn: async () => {
            /**
             Grammar calls made.
             */
            const asked: string[] = [];
            const { composed, } = composeRows({
              rows: [OPENING_ROW, MIDDLE_ROW,],
              readGrammar: function counting({ pageText, },): PageGrammarReading {
                asked.push(pageText,);
                return { kind: 'parses', };
              },
            },);
            expect(composed.step,).toBe('as-written',);
            expect(asked,).toEqual([],);
          },
        },),

        it({
          name: 'WRITES EVERY RUN AS LITERAL TEXT WHEN NO RUN IS REFUSED ALONE, the second step, with one warn line '
            + 'naming it and every source slice, and skips the first step since it would change nothing',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: TWO_RUNS,
              readGrammar: refusingBadBraces,
            },);
            expect(composed.step,).toBe('encoded-every-run',);
            expect(composed.slices,).toEqual([1, 3,],);
            expect(composed.pageText,).toContain(`${UNTRANSLATED_PASSAGE_NOTE}\n\n\\{BAD\\} 她说`,);
            expect(composed.pageText,).toContain(String.raw`\{BAD\} 她又说`,);
            expect(composed.defects,).toEqual([],);
            expect(warnings.length,).toBe(1,);
            expect(warnings[0],).toContain('step 2 of 3',);
            expect(warnings[0],).toContain('slices [1, 3]',);
            expect(warnings[0],).toContain('at 1:1 (test/braces)',);
          },
        },),

        it({
          name: 'SHIPS THE PAGE WITHOUT ITS SOURCE RUNS WHEN LITERAL TEXT DOES NOT HELP, the third step, with the '
            + 'defect page-source-unparseable naming the slices, after the warn line of each step taken',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: TWO_RUNS,
              readGrammar: refusingEveryPage,
            },);
            expect(composed.step,).toBe('dropped',);
            expect(composed.slices,).toEqual([1, 3,],);
            expect(composed.pageText,).toBe(`${OPENING_ROW.incumbentText}${MIDDLE_ROW.incumbentText}`,);
            expect(composed.defects,).toEqual([
              {
                check: 'page-source-unparseable',
                message: 'entry BookshopCat ships without the source text of slices [1, 3], since the page did not '
                  + 'parse with it as written or as literal text',
              },
            ],);
            expect(warnings.length,).toBe(2,);
            expect(warnings[0],).toContain('step 2 of 3',);
            expect(warnings[1],).toContain('step 3 of 3',);
            expect(warnings[1],).toContain('the page with every source run written as literal text does not parse',);
          },
        },),

        it({
          name: 'DROPS ONLY THE RUN THAT NO WRITING MAKES PARSE, the third step: the run that parses alone stays on '
            + 'the page, the defect names the dropped slice only, and the page is weighed without that slice',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: 'NOPE 她说\n',
                },
                MIDDLE_ROW,
                {
                  incumbentText: '',
                  sourceText: '她又说\n',
                },
              ],
              readGrammar: refusingNope,
            },);
            expect(composed.step,).toBe('dropped',);
            expect(composed.slices,).toEqual([1,],);
            expect(composed.pageText,).not.toContain('NOPE',);
            expect(composed.pageText,).toContain(`${UNTRANSLATED_PASSAGE_NOTE}\n\n她又说`,);
            expect(composed.defects,).toEqual([
              {
                check: 'page-source-unparseable',
                message: 'entry BookshopCat ships without the source text of slices [1], since the page did not '
                  + 'parse with it as written or as literal text',
              },
            ],);
            expect(wouldShipTextPerSlice({ artifact: composed.weighed, },).map(function kindOf({ reading, },): string {
              return reading.kind;
            },),).toEqual(['wording', 'nothing-ships', 'wording', 'source-ships',],);
            expect(warnings.at(-1,),).toContain('step 3 of 3',);
            expect(warnings.at(-1,),).toContain('slices [1]',);
          },
        },),

        it({
          name: 'WEIGHS AGAINST THE ARTIFACT WITHOUT ANY RECORD WHEN THE PAGE HAD TO DROP EVERY RUN, the fallback of '
            + 'the third step when no run is refused alone',
          fn: async () => {
            const { composed, } = composeRows({
              rows: TWO_RUNS,
              readGrammar: refusingEveryPage,
            },);
            expect(composed.step,).toBe('dropped',);
            expect(wouldShipTextPerSlice({ artifact: composed.weighed, },).map(function kindOf({ reading, },): string {
              return reading.kind;
            },),).toEqual(['wording', 'nothing-ships', 'wording', 'nothing-ships',],);
          },
        },),

        it({
          name: 'WEIGHS AGAINST THE ARTIFACT ITSELF FOR A PAGE THAT KEPT EVERY RUN, the control for the pages that '
            + 'dropped',
          fn: async () => {
            const { composed, } = composeRows({
              rows: TWO_RUNS,
              readGrammar: parsingEverything,
            },);
            expect(wouldShipTextPerSlice({ artifact: composed.weighed, },).map(function kindOf({ reading, },): string {
              return reading.kind;
            },),).toEqual(['wording', 'source-ships', 'wording', 'source-ships',],);
          },
        },),

        it({
          name: 'TAKES THE THIRD STEP ALONE WHEN THERE IS NOTHING TO ESCAPE, since the first two would change nothing',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: '她说\n',
                },
              ],
              readGrammar: refusingEveryPage,
            },);
            expect(composed.step,).toBe('dropped',);
            expect(warnings.length,).toBe(1,);
            expect(warnings[0],).toContain('step 3 of 3',);
            expect(warnings[0],).toContain('the page with its source text as written does not parse',);
          },
        },),

        it({
          name: 'WRITES A RUN HOLDING A COMMENT THAT NEVER CLOSES AS LITERAL TEXT EVEN WHEN THE GRAMMAR PARSES THE '
            + 'PAGE, the first step, since the package masks everything after such an opener and no grammar vouches '
            + 'for it',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: '<!-- 注释\n她说\n',
                },
                MIDDLE_ROW,
              ],
              readGrammar: parsingEverything,
            },);
            expect(composed.step,).toBe('encoded-failing-runs',);
            expect(composed.slices,).toEqual([1,],);
            expect(composed.pageText,).toContain(`${UNTRANSLATED_PASSAGE_NOTE}\n\n&lt;!-- 注释\n她说`,);
            expect(composed.pageText,).not.toContain('<!--',);
            expect(composed.defects,).toEqual([],);
            expect(warnings.length,).toBe(1,);
            expect(warnings[0],).toContain('step 1 of 3',);
            expect(warnings[0],).toContain('slices [1]',);
            expect(warnings[0],).toContain('never closes',);
          },
        },),

        it({
          name: 'SHIPS A RUN WHOSE COMMENT CLOSES AS WRITTEN, the control for the run whose comment never does',
          fn: async () => {
            const { composed, warnings, } = composeRows({
              rows: [
                OPENING_ROW,
                {
                  incumbentText: '',
                  sourceText: '<!-- 注释 -->\n她说\n',
                },
                MIDDLE_ROW,
              ],
              readGrammar: parsingEverything,
            },);
            expect(composed.step,).toBe('as-written',);
            expect(composed.pageText,).toContain('<!-- 注释 -->',);
            expect(warnings,).toEqual([],);
          },
        },),

        it({
          name: 'ENDS AT THE THIRD STEP, so the error for an exhausted ladder names the entry and nothing of the page',
          fn: async () => {
            expect(new SourcePageCompositionError({ entryId: 'BookshopCat', },).message,).toBe(
              'entry BookshopCat: the composition of the page ended without a page, which its last step cannot do',
            );
          },
        },),
      ],
    },),

    describe({
      name: sourceRunsOf.name,
      children: [
        it({
          name: 'GROUPS THE SLICES SHIPPING THEIR ORIGINAL INTO THE RUNS THE NOTE STANDS BEFORE, in page order',
          fn: async () => {
            /**
             Rows: an opening, a run of two, a paragraph, a run of one.
             */
            const rows: readonly SourceShippedRow[] = [
              OPENING_ROW,
              {
                incumbentText: '',
                sourceText: '甲\n',
              },
              {
                incumbentText: '',
                sourceText: '乙\n',
              },
              MIDDLE_ROW,
              {
                incumbentText: '',
                sourceText: '丙\n',
              },
            ];
            expect(sourceRunsOf({
              slices: wouldShipTextPerSlice({ artifact: sourceShippedArtifact({ rows, },), },),
            },),).toEqual([[1, 2,], [4,],],);
            expect(sourceRunsOf({ slices: [], },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
