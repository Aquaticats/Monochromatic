import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  campaignCaseTasks,
  casesWithin,
  fileWorkersWithin,
  mentionsProcessLimit,
  plannedShare,
  testFileTasks,
} from './process-limits.ts';

/**
 Limits from the tightest whose planned share holds one file with one case to a generous one.
 */
const limits: readonly number[] = [
  Math.ceil((testFileTasks + campaignCaseTasks) / plannedShare),
  100,
  300,
  1_000,
  5_000,
];

/**
 Processors of a large host.
 */
const processors = 28;

await describe({
  name: 'sizing the test run to a process limit',
  children: [
    describe({
      name: fileWorkersWithin.name,
      children: [
        it({
          name: 'uses every processor when there is no limit, and never fewer than one file',
          fn: async () => {
            expect(fileWorkersWithin({ processors })).toBe(processors);
            expect(fileWorkersWithin({
              limit: 1,
              processors,
            })).toBe(1);
          },
        }),
        it({
          name: 'keeps the files, each with one case, within the planned share of any limit that holds one',
          fn: async () => {
            for (const limit of limits) {
              const workers = fileWorkersWithin({
                limit,
                processors,
              });
              expect(workers).toBeGreaterThanOrEqual(1);
              expect(workers).toBeLessThanOrEqual(processors);
              expect(workers * (testFileTasks + campaignCaseTasks)).toBeLessThanOrEqual(limit * plannedShare);
            }
          },
        }),
      ],
    }),
    describe({
      name: casesWithin.name,
      children: [
        it({
          name: 'keeps the wanted width when there is no limit, and at least one case under any limit',
          fn: async () => {
            const wanted = 4;
            expect(casesWithin({
              workers: processors,
              wanted,
            })).toBe(wanted);
            expect(casesWithin({
              limit: 1,
              workers: processors,
              wanted,
            })).toBe(1);
          },
        }),
        it({
          name: 'keeps every file, with all its cases, within the planned share of the limit when the file workers are sized by the same limit',
          fn: async () => {
            const wanted = 4;
            for (const limit of limits) {
              const workers = fileWorkersWithin({
                limit,
                processors,
              });
              const cases = casesWithin({
                limit,
                workers,
                wanted,
              });
              expect(cases).toBeGreaterThanOrEqual(1);
              expect(cases).toBeLessThanOrEqual(wanted);
              expect(workers * (testFileTasks + (cases * campaignCaseTasks))).toBeLessThanOrEqual(limit * plannedShare);
            }
          },
        }),
        it({
          name: 'gives a roomy limit the wanted width',
          fn: async () => {
            const wanted = 4;
            expect(casesWithin({
              limit: 100_000,
              workers: 1,
              wanted,
            })).toBe(wanted);
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
