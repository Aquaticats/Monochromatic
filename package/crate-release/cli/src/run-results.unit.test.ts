import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findRunAddress,
  parseDispatchedRuns,
  parseRunCreatedAt,
  parseRunReference,
} from './run-results.ts';

/**
 Listed runs of the publish workflow, newest first as `gh run list` prints them.
 */
const listed = [
  { databaseId: 903, headSha: 'cccccccccccccccccccccccccccccccccccccccc', createdAt: '2026-11-01T11:00:00Z' },
  { databaseId: 902, headSha: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', createdAt: '2026-11-01T10:05:00Z' },
  { databaseId: 901, headSha: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', createdAt: '2026-11-01T09:00:00Z' },
];
const listing = JSON.stringify(listed);

await describe({
  name: '',
  children: [
    describe({
      name: parseRunReference.name,
      children: [
        it({
          name: 'accepts a run identifier as it is',
          fn: async () => {
            expect(parseRunReference('123456789')).toEqual({ found: true, value: '123456789' });
          },
        }),
        ...[
          'https://github.com/Aquaticats/Monochromatic/actions/runs/123456789',
          'https://github.com/o/r/actions/runs/123456789/job/55',
          'https://github.com/o/r/actions/runs/123456789?pr=1',
          'https://ghe.example.com/o/r/actions/runs/123456789#summary',
        ].map(function addressCase(address) {
          return it({
            name: `reads the identifier out of the address ${address}`,
            fn: async () => {
              expect(parseRunReference(address)).toEqual({ found: true, value: '123456789' });
            },
          });
        }),
        ...[
          '',
          'abc',
          '12 3',
          '-1',
          '1;2',
          'https://github.com/o/r/pull/123',
          'https://github.com/o/r/actions/runs/abc',
          'http://github.com/o/r/actions/runs/123',
          'https://github.com/o/actions/runs/123',
          ' https://github.com/o/r/actions/runs/123',
          'https://github.com/o/r/actions/runs/123 extra',
        ].map(function refusedCase(reference) {
          return it({
            name: `refuses ${JSON.stringify(reference)} and says which forms are accepted`,
            fn: async () => {
              const result = parseRunReference(reference);
              expect(result.found).toBe(false);
              expect(result.found ? '' : result.reason).toContain('run address such as');
            },
          });
        }),
      ],
    }),
    describe({
      name: findRunAddress.name,
      children: [
        it({
          name: 'finds the address gh prints after its confirmation line',
          fn: async () => {
            const output = '✓ Created workflow_dispatch event for crate-release.yml at main\nhttps://github.com/o/r/actions/runs/42\n\n'
              + 'To see the created workflow run, try: gh run view 42\n';
            expect(findRunAddress(output)).toEqual({ found: true, value: 'https://github.com/o/r/actions/runs/42' });
          },
        }),
        it({
          name: 'reports that gh printed no address, as older servers do not return one',
          fn: async () => {
            const result = findRunAddress('✓ Created workflow_dispatch event for crate-release.yml at main\n');
            expect(result.found).toBe(false);
            expect(result.found ? '' : result.reason).toContain('did not print the address');
          },
        }),
        it({
          name: 'does not take an address of something other than a run',
          fn: async () => {
            expect(findRunAddress('https://github.com/o/r/actions/workflows/crate-release.yml').found).toBe(false);
          },
        }),
      ],
    }),
    describe({
      name: parseRunCreatedAt.name,
      children: [
        it({
          name: 'reads the creation time of the run',
          fn: async () => {
            expect(parseRunCreatedAt('{"createdAt":"2026-11-01T10:00:00Z","status":"completed"}')).toEqual({
              found: true,
              value: '2026-11-01T10:00:00Z',
            });
          },
        }),
        ...[
          '',
          'not json',
          '[]',
          'null',
          '{}',
          '{"createdAt":5}',
          '{"createdAt":"yesterday"}',
        ].map(function missingCase(body) {
          return it({
            name: `reports no creation time for ${JSON.stringify(body)}`,
            fn: async () => {
              expect(parseRunCreatedAt(body).found).toBe(false);
            },
          });
        }),
      ],
    }),
    describe({
      name: parseDispatchedRuns.name,
      children: [
        it({
          name: 'lists every run oldest first with its identifier, commit and creation time, whatever order gh printed',
          fn: async () => {
            const result = parseDispatchedRuns(listing);
            expect(result.found).toBe(true);
            expect(result.found ? result.value.runs : []).toEqual(
              listed.toReversed().map(function runOf(entry) {
                return { runId: String(entry.databaseId), headSha: entry.headSha, createdAt: entry.createdAt };
              }),
            );
            expect(result.found ? result.value.unreadableEntries : -1).toBe(0);
          },
        }),
        it({
          name: 'orders by instant, not by text, so an offset spelling does not change the order',
          fn: async () => {
            const body = JSON.stringify([
              { databaseId: 1, headSha: 'a', createdAt: '2026-11-01T12:00:00+02:00' },
              { databaseId: 2, headSha: 'b', createdAt: '2026-11-01T10:30:00Z' },
            ]);
            const result = parseDispatchedRuns(body);
            expect((result.found ? result.value.runs : []).map(function idOf(run) {
              return run.runId;
            })).toEqual(['1', '2']);
          },
        }),
        it({
          name: 'finds no runs in an empty listing, which is not an error',
          fn: async () => {
            expect(parseDispatchedRuns('[]')).toEqual({ found: true, value: { runs: [], unreadableEntries: 0 } });
          },
        }),
        ...[
          '',
          'not json',
          '{"databaseId":1}',
          'null',
        ].map(function malformedCase(body) {
          return it({
            name: `reports a miss for the malformed listing ${JSON.stringify(body)}`,
            fn: async () => {
              expect(parseDispatchedRuns(body).found).toBe(false);
            },
          });
        }),
        it({
          name: 'skips malformed entries, counts them, and still reads the others',
          fn: async () => {
            const readable = { databaseId: 8, headSha: 'dddddddddddddddddddddddddddddddddddddddd', createdAt: '2026-11-01T11:30:00Z' };
            const malformed = [
              1,
              null,
              { databaseId: '5', headSha: 'a', createdAt: '2026-11-01T11:00:00Z' },
              { databaseId: 6, headSha: 'a', createdAt: 'later' },
              { databaseId: 7, createdAt: '2026-11-01T11:00:00Z' },
            ];
            const result = parseDispatchedRuns(JSON.stringify([...malformed, readable]));
            expect(result.found ? result.value.runs : []).toEqual([
              { runId: String(readable.databaseId), headSha: readable.headSha, createdAt: readable.createdAt },
            ]);
            expect(result.found ? result.value.unreadableEntries : -1).toBe(malformed.length);
          },
        }),
      ],
    }),
  ],
});
