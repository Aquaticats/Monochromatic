/**
 Rewriting tests for {@link transformImportAttributes}.
 
 Each case starts from source text, runs the transform, and asserts on the
 emitted text and on the parser's verdict, so a case proves both what the
 rewritten module says and that it still parses.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseSync,
} from 'rolldown/utils';

import {
  ATTR_QUERY_KEY,
  transformImportAttributes,
} from '../dist/final/node/index.mjs';

/**
 Runs the transform over `code` and returns the rewritten source.
 
 @param code - module source to transform
 
 @returns rewritten module source
 
 @throws Error when the transform reported no attribute clause to rewrite
 */
function transform(code: string,): string {
  const result = transformImportAttributes({ code, id: 'transform-case.ts', },);
  if (typeof result === 'symbol')
    throw new Error(`transform reported no attribute clause for: ${code}`,);
  return result.code;
}

/**
 Asserts that the transform produced code the parser still accepts.
 
 @param code - rewritten module source to parse
 */
function expectParses(code: string,): void {
  expect(function parses() {
    parseSync('transform-case.ts', code,);
  },).not.toThrow();
}

await describe({
  name: transformImportAttributes.name,
  children: [
    //region Specifier escaping

    it({
      name: 'keeps the escape sequences of a specifier holding a backslash',
      fn: async () => {
        /**
         Raw module text; `\f` is a form feed escape in the specifier, so the
         specifier's value is not the text read off disk.
         */
        const code = String.raw`import query from '.\file.sql' with { type: 'text' };`;
        const transformed = transform(code,);

        expect(transformed,).toContain(String.raw`'.\file.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(function parses() {
          parseSync('transform-case.ts', transformed,);
        },).not.toThrow();
      },
    },),

    it({
      name: 'keeps an escaped quote inside a specifier',
      fn: async () => {
        /**
         Specifier whose source text escapes the quote it is delimited by.
         */
        const code = String.raw`import query from './a\'b.sql' with { type: 'text' };`;
        const transformed = transform(code,);

        expect(transformed,).toContain(String.raw`'./a\'b.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(function parses() {
          parseSync('transform-case.ts', transformed,);
        },).not.toThrow();
      },
    },),

    it({
      name: 'keeps a unicode escape inside a specifier',
      fn: async () => {
        /**
         Specifier whose source text spells its first letter as an escape.
         */
        const code = String.raw`import query from '\u0061.sql' with { type: 'text' };`;
        const transformed = transform(code,);

        expect(transformed,).toContain(String.raw`'\u0061.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(function parses() {
          parseSync('transform-case.ts', transformed,);
        },).not.toThrow();
      },
    },),

    it({
      name: 'keeps an escape inside a dynamic import specifier',
      fn: async () => {
        /**
         Dynamic import whose specifier holds a form feed escape.
         */
        const code = String.raw`const loaded = import('.\file.sql', { with: { type: 'text' }, });`;
        const transformed = transform(code,);

        expect(transformed,).toContain(String.raw`'.\file.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(function parses() {
          parseSync('transform-case.ts', transformed,);
        },).not.toThrow();
      },
    },),

    //endregion Specifier escaping

    //region Fast-path gate

    it({
      name: 'rewrites a module whose only attribute syntax is a dynamic import options object',
      fn: async () => {
        /**
         Dynamic form documented by the README; the keyword is followed by a
         colon rather than by whitespace.
         */
        const code = "const loaded = await import('./sample.sql', { with: { type: 'text' }, });";
        const transformed = transform(code,);

        expect(transformed,).toContain(`'./sample.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(transformed,).not.toContain('with:',);
        expectParses(transformed,);
      },
    },),

    it({
      name: 'rewrites a static clause whose opening brace follows a newline',
      fn: async () => {
        /**
         Static clause written across two lines, which no whitespace-specific
         substring can match.
         */
        const code = "import query from './sample.sql' with\n{ type: 'text' };";
        const transformed = transform(code,);

        expect(transformed,).toContain(`'./sample.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(transformed,).not.toContain('with',);
        expectParses(transformed,);
      },
    },),

    it({
      name: 'rewrites a static clause whose opening brace follows a tab',
      fn: async () => {
        /**
         Static clause whose only gap between keyword and brace is a tab.
         */
        const code = "import query from './sample.sql' with\t{ type: 'text' };";
        const transformed = transform(code,);

        expect(transformed,).toContain(`'./sample.sql?${ATTR_QUERY_KEY}=text'`,);
        expect(transformed,).not.toContain('with',);
        expectParses(transformed,);
      },
    },),

    it({
      name: 'reports no rewrite for a module with no attribute clause',
      fn: async () => {
        /**
         Negative control: a plain import must stay untouched, so the gate
         keeps its fast path instead of rewriting every module.
         */
        const code = "import query from './sample.sql';";
        const result = transformImportAttributes({ code, id: 'transform-case.ts', },);

        expect(typeof result,).toBe('symbol',);
      },
    },),

    it({
      name: 'reports no rewrite for a module that only mentions the keyword in prose',
      fn: async () => {
        /**
         The gate reads a bare keyword as a candidate and lets the visitor
         decide, so a comment mentioning it must still end in no rewrite.
         */
        const code = `/** Loaded with care. */\nimport query from './sample.sql';`;
        const result = transformImportAttributes({ code, id: 'transform-case.ts', },);

        expect(typeof result,).toBe('symbol',);
      },
    },),

    //endregion Fast-path gate
  ],
},);
