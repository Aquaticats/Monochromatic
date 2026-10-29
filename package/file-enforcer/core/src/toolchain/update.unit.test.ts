/**
 Unit tests for `toolchain:update` over recorded resolver responses.

 Every case replays fixture payloads shaped like the real upstream sources
 (GitHub release JSON with asset digests, crates.io, the npm registry, PyPI,
 nodejs.org sums files, the Adoptium API, the rustup channel manifest,
 Google's Android repository XML, anaconda.org, and the zig index). No test
 touches the network.
 */

import { createHash, } from 'node:crypto';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  mapGithubRows,
  parseShasums,
  versionFromTag,
  type Fetcher,
} from './update.resolvers.ts';
import {
  patchPlanVersions,
  renderPlanArtifactsTs,
} from './update.render.ts';
import {
  applyResolutions,
  formatReport,
  resolveAll,
  type UpdateOutcome,
} from './update.ts';

//region Fixtures

/**
 Builds a fetcher that serves recorded bodies and refuses unknown URLs.
 
 @param options - URL to recorded response text pairs
 
 @returns Fetcher replaying the recordings
 */
function fixtureFetch(
  recorded: Readonly<Record<string, string>>,
): Fetcher {
  return async (url,) => {
    /**
     Recorded body for this URL.
     */
    const text = recorded[url];
    if (text === undefined)
      throw new Error(`unexpected fetch: ${url}`,);
    return {
      ok: true,
      status: 200,
      text,
    };
  };
}

/**
 Plan entry fields a fixture case sets over the base entry.
 */
type FixtureOverrides = {
  readonly id?: ToolPlanEntry['id'];
  readonly request?: ToolPlanEntry['request'];
  readonly version?: ToolPlanEntry['version'];
  readonly route?: ToolPlanEntry['route'];
  readonly packageId?: string;
  readonly bins?: ToolPlanEntry['bins'];
  readonly env?: ToolPlanEntry['env'];
  readonly artifacts?: ToolPlanEntry['artifacts'];
};

/**
 Builds a plan entry for one fixture case.
 
 @param overrides - Fields to set over the base entry
 
 @returns A plan entry shaped like the real plan
 */
function fixtureEntry(overrides: FixtureOverrides,): ToolPlanEntry {
  return {
    id: 'fixture-tool',
    request: { track: 'latest' },
    version: '1.0.0',
    route: 'bin-sourced',
    bins: ['fixture-tool'],
    options: {},
    env: {},
    artifacts: {},
    ...overrides,
  };
}

//endregion Fixtures

