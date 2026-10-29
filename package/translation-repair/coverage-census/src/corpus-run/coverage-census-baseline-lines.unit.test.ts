/**
 Tests for the reading of a run against each earlier census named: which
 stretches ran, are still cold or went cold since, which claimed sources the
 baseline holds nothing in, and which sources were edited since the baseline's
 commit. The git question is handed in and scripted. Paths and names are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { baselineReadingsOf, } from '../../dist/final/node/index.mjs';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { recorded, } from './coverage-census.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

/**
 Package directory every case asks git about.
 */
const PACKAGE_DIRECTORY = '/cats/package';

await describe({
  name: baselineReadingsOf.name,
  children: [
    it({
      name: 'READS NOTHING for no baseline and never asks git',
      fn: async () => {
        /**
         Heads git was asked about.
         */
        const asked: string[] = [];
        expect(await baselineReadingsOf({
          baselines: [],
          packageDirectory: PACKAGE_DIRECTORY,
          claimed: new Set<string>(),
          stretches: [recorded({
            source: 'src/nap.ts',
            startLine: 1,
            endLine: 2,
          },),],
          loadedSources: new Set(['src/nap.ts',],),
          editedSince: async function editedNone({ head, },): Promise<ReadonlySet<string>> {
            asked.push(head,);
            return new Set<string>();
          },
        },),).toEqual([],);
        expect(asked,).toEqual([],);
      },
    },),
    it({
      name: 'READS EACH BASELINE IN THE ORDER NAMED, asking git about each one\'s own commit, with the stretches that '
        + 'ran, are still cold, went cold since and sit in an edited source',
      fn: async () => {
        /**
         Where git was asked, by head.
         */
        const asked: string[] = [];
        expect(await baselineReadingsOf({
          baselines: [
            {
              path: '/cats/before.json',
              census: {
                head: 'c0ffee123',
                stretches: [
                  recorded({
                    source: 'src/nap.ts',
                    startLine: 3,
                    endLine: 5,
                  },),
                  recorded({
                    source: 'src/purr.ts',
                    startLine: 10,
                    endLine: 12,
                  },),
                  recorded({
                    source: 'src/loaf.ts',
                    startLine: 1,
                    endLine: 2,
                  },),
                ],
                loadedSources: new Set([
                  'src/nap.ts',
                  'src/purr.ts',
                  'src/loaf.ts',
                ],),
              },
            },
            {
              path: '/cats/older.json',
              census: {
                head: 'beef00456',
                stretches: [],
                loadedSources: new Set(['src/nap.ts',],),
              },
            },
          ],
          packageDirectory: PACKAGE_DIRECTORY,
          claimed: new Set<string>(),
          stretches: [
            recorded({
              source: 'src/purr.ts',
              startLine: 11,
              endLine: 12,
            },),
            recorded({
              source: 'src/zoomies.ts',
              startLine: 1,
              endLine: 1,
            },),
          ],
          loadedSources: new Set([
            'src/nap.ts',
            'src/purr.ts',
            'src/zoomies.ts',
          ],),
          editedSince: async function editedLoaf(
            { packageDirectory, head, },
          ): Promise<ReadonlySet<string>> {
            asked.push(`${head} in ${packageDirectory}`,);
            return new Set((head === 'c0ffee123') ? ['src/loaf.ts',] : [],);
          },
        },),).toEqual([
          [
            'against /cats/before.json at c0ffee123: ran 1, still cold 1, cold since then 1, not loaded 0, claimed '
              + 'sources with no stretch there 0, sources edited since then 1',
            '  still cold: src/purr.ts:10-12',
            '  cold since then (not loaded there): src/zoomies.ts:1-1',
            '  edited since c0ffee123, so its baseline lines name other code; this run did not load it: src/loaf.ts',
          ],
          [
            'against /cats/older.json at beef00456: ran 0, still cold 0, cold since then 2, not loaded 0, claimed '
              + 'sources with no stretch there 0, sources edited since then 0',
            '  cold since then (not loaded there): src/purr.ts:11-12',
            '  cold since then (not loaded there): src/zoomies.ts:1-1',
          ],
        ],);
        expect(asked,).toEqual([
          `c0ffee123 in ${PACKAGE_DIRECTORY}`,
          `beef00456 in ${PACKAGE_DIRECTORY}`,
        ],);
      },
    },),
    it({
      name: 'NAMES THE CLAIMED SOURCES a baseline holds no stretch in, one it ran whole and one it never loaded, '
        + 'and an edited claimed source, each with this run\'s standing in the plural',
      fn: async () => {
        expect(await baselineReadingsOf({
          baselines: [{
            path: '/cats/before.json',
            census: {
              head: 'c0ffee123',
              stretches: [recorded({
                source: 'src/nap.ts',
                startLine: 3,
                endLine: 5,
              },),],
              loadedSources: new Set([
                'src/nap.ts',
                'src/loaf.ts',
              ],),
            },
          },],
          packageDirectory: PACKAGE_DIRECTORY,
          claimed: new Set([
            'src/loaf.ts',
            'src/zoomies.ts',
            'src/nap.ts',
          ],),
          stretches: [
            recorded({
              source: 'src/zoomies.ts',
              startLine: 1,
              endLine: 1,
            },),
            recorded({
              source: 'src/zoomies.ts',
              startLine: 4,
              endLine: 6,
            },),
          ],
          loadedSources: new Set([
            'src/nap.ts',
            'src/zoomies.ts',
          ],),
          editedSince: async function editedNap(): Promise<ReadonlySet<string>> {
            return new Set(['src/nap.ts',],);
          },
        },),).toEqual([[
          'against /cats/before.json at c0ffee123: ran 0, still cold 0, cold since then 2, not loaded 0, claimed '
            + 'sources with no stretch there 2, sources edited since then 1',
          '  cold since then (not loaded there): src/zoomies.ts:1-1',
          '  cold since then (not loaded there): src/zoomies.ts:4-6',
          '  no baseline stretch (ran whole there); this run did not load it: src/loaf.ts',
          '  no baseline stretch (not loaded there, so the baseline proves nothing of it); this run loaded it and '
            + 'left 2 cold stretches: src/zoomies.ts',
          '  edited since c0ffee123, so its baseline lines name other code; this run loaded it and left 0 cold '
            + 'stretches: src/nap.ts',
        ],],);
      },
    },),
    it({
      name: 'SAYS ONE COLD STRETCH in the singular for a claimed source the baseline holds nothing in',
      fn: async () => {
        expect(await baselineReadingsOf({
          baselines: [{
            path: '/cats/before.json',
            census: {
              head: 'c0ffee123',
              stretches: [],
              loadedSources: new Set<string>(),
            },
          },],
          packageDirectory: PACKAGE_DIRECTORY,
          claimed: new Set(['src/zoomies.ts',],),
          stretches: [recorded({
            source: 'src/zoomies.ts',
            startLine: 1,
            endLine: 1,
          },),],
          loadedSources: new Set(['src/zoomies.ts',],),
          editedSince: async function editedNone(): Promise<ReadonlySet<string>> {
            return new Set<string>();
          },
        },),).toEqual([[
          'against /cats/before.json at c0ffee123: ran 0, still cold 0, cold since then 1, not loaded 0, claimed '
            + 'sources with no stretch there 1, sources edited since then 0',
          '  cold since then (not loaded there): src/zoomies.ts:1-1',
          '  no baseline stretch (not loaded there, so the baseline proves nothing of it); this run loaded it and '
            + 'left 1 cold stretch: src/zoomies.ts',
        ],],);
      },
    },),
    it({
      name: 'REFUSES with the first baseline\'s git failure when two baselines\' commits cannot be asked about and '
        + 'the second baseline\'s question is refused first',
      fn: async () => {
        /**
         The two refusals git's answers end in.
         */
        const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
        /**
         What the reading refused with.
         */
        const refusal = await rejectionOf({
          promise: baselineReadingsOf({
            baselines: [
              {
                path: '/cats/before.json',
                census: {
                  head: 'c0ffee123',
                  stretches: [],
                  loadedSources: new Set<string>(),
                },
              },
              {
                path: '/cats/older.json',
                census: {
                  head: 'beef00456',
                  stretches: [],
                  loadedSources: new Set<string>(),
                },
              },
            ],
            packageDirectory: PACKAGE_DIRECTORY,
            claimed: new Set<string>(),
            stretches: [],
            loadedSources: new Set<string>(),
            editedSince: async function refusesSecondFirst({ head, },): Promise<ReadonlySet<string>> {
              return await ((head === 'c0ffee123')
                ? refuseAfterThat(new Error('git cannot diff since c0ffee123',),)
                : refuseAtOnce(new Error('git cannot diff since beef00456',),));
            },
          },),
        },);
        expect(String(refusal,),).toBe('Error: git cannot diff since c0ffee123',);
      },
    },),
  ],
},);
