/**
 Tests for `normalizeTableHeaderKeyQuoting`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  normalizeTableHeaderKeyQuoting,
  TomlEditError,
} from '@monochromatic-dev/toml-edit-core';

await describe({
  name: '',
  children: [
    describe({
      name: normalizeTableHeaderKeyQuoting.name,
      children: [
        //region Quote style rewriting

        it({
          name: 'rewrites a single-quoted header key as a basic string',
          fn: async () => {
            /**
             Task header in the literal-quoted spelling, plus a literal string value.
             */
            const source = "[tasks.'lint:types']\nextends = 'lint:types'\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[tasks."lint:types"]\nextends = \'lint:types\'\n',
            );
          },
        },),

        it({
          name: 'rewrites every segment of a dotted sub-table header',
          fn: async () => {
            /**
             Sub-table header whose middle segment is literal-quoted.
             */
            const source = "[tasks.'lint'.env]\nA = '1'\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[tasks."lint".env]\nA = \'1\'\n',
            );
          },
        },),

        it({
          name: 'rewrites an array-of-tables header',
          fn: async () => {
            /**
             Array-of-tables header in the literal-quoted spelling.
             */
            const source = "[[tasks.'x']]\nrun = 'node'\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[[tasks."x"]]\nrun = \'node\'\n',
            );
          },
        },),

        it({
          name: 'keeps a dotted quoted name one key segment',
          fn: async () => {
            /**
             Dots inside the quotes stay part of the single key segment.
             */
            const source = "[tasks.'a.b']\nrun = 'node'\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[tasks."a.b"]\nrun = \'node\'\n',
            );
          },
        },),

        //endregion Quote style rewriting

        //region Escaping

        it({
          name: 'escapes a double quote from the literal key',
          fn: async () => {
            /**
             Literal string holding a bare double quote, which must be escaped.
             */
            const source = '[x.\'a"b\']\nrun = 1\n';
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[x."a\\"b"]\nrun = 1\n',
            );
          },
        },),

        it({
          name: 'escapes a backslash from the literal key',
          fn: async () => {
            /**
             Literal string holding a raw backslash, which a basic string escapes.
             */
            const source = "[x.'a\\b']\nrun = 1\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[x."a\\\\b"]\nrun = 1\n',
            );
          },
        },),

        //endregion Escaping

        //region Preserved bytes

        it({
          name: 'leaves bare and basic-quoted headers byte-identical',
          fn: async () => {
            /**
             Input that is already canonical in every accepted spelling.
             */
            const source = '[tasks.build]\nrun = "node"\n\n[tasks."lint:types"]\nrun = "tsc"\n';
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(source,);
          },
        },),

        it({
          name: 'leaves key-value keys untouched',
          fn: async () => {
            /**
             A literal-quoted key outside any header must not be rewritten.
             */
            const source = "'foo' = 1\n\n[x]\n'bar' = 2\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(source,);
          },
        },),

        it({
          name: 'preserves a trailing comment with an apostrophe',
          fn: async () => {
            /**
             Apostrophe after the header must not be mistaken for a key quote.
             */
            const source = "[tasks.'x'] # it's here\nrun = 'node'\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[tasks."x"] # it\'s here\nrun = \'node\'\n',
            );
          },
        },),

        it({
          name: 'preserves CRLF line endings',
          fn: async () => {
            /**
             Byte-preserving contract: only the key segment changes.
             */
            const source = "[tasks.'x']\r\nrun = 'node'\r\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[tasks."x"]\r\nrun = \'node\'\r\n',
            );
          },
        },),

        it({
          name: 'rewrites every header of a mixed document in order',
          fn: async () => {
            /**
             Mixed spellings that must all normalize without moving lines.
             */
            const source = "[tasks.'a']\nrun = 1\n\n[tasks.build]\nrun = 2\n\n[tasks.'c']\nrun = 3\n";
            expect(normalizeTableHeaderKeyQuoting({ source, },),).toBe(
              '[tasks."a"]\nrun = 1\n\n[tasks.build]\nrun = 2\n\n[tasks."c"]\nrun = 3\n',
            );
          },
        },),

        //endregion Preserved bytes

        //region Edge cases

        it({
          name: 'is idempotent',
          fn: async () => {
            /**
             Already-normalized output must pass through unchanged.
             */
            const source = "[tasks.'x']\nrun = 1\n";
            const once = normalizeTableHeaderKeyQuoting({ source, },);
            expect(normalizeTableHeaderKeyQuoting({ source: once, },),).toBe(once,);
          },
        },),

        it({
          name: 'returns an empty document unchanged',
          fn: async () => {
            expect(normalizeTableHeaderKeyQuoting({ source: '', },),).toBe('',);
          },
        },),

        it({
          name: 'rejects invalid TOML',
          fn: async () => {
            /**
             Caught error from the parse layer.
             */
            let caught: unknown;
            try {
              normalizeTableHeaderKeyQuoting({ source: "[tasks.'unterminated\n", },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(TomlEditError,);
          },
        },),

        //endregion Edge cases
      ],
    },),
  ],
},);
