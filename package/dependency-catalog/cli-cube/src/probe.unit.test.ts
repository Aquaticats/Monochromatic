/**
 Tests for the probe orchestrator and the field-level pure helpers.
 
 `probeAll` is exercised against a pre-populated file cache so no
 `gh`, npm registry, or downloads endpoint is hit. Each fixture
 covers one of the five `UnknownReason` branches plus the
 all-known case:
 
 - absent (every GH-derived field known)
 - `'no-repo'` (manifest lacks `repository`)
 - `'non-github'` (repository.url points at a non-GH host)
 - `'monorepo'` (`repository.directory` set, Linguist deliberately skipped)
 - `'private-or-404'` (GH host, Linguist fetch returns null)
 
 Pure helpers (`parseRepository`, `classifyLicense`,
 `resolveVersion`) are tested directly.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { createCache, } from './cache.ts';
import type { CatalogEntry, } from './catalog.ts';
import {
  classifyLicense,
  parseRepository,
  REPO_UNPARSEABLE,
  resolveVersion,
} from './probe-fields.ts';
import { probeAll, } from './probe.ts';

/**
 Builds an isolated cache root and pre-populates the JSON files
 the field probes would normally fetch + cache.
 
 @param entries - Map from `<name>/<version>` to the JSON to drop in.
   The `_repo` entries use `<owner>/<repo>` instead of `<name>`.
 
 @returns Cache root path and an async-disposable cleanup.
 */
