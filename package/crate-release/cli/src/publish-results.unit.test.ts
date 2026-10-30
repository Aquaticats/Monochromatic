import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  describeDuration,
  expandTemplate,
  findMissingAssets,
  formatPublishResult,
  parseAssetNames,
  parseCratesIoCreatedAt,
  selectRunId,
} from './publish-results.ts';

/**
 Hash that a listed run was built from.
 */
const fullSha = 'abcdef0123456789abcdef0123456789abcdef01';

/**
 Run listing in the shape `gh run list --json databaseId,headSha` prints.
 */
const listing = JSON.stringify([
  { databaseId: 300, headSha: '1111111111111111111111111111111111111111' },
  { databaseId: 200, headSha: fullSha },
  { databaseId: 100, headSha: fullSha },
]);

/**
 Both tarballs the release is expected to hold.
 */
const expectedAssets: readonly string[] = ['c-0.4.10-x86_64-unknown-linux-gnu.tar.gz', 'c-0.4.10-aarch64-unknown-linux-gnu.tar.gz'];

await describe({
  name: '',
  children: [
    describe({
      name: selectRunId.name,
      children: [
        it({
          name: 'picks the first listed run whose hash starts with the given prefix',
          fn: async () => {
            expect(selectRunId({ listing, sha: 'abcdef0' })).toBe('200');
          },
        }),
        it({
          name: 'accepts the whole hash',
          fn: async () => {
            expect(selectRunId({ listing, sha: fullSha })).toBe('200');
          },
        }),
        it({
          name: 'finds nothing for a prefix that no run has',
          fn: async () => {
            expect(selectRunId({ listing, sha: 'fffffff' })).toBeUndefined();
          },
        }),
        it({
          name: 'does not match a hash longer than the run hash',
          fn: async () => {
            expect(selectRunId({ listing, sha: `${fullSha}0` })).toBeUndefined();
          },
        }),
        it({
          name: 'finds nothing in an empty listing',
          fn: async () => {
            expect(selectRunId({ listing: '[]', sha: 'abcdef0' })).toBeUndefined();
          },
        }),
        ...['', 'not json', '{"databaseId":1}', 'null', '[1,"x",null]', '[{"headSha":5,"databaseId":1}]', '[{"headSha":"abcdef0"}]'].map(
          function malformedCase(text) {
            return it({
              name: `finds nothing in the malformed listing ${JSON.stringify(text)}`,
              fn: async () => {
                expect(selectRunId({ listing: text, sha: 'abcdef0' })).toBeUndefined();
              },
            });
          },
        ),
        it({
          name: 'skips a malformed entry and still finds a later run',
          fn: async () => {
            const text = JSON.stringify([null, { databaseId: 'x', headSha: fullSha }, { databaseId: 7, headSha: fullSha }]);
            expect(selectRunId({ listing: text, sha: 'abcdef0' })).toBe('7');
          },
        }),
      ],
    }),
    describe({
      name: parseCratesIoCreatedAt.name,
      children: [
        it({
          name: 'reads the creation time of the published version',
          fn: async () => {
            expect(parseCratesIoCreatedAt('{"version":{"num":"0.4.10","created_at":"2026-10-29T12:00:00.123456+00:00"}}'))
              .toBe('2026-10-29T12:00:00.123456+00:00');
          },
        }),
        it({
          name: 'ignores a creation time outside the version object',
          fn: async () => {
            expect(parseCratesIoCreatedAt('{"crate":{"created_at":"2020-01-01"},"version":{}}')).toBeUndefined();
          },
        }),
        ...['', 'not json', '{"errors":[{"detail":"Not Found"}]}', '{"version":null}', '{"version":{"created_at":5}}', '[]', 'null'].map(
          function absentCase(text) {
            return it({
              name: `reports no creation time for ${JSON.stringify(text)}`,
              fn: async () => {
                expect(parseCratesIoCreatedAt(text)).toBeUndefined();
              },
            });
          },
        ),
      ],
    }),
    describe({
      name: parseAssetNames.name,
      children: [
        it({
          name: 'lists the asset names in order',
          fn: async () => {
            expect(parseAssetNames('{"assets":[{"name":"a.tar.gz"},{"name":"b.tar.gz"}]}')).toEqual(['a.tar.gz', 'b.tar.gz']);
          },
        }),
        it({
          name: 'skips assets without a text name',
          fn: async () => {
            expect(parseAssetNames('{"assets":[{"name":"a"},{"name":5},{}, null]}')).toEqual(['a']);
          },
        }),
        ...['', 'not json', '{}', '{"assets":null}', '[]', 'null'].map(function emptyCase(text) {
          return it({
            name: `lists nothing for ${JSON.stringify(text)}`,
            fn: async () => {
              expect(parseAssetNames(text)).toEqual([]);
            },
          });
        }),
      ],
    }),
    describe({
      name: expandTemplate.name,
      children: [
        it({
          name: 'fills the crate and version placeholders wherever they stand',
          fn: async () => {
            expect(expandTemplate({ template: '{crate}-v{version}', crateName: 'c', version: '0.4.10' })).toBe('c-v0.4.10');
            expect(expandTemplate({ template: '{crate}/{version}/{version}', crateName: 'c', version: '1.0.0' })).toBe('c/1.0.0/1.0.0');
          },
        }),
        it({
          name: 'leaves a template without placeholders as it is',
          fn: async () => {
            expect(expandTemplate({ template: 'fixed', crateName: 'c', version: '1.0.0' })).toBe('fixed');
          },
        }),
        it({
          name: 'treats replacement-pattern characters in the values literally',
          fn: async () => {
            expect(expandTemplate({ template: '{crate}-{version}', crateName: 'a$&b$1', version: '$`1.0.0' })).toBe('a$&b$1-$`1.0.0');
          },
        }),
        it({
          name: 'does not expand a placeholder that a value contains',
          fn: async () => {
            expect(expandTemplate({ template: '{crate}-{version}', crateName: '{version}', version: '1.0.0' })).toBe('{version}-1.0.0');
          },
        }),
      ],
    }),
    describe({
      name: findMissingAssets.name,
      children: [
        it({
          name: 'reports the expected assets the release lacks, in the expected order',
          fn: async () => {
            expect(findMissingAssets({ expected: ['a', 'b', 'c'], actual: ['c', 'a'] })).toEqual(['b']);
          },
        }),
        it({
          name: 'reports nothing when every expected asset is present, extra assets included',
          fn: async () => {
            expect(findMissingAssets({ expected: ['a'], actual: ['x', 'a'] })).toEqual([]);
          },
        }),
        it({
          name: 'reports every expected asset when the release has none',
          fn: async () => {
            expect(findMissingAssets({ expected: ['a', 'b'], actual: [] })).toEqual(['a', 'b']);
          },
        }),
        it({
          name: 'compares whole names, not prefixes',
          fn: async () => {
            expect(findMissingAssets({ expected: ['a.tar.gz'], actual: ['a.tar.gz.sig'] })).toEqual(['a.tar.gz']);
          },
        }),
      ],
    }),
    describe({
      name: describeDuration.name,
      children: [
        ...[
          [1, '1 second'],
          [45, '45 seconds'],
          [60, '1 minute'],
          [90, '1 minute 30 seconds'],
          [900, '15 minutes'],
          [3661, '61 minutes 1 second'],
        ].map(function durationCase([seconds, text]) {
          return it({
            name: `writes ${String(seconds)} seconds as ${String(text)}`,
            fn: async () => {
              expect(describeDuration(seconds as number)).toBe(text);
            },
          });
        }),
      ],
    }),
    describe({
      name: formatPublishResult.name,
      children: [
        it({
          name: 'reports a complete publish and succeeds',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 0,
              version: '0.4.10',
              createdAt: '2026-10-29T12:00:00+00:00',
              tag: 'c-v0.4.10',
              expectedAssets,
              assetNames: [...expectedAssets, 'c-0.4.10-sha256sums.txt'],
              releaseError: '',
            });
            expect(result.ok).toBe(true);
            expect(result.lines).toEqual([
              'watch exit 0',
              'crates.io 0.4.10: 2026-10-29T12:00:00+00:00',
              `release c-v0.4.10 assets: ${[...expectedAssets, 'c-0.4.10-sha256sums.txt'].join(', ')}`,
              'PUBLISHED',
            ]);
          },
        }),
        it({
          name: 'fails and names the reason when the workflow run failed',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 1,
              version: '0.4.10',
              createdAt: '2026-10-29',
              tag: 't',
              expectedAssets: [],
              assetNames: [],
              releaseError: '',
            });
            expect(result.ok).toBe(false);
            expect(result.lines.at(-1)).toContain('PUBLISH CHECK FAILED');
            expect(result.lines.at(-1)).toContain('workflow run exited 1');
          },
        }),
        it({
          name: 'fails when the version is not on crates.io',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 0,
              version: '0.4.10',
              createdAt: undefined,
              tag: 't',
              expectedAssets: [],
              assetNames: [],
              releaseError: '',
            });
            expect(result.ok).toBe(false);
            expect(result.lines).toContain('crates.io 0.4.10: not found');
            expect(result.lines.at(-1)).toContain('not on crates.io');
          },
        }),
        it({
          name: 'fails and lists each expected asset the release lacks',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 0,
              version: '0.4.10',
              createdAt: 'x',
              tag: 'c-v0.4.10',
              expectedAssets,
              assetNames: [expectedAssets[0] ?? ''],
              releaseError: '',
            });
            expect(result.ok).toBe(false);
            expect(result.lines.at(-1)).toContain(expectedAssets[1] ?? '');
            expect(result.lines.at(-1)).not.toContain(expectedAssets[0] ?? '');
          },
        }),
        it({
          name: 'shows the release lookup error when no assets could be listed',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 0,
              version: '0.4.10',
              createdAt: 'x',
              tag: 'c-v0.4.10',
              expectedAssets,
              assetNames: [],
              releaseError: 'release not found',
            });
            expect(result.lines).toContain('release c-v0.4.10 assets: release not found');
            expect(result.ok).toBe(false);
          },
        }),
        it({
          name: 'shows none when there are no assets and no error',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 0,
              version: '1.0.0',
              createdAt: 'x',
              tag: 't',
              expectedAssets: [],
              assetNames: [],
              releaseError: '',
            });
            expect(result.lines).toContain('release t assets: none');
            expect(result.ok).toBe(true);
          },
        }),
        it({
          name: 'lists every failure reason together',
          fn: async () => {
            const result = formatPublishResult({
              watchStatus: 2,
              version: '1.0.0',
              createdAt: undefined,
              tag: 't',
              expectedAssets: ['a'],
              assetNames: [],
              releaseError: '',
            });
            const last = result.lines.at(-1) ?? '';
            expect(last).toContain('workflow run exited 2');
            expect(last).toContain('not on crates.io');
            expect(last).toContain('missing release assets: a');
          },
        }),
      ],
    }),
  ],
});
