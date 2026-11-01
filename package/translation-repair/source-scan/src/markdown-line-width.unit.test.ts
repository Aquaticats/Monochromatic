/**
 Guards the width of the family's Markdown lines (ledger B1359, B1376): a line of 120 columns or more is a finding when
 it could be broken, unless the file's allowance says it already stood.

 The rule is under 120 columns, and neither `markdown-lint-clean` nor `doc-line-limit` reads the width, so the
 re-checks of rounds w404 and w406 found hundreds of wide lines by hand. Fixing every old line is a hand wrap at
 semantic breaks, so the guard is a ratchet on the count of each file: a file may not gain a wide line, and a file
 whose allowance is higher than its count must be lowered. A line that cannot be broken (a link or a code span), and a
 fenced code block read as CommonMark reads one, are not counted. A column is a display column: a character of East Asian
 Width Wide or Fullwidth fills two.

 THE LIMIT OF "ONLY GOES DOWN" (ledger B1376): the guard holds each count to its allowance and each allowance to its
 count, so no file gains a wide line unnoticed and no allowance sits above a count. It cannot see a commit that raises
 a count and its allowance together: that is a diff of `markdown-line-width-allowance.test-fixture.ts` with a number
 that went up, which only a reader of the diff can refuse, and no ceiling the tree derives could stand against a hand
 edit of the file that holds it. An entry of zero is a finding, since a file with no entry holds none.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';
import {
  WIDE_FROM,
  wideLineCounts,
  wideLinesOf,
} from './markdown-line-width.test-fixture.ts';
import { WIDE_LINE_ALLOWANCE, } from './markdown-line-width-allowance.test-fixture.ts';

/**
 A prose line of exactly the width at which a line is wide.
 */
const WIDE_PROSE = ('The cat naps on the warm sill. '.repeat(Math.ceil(WIDE_FROM / 31,),)).slice(0, WIDE_FROM,);

/**
 Findings of the allowance against the counts: a file over its allowance, and a file whose allowance is above its count.

 @param counts - wide lines of each file that has any
 @param allowance - wide lines each file may hold

 @returns One finding for each file that parts from its allowance, in path order

 @example
 ```ts
 const findings = allowanceFindings({ counts: new Map([['cat.md', 2,],],), allowance: { 'cat.md': 1, }, },);
 ```
 */
function allowanceFindings(
  { counts, allowance, }: {
    readonly counts: ReadonlyMap<string, number>;
    readonly allowance: Readonly<Record<string, number>>;
  },
): readonly string[] {
  /**
   Every path either side names, in order.
   */
  const paths = [...new Set([...counts.keys(), ...Object.keys(allowance,),],),].toSorted();
  return paths.flatMap(function findingsOf(path,): readonly string[] {
    /**
     Wide lines the file holds, and the number it may hold.
     */
    const held = counts.get(path,) ?? 0;
    const allowed = allowance[path] ?? 0;
    if (Object.hasOwn(allowance, path,) && (allowed === 0))
      return [`${path} has an allowance of 0; remove the entry, since a file with no entry holds none`,];
    if (held > allowed)
      return [`${path} holds ${String(held,)} lines of ${String(WIDE_FROM,)} columns or more where ${String(allowed,)} are allowed; wrap the new ones at a semantic break`,];
    if (held < allowed)
      return [`${path} holds ${String(held,)} such lines and its allowance is ${String(allowed,)}; lower the allowance to ${String(held,)}`,];
    return [];
  },);
}

