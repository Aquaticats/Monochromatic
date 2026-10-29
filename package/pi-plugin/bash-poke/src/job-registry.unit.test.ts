/**
 Tests for the background job registry in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createJobRegistry,
  createOutputRecorder,
  type RunningJob,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Builds a stand-in job whose cancellation is recorded instead of performed.
 
 @param id - identifier the registry keys on
 
 @param cancels - list collecting cancelled identifiers
 
 @returns job suitable for registry cases
 
 @example
 ```ts
 fakeJob({ id: 'a', cancels: [], },);
 ```
 */
function fakeJob(
  {
    id,
    cancels,
  }: {
    readonly id: string;
    readonly cancels: string[];
  },
): RunningJob {
  return {
    id,
    command: `command-${id}`,
    startedAt: 0,
    recorder: createOutputRecorder({ headChars: 4, tailChars: 4, }, ),
    cancel(): void {
      cancels.push(id, );
    },
    finished: Promise.resolve({ cancelled: false, }, ),
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region createJobRegistry

    describe({
      name: createJobRegistry.name,
      children: [
        it({
          name: 'starts empty',
          fn: async () => {
            const registry = createJobRegistry();
            expect(registry.size(), ).toBe(0);
            expect(registry.list(), ).toEqual([]);
          },
        }, ),
        it({
          name: 'tracks added jobs in start order',
          fn: async () => {
            const cancels: string[] = [];
            const registry = createJobRegistry();
            registry.add(fakeJob({ id: 'a', cancels, }, ), );
            registry.add(fakeJob({ id: 'b', cancels, }, ), );
            expect(registry.size(), ).toBe(2);
            expect(registry.list().map(function toId(job: RunningJob, ): string {
              return job.id;
            }, ), ).toEqual(['a', 'b', ]);
          },
        }, ),
        it({
          name: 'stops tracking a removed job',
          fn: async () => {
            const cancels: string[] = [];
            const registry = createJobRegistry();
            registry.add(fakeJob({ id: 'a', cancels, }, ), );
            registry.remove('a', );
            expect(registry.size(), ).toBe(0);
          },
        }, ),
        it({
          name: 'ignores removal of an unknown identifier',
          fn: async () => {
            const registry = createJobRegistry();
            registry.remove('absent', );
            expect(registry.size(), ).toBe(0);
          },
        }, ),
        it({
          name: 'cancels every tracked job and reports the count',
          fn: async () => {
            const cancels: string[] = [];
            const registry = createJobRegistry();
            registry.add(fakeJob({ id: 'a', cancels, }, ), );
            registry.add(fakeJob({ id: 'b', cancels, }, ), );
            expect(registry.cancelAll(), ).toBe(2);
            expect(cancels, ).toEqual(['a', 'b', ]);
          },
        }, ),
        it({
          name: 'reports zero when nothing is tracked',
          fn: async () => {
            const registry = createJobRegistry();
            expect(registry.cancelAll(), ).toBe(0);
          },
        }, ),
        it({
          name: 'keeps jobs tracked after cancellation so completion can remove them',
          fn: async () => {
            const cancels: string[] = [];
            const registry = createJobRegistry();
            registry.add(fakeJob({ id: 'a', cancels, }, ), );
            registry.cancelAll();
            expect(registry.size(), ).toBe(1);
          },
        }, ),
      ],
    }, ),

    //endregion createJobRegistry
  ],
}, );
