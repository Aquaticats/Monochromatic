/**
 Tests for the surgical catalog-range rewriter.
 
 Pins the #258 write-back fix: the prior writer matched only double quotes and
 silently rewrote nothing on the single-quoted file. These cases lock that the
 rewriter replaces only the value token, preserving the entry's quote style,
 indentation, trailing comments, and every non-matching line.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  rewriteCatalogRanges,
} from './yaml-rewrite.ts';

await describe({
  name: 'yaml-rewrite',
  children: [
    it({
      name: 'rewrites a single-quoted entry, preserving the single quotes',
      fn: async () => {
        /** Single-quoted shape, as the real pnpm-workspace.yaml uses. */
        const content = [
          'catalog:',
          '  \'oxlint\': \'>=1.71.0\'',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'oxlint',
            oldRange: '>=1.71.0',
            newRange: '>=1.71.1',
          },],
        },),).toBe([
          'catalog:',
          '  \'oxlint\': \'>=1.71.1\'',
        ].join('\n',),);
      },
    },),

    it({
      name: 'rewrites a double-quoted entry, preserving the double quotes',
      fn: async () => {
        /** Double-quoted shape; quote style must round-trip unchanged. */
        const content = [
          'catalog:',
          '  "foo": ">=1.0.0"',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'foo',
            oldRange: '>=1.0.0',
            newRange: '>=1.1.0',
          },],
        },),).toBe([
          'catalog:',
          '  "foo": ">=1.1.0"',
        ].join('\n',),);
      },
    },),

    it({
      name: 'replaces the full value of an npm: aliased entry',
      fn: async () => {
        /** Aliased entry: the whole `npm:<target>@<range>` value is the replacement unit. */
        const content = [
          'catalog:',
          '  \'zod\': \'npm:@jsr/zod__zod@>=4.1.8\'',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'zod',
            oldRange: 'npm:@jsr/zod__zod@>=4.1.8',
            newRange: 'npm:@jsr/zod__zod@>=4.4.0',
          },],
        },),).toBe([
          'catalog:',
          '  \'zod\': \'npm:@jsr/zod__zod@>=4.4.0\'',
        ].join('\n',),);
      },
    },),

    it({
      name: 'preserves a trailing comment on the entry line',
      fn: async () => {
        /** Trailing inline comment must survive a value rewrite. */
        const content = [
          'catalog:',
          '  foo: ">=1.0.0" # keep me',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'foo',
            oldRange: '>=1.0.0',
            newRange: '>=1.1.0',
          },],
        },),).toBe([
          'catalog:',
          '  foo: ">=1.1.0" # keep me',
        ].join('\n',),);
      },
    },),

    it({
      name: 'leaves comment lines and non-matching entries untouched',
      fn: async () => {
        /** Only the `bar` entry is tightened; the comment and `foo` line pass through verbatim. */
        const content = [
          'catalog:',
          '  # a note',
          '  \'foo\': \'>=1.0.0\'',
          '  \'bar\': \'>=2.0.0\'',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'bar',
            oldRange: '>=2.0.0',
            newRange: '>=2.1.0',
          },],
        },),).toBe([
          'catalog:',
          '  # a note',
          '  \'foo\': \'>=1.0.0\'',
          '  \'bar\': \'>=2.1.0\'',
        ].join('\n',),);
      },
    },),

    it({
      name: 'rewrites only the default catalog when a named block repeats the key',
      fn: async () => {
        /** Default and named entries deliberately share both key and old value. */
        const content = [
          'catalog:',
          '  foo: ">=1.0.0"',
          'catalogs:',
          '  legacy:',
          '    foo: ">=1.0.0"',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'foo',
            oldRange: '>=1.0.0',
            newRange: '>=1.1.0',
          },],
        },),).toBe([
          'catalog:',
          '  foo: ">=1.1.0"',
          'catalogs:',
          '  legacy:',
          '    foo: ">=1.0.0"',
        ].join('\n',),);
      },
    },),

    it({
      name: 'keeps rewriting after a zero-indented comment containing a colon',
      fn: async () => {
        /** A top-level comment must not terminate the default catalog state. */
        const content = [
          'catalog:',
          '  foo: ">=1.0.0"',
          '# migration: keep pinned',
          '  bar: ">=2.0.0"',
        ].join('\n',);
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'bar',
            oldRange: '>=2.0.0',
            newRange: '>=2.1.0',
          },],
        },),).toBe([
          'catalog:',
          '  foo: ">=1.0.0"',
          '# migration: keep pinned',
          '  bar: ">=2.1.0"',
        ].join('\n',),);
      },
    },),

    it({
      name: 'leaves content without a default catalog header unchanged',
      fn: async () => {
        /** Unscoped YAML content must not be mistaken for the default catalog. */
        const content = '  foo: ">=1.0.0"';
        expect(rewriteCatalogRanges({
          content,
          results: [{
            name: 'foo',
            oldRange: '>=1.0.0',
            newRange: '>=1.1.0',
          },],
        },),).toBe(content,);
      },
    },),
  ],
},);
