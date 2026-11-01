/**
 Tests for the document-level either-rendering rule over destinations.

 WHAT THESE PIN: a source destination the archive rendered another way is
 owed as one rendering from either side; carrying neither drops it, carrying
 both is a finding; a destination both sides carry as written stays owed
 outright; with no archive every source destination is owed, as the check
 always demanded; and an archive addition the page lost is not a source
 destination and is not reported.

 Fixtures are invented addresses, so there is no corpus text here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ARCHIVE_RENDERING_FINDING,
  BOTH_RENDERINGS_FINDING,
  judgeDestinationRenderings,
  sameAddress,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Address the source links to.
 */
const HOME = 'https://example.org/tabby';

/**
 How the archive rendered the same reference.
 */
const HOME_MOVED = 'https://example.net/tabby';

/**
 Address both sides carry as written.
 */
const ALBUM = 'https://example.org/album';

/**
 Address only the archive carries.
 */
const SHOP = 'https://example.org/shop';

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sameAddress.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'sheds one trailing slash and nothing else',
          fn: async () => {
            expect(sameAddress({ url: `${HOME}/`, },),).toBe(HOME,);
            expect(sameAddress({ url: HOME, },),).toBe(HOME,);
          },
        },),

        it({
          name: 'KEYS the site root as itself, so it never compares equal to the empty string',
          fn: async () => {
            expect({
              root: sameAddress({ url: '/', },),
              verdict: judgeDestinationRenderings({
                source: ['/',],
                page: ['',],
                archive: [],
              },),
            },).toStrictEqual({
              root: '/',
              verdict: {
                dropped: ['/',],
                findings: [],
              },
            },);
          },
        },),
      ],
    },),

    describe({
      name: judgeDestinationRenderings.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS the archive rendering in place of the source destination, and names it',
          fn: async () => {
            const verdict = judgeDestinationRenderings({
              source: [HOME,],
              page: [HOME_MOVED,],
              archive: [HOME_MOVED,],
            },);

            expect(verdict.dropped,).toStrictEqual([],);
            expect(verdict.findings,).toStrictEqual([ARCHIVE_RENDERING_FINDING,],);
          },
        },),

        it({
          name: 'ACCEPTS the source destination where the archive rendered it another way, with no finding',
          fn: async () => {
            const verdict = judgeDestinationRenderings({
              source: [HOME,],
              page: [HOME,],
              archive: [HOME_MOVED,],
            },);

            expect(verdict,).toStrictEqual({
              dropped: [],
              findings: [],
            },);
          },
        },),

        it({
          name: 'DROPS the source destination when the page carries neither rendering',
          fn: async () => {
            const verdict = judgeDestinationRenderings({
              source: [HOME,],
              page: [],
              archive: [HOME_MOVED,],
            },);

            expect(verdict.dropped,).toStrictEqual([HOME,],);
            expect(verdict.findings,).toStrictEqual([],);
          },
        },),

        it({
          name: 'names both renderings when the page carries the original and the archive one',
          fn: async () => {
            const verdict = judgeDestinationRenderings({
              source: [HOME,],
              page: [
                HOME,
                HOME_MOVED,
              ],
              archive: [HOME_MOVED,],
            },);

            expect(verdict.dropped,).toStrictEqual([],);
            expect(verdict.findings,).toStrictEqual([BOTH_RENDERINGS_FINDING,],);
          },
        },),

        it({
          name: 'still owes a destination both sides carry as written, whatever the pool says',
          fn: async () => {
            const verdict = judgeDestinationRenderings({
              source: [
                HOME,
                ALBUM,
              ],
              page: [HOME_MOVED,],
              archive: [
                HOME_MOVED,
                ALBUM,
              ],
            },);

            expect(verdict.dropped,).toStrictEqual([ALBUM,],);
            expect(verdict.findings,).toStrictEqual([ARCHIVE_RENDERING_FINDING,],);
          },
        },),

        it({
          name: 'owes every source destination when there is no archive, a trailing slash notwithstanding',
          fn: async () => {
            expect(judgeDestinationRenderings({
              source: [
                HOME,
                ALBUM,
              ],
              page: [`${ALBUM}/`,],
              archive: [],
            },),).toStrictEqual({
              dropped: [HOME,],
              findings: [],
            },);
            expect(judgeDestinationRenderings({
              source: [
                HOME,
                ALBUM,
              ],
              page: [
                ALBUM,
                HOME,
              ],
              archive: [],
            },),).toStrictEqual({
              dropped: [],
              findings: [],
            },);
          },
        },),

        it({
          name: 'does not report an archive addition the page lost, since it is no source destination',
          fn: async () => {
            const verdict = judgeDestinationRenderings({
              source: [HOME,],
              page: [HOME,],
              archive: [
                HOME,
                SHOP,
              ],
            },);

            expect(verdict,).toStrictEqual({
              dropped: [],
              findings: [],
            },);
          },
        },),

        it({
          name: 'owes the larger side where the archive split one reference into two',
          fn: async () => {
            /**
             Pool of one source rendering and two archive renderings: two owed.
             */
            const ask = {
              source: [HOME,],
              archive: [
                HOME_MOVED,
                SHOP,
              ],
            };

            expect(judgeDestinationRenderings({
              ...ask,
              page: [
                HOME_MOVED,
                SHOP,
              ],
            },).dropped,).toStrictEqual([],);
            expect(judgeDestinationRenderings({
              ...ask,
              page: [HOME_MOVED,],
            },).dropped,).toStrictEqual([HOME,],);
          },
        },),
      ],
    },),
  ],
},);
