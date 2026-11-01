/**
 Tests for the reading of unpositioned runs: which marker shapes beside an
 autolink literal micromark did not tokenize are references, and which are
 the literal's own URL or its parent's markup (ledger B123). Each case
 drives the footnote relabel of the built package, whose rewrite shows the
 exact spans read. Fixtures are cat-themed invention.

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
import { applyFootnoteRelabel, } from '../dist/final/node/index.mjs';

/**
 Whole text of the refusal for a map naming a label no active marker of the
 page carries.
 */
const MISSING_SOURCE_REFUSAL = 'FootnoteRewriteError: footnote rewrite: a changing map identifier is absent from the '
  + 'current active document; rebuild correspondence before retrying';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'unpositioned runs under the footnote relabel',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS a marker shape glued to an untokenized autolink literal byte-identical and REFUSES a '
            + 'map that names it, since the parse reads its opening as the link URL',
          fn: async () => {
            /**
             The transform takes `www.example.com[^9` for the link and its
             URL, leaving only the closing bracket as text.
             */
            const text = 'A cat[^1] naps ，www.example.com[^9] tail.\n\n[^1]: The cat.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '1', to: '2', },],
            },),).toBe('A cat[^2] naps ，www.example.com[^9] tail.\n\n[^2]: The cat.\n',);
            const refusal = caught(function relabelsTheUrl(): unknown {
              return applyFootnoteRelabel({
                text,
                map: [{ from: '1', to: '2', }, { from: '9', to: '10', },],
              },);
            },);
            expect(refusal,).toBeInstanceOf(FootnoteRewriteError,);
            expect(String(refusal,),).toBe(MISSING_SOURCE_REFUSAL,);
          },
        },),
        it({
          name: 'REFUSES a map naming a marker shape that sits whole inside the untokenized literal, or '
            + 'whose label holds one, a URL or an email address, since a link stands in or between its brackets',
          fn: async () => {
            /**
             Pages whose only marker shape the parse splits: the first lies
             inside the link's URL, the other two have the link between
             their brackets.
             */
            const pages = [
              { text: 'A cat naps ，www.example.com/[^9]x tail.\n', from: '9', },
              { text: 'A cat [^www.example.com] naps.\n', from: 'www.example.com', },
              { text: 'A cat [^paw@cat.example] naps.\n', from: 'paw@cat.example', },
            ];
            for (const page of pages) {
              const refusal = caught(function relabelsLinkText(): unknown {
                return applyFootnoteRelabel({
                  text: page.text,
                  map: [{ from: page.from, to: 'paw', },],
                },);
              },);
              expect(refusal,).toBeInstanceOf(FootnoteRewriteError,);
              expect(String(refusal,),).toBe(MISSING_SOURCE_REFUSAL,);
            }
          },
        },),
        it({
          name: 'RELABELS a marker in plain text and KEEPS the same shape inside the literal URL after it, '
            + 'where the URL text also stands earlier unlinked',
          fn: async () => {
            /**
             `awww.example.com[^9]` is plain text, a letter before `www`
             making it no literal, and the second `www.example.com[^9` is the
             link: the first marker is a reference and the second is URL.
             */
            const text = 'awww.example.com[^9] ,www.example.com[^9] tail.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '9', to: '10', },],
            },),).toBe('awww.example.com[^10] ,www.example.com[^9] tail.\n',);
          },
        },),
        it({
          name: 'KEEPS a JSX attribute shaped like a marker byte-identical where the element text is an '
            + 'untokenized literal, a quoted value and an expression value alike',
          fn: async () => {
            /**
             Opening tags whose attribute holds the page's own label, the
             element's text one run the tag bounds at its head.
             */
            const tags = [
              '<Cat title="[^1]">',
              '<Cat note={"[^1]"}>',
            ];
            for (const tag of tags) {
              expect(applyFootnoteRelabel({
                text: `A cat[^1] naps ${tag}，www.example.com</Cat> here.\n\n[^1]: The cat.\n`,
                map: [{ from: '1', to: '2', },],
              },),).toBe(`A cat[^2] naps ${tag}，www.example.com</Cat> here.\n\n[^2]: The cat.\n`,);
            }
          },
        },),
        it({
          name: 'RELABELS a reference in the text of a JSX element and KEEPS the same shape in its attribute, '
            + 'the run bounded after the last attribute',
          fn: async () => {
            const text = 'A cat <Cat title="[^9]">，www.example.com [^9]</Cat> naps.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '9', to: '10', },],
            },),).toBe('A cat <Cat title="[^9]">，www.example.com [^10]</Cat> naps.\n',);
          },
        },),
        it({
          name: 'RELABELS the written marker and KEEPS a marker shape spelled with character references '
            + 'byte-identical: decimal, hexadecimal, named, zero-padded, inside the literal URL, after an '
            + 'escaped backslash, and beside spellings micromark does not decode',
          fn: async () => {
            /**
             Pages whose decoded text holds brackets the raw spells as
             references, so the raw and decoded brackets pair only once each
             reference is counted; the last page holds only spellings that
             stay text (an escaped ampersand, a digit too many in each base,
             no semicolon, an ampersand that is itself a reference).
             */
            const texts = [
              '，www.example.com &#91;^9&#93; [^8]\n',
              '，www.example.com &#x5B;^9&#X5d; [^8]\n',
              '，www.example.com &lbrack;^9&rsqb; &lsqb;^9&rbrack; [^8]\n',
              '，www.example.com &#0000091;^9&#x00005d; [^8]\n',
              '，www.example.com/&#91;^9&#93;x [^8]\n',
              '，www.example.com [^9&#93; &#91;^9] [^8]\n',
              '，www.example.com \\\\&#91;^9] \\\\[^8]\n',
              '，www.example.com \\&#91; &#00000091; &#x000005b; &#91 &amp;lbrack; [^8]\n',
            ];
            for (const text of texts) {
              expect(applyFootnoteRelabel({
                text,
                map: [{ from: '8', to: '7', },],
              },),).toBe(text.replaceAll('[^8]', '[^7]',),);
            }
          },
        },),
        it({
          name: 'REFUSES a map naming a marker shape that only character references spell, since the raw '
            + 'holds no marker there',
          fn: async () => {
            const refusal = caught(function relabelsTheSpelling(): unknown {
              return applyFootnoteRelabel({
                text: '，www.example.com &#91;^9&#93; [^8]\n',
                map: [{ from: '9', to: '10', },],
              },);
            },);
            expect(refusal,).toBeInstanceOf(FootnoteRewriteError,);
            expect(String(refusal,),).toBe(MISSING_SOURCE_REFUSAL,);
          },
        },),
        it({
          name: 'RELABELS a call touching an untokenized literal and a call after a second one, each run '
            + 'bounded by the calls beside it',
          fn: async () => {
            /**
             Two runs in one paragraph: the first opens it and ends at a
             call glued to its literal, the second sits between the two calls.
             */
            const text = '，www.example.com[^1] ，www.example.com [^1].\n\n[^1]: The cat.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '1', to: '2', },],
            },),).toBe('，www.example.com[^2] ，www.example.com [^2].\n\n[^2]: The cat.\n',);
          },
        },),
        it({
          name: 'RELABELS an undefined reference beside an untokenized literal inside an emphasis, a strong, a '
            + 'strikethrough, a heading, a list item, a table cell, a block quote, a definition body and a JSX '
            + 'element that has no attribute',
          fn: async () => {
            /**
             Pages whose run sits against its parent's own markup or runs
             over a continuation line, each reference set off by a space.
             */
            const texts = [
              'A cat *paws ，www.example.com [^9]* naps.\n',
              '**，www.example.com [^9]** naps.\n',
              'A cat <Cat>，www.example.com [^9]</Cat> naps.\n',
              '~~，www.example.com [^9]~~\n',
              '# Naps ，www.example.com [^9] #\n',
              '- paws ，www.example.com [^9]\n  more [^9] ，www.example.com\n',
              '| paws | naps |\n| - | - |\n| ，www.example.com [^9] | [^9] ，www.example.com |\n',
              '> paws ，www.example.com [^9]\n> more [^9] ，www.example.com\n',
              'A cat[^1].\n\n[^1]: ，www.example.com [^9]\n',
            ];
            for (const text of texts) {
              expect(applyFootnoteRelabel({
                text,
                map: [{ from: '9', to: '10', },],
              },),).toBe(text.replaceAll('[^9]', '[^10]',),);
            }
          },
        },),
        it({
          name: 'RELABELS calls, references in positioned text and references in two runs each at its own '
            + 'offset, the labels differing in width and two of them used twice',
          fn: async () => {
            /**
             One paragraph interleaving the three sources of a marker, with
             an inline code span whose marker shape is no marker.
             */
            const text = '[^paw] ，www.example.com [^1] *tail [^猫]* ，www.example.com [^paw] [^1] `[^9]`\n\n'
              + '[^1]: The cat.\n';
            const expected = '[^whisker] ，www.example.com [^10] *tail [^cat]* ，www.example.com [^whisker] [^10] '
              + '`[^9]`\n\n[^10]: The cat.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: 'paw', to: 'whisker', }, { from: '1', to: '10', }, { from: '猫', to: 'cat', },],
            },),).toBe(expected,);
          },
        },),
        it({
          name: 'RELABELS a reference in a link label beside an untokenized literal and KEEPS the same shape '
            + 'in the link destination',
          fn: async () => {
            const text = 'A cat ，www.example.com [paws [^9] nap](https://cat.example/[^9]) naps.\n';
            expect(applyFootnoteRelabel({
              text,
              map: [{ from: '9', to: '10', },],
            },),).toBe('A cat ，www.example.com [paws [^10] nap](https://cat.example/[^9]) naps.\n',);
          },
        },),
      ],
    },),
  ],
},);
