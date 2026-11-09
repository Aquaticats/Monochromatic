/**
 Equivalence tests for `skipWithClauseWhitespace`.
 
 Capture the pre-refactor behavior of the with-clause whitespace skipper
 so the linear-pass rewrite stays behavior-identical: empty input, an
 all-whitespace run, an immediate non-whitespace char (no match), a
 cursor at or past `s.length`, each of the four permitted whitespace
 chars, a char outside that set (form feed, which `\s` would match but
 this scanner must not), a mid-string scan after a quoted path, trailing
 whitespace with no following token, and a long repeated run that would
 overflow the prior recursive scan's stack.
 
 The generic delimiter edge cases (unmatched delimiter, both path
 separators) do not apply: this function scans a whitespace prefix and
 has no delimiter or path concept.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  importAttributesOf,
  stringLiteralSourceOf,
} from './test-support.ts';
import {
  ATTR_QUERY_KEY,
  collectStaticReplacements,
  skipWithClauseWhitespace,
} from '../dist/final/node/index.mjs';

/** Whitespace-run length for the long-run case; large enough to overflow the prior recursive scan, fast to compare. */
const LONG_RUN = 100_000;

await describe({
  name: '',
  children: [
    describe({
      name: skipWithClauseWhitespace.name,
      children: [
        it({
          name: 'returns the starting index for empty input',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '', idx: 0, },),).toBe(0,);
          },
        },),

        it({
          name: 'skips an all-whitespace run to end of string',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '    ', idx: 0, },),).toBe(4,);
          },
        },),

        it({
          name: 'returns the starting index when the cursor is already on a non-whitespace char',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'with', idx: 0, },),).toBe(0,);
          },
        },),

        it({
          name: 'returns the index unchanged when the cursor equals s.length',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'ab', idx: 2, },),).toBe(2,);
          },
        },),

        it({
          name: 'returns the index unchanged when the cursor is past s.length',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'ab', idx: 5, },),).toBe(5,);
          },
        },),

        it({
          name: 'advances past a single space',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: ' x', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a single tab',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\tx', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a single newline',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\nx', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a single carriage return',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\rx', idx: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'advances past a mixed run of all four whitespace chars',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: ' \t\n\rwith', idx: 0, },),).toBe(4,);
          },
        },),

        it({
          name: String.raw`stops at a form feed (outside the four-char set that \s would match)`,
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\fwith', idx: 0, },),).toBe(0,);
          },
        },),

        it({
          name: 'scans the whitespace gap mid-string after a quoted path',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: '\'./x.sql\'   with', idx: 9, },),).toBe(12,);
          },
        },),

        it({
          name: 'skips trailing whitespace with no following token to end of string',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: 'with   ', idx: 4, },),).toBe(7,);
          },
        },),

        it({
          name: 'skips a long whitespace run in one linear pass without stack overflow',
          fn: async () => {
            expect(skipWithClauseWhitespace({ s: ' '.repeat(LONG_RUN,), idx: 0, },),).toBe(LONG_RUN,);
          },
        },),

        it({
          name: 'finds the first non-whitespace char after a long whitespace run',
          fn: async () => {
            expect(
              skipWithClauseWhitespace({ s: `${' '.repeat(LONG_RUN,)}with`, idx: 0, },),
            ).toBe(LONG_RUN,);
          },
        },),
      ],
    },),
    describe({
      name: collectStaticReplacements.name,
      children: [
        it({
          name: 'rewrites the specifier and elides the clause it found',
          fn: async () => {
            /**
             Source with one static import carrying an attribute clause.
             */
            const code = "import query from './schema.sql' with { type: 'text' };";
            /**
             Offset of the clause keyword, which is where the elision starts.
             */
            const clauseStart = code.indexOf(' with',);
            /**
             Offset just past the clause's closing brace, which is where the elision ends.
             */
            const clauseEnd = code.indexOf('};',) + '}'.length;

            expect(collectStaticReplacements({
              source: stringLiteralSourceOf(code,),
              attributes: importAttributesOf(code,),
              attrType: 'text',
              code,
            },),).toEqual([
              {
                start: stringLiteralSourceOf(code,).start,
                end: stringLiteralSourceOf(code,).end,
                text: `'./schema.sql?${ATTR_QUERY_KEY}=text'`,
              },
              {
                start: clauseStart,
                end: clauseEnd,
                text: '',
              },
            ],);
          },
        },),
        it({
          name: 'keeps an escaped specifier escaped while tagging it',
          fn: async () => {
            /**
             Source whose specifier spells a backslash as an escape.
             */
            const code = String.raw`import query from '.\schema.sql' with { type: 'text' };`;
            expect(collectStaticReplacements({
              source: stringLiteralSourceOf(code,),
              attributes: importAttributesOf(code,),
              attrType: 'text',
              code,
            },)[0]?.text,).toBe(String.raw`'.\schema.sql?${ATTR_QUERY_KEY}=text'`,);
          },
        },),
        it({
          name: 'elides a clause whose keyword follows a comment',
          fn: async () => {
            /**
             Source whose comment sits between the specifier and the keyword.
             */
            const code = "import query from './schema.sql' /* note */ with { type: 'text' };";
            /**
             Clause elision replacement, the second the helper returns.
             */
            const elision = collectStaticReplacements({
              source: stringLiteralSourceOf(code,),
              attributes: importAttributesOf(code,),
              attrType: 'text',
              code,
            },)[1];

            expect(elision?.start,).toBe(code.indexOf(' /* note',),);
            expect(code.slice(elision?.start, elision?.end,),).toBe(' /* note */ with { type: \'text\' }',);
          },
        },),
        it({
          name: 'elides a clause spelled with the deprecated assert keyword',
          fn: async () => {
            /**
             Source whose clause uses the keyword rolldown still accepts in place of `with`.
             */
            const code = "import query from './schema.sql' assert { type: 'text' };";
            expect(collectStaticReplacements({
              source: stringLiteralSourceOf(code,),
              attributes: importAttributesOf(code,),
              attrType: 'text',
              code,
            },),).toHaveLength(2,);
          },
        },),
        it({
          name: 'returns only the specifier replacement when the declaration carries no attributes',
          fn: async () => {
            /**
             Source with a clause but an empty attribute list, the shape a caller reaches by passing the list it read.
             */
            const code = "import query from './schema.sql' with { type: 'text' };";
            /**
             Replacements for a declaration whose attribute list is empty.
             */
            const replacements = collectStaticReplacements({
              source: stringLiteralSourceOf(code,),
              attributes: [],
              attrType: 'text',
              code,
            },);
            expect(replacements,).toHaveLength(1,);
          },
        },),
      ],
    },),
  ],
},);
