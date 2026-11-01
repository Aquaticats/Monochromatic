/**
 Tests for the heading affinity that drives section alignment.

 These moved here from `align-sections-order.unit.test.ts` when that
 prototype was deleted. The functions under test are LIVE, reached through
 `align-headings-grid.ts` and so through the shipped forced aligner; only the
 prototype that shared their old test file was superseded. Leaving them in a
 file named for deleted code is how live coverage gets thrown away with dead
 code one day.

 Fixtures mirror the corpus shape that motivated the aligner; names are the
 real ones, since the whole point is that handles survive translation.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  headingAffinity,
  latinTokens,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: latinTokens.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'pulls the handle out of a heading that is otherwise Chinese, which '
            + 'is the signal that survives translation in this archive',
          fn: async () => {
            expect([...latinTokens({ text: '### 其八：白毛 suki', },),],)
              .toEqual(['suki',],);
          },
        },),

        it({
          name: 'ignores runs shorter than three letters, since initials and markup '
            + 'fragments match far too freely across unrelated headings',
          fn: async () => {
            expect([...latinTokens({ text: '### a b 其一', },),],).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: headingAffinity.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'scores a shared handle at 1, which is what pairs 其七：wing with '
            + 'wing however far apart the aligner would otherwise place them',
          fn: async () => {
            expect(headingAffinity({
              source: '### 其七：wing',
              target: '### wing',
            },),).toBe(1,);
          },
        },),

        it({
          name: 'scores 0 when either heading offers no Latin run at all, because no '
            + 'evidence is not weak evidence and an aligner should say so rather '
            + 'than guess from a coincidence',
          fn: async () => {
            expect(headingAffinity({
              source: '### 其九：空白',
              target: '### Shinonome',
            },),).toBe(0,);
          },
        },),

        it({
          name: 'scores 1 for a handle written with its accent, with a separate accent, or without one, '
            + 'since a translator carries a handle across any of the three ways (ledger B18)',
          fn: async () => {
            // ASCII runs alone cut `Kätzchen` to `tzchen` and the other two
            // spellings to `katzchen`, so only the identical spelling matched.
            expect(latinTokens({ text: '### 其六：Kätzchen', },),).toEqual(new Set(['katzchen',],),);
            expect([
              '### Kätzchen',
              '### Ka\u{0308}tzchen',
              '### Katzchen',
            ].map(function affinity(target,): number {
              return headingAffinity({
                source: '### 其六：Kätzchen',
                target,
              },);
            },),).toEqual([1, 1, 1,],);
          },
        },),
      ],
    },),
  ],
},);
