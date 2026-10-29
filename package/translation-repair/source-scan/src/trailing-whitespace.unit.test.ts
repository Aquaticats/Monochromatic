/**
 Guards the repository's `.editorconfig`, which sets
 `trim_trailing_whitespace = true` for every file but HTML (ledger B119): no
 line of the package's TypeScript ends in whitespace, except where that
 whitespace is part of a template literal's text, where it is the value (a
 string or regular-expression literal cannot hold a line end, so a template's
 text is the only literal a trailing run can sit in). No configured check enforces the setting for
 TypeScript (dprint's TypeScript plugin is disabled in
 `package/config/dprint/index.json`, and the repository's stylistic oxlint
 plugin has no such rule), so an editor that honours it strips the lines it
 touches and leaves whitespace-only noise in every diff; before B119 the
 package held 16,856 such lines in 1,316 files, nearly all of them TSDoc
 blank lines written as a lone space.

 WHAT THE SCAN READS. Every TypeScript file under `src`, tests included,
 line by line: a line ending in anything `String.prototype.trimEnd` removes
 (spaces, tabs, a carriage return before the newline, and the other Unicode
 whitespace) is a finding unless a template's text covers part of that run.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed;
 the package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  nodesUnder,
  parseSource,
  type SourceText,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Trailing whitespace
// Lines that end in whitespace outside every template's text, found by reading
// each line's trailing run against the offsets of the template text runs the
// parser reports.

/**
 Lines of the files given that end in whitespace outside every template's
 text, as `path:line`.

 @param files - files read, package source and tests alike

 @returns Findings, in file order and line order within a file

 @example
 ```ts
 const lines = trailingWhitespaceLines({ files: await readPackageSource(), },);
 ```
 */
function trailingWhitespaceLines({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found: string[] = [];
  for (const file of files) {
    /**
     This file's program.
     */
    const { program, } = parseSource({ file, },);
    /**
     Text runs of this file's template literals, whose whitespace is their
     value; a template's substitutions are code, and are not among them.
     */
    const templateTexts = nodesUnder({ root: program, },)
      .filter(function isTemplateText(node,): boolean {
        return node.type === 'TemplateElement';
      },);
    /**
     Offset the line being read starts at.
     */
    let lineStart = 0;
    for (const [index, line,] of file.text
      .split('\n',)
      .entries()) {
      /**
       Offsets of the line's trailing whitespace run, equal when it has none.
       */
      const runStart = lineStart + line.trimEnd().length;
      const runEnd = lineStart + line.length;
      lineStart = runEnd + 1;
      if (runStart === runEnd)
        continue;
      if (templateTexts.some(function holdsRun(text,): boolean {
        return (text.start < runEnd) && (runStart < text.end);
      },))
        continue;
      found.push(`${file.path}:${String(index + 1,)}`,);
    }
  }
  return found;
}

//endregion Trailing whitespace

await describe({
  name: 'trailing whitespace outside template text (ledger B119)',
  children: [
    it({
      name: 'FINDS a spaced TSDoc blank line, a substitution line, code after a template\'s close, a line comment, '
        + 'an ideographic space, carriage returns and an unterminated last line, and leaves whitespace inside a '
        + 'template\'s text',
      fn: async () => {
        expect(trailingWhitespaceLines({
          files: [
            {
              path: 'cat.ts',
              text: [
                '/**',
                ' A nap.',
                ' ',
                ' @returns Nothing',
                ' */',
                'export const nap = `curled ${',
                '  whiskers\t',
                '} up  ',
                'done`;  ',
                '// a sunny windowsill  ',
                'export const den = `',
                '  basket  ',
                '`;',
                '// nap\u3000',
                '',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'kitten.unit.test.ts',
              text: [
                'export const purr = 1;\r',
                'export const meow = 2;\r',
                '',
              ].join('\n',),
              isTest: true,
            },
            {
              path: 'siamese.ts',
              text: 'export const tabby = 3;  ',
              isTest: false,
            },
          ],
        },),).toEqual([
          'cat.ts:3',
          'cat.ts:7',
          'cat.ts:9',
          'cat.ts:10',
          'cat.ts:14',
          'kitten.unit.test.ts:1',
          'kitten.unit.test.ts:2',
          'siamese.ts:1',
        ],);
      },
    },),
    it({
      name: 'FINDS NO LINE ending in whitespace outside a template\'s text across the package\'s source and tests',
      fn: async () => {
        expectNoFindings({ findings: trailingWhitespaceLines({ files: await readPackageSource(), },), },);
      },
    },),
  ],
},);
