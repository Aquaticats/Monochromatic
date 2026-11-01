import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import { mapInSequence } from './in-sequence.ts';
import {
  coverageSummary,
  takeMutantRegex,
  takeShard,
} from './scope-tools.ts';

/**
 Run a parser that must fail and return the message.

 @param run - the call under test
 @returns the message of the thrown tooling error
 */
function failureOf(run: () => unknown): string {
  try {
    run();
  } catch (error) {
    expect(error).toBeInstanceOf(CargoMutantsToolingError);
    return (error as Error).message;
  }
  return '';
}

await describe({
  name: '',
  children: [
    describe({
      name: takeShard.name,
      children: [
        it({
          name: 'leaves the arguments alone when there is no shard',
          fn: async () => {
            expect(takeShard(['--core'])).toEqual({ rest: ['--core'] });
          },
        }),
        it({
          name: 'takes the shard and its value out, wherever they stand',
          fn: async () => {
            expect(takeShard(['--core', '--shard', '1/3', '--re', 'x'])).toEqual({
              shard: '1/3',
              rest: ['--core', '--re', 'x'],
            });
            expect(takeShard(['--shard', '0/1', '--core'])).toEqual({
              shard: '0/1',
              rest: ['--core'],
            });
          },
        }),
        ...[
          [['--shard', '3/3'], '3/3'],
          [['--shard', '4/3'], '4/3'],
          [['--shard', '0/0'], '0/0'],
          [['--shard', '1'], '1'],
          [['--shard', '1/2/3'], '1/2/3'],
          [['--shard', 'a/3'], 'a/3'],
          [['--shard', '1/'], '1/'],
          [['--shard', '-1/3'], '-1/3'],
          [['--shard'], 'undefined'],
        ].map(function rejection([options, shown]) {
          return it({
            name: `rejects ${JSON.stringify(options)} naming the value`,
            fn: async () => {
              expect(failureOf(() => takeShard(options as readonly string[])))
                .toBe(`--shard needs k/n with k from 0 to n - 1, not ${String(shown)}.`);
            },
          });
        }),
      ],
    }),
    describe({
      name: takeMutantRegex.name,
      children: [
        it({
          name: 'leaves the arguments alone when there is no pattern',
          fn: async () => {
            expect(takeMutantRegex(['--core'])).toEqual({ rest: ['--core'] });
          },
        }),
        it({
          name: 'takes the pattern and its value out',
          fn: async () => {
            expect(takeMutantRegex(['--core', '--re', 'Foo::bar'])).toEqual({
              examine: 'Foo::bar',
              rest: ['--core'],
            });
          },
        }),
        it({
          name: 'rejects an empty pattern and a missing one',
          fn: async () => {
            expect(failureOf(() => takeMutantRegex(['--re', '']))).toBe('--re needs a mutant-name regex.');
            expect(failureOf(() => takeMutantRegex(['--re']))).toBe('--re needs a mutant-name regex.');
          },
        }),
      ],
    }),
    describe({
      name: coverageSummary.name,
      children: [
        it({
          name: 'reports no difference when the scopes hold exactly the unscoped mutants',
          fn: async () => {
            expect(coverageSummary({
              all: ['a', 'b', 'c'],
              listings: [['--one', ['a', 'b']], ['--two', ['b', 'c']]],
            })).toEqual({
              unscoped: 3,
              unscopedDistinct: 3,
              union: 3,
              perScope: { '--one': 2, '--two': 2 },
              outside: [],
              unknown: [],
            });
          },
        }),
        it({
          name: 'names the mutants outside every scope and the ones only a scope lists',
          fn: async () => {
            expect(coverageSummary({
              all: ['a', 'b', 'c'],
              listings: [['--one', ['a', 'z']]],
            })).toEqual({
              unscoped: 3,
              unscopedDistinct: 3,
              union: 2,
              perScope: { '--one': 2 },
              outside: ['b', 'c'],
              unknown: ['z'],
            });
          },
        }),
        it({
          name: 'counts a repeated unscoped name once as distinct',
          fn: async () => {
            expect(coverageSummary({
              all: ['a', 'a'],
              listings: [['--one', ['a']]],
            })).toMatchObject({
              unscoped: 2,
              unscopedDistinct: 1,
            });
          },
        }),
      ],
    }),
    describe({
      name: mapInSequence.name,
      children: [
        it({
          name: 'starts each item only after the one before finished and keeps the order',
          fn: async () => {
            const events: string[] = [];
            const results = await mapInSequence({
              items: [3, 1, 2],
              run: async (item) => {
                events.push(`start ${item}`);
                await new Promise<void>((resolve) => {
                  setTimeout(resolve, item);
                });
                events.push(`end ${item}`);
                return item * 10;
              },
            });
            expect(results).toEqual([30, 10, 20]);
            expect(events).toEqual(['start 3', 'end 3', 'start 1', 'end 1', 'start 2', 'end 2']);
          },
        }),
        it({
          name: 'gives an empty list for no items',
          fn: async () => {
            expect(await mapInSequence({
              items: [],
              run: async (item: number) => item,
            })).toEqual([]);
          },
        }),
      ],
    }),
  ],
});
