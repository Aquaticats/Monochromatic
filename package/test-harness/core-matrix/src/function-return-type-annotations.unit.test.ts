/**
 Every function declaration and named function expression in this package's library sources states its return type.

 The repository style asks for it, and `oxlint`'s `typescript/explicit-function-return-type` rule reports a function
 declaration and a function expression assigned to a variable. It reports neither a named function expression passed
 as a call argument nor a declaration nested inside a function body, so an inferred callback such as
 `combinations.filter(function isNotExcluded(combination,) {` passes every lint task while its contract changes with
 the surrounding call (issue #1420). The scan below reads this package's own sources and fails on one.

 Comments are blanked before the scan, because the TSDoc examples in `types.ts` show callback-shaped code a reader must
 not mistake for a declaration the rule covers.

 @module
 */

import { stripSourceComments, } from '@monochromatic-dev/container-limit-core/ts/source-text-without-comments.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Suffix of a source file the rule covers; test and benchmark files carry the same rule through their own override
 being off, so they are left out here.
 */
const LIBRARY_SOURCE_SUFFIX = '.ts';

/**
 Suffixes that mark a test, benchmark or declaration file rather than library code.
 */
const NON_LIBRARY_SOURCE_SUFFIXES: readonly string[] = [
  '.bench.ts',
  '.d.ts',
  '.test.ts',
];

/**
 A `function` keyword followed by a name, an optional type parameter list and the opening parenthesis of the
 parameter list. The type parameter list is matched without parentheses inside it, which is every generic function
 this package declares.
 */
const NAMED_FUNCTION_PATTERN = /\bfunction\s+(?<name>[A-Za-z_$][\w$]*)\s*(?:<[^()]*>\s*)?\(/gu;

/**
 One declaration whose return type is inferred rather than stated.
 */
type MissingReturnTypeSite = {
  /** 1-based line of the `function` keyword in the scanned text */
  readonly line: number;
  /** Name the source gives the function */
  readonly name: string;
};

/**
 Gives the path of the parenthesis that closes the one at `openIndex`.

 Parentheses inside a string literal or a regular expression held in a parameter's default value would be counted as
 nesting; no declaration in this package carries one.

 @param text - source text whose parentheses are counted

 @param openIndex - index of the opening parenthesis

 @returns index of the matching closing parenthesis

 @throws Error when the text ends before the parenthesis closes
 */
function closingParenthesisIndex({
  text,
  openIndex,
}: {
  readonly text: string;
  readonly openIndex: number;
},): number {
  /**
   Unclosed parentheses seen so far, this one included.
   */
  let depth = 0;

  for (let index = openIndex; index < text.length; index += 1) {
    if (text[index] === '(')
      depth += 1;
    else if (text[index] === ')') {
      depth -= 1;
      if (depth === 0)
        return index;
    }
  }

  throw new Error(`Unbalanced parentheses from index ${String(openIndex,)} on`,);
}

/**
 Lists the declarations in one source text whose return type is inferred.

 @param text - source text, comments already blanked

 @returns one entry per declaration that states no return type

 @example
 ```ts
 missingReturnTypeSites('[1].map(function doubled(value,) { return value * 2; },);');
 // [{ line: 1, name: 'doubled' }]
 ```
 */
function missingReturnTypeSites(text: string,): readonly MissingReturnTypeSite[] {
  /**
   Declarations found so far, in source order.
   */
  const sites: MissingReturnTypeSite[] = [];

  for (const match of text.matchAll(NAMED_FUNCTION_PATTERN,)) {
    /**
     Index of the opening parenthesis of the parameter list this match ends on.
     */
    const openIndex = (match.index
      + match[0].length)
      - 1;
    /**
     Source text right after the parameter list, where a declaration states its return type.
     */
    const afterParameters = text.slice(closingParenthesisIndex({
      text,
      openIndex,
    },)
    + 1,)
      .trimStart();

    if (afterParameters.startsWith(':',))
      continue;

    sites.push({
      line: text.slice(
        0,
        match.index,
      )
        .split('\n',)
        .length,
      name: match.groups?.['name'] ?? '',
    },);
  }

  return sites;
}

/**
 Lists this package's library source files, each with the text the scan reads.

 @returns file paths and their comment-blanked text, in directory order
 */
async function librarySources(): Promise<readonly { readonly path: string; readonly text: string; }[]> {
  /**
   Every file below this test's own directory, tests and declarations included so the filter below can drop them.
   */
  const entries = await readdir(
    import.meta.dirname,
    {
      recursive: true,
      withFileTypes: true,
    },
  );

  /**
   Library sources, each paired with its blanked text.
   */
  const sources: { path: string; text: string; }[] = [];

  for (const entry of entries) {
    if (!entry.isFile())
      continue;
    if (!entry.name.endsWith(LIBRARY_SOURCE_SUFFIX,))
      continue;
    if (NON_LIBRARY_SOURCE_SUFFIXES.some((suffix,) => entry.name.endsWith(suffix,),))
      continue;

    /**
     Absolute path of this entry.
     */
    const path = join(
      entry.parentPath,
      entry.name,
    );

    sources.push({
      path,
      text: stripSourceComments({
        text: await readFile(
          path,
          'utf8',
        ),
        language: 'typescript',
      },),
    },);
  }

  return sources;
}

await describe({
  name: 'function return type annotations',
  children: [
    it({
      name: 'reports a callback that states no return type',
      fn: () => {
        expect(missingReturnTypeSites('[1,].map(function doubled(value: number,) {\n  return value * 2;\n},);',),)
          .toEqual([{ line: 1, name: 'doubled', },],);
      },
    },),

    it({
      name: 'reports a declaration nested in a function body',
      fn: () => {
        expect(missingReturnTypeSites('function outer(): void {\n  function inner() {}\n}',),)
          .toEqual([{ line: 2, name: 'inner', },],);
      },
    },),

    it({
      name: 'accepts a callback that states its return type',
      fn: () => {
        expect(missingReturnTypeSites('[1,].map(function doubled(value: number,): number {\n  return value * 2;\n},);',),)
          .toEqual([],);
      },
    },),

    it({
      name: 'accepts a generic declaration that states its return type',
      fn: () => {
        expect(missingReturnTypeSites('function testMatcher<T,>({ value }: { value: T; },): boolean { return true; }',),)
          .toEqual([],);
      },
    },),

    it({
      name: 'ignores a callback shape shown inside a TSDoc example',
      fn: () => {
        /**
         An example-only callback, as `types.ts` writes one inside a fenced code block.
         */
        const example = '/**\n ```ts\n { user: function notRoot(user) { return user !== \'root\'; } }\n ```\n */\n';

        expect(missingReturnTypeSites(stripSourceComments({
          text: example,
          language: 'typescript',
        },),),)
          .toEqual([],);
      },
    },),

    it({
      name: 'states the return type of every function in this package\'s library sources',
      fn: async () => {
        /**
         Library sources this scan reads.
         */
        const sources = await librarySources();

        expect(sources.length,).toBeGreaterThan(0,);

        /**
         Every declaration that states no return type, named by file and line.
         */
        const reported = sources.flatMap(({ path, text, },) => missingReturnTypeSites(text,)
          .map((site,) => `${path}:${String(site.line,)}: ${site.name}`,),);

        expect(reported,).toEqual([],);
      },
    },),
  ],
},);
