import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isRecentTimestamp,
  NO_MATCH,
  parseArtifactDir,
  parseFailureDir,
} from './linter-artifacts-timestamp.ts';

await describe({
  name: parseArtifactDir.name,
  children: [
    it({
      name: 'parses an initial-pass directory into probe, pass, and timestamp',
      fn: async () => {
        expect(parseArtifactDir('csv-rfc4180-initial-2026-03-06T12-00-00.000Z',),).toEqual({
          probe: 'csv-rfc4180',
          pass: 'initial',
          timestamp: '2026-03-06T12-00-00.000Z',
        },);
      },
    },),
    it({
      name: 'parses a fix-pass directory',
      fn: async () => {
        expect(parseArtifactDir('my-probe-fix-2026-01-01T00-00-00.000Z',),).toEqual({
          probe: 'my-probe',
          pass: 'fix',
          timestamp: '2026-01-01T00-00-00.000Z',
        },);
      },
    },),
    it({
      name: 'prefers the initial marker over a fix marker when both are valid',
      fn: async () => {
        expect(parseArtifactDir('a-fix-b-initial-2026-01-01',),).toEqual({
          probe: 'a-fix-b',
          pass: 'initial',
          timestamp: '2026-01-01',
        },);
      },
    },),
    it({
      name: 'takes the rightmost valid marker so the longest probe prefix wins',
      fn: async () => {
        expect(parseArtifactDir('a-fix-b-fix-2026-01-01',),).toEqual({
          probe: 'a-fix-b',
          pass: 'fix',
          timestamp: '2026-01-01',
        },);
      },
    },),
    it({
      name: 'scans left to an earlier marker when the rightmost tail is not year-anchored',
      fn: async () => {
        expect(parseArtifactDir('a-fix-2026-01-01-fix-notyear',),).toEqual({
          probe: 'a',
          pass: 'fix',
          timestamp: '2026-01-01-fix-notyear',
        },);
      },
    },),
    it({
      name: 'returns NO_MATCH when no pass marker is present',
      fn: async () => {
        expect(parseArtifactDir('foo-bar',),).toBe(NO_MATCH,);
      },
    },),
    it({
      name: 'returns NO_MATCH for the empty string',
      fn: async () => {
        expect(parseArtifactDir('',),).toBe(NO_MATCH,);
      },
    },),
    it({
      name: 'returns NO_MATCH when the marker sits at index 0 (empty probe)',
      fn: async () => {
        expect(parseArtifactDir('-fix-2026-01-01',),).toBe(NO_MATCH,);
      },
    },),
    it({
      name: 'returns NO_MATCH when the tail after the marker is not year-anchored',
      fn: async () => {
        expect(parseArtifactDir('probe-fix-notyear',),).toBe(NO_MATCH,);
      },
    },),
    it({
      name: 'handles tens of thousands of markers without overflowing the stack',
      fn: async () => {
        const markerCount = 50_000;
        const longName = `a${'-fix-z'.repeat(markerCount,)}`;
        expect(parseArtifactDir(longName,),).toBe(NO_MATCH,);
      },
    },),
  ],
},);

await describe({
  name: parseFailureDir.name,
  children: [
    it({
      name: 'parses a failure directory into its timestamp',
      fn: async () => {
        expect(parseFailureDir('failure-2026-03-06T12-00-00.000Z',),).toEqual({
          timestamp: '2026-03-06T12-00-00.000Z',
        },);
      },
    },),
    it({
      name: 'returns NO_MATCH without the failure- prefix',
      fn: async () => {
        expect(parseFailureDir('other-2026-03-06T12-00-00.000Z',),).toBe(NO_MATCH,);
      },
    },),
    it({
      name: 'returns NO_MATCH when the prefix is present but the tail is not year-anchored',
      fn: async () => {
        expect(parseFailureDir('failure-notyear',),).toBe(NO_MATCH,);
      },
    },),
    it({
      name: 'returns NO_MATCH for a bare failure- prefix with no timestamp',
      fn: async () => {
        expect(parseFailureDir('failure-',),).toBe(NO_MATCH,);
      },
    },),
  ],
},);

await describe({
  name: isRecentTimestamp.name,
  children: [
    it({
      name: 'returns true when the restored timestamp is after the cutoff',
      fn: async () => {
        const cutoff = new Date('2026-03-06T11:00:00.000Z',).getTime();
        expect(isRecentTimestamp({ rawTimestamp: '2026-03-06T12-00-00.000Z', cutoff, },),).toBe(true,);
      },
    },),
    it({
      name: 'returns true when the restored timestamp equals the cutoff exactly',
      fn: async () => {
        const cutoff = new Date('2026-03-06T12:00:00.000Z',).getTime();
        expect(isRecentTimestamp({ rawTimestamp: '2026-03-06T12-00-00.000Z', cutoff, },),).toBe(true,);
      },
    },),
    it({
      name: 'returns false when the restored timestamp is before the cutoff',
      fn: async () => {
        const cutoff = new Date('2026-03-06T13:00:00.000Z',).getTime();
        expect(isRecentTimestamp({ rawTimestamp: '2026-03-06T12-00-00.000Z', cutoff, },),).toBe(false,);
      },
    },),
    it({
      name: 'returns false when the slug cannot be parsed into a date',
      fn: async () => {
        expect(isRecentTimestamp({ rawTimestamp: 'not-a-date', cutoff: 0, },),).toBe(false,);
      },
    },),
  ],
},);
