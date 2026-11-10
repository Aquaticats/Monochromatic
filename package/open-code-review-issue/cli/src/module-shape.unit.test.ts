/**
 Guards on the iteration shape two modules of this package depend on, which no
 lint rule enforces.

 `.agents/skills/project-code-review/SKILL.md` prefers functional iteration,
 then `for...of`, over a classic `for` loop, and allows the classic form only
 beside a comment stating why the preferred form is not feasible. No linter
 reads that comment, so the modules the standard flagged are read here and
 checked directly. Keeping a counted loop on purpose means updating this guard,
 which is the point.

 Only these modules are held: the package's other counted loops (`create-with-retry.ts`
 walks its attempt budget and `reconcile.ts` walks an Issue-number range, each
 using the counter itself) are a sweep of their own, not a character scan.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 Names of the modules whose iteration shape this guard holds.
 */
const GUARDED_MODULE_FILE_NAMES: readonly string[] = [
  'github-version.ts',
  'repository.ts',
];

/**
 Heads opening a counted loop with a mutable counter binding.
 */
const COUNTED_LOOP_HEADS: readonly string[] = [
  'for (let ',
  'for (var ',
];

/**
 Opening of a block comment, whose text is prose rather than a loop the module runs.
 */
const BLOCK_COMMENT_OPENER = '/*';

/**
 Closing of a block comment.
 */
const BLOCK_COMMENT_CLOSER = '*/';

/**
 Source line opening a counted loop, used to prove the scan reports one.
 */
const SAMPLE_COUNTED_LOOP_LINE = 'for (let index = 0; index < length; index += 1) {';

/**
 Source text the scan must report exactly one loop head for.
 */
const SAMPLE_SOURCE = [
  SAMPLE_COUNTED_LOOP_LINE,
  '}',
].join('\n',);

/**
 Source text stating why the loop beside it keeps the index form.
 */
const SAMPLE_JUSTIFICATION = '/* index keeps UTF-16 unit semantics */';

/**
 Source text holding a loop the scan must report after a leading block comment.
 */
const COMMENT_PREFIXED_SOURCE = [
  `${SAMPLE_JUSTIFICATION} ${SAMPLE_COUNTED_LOOP_LINE}`,
  '}',
].join('\n',);

/**
 Source text quoting a counted loop in prose and in a string, and running none.
 */
const QUOTED_SOURCE = [
  `// ${SAMPLE_COUNTED_LOOP_LINE}`,
  '/**',
  ` * ${SAMPLE_COUNTED_LOOP_LINE}`,
  ' */',
  `const note = '${SAMPLE_COUNTED_LOOP_LINE}';`,
].join('\n',);

//endregion

//region Helpers

/**
 Reports the source text a line starts its statement with, dropping a leading
 block comment.

 A head that does not open a statement is quoted prose, so requiring the
 statement start keeps both comment text and string literals out of the report.
 A block comment continuing from an earlier line without a `*` prefix would be
 read as code, which reports a loop the module does not run: a visible false
 report, never a silent miss.

 @param line - One source line.

 @returns Text from the first code token on the line, or the empty string when the line only opens a comment.
 */
function statementText(line: string,): string {
  /**
   Line without its indentation.
   */
  const trimmed = line.trimStart();
  if (!trimmed.startsWith(BLOCK_COMMENT_OPENER,)) {
    return trimmed;
  }
  /**
   Offset of the block comment closer on this line, or -1 when it continues below.
   */
  const closerIndex = trimmed.indexOf(BLOCK_COMMENT_CLOSER,);
  if (closerIndex === (-1)) {
    return '';
  }
  return trimmed
    .slice(closerIndex + BLOCK_COMMENT_CLOSER.length,)
    .trimStart();
}

/**
 Reports the counted loop heads a module's source text runs.

 @param source - One module's text.

 @returns Statement lines holding a counted loop head, in source order.
 */
function countedLoopHeads(source: string,): readonly string[] {
  return source.split('\n',)
    .map(statementText,)
    .filter(function holdsCountedLoop(statement,): boolean {
      return COUNTED_LOOP_HEADS.some(function namesHead(head,): boolean {
        return statement.startsWith(head,);
      },);
    });
}

//endregion

//region Guard

await describe({
  name: 'guarded module iteration shape',
  children: [
    it({
      name: 'reports a counted loop the source runs',
      fn: async () => {
        expect(countedLoopHeads(SAMPLE_SOURCE,),).toStrictEqual([SAMPLE_COUNTED_LOOP_LINE,],);
      },
    },),
    it({
      name: 'reports a counted loop a justification comment precedes',
      fn: async () => {
        expect(countedLoopHeads(COMMENT_PREFIXED_SOURCE,),)
          .toStrictEqual([SAMPLE_COUNTED_LOOP_LINE,],);
      },
    },),
    it({
      name: 'ignores a counted loop quoted in a comment or a string',
      fn: async () => {
        expect(countedLoopHeads(QUOTED_SOURCE,),).toStrictEqual([],);
      },
    },),
    it({
      name: 'holds every guarded module on functional iteration or for...of',
      fn: async () => {
        /**
         Counted loops the guarded modules still run, named by module.
         */
        const offenders = (await Promise.all(GUARDED_MODULE_FILE_NAMES.map(
          async function inspect(fileName,): Promise<{
            readonly fileName: string;
            readonly heads: readonly string[];
          }> {
            /**
             Guarded module's source text.
             */
            const source = await readFile(join(import.meta.dirname, fileName,), 'utf8',);
            return { fileName, heads: countedLoopHeads(source,), };
          },
        ),)).filter(function ranCountedLoop(module,): boolean {
          return module.heads.length > 0;
        },);
        expect(offenders,).toStrictEqual([],);
      },
    },),
  ],
},);

//endregion
