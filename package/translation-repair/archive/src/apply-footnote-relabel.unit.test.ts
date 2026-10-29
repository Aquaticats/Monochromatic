/**
 Tests for the simultaneous syntax-positioned footnote relabel rewrite.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { FootnoteRewriteError, } from '@monochromatic-dev/translation-repair-document-structure';
import { applyFootnoteRelabel, } from '@monochromatic-dev/translation-repair-page-pass';
import { documentLabels, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: applyFootnoteRelabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns text unchanged when a self-rename finds no footnote markers in it at all',
          fn: async () => {
            const text = 'Cats nap all afternoon, holding no footnote markers at all.';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: 'whisker', to: 'whisker', },],
            },),).toBe(text,);
          },
        },),
        it({
          name: 'relabels a page holding an autolink literal micromark did not tokenize beside its '
            + 'footnote, rather than refusing the whole page (ledger B123)',
          fn: async () => {
            /**
             Literal shapes micromark leaves untokenized and the autolink
             transform then rebuilds nodes for without positions
             (the probe inputs of ledger B123).
             */
            const literals = [
              '"www.example.com"',
              '[www.example.com',
              '，www.example.com。',
              ',www.example.com',
            ];
            for (const literal of literals) {
              /**
               Page whose footnote sits beside that literal.
               */
              const text = `A cat[^1] naps.\n\n[^1]: The cat.\n\n${literal}\n`;
              /**
               The same page under the one-label relabel, every byte outside
               the two markers unchanged.
               */
              const expected = `A cat[^2] naps.\n\n[^2]: The cat.\n\n${literal}\n`;
              expect(applyFootnoteRelabel({
                text,
                map: [{ from: '1', to: '2', },],
              },),).toBe(expected,);
            }
          },
        },),
        it({
          name: 'relabels a marker at each end of a run that is its whole paragraph, the run bounded by '
            + 'the paragraph at its head and at its tail',
          fn: async () => {
            /**
             Paragraph whose every child the transform rebuilt, a marker
             opening at the paragraph's first character and another closing
             at its last.
             */
            const text = '[^9]，www.example.com [^8]\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '9', to: '10', }, { from: '8', to: '7', },],
            },),).toBe('[^10]，www.example.com [^7]\n',);
          },
        },),
        it({
          name: 'relabels a reference in a run that follows a positioned call and ends its paragraph, the '
            + 'reference set off from the literal by a space',
          fn: async () => {
            /**
             One paragraph whose untokenized literal follows a call that
             keeps its positions, the run reaching the paragraph's end.
             */
            const text = 'A cat[^1] naps ，www.example.com [^9] tail.\n\n[^1]: The cat.\n';
            const expected = 'A cat[^2] naps ，www.example.com [^10] tail.\n\n[^2]: The cat.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '1', to: '2', }, { from: '9', to: '10', },],
            },),).toBe(expected,);
          },
        },),
        it({
          name: 'relabels a marker touching each bound of a run that sits between two positioned emphasis '
            + 'nodes',
          fn: async () => {
            /**
             The run opens where the first emphasis closes and closes where
             the second opens, a marker against each of those bounds.
             */
            const text = '*Paw*[^8]，www.example.com [^9]*tail* naps.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '8', to: '7', }, { from: '9', to: '10', },],
            },),).toBe('*Paw*[^7]，www.example.com [^10]*tail* naps.\n',);
          },
        },),
        it({
          name: 'relabels a marker ending where a positioned emphasis begins, the run bounded at its head '
            + 'by the paragraph and at its tail by the emphasis',
          fn: async () => {
            const text = '，www.example.com [^9]*tail* naps.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '9', to: '10', },],
            },),).toBe('，www.example.com [^10]*tail* naps.\n',);
          },
        },),
        it({
          name: 'relabels a literal-looking reference set off from an autolink literal micromark did not '
            + 'tokenize, since an undefined call stays in its text (ledger B123)',
          fn: async () => {
            /**
             Pages whose rebuilt text holds an undefined reference after the
             literal: micromark's call tokenizer refuses an identifier no
             definition names, so `[^9]` stays in the text (ledger B123).
             */
            const texts = [
              'A cat[^1] naps.\n\n[^1]: The cat.\n\n，www.example.com [^9] tail.\n',
              'A cat[^1] naps.\n\n[^1]: The cat.\n\n[www.example.com [^9]\n',
            ];
            for (const text of texts) {
              expect(applyFootnoteRelabel({
                text,
                map: [{ from: '9', to: 'x', },],
              },),).toBe(text.replaceAll('[^9]', '[^x]',),);
            }
          },
        },),
        it({
          name: 'keeps an escaped opening beside an autolink literal byte-identical while the footnote '
            + 'relabels, and reads no label off it (ledger B123)',
          fn: async () => {
            const text = 'A cat[^1] naps.\n\n[^1]: The cat.\n\n，www.example.com \\[^9\\]\n';
            const expected = 'A cat[^2] naps.\n\n[^2]: The cat.\n\n，www.example.com \\[^9\\]\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '1', to: '2', },],
            },),).toBe(expected,);
            // The relabel names only `1`, so the inventory is what shows the
            // escaped shape was read as no marker.
            expect(documentLabels({ text, },),).toEqual(['1',],);
          },
        },),
        it({
          name: 'keeps malformed openings beside an autolink literal byte-identical and reads no label off '
            + 'them, since none of them is a marker (ledger B123)',
          fn: async () => {
            const text = 'A cat[^1] naps.\n\n[^1]: The cat.\n\n，www.example.com[^] [^a b] [^x\n';
            const expected = 'A cat[^2] naps.\n\n[^2]: The cat.\n\n，www.example.com[^] [^a b] [^x\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '1', to: '2', },],
            },),).toBe(expected,);
            expect(documentLabels({ text, },),).toEqual(['1',],);
          },
        },),
        it({
          name: 'relabels a reference beside a link whose label is empty and keeps the same shape in that '
            + 'link\'s destination',
          fn: async () => {
            const text = 'A cat [](https://cat.example/[^9]) naps [^9].\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '9', to: '10', },],
            },),).toBe('A cat [](https://cat.example/[^9]) naps [^10].\n',);
          },
        },),
        it({
          name: 'refuses a map naming a marker shape inside a URL micromark tokenized as an autolink '
            + 'literal, whose text is the URL itself',
          fn: async () => {
            const refusal = caught(function relabelsTheUrl(): unknown {
              return applyFootnoteRelabel({
                text: 'A cat naps https://cat.example/[^9]x tail.\n',
                map: [{ from: '9', to: '10', },],
              },);
            },);
            expect(refusal,).toBeInstanceOf(FootnoteRewriteError,);
            expect(String(refusal,),).toBe(
              'FootnoteRewriteError: footnote rewrite: a changing map identifier is absent from the current '
                + 'active document; rebuild correspondence before retrying',
            );
          },
        },),
      ],
    },),
  ],
},);