async function tempPopulatedCache(
  { entries, }: {
    entries: Record<string, { field: string; value: unknown; }[]>;
  },
): Promise<{
  rootDir: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}> {
  const rootDir = await mkdtemp(join(scratchRoot(), 'deps-cube-probe-',),);
  /** Pre-fill operations queued from `entries`, awaited together so each cache entry is written without serial-await overhead. */
  const fillTasks = Object.entries(entries,).map(async function fillOne(
    [nameAndVersion, fields,],
  ) {
    const slashIdx = nameAndVersion.lastIndexOf('/',);
    const name = nameAndVersion.slice(
      0,
      slashIdx,
    );
    const version = nameAndVersion.slice(slashIdx + 1,);
    const dir = join(
      rootDir,
      name,
    );
    await mkdir(
      dir,
      { recursive: true, },
    );
    const payload: Record<string, { value: unknown; fetchedAt: number; }> = {};
    for (const { field, value, } of fields) {
      payload[field] = {
        value,
        fetchedAt: Date.now(),
      };
    }
    await writeFile(
      join(
        dir,
        `${version}.json`,
      ),
      JSON.stringify(
        payload,
        null,
        2,
      ),
      'utf8',
    );
  },);
  await Promise.all(fillTasks,);
  return {
    rootDir,
    [Symbol.asyncDispose]: async function dispose() {
      await rm(rootDir, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'probe',
  children: [
    //region parseRepository
    it({
      name: 'parseRepository handles plain github.com URLs',
      fn: async () => {
        const info = parseRepository('https://github.com/preactjs/preact',);
        expect(info,).toMatchObject({
          host: 'github',
          owner: 'preactjs',
          repo: 'preact',
        },);
      },
    },),

    it({
      name: 'parseRepository handles git+https URLs with .git suffix',
      fn: async () => {
        const info = parseRepository('git+https://github.com/owner/repo.git',);
        expect(info,).toMatchObject({
          host: 'github',
          owner: 'owner',
          repo: 'repo',
        },);
      },
    },),

    it({
      name: 'parseRepository handles github:owner/repo shorthand',
      fn: async () => {
        const info = parseRepository('github:foo/bar',);
        expect(info,).toMatchObject({
          host: 'github',
          owner: 'foo',
          repo: 'bar',
        },);
      },
    },),

    it({
      name: 'parseRepository preserves directory from object form',
      fn: async () => {
        const info = parseRepository({
          type: 'git',
          url: 'https://github.com/lezer-parser/common.git',
          directory: 'package/common',
        },);
        expect(info,).toMatchObject({
          host: 'github',
          directory: 'package/common',
        },);
      },
    },),

    it({
      name: 'parseRepository returns other host for non-GitHub URLs',
      fn: async () => {
        const info = parseRepository('https://gitlab.com/owner/repo',);
        expect(info,).not.toBe(REPO_UNPARSEABLE,);
        if (info === REPO_UNPARSEABLE)
          return;
        expect(info.host,).toBe('other',);
      },
    },),

    it({
      name: 'parseRepository returns REPO_UNPARSEABLE for undefined and empty',
      fn: async () => {
        expect(parseRepository(undefined,),).toBe(REPO_UNPARSEABLE,);
        expect(parseRepository('',),).toBe(REPO_UNPARSEABLE,);
        expect(parseRepository({ url: '', },),).toBe(REPO_UNPARSEABLE,);
      },
    },),
    //endregion parseRepository

    //region classifyLicense
    it({
      name: 'classifyLicense maps MIT/Apache/BSD/ISC to permissive',
      fn: async () => {
        for (const lic of ['MIT', 'Apache-2.0', 'BSD-3-Clause', 'BSD-2-Clause', 'ISC',])
          expect(classifyLicense(lic,),).toBe('permissive',);
      },
    },),

    it({
      name: 'classifyLicense maps GPL/LGPL/AGPL to copyleft',
      fn: async () => {
        for (const lic of ['GPL-3.0', 'LGPL-2.1', 'AGPL-3.0-or-later',])
          expect(classifyLicense(lic,),).toBe('copyleft',);
      },
    },),

    it({
      name: 'classifyLicense maps custom-license markers to non-oss',
      fn: async () => {
        for (const lic of [
          'UNLICENSED',
          'PROPRIETARY Foo',
          'SEE LICENSE IN LICENSE.md',
          'see license in license.md',
          'SEE LGPL-3.0-or-later.txt IN LGPL-3.0-or-later.txt.md',
        ])
          expect(classifyLicense(lic,),).toBe('non-oss',);
      },
    },),

    it({
      name: 'classifyLicense maps missing/unknown to unknown',
      fn: async () => {
        expect(classifyLicense(undefined,),).toBe('unknown',);
        expect(classifyLicense('',),).toBe('unknown',);
        expect(classifyLicense('Funky-Custom-1.0',),).toBe('unknown',);
      },
    },),

    it({
      name: 'classifyLicense accepts object license form',
      fn: async () => {
        expect(classifyLicense({ type: 'MIT', },),).toBe('permissive',);
      },
    },),
    //endregion classifyLicense

    //region resolveVersion
    it({
      name: 'resolveVersion returns the pinned version when it exists',
      fn: async () => {
        const pkg = {
          versions: { '1.2.3': {}, },
          'dist-tags': { latest: '1.2.3', },
        };
        expect(resolveVersion({ range: '1.2.3', pkg, },),).toBe('1.2.3',);
      },
    },),

    it({
      name: 'resolveVersion falls back to dist-tags.latest for ranges',
      fn: async () => {
        const pkg = {
          versions: { '1.0.0': {}, '2.0.0': {}, },
          'dist-tags': { latest: '2.0.0', },
        };
        expect(resolveVersion({ range: '^1.0.0', pkg, },),).toBe('2.0.0',);
      },
    },),
    //endregion resolveVersion

    //region probeAll: each UnknownReason branch
    it({
      name: 'probeAll resolves a healthy GH-hosted entry to absent unknownReason',
      fn: async () => {
        await using cacheDir = await tempPopulatedCache({
          entries: {
            'preact/_pkg': [
              {
                field: 'registry',
                value: {
                  'dist-tags': { latest: '10.26.0', },
                  time: { created: '2014-01-01T00:00:00.000Z', },
                  versions: {
                    '10.26.0': {
                      license: 'MIT',
                      repository: 'https://github.com/preactjs/preact',
                      dependencies: {},
                      dist: { unpackedSize: 250_000, },
                    },
                  },
                },
              },
              {
                field: 'downloads',
                value: { downloads: 5_000_000, },
              },
            ],
            'preactjs/preact/_repo': [
              { field: 'languages', value: { TypeScript: 900, JavaScript: 100, }, },
              { field: 'pushed_at',
                value: new Date(Date.now() - (14 * 86_400_000),).toISOString(), },
            ],
            'preact/10.26.0': [
              { field: 'transitive', value: 0, },
            ],
          },
        },);
        const cache = createCache({ rootDir: cacheDir.rootDir, },);
        const entries: readonly CatalogEntry[] = [
          { catalogKey: 'preact', npmName: 'preact', range: '^10.26.0', },
        ];
        const [probe,] = await probeAll({ entries, cache, },);
        expect(probe,).toBeDefined();
        expect(probe?.unknownReason,).toBeUndefined();
        expect(probe?.tsRatioOrNull,).toBeCloseTo(0.9, 6,);
        expect(probe?.licenseClass,).toBe('permissive',);
        expect(probe?.isLeaf,).toBe(true,);
        expect(probe?.weeklyDownloads,).toBe(5_000_000,);
      },
    },),

    it({
      name: 'probeAll marks manifests without repository as no-repo',
      fn: async () => {
        await using cacheDir = await tempPopulatedCache({
          entries: {
            'norepo/_pkg': [
              {
                field: 'registry',
                value: {
                  'dist-tags': { latest: '1.0.0', },
                  time: { created: '2020-01-01T00:00:00.000Z', },
                  versions: {
                    '1.0.0': {
                      license: 'MIT',
                      dependencies: {},
                      dist: { unpackedSize: 1_000, },
                    },
                  },
                },
              },
              { field: 'downloads', value: { downloads: 10, }, },
            ],
            'norepo/1.0.0': [
              { field: 'transitive', value: 0, },
            ],
          },
        },);
        const cache = createCache({ rootDir: cacheDir.rootDir, },);
        const [probe,] = await probeAll({
          entries: [
            { catalogKey: 'norepo', npmName: 'norepo', range: '^1.0.0', },
          ],
          cache,
        },);
        expect(probe?.unknownReason,).toBe('no-repo',);
        expect(probe?.tsRatioOrNull,).toBeUndefined();
      },
    },),

    it({
      name: 'probeAll marks non-GitHub hosts as non-github',
      fn: async () => {
        await using cacheDir = await tempPopulatedCache({
          entries: {
            'gitlab-pkg/_pkg': [
              {
                field: 'registry',
                value: {
                  'dist-tags': { latest: '1.0.0', },
                  time: { created: '2020-01-01T00:00:00.000Z', },
                  versions: {
                    '1.0.0': {
                      license: 'MIT',
                      repository: 'https://gitlab.com/owner/repo',
                      dependencies: {},
                      dist: { unpackedSize: 1_000, },
                    },
                  },
                },
              },
              { field: 'downloads', value: { downloads: 10, }, },
            ],
            'gitlab-pkg/1.0.0': [
              { field: 'transitive', value: 0, },
            ],
          },
        },);
        const cache = createCache({ rootDir: cacheDir.rootDir, },);
        const [probe,] = await probeAll({
          entries: [
            { catalogKey: 'gitlab-pkg', npmName: 'gitlab-pkg', range: '^1.0.0', },
          ],
          cache,
        },);
        expect(probe?.unknownReason,).toBe('non-github',);
      },
    },),

    it({
      name:
        'probeAll marks monorepo-housed entries as monorepo when Linguist would mismeasure',
      fn: async () => {
        await using cacheDir = await tempPopulatedCache({
          entries: {
            '@lezer/common/_pkg': [
              {
                field: 'registry',
                value: {
                  'dist-tags': { latest: '1.0.0', },
                  time: { created: '2020-01-01T00:00:00.000Z', },
                  versions: {
                    '1.0.0': {
                      license: 'MIT',
                      repository: {
                        type: 'git',
                        url: 'https://github.com/lezer-parser/common',
                        directory: 'package/common',
                      },
                      dependencies: {},
                      dist: { unpackedSize: 50_000, },
                    },
                  },
                },
              },
              { field: 'downloads', value: { downloads: 200_000, }, },
            ],
            '@lezer/common/1.0.0': [
              { field: 'transitive', value: 0, },
            ],
            // Note: no 'lezer-parser/common/_repo' entry, so Linguist + pushed_at
            // are treated as cache miss → null (with no network they stay null).
          },
        },);
        const cache = createCache({ rootDir: cacheDir.rootDir, },);
        const [probe,] = await probeAll({
          entries: [
            { catalogKey: '@lezer/common', npmName: '@lezer/common', range: '^1.0.0', },
          ],
          cache,
        },);
        expect(probe?.isMonorepoHoused,).toBe(true,);
        expect(probe?.unknownReason,).toBe('monorepo',);
        expect(probe?.tsRatioOrNull,).toBeUndefined();
      },
    },),

    it({
      name: 'probeAll handles failed entries via the failedProbe stub (private-or-404)',
      fn: async ({ sinon, },) => {
        // No cache entries for this package; manifest fetch will fail because
        // the test runs without network credentials. The orchestrator catches
        // the failure and emits a failedProbe with unknownReason='private-or-404'.
        await using cacheDir = await tempPopulatedCache({ entries: {}, },);
        const cache = createCache({ rootDir: cacheDir.rootDir, },);
        sinon.stub(globalThis, 'fetch',).rejects(
          new Error('network disabled for test',),
        );
        const [probe,] = await probeAll({
          entries: [
            { catalogKey: 'definitely-not-real', npmName: 'definitely-not-real',
              range: '^1.0.0', },
          ],
          cache,
        },);
        expect(probe?.unknownReason,).toBe('private-or-404',);
        expect(probe?.tsRatioOrNull,).toBeUndefined();
        expect(probe?.weeklyDownloads,).toBe(0,);
      },
      timeout: 10_000,
    },),
    //endregion probeAll: each UnknownReason branch
  ],
},);
