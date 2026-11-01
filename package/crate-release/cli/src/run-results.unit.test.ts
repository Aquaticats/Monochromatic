import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findRunAddress,
  parseRunCreatedAt,
  parseRunReference,
  selectDispatchedRun,
} from './run-results.ts';

/**
 Listed runs of the publish workflow, newest first as `gh run list` prints them.
 */
const listing = JSON.stringify([
  { databaseId: 903, headSha: 'cccccccccccccccccccccccccccccccccccccccc', createdAt: '2026-11-01T11:00:00Z' },
  { databaseId: 902, headSha: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb', createdAt: '2026-11-01T10:05:00Z' },
  { databaseId: 901, headSha: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', createdAt: '2026-11-01T09:00:00Z' },
]);

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
      name: selectDispatchedRun.name,
      children: [
        it({
          name: 'picks the earliest run created at or after the release run, not the newest',
          fn: async () => {
            expect(selectDispatchedRun({ listing, notBefore: '2026-11-01T10:00:00Z' })).toEqual({
              found: true,
              value: { runId: '902', headSha: 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb' },
            });
          },
        }),
        it({
          name: 'takes a run created at exactly the release run time',
          fn: async () => {
            const result = selectDispatchedRun({ listing, notBefore: '2026-11-01T10:05:00Z' });
            expect(result.found ? result.value.runId : '').toBe('902');
          },
        }),
        it({
          name: 'compares instants, not text, so an offset spelling does not change the answer',
          fn: async () => {
            const result = selectDispatchedRun({ listing, notBefore: '2026-11-01T12:00:00+02:00' });
            expect(result.found ? result.value.runId : '').toBe('902');
          },
        }),
        it({
          name: 'skips every run older than the release run, which a manual dry run could be',
          fn: async () => {
            const result = selectDispatchedRun({ listing, notBefore: '2026-11-01T10:06:00Z' });
            expect(result.found ? result.value.runId : '').toBe('903');
            const none = selectDispatchedRun({ listing, notBefore: '2026-11-01T11:01:00Z' });
            expect(none.found).toBe(false);
            expect(none.found ? '' : none.reason).toContain('2026-11-01T11:01:00Z');
          },
        }),
        it({
          name: 'finds nothing in an empty listing',
          fn: async () => {
            expect(selectDispatchedRun({ listing: '[]', notBefore: '2026-11-01T10:00:00Z' }).found).toBe(false);
          },
        }),
        ...[
          '',
          'not json',
          '{"databaseId":1}',
          'null',
        ].map(function malformedCase(body) {
          return it({
            name: `finds nothing in the malformed listing ${JSON.stringify(body)}`,
            fn: async () => {
              expect(selectDispatchedRun({ listing: body, notBefore: '2026-11-01T10:00:00Z' }).found).toBe(false);
            },
          });
        }),
        it({
          name: 'skips malformed entries and still finds a later run',
          fn: async () => {
            const body = JSON.stringify([
              1,
              null,
              { databaseId: '5', headSha: 'a', createdAt: '2026-11-01T11:00:00Z' },
              { databaseId: 6, headSha: 'a', createdAt: 'later' },
              { databaseId: 7, createdAt: '2026-11-01T11:00:00Z' },
              { databaseId: 8, headSha: 'dddddddddddddddddddddddddddddddddddddddd', createdAt: '2026-11-01T11:30:00Z' },
            ]);
            const result = selectDispatchedRun({ listing: body, notBefore: '2026-11-01T10:00:00Z' });
            expect(result.found ? result.value.runId : '').toBe('8');
          },
        }),
      ],
    }),
  ],
});