await describe({
  name: 'toolchain update',
  children: [
    //region Tag mapping

    describe({
      name: versionFromTag.name,
      children: [
        it({
          name: 'carries the v prefix and bare tags',
          fn: async () => {
            expect(versionFromTag({
              oldTag: 'v1.9.0',
              oldVersion: '1.9.0',
              newTag: 'v2.0.0',
            },),)
              .toBe('2.0.0',);
            expect(versionFromTag({
              oldTag: '15.2.0',
              oldVersion: '15.2.0',
              newTag: '16.0.0',
            },),)
              .toBe('16.0.0',);
          },
        },),
        it({
          name: 'carries a name prefix like bun-v',
          fn: async () => {
            expect(versionFromTag({
              oldTag: 'bun-v1.3.14',
              oldVersion: '1.3.14',
              newTag: 'bun-v1.3.15',
            },),)
              .toBe('1.3.15',);
          },
        },),
        it({
          name: 'refuses tags that do not carry the old version',
          fn: async () => {
            let caught: unknown;
            try {
              versionFromTag({
                oldTag: 'jdk-21.0.12%2B8',
                oldVersion: 'temurin-21.0.12+8.0.LTS',
                newTag: 'jdk-21.0.13%2B9',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('cannot derive a version from tag jdk-21.0.13%2B9',);
          },
        },),
      ],
    },),

    describe({
      name: mapGithubRows.name,
      children: [
        it({
          name: 'maps recorded rows onto release assets by name and digest',
          fn: async () => {
            /**
             Recorded rows of one platform pair.
             */
            const rows: ToolArtifacts = {
              'linux-x64': {
                url: 'https://github.com/example/demo/releases/download/v1.0.0/demo_1.0.0_linux_x64.tar.gz',
                sha256: '0'.repeat(64,),
              },
            };
            expect(await mapGithubRows({
              rows,
              release: {
                tag_name: 'v1.1.0',
                assets: [
                  {
                    name: 'demo_1.1.0_linux_x64.tar.gz',
                    digest: `sha256:${'a'.repeat(64,)}`,
                    browser_download_url: 'https://github.com/example/demo/releases/download/v1.1.0/demo_1.1.0_linux_x64.tar.gz',
                  },
                ],
              },
              oldVersion: '1.0.0',
              newVersion: '1.1.0',
            },),)
              .toEqual({
                'linux-x64': {
                  url: 'https://github.com/example/demo/releases/download/v1.1.0/demo_1.1.0_linux_x64.tar.gz',
                  sha256: 'a'.repeat(64,),
                },
              },);
          },
        },),
        it({
          name: 'throws when no asset matches the recorded name',
          fn: async () => {
            let caught: unknown;
            try {
              await mapGithubRows({
                rows: {
                  'linux-x64': {
                    url: 'https://github.com/example/demo/releases/download/v1.0.0/demo_1.0.0_linux_x64.tar.gz',
                    sha256: '0'.repeat(64,),
                  },
                },
                release: { tag_name: 'v1.1.0', assets: [], },
                oldVersion: '1.0.0',
                newVersion: '1.1.0',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
          },
        },),
      ],
    },),

    describe({
      name: parseShasums.name,
      children: [
        it({
          name: 'reads digests by file name',
          fn: async () => {
            expect(parseShasums({
              text: `${
                'a'.repeat(64,)
              }  node-v26.12.1-linux-x64.tar.gz\n${
                'b'.repeat(64,)
              } *node-v26.12.1-linux-arm64-musl.tar.gz\n`,
            },)
              .get('node-v26.12.1-linux-x64.tar.gz',),)
              .toBe('a'.repeat(64,),);
          },
        },),
      ],
    },),

    //endregion Tag mapping

    //region Family resolutions

    describe({
      name: resolveAll.name,
      children: [
        it({
          name: 'resolves the GitHub family from the latest release',
          fn: async () => {
            /**
             Entry pinned to a recorded release row.
             */
            const entry = fixtureEntry({
              id: 'demo',
              version: '1.0.0',
              artifacts: {
                'linux-x64': {
                  url: 'https://github.com/example/demo/releases/download/v1.0.0/demo_1.0.0_linux_x64.tar.gz',
                  sha256: '0'.repeat(64,),
                },
              },
            },);
            /**
             Outcomes replaying the release payload.
             */
            const outcomes = await resolveAll({
              tools: [entry],
              fetcher: fixtureFetch({
                'https://api.github.com/repos/example/demo/releases/latest': JSON.stringify({
                  tag_name: 'v1.1.0',
                  assets: [
                    {
                      name: 'demo_1.1.0_linux_x64.tar.gz',
                      digest: `sha256:${'a'.repeat(64,)}`,
                      browser_download_url: 'https://github.com/example/demo/releases/download/v1.1.0/demo_1.1.0_linux_x64.tar.gz',
                    },
                  ],
                },),
              },),
            },);
            expect(outcomes[0]?.kind,)
              .toBe('updated',);
            expect(outcomes[0]?.newVersion,)
              .toBe('1.1.0',);
            expect(outcomes[0]?.artifacts['linux-x64']?.sha256,)
              .toBe('a'.repeat(64,),);
          },
        },),

        it({
          name: 'reports pins with newer-availability and never moves them',
          fn: async () => {
            /**
             Pinned entry whose upstream moved.
             */
            const entry = fixtureEntry({
              id: 'demo',
              request: { pinned: '1.0.0', },
              version: '1.0.0',
              artifacts: {
                'linux-x64': {
                  url: 'https://github.com/example/demo/releases/download/v1.0.0/demo_1.0.0_linux_x64.tar.gz',
                  sha256: '0'.repeat(64,),
                },
              },
            },);
            /**
             Outcomes replaying a newer release.
             */
            const outcomes = await resolveAll({
              tools: [entry],
              fetcher: fixtureFetch({
                'https://api.github.com/repos/example/demo/releases/latest': JSON.stringify({
                  tag_name: 'v1.1.0',
                  assets: [
                    {
                      name: 'demo_1.1.0_linux_x64.tar.gz',
                      digest: `sha256:${'a'.repeat(64,)}`,
                      browser_download_url: 'https://github.com/example/demo/releases/download/v1.1.0/demo_1.1.0_linux_x64.tar.gz',
                    },
                  ],
                },),
              },),
            },);
            expect(outcomes[0]?.kind,)
              .toBe('pinned',);
            expect(outcomes[0]?.note,)
              .toBe('newer available: 1.1.0',);
            expect(outcomes[0]?.artifacts,)
              .toEqual(entry.artifacts,);
          },
        },),

        it({
          name: 'resolves cargo tools from crates.io',
          fn: async () => {
            /**
             Outcomes replaying a crate record.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'cargo:demo',
                route: 'cargo-binstall',
                packageId: 'demo-crate',
                bins: ['demo'],
              },)],
              fetcher: fixtureFetch({
                'https://crates.io/api/v1/crates/demo-crate': JSON.stringify({
                  crate: {
                    max_stable_version: '9.9.9',
                  },
                },),
              },),
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('9.9.9',);
          },
        },),

        it({
          name: 'resolves npm tools from the registry dist-tag',
          fn: async () => {
            /**
             Outcomes replaying a packument.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'npm:demo',
                route: 'mpm-pnpm',
                packageId: '@scope/demo',
                bins: ['demo'],
              },)],
              fetcher: fixtureFetch({
                'https://registry.npmjs.org/@scope/demo': JSON.stringify({
                  'dist-tags': { latest: '3.2.1', },
                },),
              },),
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('3.2.1',);
          },
        },),

        it({
          name: 'resolves uvx tools from PyPI',
          fn: async () => {
            /**
             Outcomes replaying a project record.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'pipx:demo',
                route: 'mpm-uvx',
                packageId: 'demo-tool',
                bins: ['demo'],
              },)],
              fetcher: fixtureFetch({
                'https://pypi.org/pypi/demo-tool/json': JSON.stringify({
                  info: { version: '0.5.0', },
                },),
              },),
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('0.5.0',);
          },
        },),

        it({
          name: 'resolves the rust channel stamp from the nightly manifest',
          fn: async () => {
            /**
             Outcomes replaying the channel manifest.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'rust',
                request: { channel: 'nightly', },
                route: 'rustup-toolchain',
                version: 'nightly-2026-01-01',
                bins: [],
              },)],
              fetcher: fixtureFetch({
                'https://static.rust-lang.org/dist/channel-rust-nightly.toml': 'manifest-version = "2"\ndate = "2026-10-26"\n',
              },),
            },);
            expect(outcomes[0]?.kind,)
              .toBe('updated',);
            expect(outcomes[0]?.newVersion,)
              .toBe('nightly-2026-10-26',);
          },
        },),

        it({
          name: 'resolves node from the release index and sums files',
          fn: async () => {
            /**
             Outcomes replaying index and sums bodies.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'node',
                version: '26.12.0',
                route: 'archive',
                bins: ['node'],
                artifacts: {
                  'linux-x64': {
                    url: 'https://nodejs.org/dist/v26.12.0/node-v26.12.0-linux-x64.tar.gz',
                    sha256: '0'.repeat(64,),
                  },
                  'linux-x64-musl': {
                    url: 'https://unofficial-builds.nodejs.org/download/release/v26.12.0/node-v26.12.0-linux-x64-musl.tar.gz',
                    sha256: '1'.repeat(64,),
                  },
                },
              },)],
              fetcher: fixtureFetch({
                'https://nodejs.org/dist/index.json': JSON.stringify([
                  { version: 'v26.12.1', },
                ],),
                'https://nodejs.org/dist/v26.12.1/SHASUMS256.txt': `${
                  'a'.repeat(64,)
                }  node-v26.12.1-linux-x64.tar.gz\n`,
                'https://unofficial-builds.nodejs.org/download/release/v26.12.1/SHASUMS256.txt': `${
                  'b'.repeat(64,)
                }  node-v26.12.1-linux-x64-musl.tar.gz\n`,
              },),
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('26.12.1',);
            expect(outcomes[0]?.artifacts['linux-x64'],)
              .toEqual({
                url: 'https://nodejs.org/dist/v26.12.1/node-v26.12.1-linux-x64.tar.gz',
                sha256: 'a'.repeat(64,),
              },);
            expect(outcomes[0]?.artifacts['linux-x64-musl']?.url,)
              .toBe('https://unofficial-builds.nodejs.org/download/release/v26.12.1/node-v26.12.1-linux-x64-musl.tar.gz',);
          },
        },),

        it({
          name: 'resolves java from Adoptium within the temurin-21 channel',
          fn: async () => {
            /**
             Recorded Adoptium release name plus the GitHub asset table.
             */
            const adoptium = JSON.stringify([
              { release_name: 'jdk-21.0.13+9', },
            ],);
            const adoptiumRelease = JSON.stringify({
              tag_name: 'jdk-21.0.13+9',
              assets: [
                {
                  name: 'OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz',
                  digest: `sha256:${'c'.repeat(64,)}`,
                  browser_download_url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.13%2B9/OpenJDK21U-jdk_x64_linux_hotspot_21.0.13_9.tar.gz',
                },
                {
                  name: 'OpenJDK21U-jdk_aarch64_linux_hotspot_21.0.13_9.tar.gz',
                  digest: `sha256:${'d'.repeat(64,)}`,
                  browser_download_url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.13%2B9/OpenJDK21U-jdk_aarch64_linux_hotspot_21.0.13_9.tar.gz',
                },
              ],
            },);
            /**
             Outcomes replaying the Adoptium API.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'java',
                request: { channel: 'temurin-21', },
                route: 'archive',
                version: 'temurin-21.0.12+8.0.LTS',
                bins: ['java'],
                artifacts: {
                  'linux-x64': {
                    url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12_8.tar.gz',
                    sha256: '0'.repeat(64,),
                  },
                  'linux-arm64': {
                    url: 'https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.12%2B8/OpenJDK21U-jdk_aarch64_linux_hotspot_21.0.12_8.tar.gz',
                    sha256: '1'.repeat(64,),
                  },
                },
              },)],
              fetcher: fixtureFetch({
                'https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=linux': adoptium,
                'https://api.github.com/repos/adoptium/temurin21-binaries/releases/tags/jdk-21.0.13%2B9': adoptiumRelease,
              },),
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('temurin-21.0.13+9.0.LTS',);
            expect(outcomes[0]?.artifacts['linux-x64']?.sha256,)
              .toBe('c'.repeat(64,),);
          },
        },),

        it({
          name: 'keeps android rows when the cmdline-tools revision is current',
          fn: async () => {
            /**
             Entry whose recorded archive is still the newest.
             */
            const entry = fixtureEntry({
              id: 'android-sdk',
              route: 'android-sdk',
              version: '23.0',
              bins: ['android'],
              artifacts: {
                'linux-x64': {
                  url: 'https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip',
                  sha256: '0'.repeat(64,),
                },
              },
            },);
            /**
             Outcomes replaying the repository XML.
             */
            const outcomes = await resolveAll({
              tools: [entry],
              fetcher: fixtureFetch({
                'https://dl.google.com/android/repository/repository2-3.xml': [
                  '<remotePackage path="cmdline-tools;23.0">',
                  '<revision><major>23</major><minor>0</minor></revision>',
                  '<archive><complete><size>1</size>',
                  '<checksum type="sha1">e025</checksum>',
                  '<url>commandlinetools-linux-16111833_latest.zip</url>',
                  '</complete></archive></remotePackage>',
                ].join('\n',),
              },),
            },);
            expect(outcomes[0]?.kind,)
              .toBe('unchanged',);
            expect(outcomes[0]?.artifacts,)
              .toEqual(entry.artifacts,);
          },
        },),

        it({
          name: 'hashes the new android archive when the revision moves',
          fn: async () => {
            /**
             Outcomes replaying a newer repository and the zip bytes.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'android-sdk',
                route: 'android-sdk',
                version: '23.0',
                bins: ['android'],
                artifacts: {
                  'linux-x64': {
                    url: 'https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip',
                    sha256: '0'.repeat(64,),
                  },
                },
              },)],
              fetcher: async (url,) => {
                if (url.endsWith('.xml',)) {
                  return {
                    ok: true,
                    status: 200,
                    text: [
                      '<remotePackage path="cmdline-tools;24.0">',
                      '<revision><major>24</major><minor>0</minor></revision>',
                      '<archive><complete><size>1</size>',
                      '<checksum type="sha1">abcd</checksum>',
                      '<url>commandlinetools-linux-17000000_latest.zip</url>',
                      '</complete></archive></remotePackage>',
                    ].join('\n',),
                  };
                }
                return {
                  ok: true,
                  status: 200,
                  text: '',
                  bytes: new Uint8Array([1, 2, 3,],),
                };
              },
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('24.0',);
            /**
             SHA-256 of the three fixture bytes.
             */
            /**
             SHA-256 of the three fixture bytes.
             */
            expect(outcomes[0]?.artifacts['linux-x64']?.sha256,)
              .toBe(createHash('sha256',)
                .update(new Uint8Array([1, 2, 3,],),)
                .digest('hex',),);
            expect(outcomes[0]?.artifacts['linux-x64']?.url,)
              .toBe('https://dl.google.com/android/repository/commandlinetools-linux-17000000_latest.zip',);
          },
        },),

        it({
          name: 'resolves conda tools from the anaconda.org record',
          fn: async () => {
            /**
             Outcomes replaying a package record.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'conda:demo',
                route: 'archive',
                version: '0.1.0',
                bins: ['demo'],
                artifacts: {
                  'linux-x64': {
                    url: 'https://conda.anaconda.org/conda-forge/linux-64/demo-0.1.0-h123_0.conda',
                    sha256: '0'.repeat(64,),
                  },
                },
              },)],
              fetcher: fixtureFetch({
                'https://api.anaconda.org/package/conda-forge/demo': JSON.stringify({
                  latest_version: '0.2.0',
                  files: [
                    {
                      basename: 'linux-64/demo-0.2.0-h456_0.conda',
                      version: '0.2.0',
                      sha256: 'd'.repeat(64,),
                      attrs: { subdir: 'linux-64', build_number: 0, },
                    },
                  ],
                },),
              },),
            },);
            expect(outcomes[0]?.newVersion,)
              .toBe('0.2.0',);
            expect(outcomes[0]?.artifacts['linux-x64'],)
              .toEqual({
                url: 'https://conda.anaconda.org/conda-forge/linux-64/demo-0.2.0-h456_0.conda',
                sha256: 'd'.repeat(64,),
              },);
          },
        },),

        it({
          name: 'records a failed resolution without aborting the run',
          fn: async () => {
            /**
             Outcomes replaying an empty release.
             */
            const outcomes = await resolveAll({
              tools: [fixtureEntry({
                id: 'demo',
                artifacts: {
                  'linux-x64': {
                    url: 'https://github.com/example/demo/releases/download/v1.0.0/demo.tar.gz',
                    sha256: '0'.repeat(64,),
                  },
                },
              },)],
              fetcher: fixtureFetch({
                'https://api.github.com/repos/example/demo/releases/latest': JSON.stringify({
                  tag_name: 'v2.0.0',
                  assets: [],
                },),
              },),
            },);
            expect(outcomes[0]?.kind,)
              .toBe('failed',);
            expect(outcomes[0]?.note,)
              .toContain('no release asset matches',);
          },
        },),
      ],
    },),

    //endregion Family resolutions

    //region Report and apply

    describe({
      name: formatReport.name,
      children: [
        it({
          name: 'names every outcome kind and summarizes the counts',
          fn: async () => {
            /**
             Report for one outcome of each kind.
             */
            const report = formatReport({
              outcomes: [
                {
                  id: 'a',
                  kind: 'updated',
                  oldVersion: '1.0.0',
                  newVersion: '1.1.0',
                  artifacts: {},
                },
                {
                  id: 'b',
                  kind: 'unchanged',
                  oldVersion: '2.0.0',
                  newVersion: '2.0.0',
                  artifacts: {},
                },
                {
                  id: 'c',
                  kind: 'pinned',
                  oldVersion: '3.0.0',
                  newVersion: '3.0.0',
                  artifacts: {},
                  note: 'newer available: 3.1.0',
                },
                {
                  id: 'd',
                  kind: 'failed',
                  oldVersion: '4.0.0',
                  newVersion: '4.0.0',
                  artifacts: {},
                  note: 'boom',
                },
              ] satisfies readonly UpdateOutcome[],
            },);
            expect(report,)
              .toContain('a 1.0.0 -> 1.1.0',);
            expect(report,)
              .toContain('b unchanged',);
            expect(report,)
              .toContain('c pinned (newer available: 3.1.0)',);
            expect(report,)
              .toContain('d failed: boom',);
            expect(report,)
              .toContain('1 updated, 1 unchanged, 1 pinned, 1 failed',);
          },
        },),
      ],
    },),

    describe({
      name: applyResolutions.name,
      children: [
        it({
          name: 'moves only updated entries and keeps every pin intact',
          fn: async () => {
            /**
             Base entries for one updated and one pinned tool.
             */
            const pinned = fixtureEntry({
              id: 'pinned-tool',
              request: { pinned: '1.0.0', },
              version: '1.0.0',
            },);
            const moving = fixtureEntry({
              id: 'moving-tool',
              version: '1.0.0',
            },);
            /**
             Entries after applying fixture outcomes.
             */
            const applied = applyResolutions({
              tools: [pinned, moving],
              outcomes: [
                {
                  id: 'pinned-tool',
                  kind: 'pinned',
                  oldVersion: '1.0.0',
                  newVersion: '1.0.0',
                  artifacts: pinned.artifacts,
                },
                {
                  id: 'moving-tool',
                  kind: 'updated',
                  oldVersion: '1.0.0',
                  newVersion: '2.0.0',
                  artifacts: {
                    'linux-x64': {
                      url: 'https://example/moved.tar.gz',
                      sha256: 'e'.repeat(64,),
                    },
                  },
                },
              ],
            },);
            expect(applied[0],)
              .toEqual(pinned,);
            expect(applied[1]?.version,)
              .toBe('2.0.0',);
            expect(applied[1]?.artifacts['linux-x64']?.url,)
              .toBe('https://example/moved.tar.gz',);
          },
        },),

        it({
          name: 'carries packageId and env pins along with a moved resolution',
          fn: async () => {
            /**
             Entry whose package id and env pin the dated toolchain name.
             */
            const rustLike = fixtureEntry({
              id: 'rust-like',
              request: { channel: 'nightly' },
              version: 'nightly-2026-10-21',
              packageId: 'nightly-2026-10-21',
              env: { RUSTUP_TOOLCHAIN: 'nightly-2026-10-21' },
            },);
            /**
             Entry whose env embeds the resolved version in a root-relative path.
             */
            const javaLike = fixtureEntry({
              id: 'java-like',
              version: 'temurin-21.0.12+8.0.LTS',
              env: { JAVA_HOME: 'java/temurin-21.0.12+8.0.LTS' },
            },);
            /**
             Entries after applying fixture outcomes.
             */
            const applied = applyResolutions({
              tools: [rustLike, javaLike],
              outcomes: [
                {
                  id: 'rust-like',
                  kind: 'updated',
                  oldVersion: 'nightly-2026-10-21',
                  newVersion: 'nightly-2026-10-28',
                  artifacts: {},
                },
                {
                  id: 'java-like',
                  kind: 'updated',
                  oldVersion: 'temurin-21.0.12+8.0.LTS',
                  newVersion: 'temurin-21.0.13+9.0.LTS',
                  artifacts: {},
                },
              ],
            },);
            expect(applied[0]?.version,)
              .toBe('nightly-2026-10-28',);
            expect(applied[0]?.packageId,)
              .toBe('nightly-2026-10-28',);
            expect(applied[0]?.env.RUSTUP_TOOLCHAIN,)
              .toBe('nightly-2026-10-28',);
            expect(applied[1]?.env.JAVA_HOME,)
              .toBe('java/temurin-21.0.13+9.0.LTS',);
            expect((applied[1] !== undefined) && Object.hasOwn(applied[1], 'packageId',),)
              .toBe(false,);
          },
        },),
      ],
    },),

    //endregion Report and apply

    //region Plan rendering

    describe({
      name: renderPlanArtifactsTs.name,
      children: [
        it({
          name: 'renders sorted tool and platform keys in stable shape',
          fn: async () => {
            /**
             First render of a two-tool map.
             */
            const once = renderPlanArtifactsTs({
              artifacts: {
                'zeta': {
                  'linux-x64': {
                    url: 'https://example/z.tar.gz',
                    sha256: '1'.repeat(64,),
                  },
                },
                'alpha': {
                  'linux-x64': {
                    url: 'https://example/a.tar.gz',
                    sha256: '2'.repeat(64,),
                  },
                  'linux-arm64': {
                    url: 'https://example/a-arm.tar.gz',
                    sha256: '3'.repeat(64,),
                  },
                },
              },
            },);
            expect(once,)
              .toBe(renderPlanArtifactsTs({
                artifacts: {
                  'alpha': {
                    'linux-arm64': {
                      url: 'https://example/a-arm.tar.gz',
                      sha256: '3'.repeat(64,),
                    },
                    'linux-x64': {
                      url: 'https://example/a.tar.gz',
                      sha256: '2'.repeat(64,),
                    },
                  },
                  'zeta': {
                    'linux-x64': {
                      url: 'https://example/z.tar.gz',
                      sha256: '1'.repeat(64,),
                    },
                  },
                },
              },),);
            expect(once.indexOf("'alpha'",),)
              .toBeLessThan(once.indexOf("'zeta'",),);
            expect(once.indexOf("'linux-arm64'",),)
              .toBeLessThan(once.indexOf("'linux-x64'",),);
          },
        },),
      ],
    },),

    describe({
      name: patchPlanVersions.name,
      children: [
        it({
          name: 'rewrites the version field of the named entry only',
          fn: async () => {
            /**
             Patched plan text for a two-entry source.
             */
            const patched = patchPlanVersions({
              source: [
                '  {',
                "    id: 'alpha',",
                "    request: { track: 'latest' },",
                "    version: '1.0.0',",
                '  },',
                '  {',
                "    id: 'beta',",
                "    request: { track: 'latest' },",
                "    version: '2.0.0',",
                '  },',
              ].join('\n',),
              versions: new Map([
                ['beta', '2.1.0',],
              ],),
            },);
            expect(patched,)
              .toContain("version: '1.0.0',",);
            expect(patched,)
              .toContain("version: '2.1.0',",);
            expect(patched.includes("version: '2.0.0',",),)
              .toBe(false,);
          },
        },),

        it({
          name: 'carries pin literals with the version and leaves comment prose',
          fn: async () => {
            /**
             Patched plan text for an entry whose package id and env pin the
             resolved version string, beside a comment citing it.
             */
            const patched = patchPlanVersions({
              source: [
                '  {',
                "    id: 'demo',",
                "    // resolved to '1.0.0' from the channel manifest",
                "    version: '1.0.0',",
                "    packageId: '1.0.0',",
                "    env: { TOOLCHAIN_HOME: 'toolchains/1.0.0' },",
                '  },',
              ].join('\n',),
              versions: new Map([
                ['demo', '2.0.0',],
              ],),
            },);
            expect(patched,)
              .toContain("version: '2.0.0',",);
            expect(patched,)
              .toContain("packageId: '2.0.0',",);
            expect(patched,)
              .toContain("env: { TOOLCHAIN_HOME: 'toolchains/2.0.0' },",);
            expect(patched,)
              .toContain("// resolved to '1.0.0' from the channel manifest",);
          },
        },),

        it({
          name: 'rewrites several entries in one pass and refuses overlapping entries',
          fn: async () => {
            /**
             Patched text for three entries where the edits arrive out of source order.
             */
            const patched = patchPlanVersions({
              source: [
                '  {', "    id: 'a',", "    version: '1.0.0',", '  },',
                '  {', "    id: 'b',", "    version: '2.0.0',", '  },',
                '  {', "    id: 'c',", "    version: '3.0.0',", '  },',
              ].join('\n',),
              versions: new Map([
                ['c', '3.1.0',],
                ['a', '1.1.0',],
              ],),
            },);
            expect(patched,)
              .toContain("version: '1.1.0',",);
            expect(patched,)
              .toContain("version: '2.0.0',",);
            expect(patched,)
              .toContain("version: '3.1.0',",);
            let caught: unknown;
            try {
              patchPlanVersions({
                source: "  {\n    id: 'outer',\n    note: { id: 'inner', },\n    version: '1.0.0',\n  },",
                versions: new Map([
                  ['outer', '2.0.0',],
                  ['inner', '2.0.0',],
                ],),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).message,)
              .toContain('starts inside the previous edited entry',);
          },
        },),

        it({
          name: 'throws naming the entry when it or its version field is missing',
          fn: async () => {
            let missingEntry: unknown;
            try {
              patchPlanVersions({
                source: "  {\n    id: 'alpha',\n    version: '1.0.0',\n  },",
                versions: new Map([
                  ['beta', '2.0.0',],
                ],),
              },);
            }
            catch (error) {
              missingEntry = error;
            }
            let missingVersion: unknown;
            try {
              patchPlanVersions({
                source: "  {\n    id: 'alpha',\n    request: { track: 'latest' },\n  },",
                versions: new Map([
                  ['alpha', '2.0.0',],
                ],),
              },);
            }
            catch (error) {
              missingVersion = error;
            }
            expect((missingEntry as Error).message,)
              .toContain('no entry found for plan entry beta',);
            expect((missingVersion as Error).message,)
              .toContain('no version field found for plan entry alpha',);
          },
        },),
      ],
    },),

    //endregion Plan rendering
  ],
},);
