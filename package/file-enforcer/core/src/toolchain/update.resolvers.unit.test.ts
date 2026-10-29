/**
 Unit tests for the `toolchain:update` family resolvers over recorded
 upstream responses.

 Every case replays fixture payloads shaped like the real sources
 (GitHub release JSON with asset digests, the npm registry packument, the
 Go module proxy `@latest` record, go.dev's download index, and the rustup
 channel manifests). No test touches the network.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ToolArtifacts, } from './plan.artifacts.data.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  mapGithubRows,
  resolveGoModule,
  resolveGoRuntime,
  resolveNpm,
  resolveRust,
  resolveRustup,
  resolveZigLatest,
  type Fetcher,
} from './update.resolvers.ts';

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
  readonly packageId?: ToolPlanEntry['packageId'];
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
    manager: null,
    packageId: null,
    bins: ['fixture-tool'],
    os: null,
    options: {},
    env: {},
    artifacts: {},
    ...overrides,
  };
}

//endregion Fixtures

await describe({
  name: 'toolchain update resolvers',
  children: [
    //region GitHub asset selection

    describe({
      name: mapGithubRows.name,
      children: [
        it({
          name: 'selects the single asset the pattern names when row names do not line up',
          fn: async () => {
            /**
             Recorded row whose asset name the new release no longer uses.
             */
            const rows: ToolArtifacts = {
              'linux-x64': {
                url: 'https://github.com/example/llamacpp/releases/download/v1.0.0/legacy-binary.zip',
                sha256: '0'.repeat(64,),
              },
            };
            expect(await mapGithubRows({
              rows,
              release: {
                tag_name: 'v1.1.0',
                assets: [
                  {
                    name: 'llama-b1234-ubuntu-rocm-gfx110X-x64.zip',
                    digest: `sha256:${'a'.repeat(64,)}`,
                    browser_download_url: 'https://github.com/example/llamacpp/releases/download/v1.1.0/llama-b1234-ubuntu-rocm-gfx110X-x64.zip',
                  },
                ],
              },
              oldVersion: '1.0.0',
              newVersion: '1.1.0',
              assetPattern: 'llama-*-ubuntu-rocm-gfx110X-x64.zip',
            },),)
              .toEqual({
                'linux-x64': {
                  url: 'https://github.com/example/llamacpp/releases/download/v1.1.0/llama-b1234-ubuntu-rocm-gfx110X-x64.zip',
                  sha256: 'a'.repeat(64,),
                },
              },);
          },
        },),
        it({
          name: 'throws when the pattern selects no asset',
          fn: async () => {
            let caught: unknown;
            try {
              await mapGithubRows({
                rows: {
                  'linux-x64': {
                    url: 'https://github.com/example/llamacpp/releases/download/v1.0.0/legacy-binary.zip',
                    sha256: '0'.repeat(64,),
                  },
                },
                release: {
                  tag_name: 'v1.1.0',
                  assets: [
                    {
                      name: 'unrelated.tar.gz',
                      digest: `sha256:${'a'.repeat(64,)}`,
                      browser_download_url: 'https://github.com/example/llamacpp/releases/download/v1.1.0/unrelated.tar.gz',
                    },
                  ],
                },
                oldVersion: '1.0.0',
                newVersion: '1.1.0',
                assetPattern: 'llama-*-ubuntu-rocm-gfx110X-x64.zip',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('llama-*-ubuntu-rocm-gfx110X-x64.zip',);
            expect((caught as Error).message,)
              .toContain('matches 0',);
          },
        },),
        it({
          name: 'throws when the pattern selects several assets',
          fn: async () => {
            let caught: unknown;
            try {
              await mapGithubRows({
                rows: {
                  'linux-x64': {
                    url: 'https://github.com/example/openi/releases/download/v1.0.0/legacy-binary.zip',
                    sha256: '0'.repeat(64,),
                  },
                },
                release: {
                  tag_name: 'v1.1.0',
                  assets: [
                    {
                      name: 'open-interpreter-package-a.zip',
                      digest: `sha256:${'a'.repeat(64,)}`,
                      browser_download_url: 'https://github.com/example/openi/releases/download/v1.1.0/open-interpreter-package-a.zip',
                    },
                    {
                      name: 'open-interpreter-package-b.zip',
                      digest: `sha256:${'b'.repeat(64,)}`,
                      browser_download_url: 'https://github.com/example/openi/releases/download/v1.1.0/open-interpreter-package-b.zip',
                    },
                  ],
                },
                oldVersion: '1.0.0',
                newVersion: '1.1.0',
                assetPattern: '*open-interpreter-package*',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('*open-interpreter-package*',);
            expect((caught as Error).message,)
              .toContain('matches 2',);
          },
        },),
        it({
          name: 'keeps name substitution when it matches even with a pattern present',
          fn: async () => {
            /**
             Rows whose version-substituted names both match release assets;
             the pattern alone would select two assets and throw.
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
                  {
                    name: 'demo_notes.txt',
                    digest: `sha256:${'b'.repeat(64,)}`,
                    browser_download_url: 'https://github.com/example/demo/releases/download/v1.1.0/demo_notes.txt',
                  },
                ],
              },
              oldVersion: '1.0.0',
              newVersion: '1.1.0',
              assetPattern: 'demo_*',
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
          name: 'does not let the glob prefix and suffix share characters',
          fn: async () => {
            let caught: unknown;
            try {
              await mapGithubRows({
                rows: {
                  'linux-x64': {
                    url: 'https://github.com/example/demo/releases/download/v1.0.0/legacy-binary.zip',
                    sha256: '0'.repeat(64,),
                  },
                },
                release: {
                  tag_name: 'v1.1.0',
                  assets: [
                    {
                      name: 'abc',
                      digest: `sha256:${'a'.repeat(64,)}`,
                      browser_download_url: 'https://github.com/example/demo/releases/download/v1.1.0/abc',
                    },
                  ],
                },
                oldVersion: '1.0.0',
                newVersion: '1.1.0',
                assetPattern: 'ab*bc',
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('matches 0',);
          },
        },),
      ],
    },),

    //endregion GitHub asset selection

    //region npm dist-tags

    describe({
      name: resolveNpm.name,
      children: [
        it({
          name: 'reads the latest dist-tag for the floating track',
          fn: async () => {
            expect(await resolveNpm({
              entry: fixtureEntry({
                id: 'npm:demo',
                route: 'mpm-pnpm',
                packageId: '@scope/demo',
                request: { track: 'latest' },
              },),
              fetcher: fixtureFetch({
                'https://registry.npmjs.org/@scope/demo': JSON.stringify({
                  'dist-tags': {
                    latest: '3.2.1',
                    next: '4.0.0-beta.1',
                  },
                },),
              },),
            },),)
              .toEqual({
                version: '3.2.1',
                artifacts: {},
              },);
          },
        },),
        it({
          name: 'reads the named channel dist-tag for a channel request',
          fn: async () => {
            expect(await resolveNpm({
              entry: fixtureEntry({
                id: 'npm:demo',
                route: 'mpm-pnpm',
                packageId: '@scope/demo',
                request: { channel: 'next' },
              },),
              fetcher: fixtureFetch({
                'https://registry.npmjs.org/@scope/demo': JSON.stringify({
                  'dist-tags': {
                    latest: '3.2.1',
                    next: '4.0.0-beta.1',
                  },
                },),
              },),
            },),)
              .toEqual({
                version: '4.0.0-beta.1',
                artifacts: {},
              },);
          },
        },),
        it({
          name: 'throws naming the tag and package when the dist-tag is missing',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveNpm({
                entry: fixtureEntry({
                  id: 'npm:demo',
                  route: 'mpm-pnpm',
                  packageId: '@scope/demo',
                  request: { channel: 'beta' },
                },),
                fetcher: fixtureFetch({
                  'https://registry.npmjs.org/@scope/demo': JSON.stringify({
                    'dist-tags': {
                      latest: '3.2.1',
                      next: '4.0.0-beta.1',
                    },
                  },),
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('beta',);
            expect((caught as Error).message,)
              .toContain('@scope/demo',);
          },
        },),
      ],
    },),

    //endregion npm dist-tags

    //region Go family

    describe({
      name: resolveGoRuntime.name,
      children: [
        it({
          name: 'keeps the recorded version and maps the linux archive rows',
          fn: async () => {
            expect(await resolveGoRuntime({
              entry: fixtureEntry({
                id: 'go',
                route: 'archive',
                version: '1.27.3',
                bins: ['go', 'gofmt'],
              },),
              fetcher: fixtureFetch({
                'https://go.dev/dl/?mode=json&include=all': JSON.stringify([
                  {
                    version: 'go1.28.0',
                    stable: true,
                    files: [
                      {
                        filename: 'go1.28.0.linux-amd64.tar.gz',
                        os: 'linux',
                        arch: 'amd64',
                        kind: 'archive',
                        sha256: 'c'.repeat(64,),
                      },
                    ],
                  },
                  {
                    version: 'go1.27.3',
                    stable: true,
                    files: [
                      {
                        filename: 'go1.27.3.linux-amd64.tar.gz',
                        os: 'linux',
                        arch: 'amd64',
                        kind: 'archive',
                        sha256: 'a'.repeat(64,),
                      },
                      {
                        filename: 'go1.27.3.linux-arm64.tar.gz',
                        os: 'linux',
                        arch: 'arm64',
                        kind: 'archive',
                        sha256: 'b'.repeat(64,),
                      },
                      {
                        filename: 'go1.27.3.src.tar.gz',
                        os: '',
                        arch: '',
                        kind: 'source',
                        sha256: 'd'.repeat(64,),
                      },
                    ],
                  },
                ],),
              },),
            },),)
              .toEqual({
                version: '1.27.3',
                artifacts: {
                  'linux-x64': {
                    url: 'https://go.dev/dl/go1.27.3.linux-amd64.tar.gz',
                    sha256: 'a'.repeat(64,),
                  },
                  'linux-arm64': {
                    url: 'https://go.dev/dl/go1.27.3.linux-arm64.tar.gz',
                    sha256: 'b'.repeat(64,),
                  },
                },
              },);
          },
        },),
        it({
          name: 'falls back to the newest stable release when the recorded version is gone',
          fn: async () => {
            expect(await resolveGoRuntime({
              entry: fixtureEntry({
                id: 'go',
                route: 'archive',
                version: '9.9.9',
                bins: ['go', 'gofmt'],
              },),
              fetcher: fixtureFetch({
                'https://go.dev/dl/?mode=json&include=all': JSON.stringify([
                  {
                    version: 'go1.29.0',
                    stable: false,
                    files: [
                      {
                        filename: 'go1.29.0.linux-amd64.tar.gz',
                        os: 'linux',
                        arch: 'amd64',
                        kind: 'archive',
                        sha256: 'e'.repeat(64,),
                      },
                    ],
                  },
                  {
                    version: 'go1.28.0',
                    stable: true,
                    files: [
                      {
                        filename: 'go1.28.0.linux-amd64.tar.gz',
                        os: 'linux',
                        arch: 'amd64',
                        kind: 'archive',
                        sha256: 'c'.repeat(64,),
                      },
                    ],
                  },
                ],),
              },),
            },),)
              .toEqual({
                version: '1.28.0',
                artifacts: {
                  'linux-x64': {
                    url: 'https://go.dev/dl/go1.28.0.linux-amd64.tar.gz',
                    sha256: 'c'.repeat(64,),
                  },
                },
              },);
          },
        },),
        it({
          name: 'throws when the chosen release lists no linux archive',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveGoRuntime({
                entry: fixtureEntry({
                  id: 'go',
                  route: 'archive',
                  version: '1.27.3',
                  bins: ['go', 'gofmt'],
                },),
                fetcher: fixtureFetch({
                  'https://go.dev/dl/?mode=json&include=all': JSON.stringify([
                    {
                      version: 'go1.27.3',
                      stable: true,
                      files: [
                        {
                          filename: 'go1.27.3.src.tar.gz',
                          os: '',
                          arch: '',
                          kind: 'source',
                          sha256: 'd'.repeat(64,),
                        },
                        {
                          filename: 'go1.27.3.windows-amd64.zip',
                          os: 'windows',
                          arch: 'amd64',
                          kind: 'archive',
                          sha256: 'f'.repeat(64,),
                        },
                      ],
                    },
                  ],),
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('go1.27.3',);
            expect((caught as Error).message,)
              .toContain('no linux archive',);
          },
        },),
        it({
          name: 'throws when the index carries neither the recorded version nor a stable release',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveGoRuntime({
                entry: fixtureEntry({
                  id: 'go',
                  route: 'archive',
                  version: '9.9.9',
                  bins: ['go', 'gofmt'],
                },),
                fetcher: fixtureFetch({
                  'https://go.dev/dl/?mode=json&include=all': JSON.stringify([
                    {
                      version: 'go1.30.0',
                      stable: false,
                      files: [],
                    },
                  ],),
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('9.9.9',);
          },
        },),
      ],
    },),

    describe({
      name: resolveGoModule.name,
      children: [
        it({
          name: 'strips one leading v from the proxy version tag',
          fn: async () => {
            expect(await resolveGoModule({
              entry: fixtureEntry({
                id: 'go:github.com/charmbracelet/crush',
                route: 'go-install',
                packageId: 'github.com/charmbracelet/crush',
                version: '0.97.0',
              },),
              fetcher: fixtureFetch({
                'https://proxy.golang.org/github.com/charmbracelet/crush/@latest': JSON.stringify({
                  Version: 'v0.98.1',
                  Time: '2026-10-27T00:00:00Z',
                },),
              },),
            },),)
              .toEqual({
                version: '0.98.1',
                artifacts: {},
              },);
          },
        },),
        it({
          name: 'reads the module path from the entry id when no packageId is set',
          fn: async () => {
            expect(await resolveGoModule({
              entry: fixtureEntry({
                id: 'go:github.com/example/tool',
                route: 'go-install',
                version: '1.0.0',
              },),
              fetcher: fixtureFetch({
                'https://proxy.golang.org/github.com/example/tool/@latest': JSON.stringify({
                  Version: 'v2.0.0',
                },),
              },),
            },),)
              .toEqual({
                version: '2.0.0',
                artifacts: {},
              },);
          },
        },),
        it({
          name: 'throws when the proxy rejects the query',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveGoModule({
                entry: fixtureEntry({
                  id: 'go:github.com/charmbracelet/crush',
                  route: 'go-install',
                  packageId: 'github.com/charmbracelet/crush',
                  version: '0.97.0',
                },),
                fetcher: async () => ({
                  ok: false,
                  status: 404,
                  text: 'not found',
                }),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('github.com/charmbracelet/crush',);
            expect((caught as Error).message,)
              .toContain('404',);
          },
        },),
        it({
          name: 'throws when the proxy record carries no Version',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveGoModule({
                entry: fixtureEntry({
                  id: 'go:github.com/charmbracelet/crush',
                  route: 'go-install',
                  packageId: 'github.com/charmbracelet/crush',
                  version: '0.97.0',
                },),
                fetcher: fixtureFetch({
                  'https://proxy.golang.org/github.com/charmbracelet/crush/@latest': JSON.stringify({
                    Time: '2026-10-27T00:00:00Z',
                  },),
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('github.com/charmbracelet/crush',);
          },
        },),
      ],
    },),

    //endregion Go family

    //region Rust channels

    describe({
      name: resolveRust.name,
      children: [
        it({
          name: 'returns the nightly date stamp for the nightly channel',
          fn: async () => {
            expect(await resolveRust({
              entry: fixtureEntry({
                id: 'rust',
                request: { channel: 'nightly' },
                route: 'rustup-toolchain',
                version: 'nightly-2026-01-01',
                bins: [],
              },),
              fetcher: fixtureFetch({
                'https://static.rust-lang.org/dist/channel-rust-nightly.toml': 'manifest-version = "2"\ndate = "2026-10-26"\n',
              },),
            },),)
              .toEqual({
                version: 'nightly-2026-10-26',
                artifacts: {},
              },);
          },
        },),
        it({
          name: 'returns the [pkg.rust] version for the stable channel',
          fn: async () => {
            expect(await resolveRust({
              entry: fixtureEntry({
                id: 'rust',
                request: { channel: 'stable' },
                route: 'rustup-toolchain',
                version: '1.97.0',
                bins: [],
              },),
              fetcher: fixtureFetch({
                'https://static.rust-lang.org/dist/channel-rust-stable.toml': [
                  'manifest-version = "2"',
                  'date = "2026-10-20"',
                  '',
                  '[pkg.rust]',
                  'version = "1.98.1"',
                  '[pkg.rust.target.x86_64-unknown-linux-gnu]',
                  'available = true',
                  'version = "9.9.9"',
                  '',
                  '[pkg.rust-preview]',
                  'version = "8.8.8"',
                  '',
                ].join('\n',),
              },),
            },),)
              .toEqual({
                version: '1.98.1',
                artifacts: {},
              },);
          },
        },),
        it({
          name: 'throws when the nightly manifest carries no date',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveRust({
                entry: fixtureEntry({
                  id: 'rust',
                  request: { channel: 'nightly' },
                  route: 'rustup-toolchain',
                  version: 'nightly-2026-01-01',
                  bins: [],
                },),
                fetcher: fixtureFetch({
                  'https://static.rust-lang.org/dist/channel-rust-nightly.toml': 'manifest-version = "2"\n',
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('date',);
          },
        },),
        it({
          name: 'throws when the channel manifest carries no [pkg.rust] version',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveRust({
                entry: fixtureEntry({
                  id: 'rust',
                  request: { channel: 'stable' },
                  route: 'rustup-toolchain',
                  version: '1.97.0',
                  bins: [],
                },),
                fetcher: fixtureFetch({
                  'https://static.rust-lang.org/dist/channel-rust-stable.toml': 'manifest-version = "2"\ndate = "2026-10-20"\n\n[pkg.rust]\navailable = true\n',
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('[pkg.rust]',);
          },
        },),
      ],
    },),

    describe({
      name: resolveZigLatest.name,
      children: [
        it({
          name: 'returns the newest stable key and skips master and dev builds',
          fn: async () => {
            expect(await resolveZigLatest({
              fetcher: fixtureFetch({
                'https://ziglang.org/download/index.json': JSON.stringify({
                  'master': { version: '0.17.0-dev.12+abc', },
                  '0.9.1': {},
                  '0.16.0': {},
                  '0.15.1': {},
                  '0.17.0-dev.3': {},
                },),
              },),
            },),)
              .toBe('0.16.0',);
          },
        },),
        it({
          name: 'returns an empty key when only dev builds are listed',
          fn: async () => {
            expect(await resolveZigLatest({
              fetcher: fixtureFetch({
                'https://ziglang.org/download/index.json': JSON.stringify({
                  'master': {},
                  '0.17.0-dev.3': {},
                },),
              },),
            },),)
              .toBe('',);
          },
        },),
      ],
    },),

    //endregion Rust channels

    //region rustup channel archive

    describe({
      name: resolveRustup.name,
      children: [
        it({
          name: 'rebuilds the channel rows when the release manifest moves',
          fn: async () => {
            expect(await resolveRustup({
              entry: fixtureEntry({
                id: 'rustup',
                request: { track: 'latest', },
                route: 'archive',
                version: '1.29.2',
                bins: ['rustup'],
                artifacts: {
                  'linux-x64': {
                    url: 'https://static.rust-lang.org/rustup/archive/1.29.2/x86_64-unknown-linux-gnu/rustup-init',
                    sha256: 'b'
                      .repeat(64,),
                  },
                },
              },),
              fetcher: fixtureFetch({
                'https://static.rust-lang.org/rustup/release-stable.toml': `schema-version = '1'\nversion = '1.29.3'\n`,
                'https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init.sha256': `${
                  'c'
                    .repeat(64,)
                } *./rustup-init\n`,
              },),
            },))
              .toEqual({
                version: '1.29.3',
                artifacts: {
                  'linux-x64': {
                    url: 'https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init',
                    sha256: 'c'
                      .repeat(64,),
                  },
                },
              },);
          },
        },),
        it({
          name: 'keeps the recorded rows when the manifest names the locked version',
          fn: async () => {
            /**
             Recorded rows the resolver must not touch.
             */
            const artifacts = {
              'linux-x64': {
                url: 'https://static.rust-lang.org/rustup/archive/1.29.2/x86_64-unknown-linux-gnu/rustup-init',
                sha256: 'b'
                  .repeat(64,),
              },
            };
            expect(await resolveRustup({
              entry: fixtureEntry({
                id: 'rustup',
                request: { track: 'latest', },
                route: 'archive',
                version: '1.29.2',
                bins: ['rustup'],
                artifacts,
              },),
              fetcher: fixtureFetch({
                'https://static.rust-lang.org/rustup/release-stable.toml': `schema-version = '1'\nversion = '1.29.2'\n`,
              },),
            },))
              .toEqual({
                version: '1.29.2',
                artifacts,
              },);
          },
        },),
        it({
          name: 'throws when a checksum line is not a sha256',
          fn: async () => {
            /**
             Rejection of the malformed checksum.
             */
            let caught: unknown;
            try {
              await resolveRustup({
                entry: fixtureEntry({
                  id: 'rustup',
                  request: { track: 'latest', },
                  route: 'archive',
                  version: '1.29.2',
                  bins: ['rustup'],
                  artifacts: {
                    'linux-x64': {
                      url: 'https://static.rust-lang.org/rustup/archive/1.29.2/x86_64-unknown-linux-gnu/rustup-init',
                      sha256: 'b'
                        .repeat(64,),
                    },
                  },
                },),
                fetcher: fixtureFetch({
                  'https://static.rust-lang.org/rustup/release-stable.toml': `schema-version = '1'\nversion = '1.29.3'\n`,
                  'https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init.sha256': 'not-a-digest *./rustup-init\n',
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('sha256',);
          },
        },),
        it({
          name: 'throws when a recorded row url carries no target triple',
          fn: async () => {
            let caught: unknown;
            try {
              await resolveRustup({
                entry: fixtureEntry({
                  id: 'rustup',
                  request: { track: 'latest', },
                  route: 'archive',
                  version: '1.29.2',
                  bins: ['rustup'],
                  artifacts: {
                    'linux-x64': {
                      url: 'https://static.rust-lang.org/rustup/archive/1.29.2/rustup-init',
                      sha256: 'b'.repeat(64,),
                    },
                  },
                },),
                fetcher: fixtureFetch({
                  'https://static.rust-lang.org/rustup/release-stable.toml': `schema-version = '1'\nversion = '1.29.3'\n`,
                },),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,)
              .toBeInstanceOf(Error,);
            expect((caught as Error).message,)
              .toContain('carries no target triple',);
          },
        },),
      ],
    },),

    //endregion rustup channel archive
  ],
},);