await describe({
  name: 'Markdown line width',
  children: [
    it({
      name: 'READS A LINE OF THE WIDTH AS WIDE AND A LINE JUST UNDER IT AS NOT, outside a fence',
      fn: async () => {
        expect(WIDE_PROSE.length,).toBe(WIDE_FROM,);
        expect(wideLinesOf({ text: `A short line.\n${WIDE_PROSE}\n${WIDE_PROSE.slice(1,)}\n`, },),).toEqual([2,],);
        expect(wideLinesOf({ text: `\`\`\`text\n${WIDE_PROSE}\n\`\`\`\n${WIDE_PROSE}\n`, },),).toEqual([4,],);
      },
    },),

    it({
      name: 'LEAVES A LINE THAT COULD NOT BE BROKEN, a link or a code span being the one exception the rule names',
      fn: async () => {
        /**
         A list item that is one link with a long destination, a reference definition and a line of one long code span.
         */
        const unbreakable = [
          `- [${'cat '.repeat(20,)}](${'long-file-name'.repeat(8,)}.md)`,
          `[label]: ${'long-file-name'.repeat(10,)}.md`,
          `\`${'cat nap '.repeat(20,)}\``,
        ].join('\n',);
        expect(wideLinesOf({ text: unbreakable, },),).toEqual([],);
        // A BREAKABLE LINE BESIDE THE LINK IS WIDE: the control that shows the exception is not a blanket.
        expect(wideLinesOf({ text: `${WIDE_PROSE} [cat](nap.md)\n`, },),).toEqual([1,],);
      },
    },),

    it({
      name: 'READS A FENCE AS COMMONMARK DOES (ledger B1376): three or more backticks or tildes open it, a backtick fence '
        + 'has no backtick in its info string, and only a run of the same character at least as long closes it',
      fn: async () => {
        /**
         A line of inline code that opens with three backticks and is no fence, and the wide prose after it.
         */
        const notAFence = `\`\`\`cat\`\`\`\n${WIDE_PROSE}\n`;
        expect(wideLinesOf({ text: notAFence, },),).toEqual([2,],);
        /**
         A fence of four backticks, a run of three inside it that closes nothing, its close, and wide prose after.
         */
        const longFence = [
          '````md',
          '```',
          WIDE_PROSE,
          '````',
          WIDE_PROSE,
        ].join('\n',);
        expect(wideLinesOf({ text: longFence, },),).toEqual([5,],);
        /**
         A fence of tildes, a backtick run inside it that closes nothing, its close, and wide prose after.
         */
        const tildeFence = [
          '~~~text',
          '```',
          WIDE_PROSE,
          '~~~',
          WIDE_PROSE,
        ].join('\n',);
        expect(wideLinesOf({ text: tildeFence, },),).toEqual([5,],);
        // The controls: the plain fence of three backticks still hides what it holds.
        expect(wideLinesOf({ text: `\`\`\`text\n${WIDE_PROSE}\n\`\`\`\n${WIDE_PROSE}\n`, },),).toEqual([4,],);
      },
    },),

    it({
      name: 'READS A FENCE ONLY AT THE INDENT COMMONMARK ALLOWS (ledger B1413): an opening or closing run indented four '
        + 'columns past its container is no fence, so the prose after an indented run is measured and an over-indented '
        + 'run closes nothing; a fence inside a list item, after a marker or inside a block quote is still a fence',
      fn: async () => {
        /**
         The reviewer's reproducer: four spaces and three backticks, a blank line, and a wide line of prose.
         */
        const indentedCode = `    \`\`\`\n\n${'cat '.repeat(WIDE_FROM / 4,)}`;
        expect(wideLinesOf({ text: indentedCode, },),).toEqual([3,],);
        // A paragraph line, then a run indented four columns: it continues the paragraph and opens nothing.
        expect(wideLinesOf({ text: `A line.\n    \`\`\`\n${WIDE_PROSE}\n`, },),).toEqual([3,],);
        /**
         A fence, an over-indented run that closes nothing, wide prose inside it, the real close and wide prose after.
         */
        const overIndentedClose = [
          '```text',
          WIDE_PROSE,
          '    ```',
          WIDE_PROSE,
          '```',
          WIDE_PROSE,
        ].join('\n',);
        expect(wideLinesOf({ text: overIndentedClose, },),).toEqual([6,],);
        // THE CONTROLS: three columns of indent open and close a fence, and a fence inside a numbered item, indented to
        // the item's text, hides what it holds (the shape of the corpus-pass runbook).
        expect(wideLinesOf({ text: `   \`\`\`\n${WIDE_PROSE}\n   \`\`\`\n${WIDE_PROSE}\n`, },),).toEqual([4,],);
        /**
         A numbered item, a fence indented four columns under it, its wide code, its close, and wide prose after.
         */
        const itemFence = [
          '1.  Confirm the identity exists.',
          '',
          '    ```sh',
          `    ${WIDE_PROSE}`,
          '    ```',
          '',
          WIDE_PROSE,
        ].join('\n',);
        expect(wideLinesOf({ text: itemFence, },),).toEqual([7,],);
        /**
         A bullet that opens a fence on its marker line, and a block quote that does, each followed by wide prose.
         */
        const markerFence = ['- ```ts', `  ${WIDE_PROSE}`, '  ```', WIDE_PROSE,].join('\n',);
        const quotedFence = ['> ```ts', `> ${WIDE_PROSE}`, '> ```', WIDE_PROSE,].join('\n',);
        expect(wideLinesOf({ text: markerFence, },),).toEqual([4,],);
        expect(wideLinesOf({ text: quotedFence, },),).toEqual([4,],);
      },
    },),

    it({
      name: 'COUNTS A WIDE CHARACTER OF THE UNICODE TABLE AS TWO COLUMNS (ledger B1413): U+2329 and U+2FF0, which the '
        + 'reviewer found counted as one, fill two, and a flag of two regional indicators fills two and not four',
      fn: async () => {
        // Forty of the character and a space fill 120 columns.
        expect(wideLinesOf({ text: '\u{2329} '.repeat(WIDE_FROM / 3,), },),).toEqual([1,],);
        expect(wideLinesOf({ text: '\u{2FF0} '.repeat(WIDE_FROM / 3,), },),).toEqual([1,],);
        // A flag and a space fill three columns, so 24 of them fill 72; counted four for the pair, they filled 120.
        expect(wideLinesOf({ text: '\u{1F1E8}\u{1F1E6} '.repeat(WIDE_FROM / 5,), },),).toEqual([],);
      },
    },),

    it({
      name: 'COUNTS A COLUMN AS A DISPLAY COLUMN (ledger B1376): a wide character fills two, so a line of 80 code points '
        + 'that fills 120 columns is wide and one that fills 119 is not',
      fn: async () => {
        /**
         Eighty code points of one wide character and a space, which fill 120 columns.
         */
        const wide = '猫 '.repeat(WIDE_FROM / 3,);
        /**
         The same with the last space taken off and one narrow character in its place, which fill 119 columns.
         */
        const narrower = `${'猫 '.repeat((WIDE_FROM / 3) - 1,)}猫`;
        expect(wide.length,).toBeLessThan(WIDE_FROM,);
        expect(wideLinesOf({ text: wide, },),).toEqual([1,],);
        expect(wideLinesOf({ text: narrower, },),).toEqual([],);
        // A combining mark fills no column of its own: 177 code points fill 118 columns.
        expect(wideLinesOf({ text: 'a\u0301 '.repeat((WIDE_FROM / 2) - 1,), },),).toEqual([],);
      },
    },),

    it({
      name: 'FINDS AN ALLOWANCE ABOVE THE COUNT OF A REAL DOCUMENT, and an entry of zero (ledger B1376, B1413): the '
        + 'allowance of the package README is raised by one over the count the scan reads from its file, whatever that '
        + 'count is, so a clean wrap of its last wide line leaves the case as it was',
      fn: async () => {
        /**
         The README of the package, whose count is zero or more.
         */
        const readme = 'package/translation-repair/README.md';
        const counts = await wideLineCounts();
        /**
         The findings with the allowance of that file one above its count.
         */
        const raised = allowanceFindings({
          counts,
          allowance: { ...WIDE_LINE_ALLOWANCE, [readme]: (counts.get(readme,) ?? 0) + 1, },
        },);
        expect(raised.length,).toBe(1,);
        expect(raised[0],).toContain(readme,);
        expect(raised[0],).toContain('lower the allowance',);
        expect(allowanceFindings({ counts, allowance: { ...WIDE_LINE_ALLOWANCE, 'cat.md': 0, }, },),).toEqual([
          'cat.md has an allowance of 0; remove the entry, since a file with no entry holds none',
        ],);
      },
    },),

    it({
      name: 'FINDS A FILE OVER ITS ALLOWANCE, and a file whose allowance is above its count, and passes a file at it',
      fn: async () => {
        /**
         The allowance of two files, one of which is over it and one under.
         */
        const allowance = { 'nap.md': 1, 'sill.md': 2, 'warm.md': 1, };
        /**
         The counts the files hold.
         */
        const counts = new Map([['nap.md', 3,], ['sill.md', 1,], ['warm.md', 1,], ['new.md', 1,],],);
        expect(allowanceFindings({ counts, allowance, },).map(function pathOf(finding,): string {
          return finding.split(' ',)[0] ?? '';
        },),).toEqual(['nap.md', 'new.md', 'sill.md',],);
      },
    },),

    it({
      name: 'FINDS NO FILE OF THE FAMILY WIDER THAN ITS ALLOWANCE, and no allowance above its count',
      fn: async () => {
        expectNoFindings({
          findings: allowanceFindings({ counts: await wideLineCounts(), allowance: WIDE_LINE_ALLOWANCE, },),
        },);
      },
    },),
  ],
},);
