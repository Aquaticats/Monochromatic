/**
 Tests the census's reading of the family's build entries: every package that
 owns runners lists them in its own `src/build-entries.ts`, and the census must
 name the union of those lists, or it sees a subset of the runners and counts
 a runner's main as library source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  FAMILY_ENTRY_LISTS,
  FamilyEntryConflictError,
  familyEntriesOf,
  nodeEntries,
  runnerEntrySources,
} from '../dist/final/node/index.mjs';

// The case that holds `FAMILY_ENTRY_LISTS` to the packages that have a list moved to
// `source-scan/src/family-build-entries.unit.test.ts` (ledger B1265): it listed every package's folder, which a test
// of one package must not.

await describe({
  name: familyEntriesOf.name,
  children: [
    it({
      name: 'NAMES EVERY RUNNER OF EVERY LIST ONCE, under the one library index every list names',
      fn: async () => {
        /**
         Every entry name of every list, with repeats.
         */
        const named = [...FAMILY_ENTRY_LISTS.values(),].flatMap(function namesOf(list,): readonly string[] {
          return [...list.keys(),];
        },);
        /**
         The same, without the index each list names.
         */
        const runners = named.filter(function isRunner(name,): boolean {
          return name !== 'index';
        },);

        expect(new Set(runners,).size,).toBe(runners.length,);
        expect([...nodeEntries.keys(),].filter(function isRunner(name,): boolean {
          return name !== 'index';
        },).toSorted(),).toEqual(runners.toSorted(),);
        expect(runnerEntrySources().size,).toBe(runners.length,);
        expect(runnerEntrySources().has('src/corpus-run/slice-census.ts',),).toBe(true,);
        expect(runnerEntrySources().has('src/corpus-run/corpus-pass.ts',),).toBe(true,);
        expect(runnerEntrySources().has('src/index.ts',),).toBe(false,);
      },
    },),
    it({
      name: 'REFUSES TWO PACKAGES THAT NAME ONE ENTRY AT TWO PATHS, naming the entry and both packages',
      fn: async () => {
        /**
         What the merge threw.
         */
        let thrown: unknown;
        try {
          familyEntriesOf({
            lists: new Map([
              [
                'tabby',
                new Map([['purr', './src/corpus-run/purr.ts',],],),
              ],
              [
                'calico',
                new Map([['purr', './src/corpus-run/other-purr.ts',],],),
              ],
            ],),
          },);
        }
        catch (error) {
          thrown = error;
        }

        expect(thrown,).toBeInstanceOf(FamilyEntryConflictError,);
        expect((thrown as FamilyEntryConflictError).messageNamesOnly,).toBe(true,);
        expect(String(thrown,),).toBe(
          'FamilyEntryConflictError: build entry purr is listed at ./src/corpus-run/purr.ts by tabby and at '
            + './src/corpus-run/other-purr.ts by calico, so the census cannot tell which runner it names',
        );
      },
    },),
    it({
      name: 'KEEPS AN ENTRY TWO PACKAGES LIST AT THE SAME PATH once',
      fn: async () => {
        expect([...familyEntriesOf({
          lists: new Map([
            [
              'tabby',
              new Map([['index', './src/index.ts',],],),
            ],
            [
              'calico',
              new Map([['index', './src/index.ts',],],),
            ],
          ],),
        },),],).toEqual([['index', './src/index.ts',],],);
      },
    },),
  ],
},);
