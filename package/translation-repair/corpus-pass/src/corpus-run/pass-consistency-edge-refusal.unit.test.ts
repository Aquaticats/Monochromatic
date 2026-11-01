/**
 Tests for naming why the pass left a place as it was although the record's words let code move the chosen words in: the
 splice was refused at the edge of the span, by a second mark past markup or by a mark outside the words that would read
 differently, which the record holds no English to check.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { edgeRefusalOf, } from '../../dist/final/node/index.mjs';
import { renderingsOf, } from '@monochromatic-dev/translation-repair-consistency/ts/consistency-renderings.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: edgeRefusalOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES A SECOND MARK PAST MARKUP when the chosen words carry an edge apostrophe over a place whose own mark stands '
            + 'past emphasis, which the record carries no field for',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dogs’ bowl.', quote: 'dogs’', },
              receiving: { english: 'Fill the *kittens*’ bowl.', quote: 'kittens', },
            },);
            expect(edgeRefusalOf({
              chosen,
              target,
              framed: false,
              english: 'Fill the *kittens*’ bowl.',
            },),).toBe('edge-second-mark',);
          },
        },),
        it({
          name: 'NAMES A MARK OUTSIDE THE WORDS THAT WOULD READ DIFFERENTLY, which only the English around the splice shows',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Say Come (here)\' now.', quote: 'Come (here)', },
              receiving: { english: 'He said cats 🐱\'s now.', quote: 'cats 🐱', },
            },);
            expect(edgeRefusalOf({
              chosen,
              target,
              framed: false,
              english: 'He said cats 🐱\'s now.',
            },),).toBe('edge-reading-changes',);
          },
        },),
        it({
          name: 'NAMES A QUOTATION-LIKE REFERENCE ON THE PAGE when the chosen words carry an edge apostrophe over a place whose '
            + 'page holds one, which the record holds no page to see, and the second mark first where both refuse',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dogs’ bowl.', quote: 'dogs’', },
              receiving: { english: 'Fill the cats bowl&#39;.', quote: 'cats', },
            },);
            const [marked, receiving,] = renderingsOf({
              chosen: { english: 'Fill the dogs’ bowl.', quote: 'dogs’', },
              receiving: { english: 'Fill the *kittens*’ bowl&#39;.', quote: 'kittens', },
            },);
            expect([
              edgeRefusalOf({
                chosen,
                target,
                framed: false,
                english: 'Fill the cats bowl&#39;.',
                pageReference: function pageReferences(): readonly string[] {
                  return ['&#39;',];
                },
              },),
              edgeRefusalOf({
                chosen,
                target,
                framed: false,
                english: 'Fill the cats bowl&#39;.',
                pageReference: function pageReferences(): readonly string[] {
                  return [];
                },
              },),
              edgeRefusalOf({ chosen, target, framed: false, english: 'Fill the cats bowl&#39;.', },),
              edgeRefusalOf({
                chosen: marked,
                target: receiving,
                framed: false,
                english: 'Fill the *kittens*’ bowl&#39;.',
                pageReference: function pageReferences(): readonly string[] {
                  return ['&#39;',];
                },
              },),
            ],).toEqual(['edge-reference-mark', 'unrefused', 'unrefused', 'edge-second-mark',],);
          },
        },),
        it({
          name: 'NAMES A QUOTATION-LIKE REFERENCE ON THE PAGE FOR A SPLICE THAT BRINGS NO APOSTROPHE, since every splice on such a '
            + 'page is refused, and names none for a splice that changes nothing',
          fn: async () => {
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dogs bowl.', quote: 'dogs', },
              receiving: { english: 'Fill the cats bowl.', quote: 'cats', },
            },);
            const [same, alike,] = renderingsOf({
              chosen: { english: 'Fill the dogs bowl.', quote: 'dogs', },
              receiving: { english: 'Fill the dogs bowl.', quote: 'dogs', },
            },);
            expect([
              edgeRefusalOf({
                chosen,
                target,
                framed: false,
                english: 'Fill the cats bowl.',
                pageReference: function pageReferences(): readonly string[] {
                  return ['&#39;',];
                },
              },),
              edgeRefusalOf({
                chosen,
                target,
                framed: false,
                english: 'Fill the cats bowl.',
                pageReference: function pageReferences(): readonly string[] {
                  return [];
                },
              },),
              edgeRefusalOf({
                chosen: same,
                target: alike,
                framed: false,
                english: 'Fill the dogs bowl.',
                pageReference: function pageReferences(): readonly string[] {
                  return ['&#39;',];
                },
              },),
            ],).toEqual(['edge-reference-mark', 'unrefused', 'unrefused',],);
          },
        },),
        it({
          name: 'NAMES NO REASON FOR A SPLICE THE RECORD CAN REFUSE ITSELF OR ONE THE ENGLISH LEAVES PLANNED, and none when the '
            + 'English is not held',
          fn: async () => {
            const [marked, receiving,] = renderingsOf({
              chosen: { english: 'Fill the dogs bowl.', quote: 'dogs', },
              receiving: { english: 'Fill the *kittens*’ bowl.', quote: 'kittens', },
            },);
            const [chosen, target,] = renderingsOf({
              chosen: { english: 'Fill the dogs’ bowl.', quote: 'dogs’', },
              receiving: { english: 'Fill the cats bowl.', quote: 'cats', },
            },);
            expect([
              // A chosen place with no apostrophe over a place with a mark past markup is a plain replacement.
              edgeRefusalOf({ chosen: marked, target: receiving, framed: false, english: 'Fill the *kittens*’ bowl.', },),
              // A gain over a place with nothing around it is a plain replacement.
              edgeRefusalOf({ chosen, target, framed: false, english: 'Fill the cats bowl.', },),
              // The two places hold different single marks beside their words, which the reader of the record sees too.
              edgeRefusalOf({
                chosen: { ...chosen, markAfter: '\'', },
                target,
                framed: false,
                english: 'Fill the cats bowl.',
              },),
            ],).toEqual(['unrefused', 'unrefused', 'unrefused',],);
          },
        },),
      ],
    },),
  ],
},);
