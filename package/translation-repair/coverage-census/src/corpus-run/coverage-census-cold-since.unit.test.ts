/**
 Tests how a batch's run names the code it left cold that an earlier census
 did not (ledger B61): a stretch holding a line no baseline stretch of its
 source held. Reading only the baseline's own stretches counted such code
 nowhere, so a change that left code cold read clean. Paths are cat-themed
 invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { coldSinceOf, } from '../../dist/final/node/index.mjs';
import { recorded, } from './coverage-census.test-fixture.ts';

await describe({
  name: coldSinceOf.name,
  children: [
    it({
      name: 'NAMES EACH STRETCH THIS RUN LEFT COLD IN LINES NO BASELINE STRETCH OF ITS SOURCE HELD, sorted by source '
        + 'and then line whatever order the run listed them in, including one in a source the baseline ran whole, '
        + 'and leaves one inside the baseline\'s lines',
      fn: async () => {
        expect(coldSinceOf({
          baseline: {
            head: 'c0ffee123',
            stretches: [
              recorded({
                source: 'src/nap.ts',
                startLine: 3,
                endLine: 5,
              },),
            ],
            loadedSources: new Set(['src/nap.ts', 'src/purr.ts',],),
          },
          current: [
            recorded({
              source: 'src/purr.ts',
              startLine: 1,
              endLine: 1,
            },),
            recorded({
              source: 'src/nap.ts',
              startLine: 12,
              endLine: 12,
            },),
            recorded({
              source: 'src/nap.ts',
              startLine: 8,
              endLine: 9,
            },),
            recorded({
              source: 'src/nap.ts',
              startLine: 4,
              endLine: 5,
            },),
          ],
          sources: new Set(),
          edited: new Set(),
        },).map(({
          stretch,
          loadedAtBaseline,
        },) => [stretch.source, stretch.startLine, stretch.endLine, loadedAtBaseline,]),).toEqual([
          ['src/nap.ts', 8, 9, true,],
          ['src/nap.ts', 12, 12, true,],
          ['src/purr.ts', 1, 1, true,],
        ],);
      },
    },),
    it({
      name: 'NAMES A STRETCH THAT REACHES PAST THE BASELINE\'S LINES, at either end or across the gap between two, '
        + 'since the lines it gained went cold',
      fn: async () => {
        /**
         Two baseline stretches with a line between them.
         */
        const baseline = {
          head: 'c0ffee123',
          stretches: [
            recorded({
              source: 'src/nap.ts',
              startLine: 3,
              endLine: 5,
            },),
            recorded({
              source: 'src/nap.ts',
              startLine: 7,
              endLine: 8,
            },),
          ],
          loadedSources: new Set(['src/nap.ts',],),
        };
        for (const [startLine, endLine,] of [[2, 5,], [7, 9,], [3, 8,],] as const) {
          expect(coldSinceOf({
            baseline,
            current: [
              recorded({
                source: 'src/nap.ts',
                startLine,
                endLine,
              },),
            ],
            sources: new Set(),
            edited: new Set(),
          },),).toHaveLength(1,);
        }
        expect(coldSinceOf({
          baseline,
          current: [
            recorded({
              source: 'src/nap.ts',
              startLine: 3,
              endLine: 5,
            },),
            recorded({
              source: 'src/nap.ts',
              startLine: 7,
              endLine: 8,
            },),
          ],
          sources: new Set(),
          edited: new Set(),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'MARKS A STRETCH IN A SOURCE THE BASELINE DID NOT LOAD, which proves nothing of its lines either way',
      fn: async () => {
        expect(coldSinceOf({
          baseline: {
            head: 'c0ffee123',
            stretches: [],
            loadedSources: new Set(['src/nap.ts',],),
          },
          current: [
            recorded({
              source: 'src/knead.ts',
              startLine: 2,
              endLine: 2,
            },),
          ],
          sources: new Set(),
          edited: new Set(),
        },).map(({ loadedAtBaseline, },) => loadedAtBaseline),).toEqual([false,],);
      },
    },),
    it({
      name: 'LEAVES OUT CODE NO SOURCE MAP PLACES, which has no lines to compare and takes its bundle\'s name, so a '
        + 'rebuild that renames the bundle read its unchanged stretches as cold since then',
      fn: async () => {
        expect(coldSinceOf({
          baseline: {
            head: 'c0ffee123',
            stretches: [
              recorded({
                source: '(unmapped) nap-AAAA.mjs',
                startLine: 0,
                endLine: 0,
              },),
            ],
            loadedSources: new Set(['src/nap.ts',],),
          },
          current: [
            recorded({
              source: '(unmapped) nap-BBBB.mjs',
              startLine: 0,
              endLine: 0,
            },),
          ],
          sources: new Set(),
          edited: new Set(),
        },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES OUT A SOURCE EDITED SINCE THE BASELINE, whose lines there name other code, and a source the batch '
        + 'does not claim when it claims any',
      fn: async () => {
        expect(coldSinceOf({
          baseline: {
            head: 'c0ffee123',
            stretches: [],
            loadedSources: new Set(['src/nap.ts', 'src/purr.ts', 'src/yawn.ts',],),
          },
          current: [
            recorded({
              source: 'src/nap.ts',
              startLine: 20,
              endLine: 21,
            },),
            recorded({
              source: 'src/purr.ts',
              startLine: 2,
              endLine: 2,
            },),
            recorded({
              source: 'src/yawn.ts',
              startLine: 1,
              endLine: 1,
            },),
          ],
          sources: new Set(['src/nap.ts', 'src/purr.ts',],),
          edited: new Set(['src/nap.ts',],),
        },).map(({ stretch, },) => stretch.source),).toEqual(['src/purr.ts',],);
      },
    },),
  ],
},);
