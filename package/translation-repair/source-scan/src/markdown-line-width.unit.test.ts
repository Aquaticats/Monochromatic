/**
 Guards the width of the family's Markdown lines (ledger B1359): a line of 120 code points or more is a finding when it
 could be broken, unless the file's allowance says it already stood.

 The rule is under 120 columns, and neither `markdown-lint-clean` nor `doc-line-limit` reads the width, so the
 re-checks of rounds w404 and w406 found hundreds of wide lines by hand. Fixing every old line is a hand wrap at
 semantic breaks, so the guard is a ratchet on the count of each file: a file may not gain a wide line, and a file
 whose allowance is higher than its count must be lowered, so the allowance only goes down. A line that cannot be broken
 (a link or a code span), and a fenced code block, are not counted.

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
