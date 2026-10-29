/**
 Tests for the TSDoc example scan (ledger D9): every function declaration's
 `@example` calls the function it documents and passes the keys it requires.

 THE FIXTURES COME FIRST, so the "FINDS NOTHING IN THIS PACKAGE'S SOURCE" case
 is read against a
 scan shown able to find each fault (ledger M21: a check that could not fail).
 Fixtures are cat-themed; the package case reads the source of every family package.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import { exampleFindingsOf, } from './tsdoc-example-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 A documented function, as the formatter writes one.

 @param example - example's code line

 @param declaration - declaration's lines

 @returns Source text

 @example
 ```ts
 const text = documented({ example: 'feedCat({ kibble, },);', declaration: ['export function feedCat(', ...], },);
 ```
 */
function documented(
  {
    example,
    declaration,
  }: {
    readonly example: string;
    readonly declaration: readonly string[];
  },
): string {
  return [
    '/**',
    ' Feeds the cat.',
    '',
    ' @example',
    ' ```ts',
    ` ${example}`,
    ' ```',
    ' */',
    ...declaration,
    '  return 1;',
    '}',
  ].join('\n',);
}

/**
 A declaration taking a required, an optional and a defaulted key.
 */
const FEED_CAT = [
  'export function feedCat(',
  '  { kibble, water, nap = 3, }: { readonly kibble: number; readonly water?: number; readonly nap: number; },',
  '): number {',
];

/**
 A declaration in the multi-line shape, its type wrapped and commented.
 */
const GROOM_CAT = [
  'export async function groomCat<const BrushT,>(',
  '  {',
  '    purr,',
  '    brush,',
  '  }: ForeignBorrowed<{',
  '    /**',
  '     Whether the cat purrs (it doesn\'t always; {braces} are text here).',
  '     */',
  '    readonly purr: boolean;',
  '    readonly brush: BrushT;',
  '  }>,',
  '): Promise<number> {',
];

await describe({
  name: exampleFindingsOf.name,
  children: [
    it({
      name: 'FINDS AN EXAMPLE CALLING ANOTHER FUNCTION and one leaving out a required key, the two D9 faults',
      fn: async () => {
        expect({
          otherCallee: exampleFindingsOf({
            text: documented({ example: 'const bowl = fillBowl({ kibble, },);', declaration: FEED_CAT, },),
          },),
          missingKey: exampleFindingsOf({
            text: documented({ example: 'feedCat({ water: 1, },);', declaration: FEED_CAT, },),
          },),
          missingInWrapped: exampleFindingsOf({
            text: documented({ example: 'await groomCat({ brush, },);', declaration: GROOM_CAT, },),
          },),
        },).toEqual({
          otherCallee: [{ name: 'feedCat', line: 9, problem: 'calls another function', },],
          missingKey: [{ name: 'feedCat', line: 9, problem: 'leaves out kibble', },],
          missingInWrapped: [{ name: 'groomCat', line: 9, problem: 'leaves out purr', },],
        },);
      },
    },),
    it({
      name: 'PASSES AN EXAMPLE THAT LEAVES OUT ONLY OPTIONAL OR DEFAULTED KEYS, elides them, or passes a variable',
      fn: async () => {
        expect([
          'feedCat({ kibble, },);',
          'feedCat({ ..., },);',
          'feedCat(order,);',
        ].flatMap(function findingsOf(example,) {
          return exampleFindingsOf({ text: documented({ example, declaration: FEED_CAT, },), },);
        },),).toEqual([],);
        expect(exampleFindingsOf({
          text: documented({ example: 'await groomCat({ purr: true, brush, },);', declaration: GROOM_CAT, },),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'READS AN EXAMPLE WHOLE when a string in it quotes a fence, since only a line of backticks closes one '
        + '(audit area six: a backtick run mid-line was read as the close, and the keys after it as missing)',
      fn: async () => {
        expect(exampleFindingsOf({
          text: documented({ example: 'feedCat({ note: \'a ``` b\', kibble, },);', declaration: FEED_CAT, },),
        },),).toEqual([],);
        expect(exampleFindingsOf({
          text: documented({ example: 'feedCat({ note: \'a ``` b\', water: 1, },);', declaration: FEED_CAT, },),
        },),).toEqual([{ name: 'feedCat', line: 9, problem: 'leaves out kibble', },],);
      },
    },),
    it({
      name: 'FINDS NOTHING IN THIS PACKAGE\'S SOURCE: every function declaration\'s example calls it with its '
        + 'required keys (ledger D9 listed 29 that did not, and this scan first read 42)',
      fn: async () => {
        /**
         Every source file of every family package other than tests, fixtures included.
         */
        const files = (await readPackageSource()).filter(function isSource({ path, },): boolean {
          return (!path.endsWith('.test.ts',)) && (!path.endsWith('.d.ts',));
        },);

        /**
         Findings across them, each named by file.
         */
        const findings = files.flatMap(function scan({ path, text, },): readonly string[] {
          return exampleFindingsOf({ text, },)
            .map(function named(finding,): string {
              return `${path}:${String(finding.line,)} ${finding.name} ${finding.problem}`;
            },);
        },).toSorted();
        expectNoFindings({ findings, },);
      },
    },),
  ],
},);
