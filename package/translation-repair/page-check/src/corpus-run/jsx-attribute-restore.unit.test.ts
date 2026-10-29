/**
 Guards class ninety-nine (XingZ630, 2026-09-23): a JSX or HTML tag's
 string attribute is apparatus the archive fixed, and the bench rendered a
 numeral series in it three ways on one page (`n="II"`, `n="5"`, `n="七"`)
 where the archive writes Roman numerals throughout. Where the page carries
 the same tags in the same order as the archive's slice, every attribute
 takes the archive's value; a slice whose tag sequence differs from the
 archive's is left to the judges. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { restoreJsxAttributes, } from '../../dist/final/node/index.mjs';
import { pair, } from '@monochromatic-dev/translation-repair-page-pass/ts/corpus-run/title-reference.test-fixture.ts';

/**
 Two marker slices the archive renders with Roman numerals, one heading
 with an alignment attribute.
 */
const SLICES: readonly ChunkPair[] = [
  pair({
    sliceIndex: 0,
    source: '<Paw n="五"/>\n\n猫。',
    target: '<Paw n="V"/>\n\nCat.',
  },),
  pair({
    sliceIndex: 1,
    source: '<Paw n="六"/>\n\n猫猫。',
    target: '<Paw n="VI"/>\n\nKitten.',
  },),
  pair({
    sliceIndex: 2,
    source: '<h3 align = "center">猫</h3>',
    target: '<h3 align = "center">Cat</h3>',
  },),
  pair({
    sliceIndex: 3,
    source: '<Paw n="七"/>\n\n猫。',
    target: '',
  },),
];

await describe({
  name: 'a tag attribute the archive fixed (class ninety-nine)',
  children: [
    it({
      name: 'RESTORES an attribute the bench rendered in another style to the archive\'s value',
      fn: async () => {
        /**
         Pass over a page whose bench wrote an Arabic numeral and a Han one.
         */
        const restored = restoreJsxAttributes({
          slices: SLICES,
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '<Paw n="5"/>\n\nThe cat.',
            },
            {
              sliceIndex: 1,
              replacementText: '<Paw n="六"/>\n\nThe kitten.',
            },
          ],
        },);
        expect(restored.replacements.map(function textOf(row,): string {
          return row.replacementText;
        },),).toEqual([
          '<Paw n="V"/>\n\nThe cat.',
          '<Paw n="VI"/>\n\nThe kitten.',
        ],);
        expect(restored.restored.length,).toBe(2,);
        expect(restored.findings.join('\n',),).toContain('jsx-attribute-restored (slice 0: <Paw n="5"/> to <Paw n="V"/>',);
      },
    },),
    it({
      name: 'LEAVES a slice whose tags differ from the archive\'s, an attribute already equal, and a slice the archive never carried',
      fn: async () => {
        /**
         Pass over a page whose bench dropped a tag, kept one, and wrote one
         the archive lacks.
         */
        const restored = restoreJsxAttributes({
          slices: SLICES,
          replacements: [
            {
              sliceIndex: 0,
              replacementText: 'The cat.',
            },
            {
              sliceIndex: 2,
              replacementText: '<h3 align = "center">The Cat</h3>',
            },
            {
              sliceIndex: 3,
              replacementText: '<Paw n="7"/>\n\nThe cat.',
            },
          ],
        },);
        expect(restored.restored,).toEqual([],);
        expect(restored.findings,).toEqual([],);
        expect(restored.replacements.map(function textOf(row,): string {
          return row.replacementText;
        },),).toEqual([
          'The cat.',
          '<h3 align = "center">The Cat</h3>',
          '<Paw n="7"/>\n\nThe cat.',
        ],);
      },
    },),
    it({
      name: 'LEAVES AN ATTRIBUTE THE ARCHIVE\'S TAG NAMES MORE THAN ONCE as the bench wrote it, since which of the '
        + 'archive\'s values a reader sees cannot be read, and RESTORES the tag\'s other attribute',
      fn: async () => {
        /**
         Pass over a page whose archive tag names `n` twice and `side` once.
         */
        const restored = restoreJsxAttributes({
          slices: [
            pair({
              sliceIndex: 0,
              source: '<Paw n="五" side="left"/>\n\n猫。',
              target: '<Paw n="V" n="5" side="left"/>\n\nCat.',
            },),
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '<Paw n="五" side="LEFT"/>\n\nThe cat.',
            },
          ],
        },);
        // Taking the archive's first `n`, the pass wrote "V" over the bench's value.
        expect(restored,).toEqual({
          replacements: [
            {
              sliceIndex: 0,
              replacementText: '<Paw n="五" side="left"/>\n\nThe cat.',
            },
          ],
          restored: [
            {
              sliceIndex: 0,
              replacementText: '<Paw n="五" side="left"/>\n\nThe cat.',
            },
          ],
          findings: [
            'jsx-attribute-restored (slice 0: <Paw n="五" side="LEFT"/> to <Paw n="V" n="5" side="left"/>; '
              + 'a tag attribute is apparatus the archive fixed)',
          ],
        },);
      },
    },),
  ],
},);
