/**
 * Guard tests for the stringly-compiled parallel CPU worker script.
 * The worker script is emitted as a string and passed to `node -e`,
 * so TypeScript never checks its body; a past defect (the loop body
 * referenced an undeclared variable) crashed every worker at runtime
 * while type-checking stayed green. These tests execute
 * runParallelCpuBenchmark end to end so a broken emitted script
 * fails the test suite instead of surfacing only in benchmark runs.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runParallelCpuBenchmark, } from './validate-benchmarks.ts';

await describe({
  name: '',
  children: [
    describe({
      name: runParallelCpuBenchmark.name,
      children: [
        it({
          name: 'resolves with a finite timing for one worker and one hash',
          fn: async () => {
            const ms = await runParallelCpuBenchmark(1, 1,);
            expect(ms,).toBeTypeOf('number',);
            expect(Number.isFinite(ms,),).toBe(true,);
            expect(ms,).toBeGreaterThanOrEqual(0,);
          },
          timeout: 30_000,
        },),
        it({
          name: 'resolves with a finite timing when workers fan out in parallel',
          fn: async () => {
            const ms = await runParallelCpuBenchmark(2, 3,);
            expect(ms,).toBeTypeOf('number',);
            expect(Number.isFinite(ms,),).toBe(true,);
            expect(ms,).toBeGreaterThanOrEqual(0,);
          },
          timeout: 30_000,
        },),
      ],
    },),
  ],
},);
