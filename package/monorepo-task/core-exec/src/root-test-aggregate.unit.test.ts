/** Exercise root test aggregate discovery through its real process boundary. @module */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runRootTestAggregateFixture, } from './root-test-aggregate-fixture.ts';

/**
 Every suite the aggregate must reach, once the package fanout and root discovery agree on ownership.
 */
const ALL_SUITES = [
  'browser',
  'e2e',
  'fallback-unit',
  'owned-suite',
] as const;

await describe({
  name: 'root test aggregate',
  concurrency: 1,
  timeout: 180_000,
  children: [
    it({
      name: 'reaches a package that declares its own test task, and root discovery leaves its unit files to that package',
      fn: async (): Promise<void> => {
        /** One fresh repository per case keeps discovery state out of the next case. */
        const result = await runRootTestAggregateFixture({},);
        expect(result.exitCode,).toBe(0,);
        expect(result.events,).toEqual([...ALL_SUITES,],);
      },
    },),
    it({
      name: 'names the failing package and still runs every other suite',
      fn: async (): Promise<void> => {
        /** A package suite failure is reported by package identity, not by a bare status. */
        const result = await runRootTestAggregateFixture({ ownedTestFails: true, },);
        expect(result.exitCode,).toBe(1,);
        expect(result.events,).toEqual([...ALL_SUITES,],);
        expect(result.output,).toContain('fixture owned-suite failed',);
        expect(result.output,).toContain('package fanout failed: //package/fixture/owned:test',);
        expect(result.output,).toContain('test suites failed: test:packages',);
      },
    },),
    it({
      name: 'keeps running the package suites after a root suite fails',
      fn: async (): Promise<void> => {
        /** Independent evidence for every failed suite requires the later phase to run. */
        const result = await runRootTestAggregateFixture({ browserFails: true, },);
        expect(result.exitCode,).toBe(1,);
        expect(result.events,).toEqual([...ALL_SUITES,],);
        expect(result.output,).toContain('fixture browser failed',);
        expect(result.output,).toContain('test suites failed: test:browser',);
      },
    },),
  ],
},);
