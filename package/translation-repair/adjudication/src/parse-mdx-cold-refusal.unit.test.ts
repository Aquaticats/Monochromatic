/**
 Tests that a body the parser cannot read for want of stack is refused with the grammar's own refusal even when that
 refusal is the first one the process builds.

 A refusal built with the stack almost spent needs its code compiled, and V8 compiles a function when it is first called,
 so the first refusal of a process is the one that can turn into a raw `RangeError`. The cases here run the stack-edge
 search of `stack-edge.test-fixture.ts` in a fresh child process per grammar, where no earlier case has built a refusal,
 which is the condition under which `parse-mdx.unit.test.ts` failed when it ran first in its process.

 Fixture wording is cat-themed invention, so no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltInChild, } from './corpus-run/built-child.test-fixture.ts';

/**
 Where the stack-edge search is, by URL, so a child process can import it.
 */
const STACK_EDGE_URL = import.meta.resolve('@monochromatic-dev/translation-repair-test-fixture/ts/stack-edge.test-fixture.ts',);

/**
 How many times a child runs the search, the first of them in a process that has built no refusal.
 */
const ATTEMPTS = 5;

/**
 What the stack-edge search finds for one grammar in a fresh process, attempt after attempt.

 @param parser - name of the built function that parses a body
 @param refusal - name of the built class its refusal is
 @returns For each attempt, whether the search found the grammar's refusal, overflowed in its own recursion before it reached the parser, or found another error
 @example
 ```ts
 const kinds = await edgeKindsOf({ parser: 'parseMdxBody', refusal: 'MdxParseError', },);
 ```
 */
async function edgeKindsOf(
  {
    parser,
    refusal,
  }: {
    readonly parser: string;
    readonly refusal: string;
  },
): Promise<readonly string[]> {
  /**
   What the child wrote: the kind of each attempt, as a JSON list.
   */
  const written = await runBuiltInChild({
    packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-document-structure',),
    body: [
      `const { caughtAtStackEdge, } = await import(${JSON.stringify(STACK_EDGE_URL,)});`,
      'const body = \'>\'.repeat(200,) + \' cat\';',
      'const kinds = [];',
      `for (let attempt = 0; attempt < ${String(ATTEMPTS,)}; attempt += 1) {`,
      '  let found;',
      '  try {',
      `    found = caughtAtStackEdge({ run: function parseDeep() { return built.${parser}({ body, },); }, },);`,
      '  } catch (error) {',
      '    found = error;',
      '  }',
      String.raw`  const top = String(found?.stack ?? '').split('\n')[1] ?? '';`,
      `  if (found instanceof built.${refusal}) kinds.push('refusal');`,
      '  else if ((found instanceof RangeError) && top.includes(\'callFromDepth\')) kinds.push(\'the search overflowed itself\');',
      '  else if (found instanceof RangeError) kinds.push(\'a raw RangeError out of the parser\');',
      '  else kinds.push(\'another error\');',
      '}',
      'process.stdout.write(JSON.stringify(kinds));',
    ].join('\n',),
  },);
  return JSON.parse(written,) as readonly string[];
}

/**
 The attempts of a search that went wrong in the parser's refusal: anything but the refusal or the search's own overflow.

 @param kinds - what each attempt found
 @returns Attempts that found something else
 @example
 ```ts
 const wrong = wrongIn({ kinds: ['refusal', 'a raw RangeError out of the parser',], },); // ['a raw RangeError out of the parser']
 ```
 */
function wrongIn({ kinds, }: { readonly kinds: readonly string[]; },): readonly string[] {
  return kinds.filter(function isWrong(kind,): boolean {
    return (kind !== 'refusal') && (kind !== 'the search overflowed itself');
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'refusal at the edge of the stack, first in its process',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A BODY THAT EXHAUSTS THE STACK WITH THE STRICT GRAMMAR\'S REFUSAL EVEN AS THE FIRST REFUSAL OF A '
            + 'PROCESS, never a raw RangeError',
          fn: async () => {
            const kinds = await edgeKindsOf({ parser: 'parseMdxBody', refusal: 'MdxParseError', },);
            expect(wrongIn({ kinds, },),).toEqual([],);
            expect(kinds.filter(function isRefusal(kind,): boolean {
              return kind === 'refusal';
            },).length,).toBeGreaterThanOrEqual(ATTEMPTS - 2,);
          },
        },),
        it({
          name: 'REFUSES A BODY THAT EXHAUSTS THE STACK WITH THE PLAIN GRAMMAR\'S REFUSAL EVEN AS THE FIRST REFUSAL OF A '
            + 'PROCESS, never a raw RangeError',
          fn: async () => {
            const kinds = await edgeKindsOf({ parser: 'parseMarkdownBody', refusal: 'MarkdownParseError', },);
            expect(wrongIn({ kinds, },),).toEqual([],);
            expect(kinds.filter(function isRefusal(kind,): boolean {
              return kind === 'refusal';
            },).length,).toBeGreaterThanOrEqual(ATTEMPTS - 2,);
          },
        },),
      ],
    },),
  ],
},);
