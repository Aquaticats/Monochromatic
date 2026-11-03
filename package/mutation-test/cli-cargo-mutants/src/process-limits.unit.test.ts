import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  casesWithin,
  mentionsProcessLimit,
} from './process-limits.ts';

/**
 Tasks a test file holds at the least, which includes one case.
 */
const fileTasks = 40;

/**
 Tasks one case holds at its peak.
 */
const caseTasks = 25;

/**
 Cases a file runs at once when nothing limits it.
 */
const wanted = 4;

/**
 Workers, and per-run budgets from the tightest that holds each file with one case to a generous one.
 */
const workerCounts: readonly number[] = [
  1,
  3,
  8,
];

await describe({
  name: 'sizing the cases of a test file to the budget of the run',
  children: [
    describe({
      name: casesWithin.name,
      children: [
        it({
          name: 'keeps the wanted width when the runner gave no budget',
          fn: async () => {
            expect(casesWithin({
              workers: 8,
              fileTasks,
              caseTasks,
              wanted,
            })).toBe(wanted);
          },
        }),
        it({
          name: 'gives at least one case however small the budget, and never more than wanted however large',
          fn: async () => {
            expect(casesWithin({
              budget: 1,
              workers: 8,
              fileTasks,
              caseTasks,
              wanted,
            })).toBe(1);
            expect(casesWithin({
              budget: 1_000_000,
              workers: 1,
              fileTasks,
              caseTasks,
              wanted,
            })).toBe(wanted);
          },
        }),
        it({
          name: 'keeps every file, with all its cases, within the budget whenever the budget holds each file with one case',
          fn: async () => {
            for (const workers of workerCounts) {
              for (const budget of [
                workers * fileTasks,
                workers * 100,
                workers * 1_000,
              ]) {
                const cases = casesWithin({
                  budget,
                  workers,
                  fileTasks,
                  caseTasks,
                  wanted,
                });
                expect(cases).toBeGreaterThanOrEqual(1);
                expect(cases).toBeLessThanOrEqual(wanted);
                expect(workers * ((fileTasks - caseTasks) + (cases * caseTasks))).toBeLessThanOrEqual(budget);
              }
            }
          },
        }),
      ],
    }),
    describe({
      name: mentionsProcessLimit.name,
      children: [
        it({
          name: 'recognises the ways Node and the C library report a start that the limit refused, and nothing else',
          fn: async () => {
            expect(mentionsProcessLimit('Error: spawn podman EAGAIN')).toBe(true);
            expect(mentionsProcessLimit('Assertion failed: (0) == (uv_thread_create(t.get(), start_thread, this))')).toBe(true);
            expect(mentionsProcessLimit('pthread_create failed')).toBe(true);
            expect(mentionsProcessLimit('fork: Resource temporarily unavailable')).toBe(true);
            expect(mentionsProcessLimit('ENOENT: no such file or directory')).toBe(false);
          },
        }),
      ],
    }),
  ],
});
