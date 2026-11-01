/**
 Tests for HTML comment masking:
 length preservation, newline preservation, region offsets, and the
 unterminated tail case. Fixtures are cat-themed invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { maskHtmlComments, } from '../dist/final/node/index.mjs';

await describe({
  name: maskHtmlComments.name,
  children: [
    it({
      name: 'returns comment-free text untouched with no regions',
      fn: async () => {
        /** Body without any comment. */
        const text = 'The cat naps in the sun.\n\nThe cat chases butterflies.\n';
        expect(maskHtmlComments({ text, },),).toEqual({
          masked: text,
          regions: [],
        },);
      },
    },),

    it({
      name: 'masks a terminated comment to same-length whitespace',
      fn: async () => {
        /** Body with one standalone comment line between paragraphs. */
        const text = 'The cat naps.\n\n<!-- 猫猫备注 -->\n\nThe cat purrs.\n';
        /** Masked result with one region. */
        const { masked, regions, } = maskHtmlComments({ text, },);

        expect(masked.length,).toBe(text.length,);
        expect(regions,).toEqual([{
          startOffset: text.indexOf('<!--',),
          endOffset: text.indexOf('-->',) + '-->'.length,
          terminated: true,
        },],);
        // Prose outside the region survives byte-for-byte.
        expect(masked.startsWith('The cat naps.\n\n',),).toBe(true,);
        expect(masked.endsWith('\n\nThe cat purrs.\n',),).toBe(true,);
        // The region itself is whitespace only.
        expect(
          masked
            .slice(
            regions[0]?.startOffset,
            regions[0]?.endOffset,
            )
            .trim(),
        ).toBe('',);
      },
    },),

    it({
      name: 'preserves newlines inside multi-line comments',
      fn: async () => {
        /** Body whose comment spans lines around a paragraph boundary. */
        const text = 'The cat naps.\n\n<!--\n猫猫的多行备注\n-->\n\nThe cat purrs.\n';
        /** Masked result. */
        const { masked, } = maskHtmlComments({ text, },);

        expect(masked.length,).toBe(text.length,);
        // Every original newline survives at its exact offset.
        for (
          let offset = text.indexOf('\n',);
          offset !== (-1);
          offset = text.indexOf(
            '\n',
            offset + 1,
          )
        ) {
          expect(masked.charAt(offset,),).toBe('\n',);
        }
      },
    },),

    it({
      name: 'masks multiple comments each into its own region',
      fn: async () => {
        /** Body with a comment before and after the prose. */
        const text = '<!-- top -->\n\nThe cat naps happily.\n\n<!-- bottom -->\n';
        /** Masked result with two regions. */
        const { masked, regions, } = maskHtmlComments({ text, },);

        expect(regions,).toHaveLength(2,);
        expect(masked,).toContain('The cat naps happily.',);
        expect(masked,).not.toContain('top',);
        expect(masked,).not.toContain('bottom',);
      },
    },),

    it({
      name: 'masks an unterminated comment through the end of input',
      fn: async () => {
        /** Body whose final comment never closes. */
        const text = 'The cat naps.\n\n<!-- 没有结束的备注\n还在继续';
        /** Masked result with one unterminated region. */
        const { masked, regions, } = maskHtmlComments({ text, },);

        expect(masked.length,).toBe(text.length,);
        expect(regions,).toEqual([{
          startOffset: text.indexOf('<!--',),
          endOffset: text.length,
          terminated: false,
        },],);
        expect(masked,).not.toContain('备注',);
      },
    },),
  ],
},);
