/**
 Holds every rule that reads quotes, ellipses or guillemets to the one prose mask (n156, W1; ledger B1265). The
 typography package's `proseMaskReading` is the one scanner of prose: a unit of a text is prose only where the page
 parse shows prose text, and a rule that kept a scanner of its own for code or tags would disagree with it. The case
 once sat in the typography package's tests, which read the source of four other packages; a test of one package
 must not, so the check whose subject is the source of several packages lives here, where the input is the whole
 family and the gate runs this package's whole `test:unit` whatever changed.

 WHAT THE SCAN READS: the text of each reader named in `READERS` (the family package that holds it and its path under
 that package's `src`), for the call `proseMaskReading({`. A reader the family does not hold, and a reader whose text
 lacks the call, are findings.

 OUT OF ITS REACH: a reader that calls the mask through another function of its own, and a rule that reads quotes
 and is not listed; the list is the owner's statement of which rules read them.

 THE FIXTURE CASE COMES FIRST (ledger M21). Fixtures are cat-themed.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Prose mask readers
// The listed readers, and the reading of each for the shared mask.

/**
 Source files whose rules read the mask, each named by the family package that holds it and its path under that
 package's `src`.
 */
const READERS: readonly string[] = [
  'page-pass/restore-typography.ts',
  'page-pass/restore-ellipsis.ts',
  'typography/quote-normalize.ts',
  'translate-floor/translate-guillemets.ts',
  'page-check/corpus-run/quote-style-unify.ts',
];

/**
 The call every reader makes to take its mask from the shared scanner.
 */
const MASK_CALL = 'proseMaskReading({';

/**
 The readers that do not take their mask from the shared scanner.

 @param files - every file of the family

 @param readers - readers listed, each by package directory and path under `src`

 @returns One line for each reader the files do not hold and each whose text lacks the call

 @example
 ```ts
 const findings = readersApart({ files, readers: READERS, },);
 ```
 */
function readersApart({ files, readers, }: { readonly files: readonly SourceText[]; readonly readers: readonly string[]; }): readonly string[] {
  return readers.flatMap(function findingOf(reader,): readonly string[] {
    /**
     The reader's file.
     */
    const file = files.find(function isReader({ path, },): boolean {
      return path === reader;
    },);
    if (file === undefined)
      return [`${reader}: the family holds no such file`,];
    return file.text.includes(MASK_CALL,) ? [] : [`${reader}: does not call ${MASK_CALL}`,];
  },);
}

//endregion Prose mask readers

await describe({
  name: 'every rule that reads quotes takes its mask from the shared scanner',
  children: [
    it({
      name: 'FINDS A READER THAT KEEPS A SCANNER OF ITS OWN and a reader the family does not hold, and leaves one that '
        + 'calls the shared mask',
      fn: async () => {
        expectFindingsAsListed({
          findings: readersApart({
            files: [
              {
                path: 'cat/nap.ts',
                text: 'export const nap = proseMaskReading({ text, },);\n',
                isTest: false,
              },
              {
                path: 'cat/purr.ts',
                text: 'export const purr = scanQuotes({ text, },);\n',
                isTest: false,
              },
            ],
            readers: [
              'cat/nap.ts',
              'cat/purr.ts',
              'cat/hiss.ts',
            ],
          },),
          listed: [
            'cat/purr.ts: does not call proseMaskReading({',
            'cat/hiss.ts: the family holds no such file',
          ],
        },);
      },
    },),
    it({
      name: 'IS THE ONE SCANNER OF PROSE: every rule that reads quotes, ellipses or guillemets takes its mask from '
        + '`proseMaskReading`, and none keeps a scanner of its own for code or tags',
      fn: async () => {
        expectNoFindings({
          findings: readersApart({
            files: await readPackageSource(),
            readers: READERS,
          },),
        },);
      },
    },),
  ],
},);
