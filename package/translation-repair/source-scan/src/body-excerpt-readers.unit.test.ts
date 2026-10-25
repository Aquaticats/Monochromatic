/**
 Guards that the provider's own words, kept on `ProviderHttpError.bodyExcerpt`,
 are read only by the files that have a reason to.

 WHY IT MATTERS. The field holds the opening of a provider's reply body. It is
 withheld from a marked subclass's message so no refusal printer repeats it,
 and it reaches a log line only through `exchangeFailureLogText`. A new reader
 that copies it into a stored record, a thrown message or a printed report
 would undo both, so a file that names the field must be listed here with the
 reason it may.

 WHAT IT READS. The text of every production file, comments included, for the
 word `bodyExcerpt`; a mention in a comment is listed too, so the comment is
 read when the field's readers change. A reader that reaches the field through
 another name (a destructuring rename, a copy into another binding) still
 names it once, at the read. It cannot see a reader that receives the text
 from one of the listed files.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a scan
 shown able to find a reader. Fixtures are cat-themed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { expectFindingsAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 The field this scan holds to its readers.
 */
const FIELD = 'bodyExcerpt';

/**
 Files that name the field, each with why it may.
 */
const NAMED_READERS: Readonly<Record<string, string>> = {
  'provider-stream/completion-shape.ts': 'declares the field and builds it, bounded, from the reply body',
  'provider/request-size-refusal.ts': 'comment saying where the withheld excerpt is read',
  'provider-stream/exchange-failure-text.ts': 'reads it for log lines alone, labelled as the provider\'s and quoted onto one line',
  'provider/provider-budget-refusal.ts': 'classifies a refusal by what the excerpt names and reads a wait out of it, text never kept',
  'provider/decision-context-refusal.ts': 'classifies a refusal by the error type the excerpt names, text never kept',
  'probe/corpus-run/editor-fidelity-spend-guard.ts': 'asks only whether the excerpt is empty, to know an error status delivered no body to bill, text never kept',
};

/**
 Paths of the production files that name the field.

 @param files - files to read

 @returns Paths in the order given, test files excluded

 @example
 ```ts
 const named = filesNaming({ files, });
 ```
 */
function filesNaming({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return files
    .filter(function isNamingProduction(file,): boolean {
      return (!file.isTest) && file.text.includes(FIELD,);
    },)
    .map(function pathOf(file,): string {
      return file.path;
    },)
    .toSorted();
}

await describe({
  name: 'bodyExcerpt readers',
  children: [
    it({
      name: 'FINDS A PRODUCTION FILE NAMING THE FIELD and skips a test file and a file that does not',
      fn: async () => {
        expect(filesNaming({
          files: [
            {
              path: 'cat-report.ts',
              text: 'const said = error.bodyExcerpt;',
              isTest: false,
            },
            {
              path: 'cat-report.unit.test.ts',
              text: 'expect(error.bodyExcerpt,)',
              isTest: true,
            },
            {
              path: 'cat-nap.ts',
              text: 'const nap = 1;',
              isTest: false,
            },
          ],
        },),).toEqual(['cat-report.ts',],);
      },
    },),
    it({
      name: 'HOLDS THE FIELD TO THE FILES LISTED WITH A REASON, and lists no file that stopped naming it',
      fn: async () => {
        expectFindingsAsListed({
          findings: filesNaming({ files: await readPackageSource(), },),
          listed: Object.keys(NAMED_READERS,).toSorted(),
        },);
      },
    },),
  ],
},);
