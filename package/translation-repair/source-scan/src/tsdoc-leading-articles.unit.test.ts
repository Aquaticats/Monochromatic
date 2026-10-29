/**
 Guards the repository's reading of `AGENTS.md` rule TD6 for `@returns` and `@param` descriptions (ledger B1000): none
 opens with an article. The rule reads "`@param`/`@returns`: no articles; explain why, not what". Measured on
 2026-10-29 over `origin/main` outside this package, 670 of about 22,900 such description lines opened with an
 article (2.9 percent), so the repository practises the telegraph style, "Its child nodes" and "Whether it is a
 node", and this scan holds the package to the same. Before B1000 the package held 4,029 such lines in 1,159 files.

 WHAT THE SCAN READS. Every TypeScript file under `src` of a family package, tests and fixtures included, line by
 line: a line inside a TSDoc block (`/**` to `*\/`) and outside a fence in that block, whose first words are `@returns`
 or `@param name -` and then `A`, `An`, `The`, `a`, `an` or `the` and a space, is a finding. A description that opens
 with a name that merely starts with those letters (`ArtifactParseError`, `Theme`), a parameter named `a` or `the`,
 an article later in the line, and an article in a `//` comment, a string or a fenced example are not findings.

 WHAT THIS DOES NOT CHECK. The other half of TD6, "explain why, not what", is not mechanically checkable, and
 this scan does not enforce it: a green run is no proof that a description explains why, and no proof of TD6
 compliance. An article elsewhere in a description is ordinary English and is left alone.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan shown able to find each spelling after
 each tag and to leave each look-alike alone (ledger M21). Fixtures are cat-themed; the package case reads the
 source of every family package.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Leading articles
// The lines of TSDoc blocks whose `@returns` or `@param name -` description opens with an article.

/**
 The six spellings of an article a description may open with.
 */
const ARTICLES: readonly string[] = ['A', 'An', 'The', 'a', 'an', 'the',];

/**
 A TSDoc line past its margin: leading space and an optional `*`.

 @param line - one line of a TSDoc block

 @returns Line's text as Markdown reads it

 @example
 ```ts
 pastMargin({ line: '  * @returns Kitten', },); // '@returns Kitten'
 ```
 */
function pastMargin({ line, }: { readonly line: string; },): string {
  /**
   The line without its leading space.
   */
  const trimmed = line.trimStart();
  return (trimmed.startsWith('*',) ? trimmed.slice(1,) : trimmed).trimStart();
}

/**
 Description a `@returns` or `@param name -` line carries. The tag, the name and the dash are each followed by one
 space, as the package writes them.

 @param line - one line of a TSDoc block

 @returns Text after the tag, or after the dash for a parameter; empty for any other line

 @example
 ```ts
 descriptionOf({ line: ' @param cat - the kitten', },); // 'the kitten'
 ```
 */
function descriptionOf({ line, }: { readonly line: string; },): string {
  /**
   The line as Markdown reads it.
   */
  const text = pastMargin({ line, },);
  if (text.startsWith('@returns ',))
    return text.slice('@returns '.length,);
  if (!text.startsWith('@param ',))
    return '';
  /**
   What follows the tag: the parameter's name, the dash and the description.
   */
  const rest = text.slice('@param '.length,);
  /**
   Where the parameter's name ends.
   */
  const nameEnd = rest.indexOf(' ',);
  return ((nameEnd > 0) && rest.startsWith('- ', nameEnd + 1,)) ? rest.slice(nameEnd + ' - '.length,) : '';
}

/**
 Whether a description opens with an article.

 @param description - text after a tag

 @returns Whether it starts with one of the six spellings and a space

 @example
 ```ts
 opensWithArticle({ description: 'the kitten', },); // true
 ```
 */
function opensWithArticle({ description, }: { readonly description: string; },): boolean {
  return ARTICLES.some(function opensWith(article,): boolean {
    return description.startsWith(`${article} `,);
  },);
}

/**
 Whether a TSDoc line opens or closes a fenced code block.

 @param line - one line of a TSDoc block

 @returns Whether it starts, past the margin, with backticks or tildes in a run of three

 @example
 ```ts
 isFenceLine({ line: ' ```ts', },); // true
 ```
 */
function isFenceLine({ line, }: { readonly line: string; },): boolean {
  /**
   The line as Markdown reads it.
   */
  const text = pastMargin({ line, },);
  return text.startsWith('```',) || text.startsWith('~~~',);
}

/**
 Lines of the files given inside a TSDoc block and outside a fence, whose `@returns` or `@param name -` description
 opens with an article, as `path:line`.

 @param files - files read, package source and tests alike

 @returns Findings, in file order and line order within a file

 @example
 ```ts
 const lines = leadingArticleLines({ files: await readPackageSource(), },);
 ```
 */
