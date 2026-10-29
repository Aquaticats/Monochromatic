/**
 Tests the build's entry list (ledger T8): every entry names a source under
 `src/` from the package directory, which the coverage census relies on to
 tell a runner's main from library source, and the runner sources leave the
 library index out.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  nodeEntries,
  runnerEntrySources,
} from '../dist/final/node/index.mjs';

await describe({
  name: runnerEntrySources.name,
  children: [
    it({
      name: 'NAMES EVERY RUNNER\'S SOURCE AS THE CENSUS NAMES SOURCES, without the leading ./, and leaves the library index out',
      fn: async () => {
        const runners = runnerEntrySources();
        expect([...nodeEntries.values(),].every((path,) => path.startsWith('./src/',)),).toBe(true,);
        expect(runners.has('src/index.ts',),).toBe(false,);
        expect([...runners,].toSorted(),).toEqual(
          [...nodeEntries,]
            .filter(([name,],) => name !== 'index')
            .map(([, path,],) => path.slice('./'.length,))
            .toSorted(),
        );
        expect(runners.has('src/corpus-run/coverage-census.ts',),).toBe(true,);
      },
    },),
  ],
},);
