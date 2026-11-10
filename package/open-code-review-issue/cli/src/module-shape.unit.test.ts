/**
 Guards on this package's iteration shape, which no lint rule enforces.

 `.agents/skills/project-code-review/SKILL.md` prefers functional iteration,
 then `for...of`, over a classic `for` loop, and allows the classic form only
 beside a comment stating why the preferred form is not feasible. No linter
 reads that comment, so the two modules this rule flagged are read here and
 checked directly. Keeping a counted loop on purpose means updating this guard,
 which is the point.

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
 Prefixes of a comment line, whose text is prose rather than a loop this package runs.
 */
const COMMENT_LINE_PREFIXES: readonly string[] = [
  '*',
  '//',
  '/*',
];

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
 Source text quoting a counted loop inside a block comment and running none.
 */
const COMMENT_ONLY_SOURCE = [
  '/**',
  ` * ${SAMPLE_COUNTED_LOOP_LINE}`,
  ' */',
  'return value;',
].join('\n',);

//endregion

//region Helpers

/**
 Reports the counted loop heads a module's source text runs.

 @param source - One module's text.

 @returns Loop head lines the module executes, in source order.
 */
function countedLoopHeads(source: string,): readonly string[] {
  return source.split('\n',)
    .filter(function isNotComment(line,): boolean {
      /**
       Line without its indentation.
       */
      const trimmed = line.trim();
      return !COMMENT_LINE_PREFIXES.some(function startsComment(prefix,): boolean {
        return trimmed.startsWith(prefix,);
      },);
    },)
    .filter(function holdsCountedLoop(line,): boolean {
      return COUNTED_LOOP_HEADS.some(function namesHead(head,): boolean {
        return line.includes(head,);
      },);
    });
}

//endregion

//region Guard

await describe({
  name: 'package iteration shape',
  children: [
    it({
      name: 'reports a counted loop the source runs',
      fn: async () => {
        expect(countedLoopHeads(SAMPLE_SOURCE,),).toStrictEqual([SAMPLE_COUNTED_LOOP_LINE,],);
      },
    },),
    it({
      name: 'ignores a counted loop quoted in a comment',
      fn: async () => {
        expect(countedLoopHeads(COMMENT_ONLY_SOURCE,),).toStrictEqual([],);
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