function leadingArticleLines({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found: string[] = [];
  for (const file of files) {
    /**
     Whether the lines read last are inside a TSDoc block.
     */
    let inDoc = false;
    /**
     Whether they are inside a fence of that block.
     */
    let inFence = false;
    for (const [index, line,] of file.text
      .split('\n',)
      .entries()) {
      /**
       The line without its margin.
       */
      const trimmed = line.trim();
      if ((!inDoc) && trimmed.startsWith('/**',))
        inDoc = true;
      /**
       Whether the block closes on this line.
       */
      const closes = inDoc && trimmed.includes('*/',);
      if (inDoc && isFenceLine({ line, },))
        inFence = !inFence;
      else if (inDoc && (!inFence) && opensWithArticle({ description: descriptionOf({ line, },), },))
        found.push(`${file.path}:${String(index + 1,)}`,);
      if (closes) {
        inDoc = false;
        inFence = false;
      }
    }
  }
  return found;
}

/**
 A planted file whose TSDoc block holds the lines given, after a summary line and a blank one.

 @param lines - lines of the block between its summary and its close

 @returns File named for the cat it documents

 @example
 ```ts
 const file = plantedFile({ lines: [' @returns The kitten',], });
 ```
 */
function plantedFile({ lines, }: { readonly lines: readonly string[]; },): SourceText {
  return {
    path: 'cat.ts',
    text: [
      '/**',
      ' Purrs.',
      '',
      ...lines,
      ' */',
      'export function purr(): void {}',
      '',
    ].join('\n',),
    isTest: false,
  };
}

//endregion Leading articles

await describe({
  name: 'leading articles in TSDoc @returns and @param descriptions (ledger B1000)',
  children: [
    it({
      name: 'FINDS each of the six article spellings after @returns and after @param name -, with or without a '
        + 'margin star',
      fn: async () => {
        /**
         One `@returns` and one `@param` line for each article, the first at the fourth line of the file.
         */
        const lines = ARTICLES.flatMap(function tagged(article,): readonly string[] {
          return [` @returns ${article} kitten`, ` @param cat - ${article} kitten`,];
        },);
        expect(leadingArticleLines({ files: [plantedFile({ lines, },),], },),).toEqual(
          lines.map(function placed(_, at,): string {
            return `cat.ts:${String(at + 4,)}`;
          },),
        );
        expect(leadingArticleLines({
          files: [plantedFile({ lines: [' * @returns The kitten', '   * @param cat - a kitten',], },),],
        },),).toEqual(['cat.ts:4', 'cat.ts:5',],);
      },
    },),
    it({
      name: 'LEAVES a parameter named a or the, a description that opens with a name starting like an article, an '
        + 'article later in the line, a tag with no description and a description that opens with another word',
      fn: async () => {
        expect(leadingArticleLines({
          files: [
            plantedFile({
              lines: [
                ' @param a - left side',
                ' @param the - right side',
                ' @param an - Ancient sum',
                ' @returns ArtifactParseError when the sum fails',
                ' @returns Theme of the sum',
                ' @returns Another sum',
                ' @returns Rows the census reads',
                ' @returns Sum of a and the',
                ' @returns',
                ' @returns The',
                ' @param cat - The',
                ' @returns  The doubled space',
                ' @param cat A kitten without a dash',
              ],
            },),
          ],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES an article in a fenced example of the block, in a // comment, in a string and in a template '
        + 'outside every block, and reads on past a block that ends inside a fence',
      fn: async () => {
        expect(leadingArticleLines({
          files: [
            plantedFile({
              lines: [
                ' @example',
                ' ```ts',
                ' @returns The kitten',
                ' @param cat - the kitten',
                ' ```',
                ' ~~~',
                ' @returns A kitten',
                ' ~~~',
              ],
            },),
            {
              path: 'kitten.ts',
              text: [
                '// @returns The kitten',
                '/* @param cat - the kitten */',
                'export const note = \'@returns The kitten\';',
                'export const text = `',
                ' @returns The kitten',
                ' @param cat - a kitten',
                '`;',
                '/**',
                ' Naps.',
                ' ```ts',
                ' @returns The kitten',
                ' */',
                '/**',
                ' Sleeps.',
                '',
                ' @returns The kitten',
                ' */',
                'export function sleep(): void {}',
                '',
              ].join('\n',),
              isTest: false,
            },
          ],
        },),).toEqual(['kitten.ts:16',],);
      },
    },),
    it({
      name: 'FINDS NO @returns OR @param DESCRIPTION OPENING WITH AN ARTICLE in the family\'s source and tests (ledger '
        + 'B1000: the package held 4,029 such lines before it)',
      fn: async () => {
        expectNoFindings({ findings: leadingArticleLines({ files: await readPackageSource(), },), },);
      },
    },),
  ],
},);
