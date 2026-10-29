/**
 Tests for the pools of renderings the original and the page disagree on,
 and the findings drawn from them.

 WHAT THESE PIN is the owner's rule of 2026-09-04: where the page rendered
 a reference another way, a candidate owes one rendering and not both;
 a kind that diverges one way only stays owed as an addition or a drop.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ProtectedAtom, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  atomFindings,
  renderingPoolsOf,
} from '../dist/final/node/index.mjs';

/**
 Link atom for a destination.

 @param url - destination

 @returns Atom as the skeleton reader emits it

 @example
 ```ts
 const atom = link('https://a.example');
 ```
 */
function link(url: string,): ProtectedAtom {
  return {
    kind: 'link-url',
    value: url,
  };
}

/**
 Footnote atom for a marker.

 @param marker - footnote label

 @returns Atom as the skeleton reader emits it

 @example
 ```ts
 const atom = footnote('1');
 ```
 */
function footnote(marker: string,): ProtectedAtom {
  return {
    kind: 'footnote',
    value: marker,
  };
}

/**
 Original's destination.
 */
const A = 'https://twitter.example/cat';

/**
 Page's rewritten destination.
 */
const B = 'https://x.example/cat';

/**
 Destination neither carries.
 */
const C = 'https://cats.example/naps';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: renderingPoolsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'POOLS A KIND that diverges both ways, owing the larger side',
          fn: async () => {
            expect(renderingPoolsOf({
              source: [link(A,), link(A,),],
              page: [link(B,),],
            },),).toEqual([{
              kind: 'link-url',
              fromSource: [`link-url ${A}`, `link-url ${A}`,],
              fromPage: [`link-url ${B}`,],
              owed: 2,
            },],);
          },
        },),

        it({
          name: 'LEAVES ONE-WAY DIVERGENCE OUT, whether an addition or a drop, and keeps kinds apart',
          fn: async () => {
            expect(renderingPoolsOf({
              source: [link(A,),],
              page: [link(A,), footnote('1',),],
            },),).toEqual([],);
            expect(renderingPoolsOf({
              source: [link(A,), footnote('1',),],
              page: [link(A,),],
            },),).toEqual([],);
            expect(renderingPoolsOf({
              source: [link(A,), footnote('1',),],
              page: [link(B,), footnote('2',),],
            },).map(function toKind(pool,): string {
              return pool.kind;
            },),).toEqual(['link-url', 'footnote',],);
            expect(renderingPoolsOf({
              source: [],
              page: [],
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: atomFindings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS EITHER RENDERING of a pooled reference',
          fn: async () => {
            expect(atomFindings({
              source: [link(A,),],
              page: [link(B,),],
              candidate: [link(A,),],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },),).toEqual([],);
            expect(atomFindings({
              source: [link(A,),],
              page: [link(B,),],
              candidate: [link(B,),],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },),).toEqual([],);
          },
        },),

        it({
          name: 'REFUSES NEITHER AND BOTH, naming the two renderings and the count owed',
          fn: async () => {
            /**
             Findings over a candidate that dropped the reference.
             */
            const neither = atomFindings({
              source: [link(A,),],
              page: [link(B,),],
              candidate: [],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },);
            expect(neither,).toEqual([
              `The ORIGINAL carries link-url ${A} where the PAGE AS IT STANDS carries link-url ${B}: the page rendered `
                + 'the original\'s reference another way, and your translation must carry exactly 1 of these, taken from '
                + 'either side; it carries 0.',
            ],);
            /**
             Findings over a candidate that carried both renderings.
             */
            const both = atomFindings({
              source: [link(A,),],
              page: [link(B,),],
              candidate: [link(A,), link(B,),],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },);
            expect(both,).toEqual([
              `The ORIGINAL carries link-url ${A} where the PAGE AS IT STANDS carries link-url ${B}: the page rendered `
                + 'the original\'s reference another way, and your translation must carry exactly 1 of these, taken from '
                + 'either side; it carries 2.',
            ],);
          },
        },),

        it({
          name: 'COUNTS A POOLED RENDERING carried twice as one draw and one copy too many, naming both counts',
          fn: async () => {
            expect(atomFindings({
              source: [link(A,),],
              page: [link(B,),],
              candidate: [link(A,), link(A,),],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },),).toEqual([
              `Your translation carries link-url ${A} 2 times and the ORIGINAL or the PAGE AS IT STANDS carries it once.`,
            ],);
          },
        },),

        it({
          name: 'STILL OWES an addition the page made and a reference the page dropped, and refuses an invention',
          fn: async () => {
            expect(atomFindings({
              source: [link(A,),],
              page: [link(A,), link(C,),],
              candidate: [link(A,),],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },),).toEqual([
              `The ORIGINAL or the PAGE AS IT STANDS carries link-url ${C} and your translation does not.`,
            ],);
            expect(atomFindings({
              source: [link(A,), link(C,),],
              page: [link(A,),],
              candidate: [link(A,),],
              referenceName: 'ORIGINAL or the PAGE AS IT STANDS',
            },),).toEqual([
              `The ORIGINAL or the PAGE AS IT STANDS carries link-url ${C} and your translation does not.`,
            ],);
            expect(atomFindings({
              source: [link(A,),],
              page: [],
              candidate: [link(A,), link(C,),],
              referenceName: 'ORIGINAL',
            },),).toEqual([
              `Your translation carries link-url ${C} and the ORIGINAL does not.`,
            ],);
          },
        },),

        it({
          name: 'NAMES EACH ATOM ONCE WITH ITS COUNTS, never saying a side carries none of an atom it carries fewer '
            + 'times, nor repeating one sentence per copy',
          fn: async () => {
            expect(atomFindings({
              source: [link(A,), link(C,), link(C,),],
              page: [],
              candidate: [link(A,),],
              referenceName: 'ORIGINAL',
            },),).toEqual([
              `The ORIGINAL carries link-url ${C} 2 times and your translation does not.`,
            ],);
            expect(atomFindings({
              source: [link(A,), link(C,), link(C,),],
              page: [],
              candidate: [link(A,), link(C,),],
              referenceName: 'ORIGINAL',
            },),).toEqual([
              `The ORIGINAL carries link-url ${C} 2 times and your translation carries it once.`,
            ],);
            expect(atomFindings({
              source: [link(A,),],
              page: [],
              candidate: [link(A,), link(C,), link(C,),],
              referenceName: 'ORIGINAL',
            },),).toEqual([
              `Your translation carries link-url ${C} 2 times and the ORIGINAL does not.`,
            ],);
            expect(atomFindings({
              source: [link(C,),],
              page: [],
              candidate: [link(C,), link(C,), link(C,),],
              referenceName: 'ORIGINAL',
            },),).toEqual([
              `Your translation carries link-url ${C} 3 times and the ORIGINAL carries it once.`,
            ],);
          },
        },),
      ],
    },),
  ],
},);
