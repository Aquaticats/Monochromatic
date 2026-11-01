import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  excludedMutantKinds,
  exclusionArguments,
} from './excluded-mutants.ts';

await describe({
  name: '',
  children: [
    describe({
      name: 'excludedMutantKinds',
      children: [
        it({
          name: 'is exactly the two kinds accepted on 2026-10-05, spelled as the five scripts spelled them',
          fn: async () => {
            expect(excludedMutantKinds).toEqual([
              String.raw`replace \+= with \*=`,
              'replace -= with /=',
            ]);
            expect(excludedMutantKinds[0]).toBe(String.raw`replace \+= with \*=`);
          },
        }),
      ],
    }),
    describe({
      name: exclusionArguments.name,
      children: [
        it({
          name: 'gives one pair per repository-wide kind when a product adds none',
          fn: async () => {
            expect(exclusionArguments([])).toEqual([
              '--exclude-re',
              String.raw`replace \+= with \*=`,
              '--exclude-re',
              'replace -= with /=',
            ]);
          },
        }),
        it({
          name: 'puts the product patterns after the repository-wide kinds and drops none of them',
          fn: async () => {
            expect(exclusionArguments(['a', 'b c'])).toEqual([
              '--exclude-re',
              String.raw`replace \+= with \*=`,
              '--exclude-re',
              'replace -= with /=',
              '--exclude-re',
              'a',
              '--exclude-re',
              'b c',
            ]);
          },
        }),
      ],
    }),
  ],
});
