/**
 Unit tests for the typed toolchain plan.
 */


import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ALL_TOOLS,
  BIN_BOOTSTRAP,
  MPM_BOOTSTRAP,
  TOOLS,
  TOOLS_BY_ID,
  toolById,
  toolPlanInstallableNames,
} from './plan.ts';
import { TOOL_ROUTES, } from './types.ts';

/**
 Hex digits in a SHA-256 digest.
 */
const SHA256_HEX_LENGTH = 64;

/**
 Whether text is an `https://` URL with something after the scheme and no whitespace anywhere.

 @param url - Artifact URL to test

 @returns `true` when the URL is a non-empty `https://` reference without whitespace
 */
function isHttpsUrlWithoutWhitespace(url: string,): boolean {
  if ((!url.startsWith('https://',)) || (url.length <= 'https://'.length))
    return false;
  for (const ch of url) {
    if (ch.trim() === '')
      return false;
  }
  return true;
}

/**
 Whether text is a lowercase hexadecimal SHA-256 digest.

 @param digest - Digest text to test

 @returns `true` for exactly 64 lowercase hex digits
 */
function isSha256Hex(digest: string,): boolean {
  if (digest.length !== SHA256_HEX_LENGTH)
    return false;
  for (const ch of digest) {
    if (!'0123456789abcdef'.includes(ch,))
      return false;
  }
  return true;
}

await describe({
  name: 'toolchain plan',
  children: [
    describe({
      name: 'TOOLS',
      children: [
        it({
          name: 'declares 60 tools with unique ids',
          fn: async () => {
            expect(TOOLS.length,)
              .toBe(60,);
            /**
             Ids seen once each.
             */
            const ids = TOOLS.map(
              (entry,) => entry.id,
            );
            expect(new Set(ids,)
              .size,)
              .toBe(ids.length,);
          },
        },),
        it({
          name: 'keeps 54 floating requests, 4 pins and 2 channels',
          fn: async () => {
            /**
             Request shapes counted across the 60 tools.
             */
            const counts = {
              track: 0,
              pinned: 0,
              channel: 0,
            };
            for (const entry of TOOLS) {
              if ('track' in entry.request)
                counts.track += 1;
              else if ('pinned' in entry.request)
                counts.pinned += 1;
              else
                counts.channel += 1;
            }
            expect(counts,)
              .toEqual({
                track: 54,
                pinned: 4,
                channel: 2,
              },);
          },
        },),
        it({
          name: 'records the four explicit pins exactly',
          fn: async () => {
            expect(toolById('bun',)
              .request)
              .toEqual({ pinned: '1.3.14' },);
            expect(toolById('cargo:cargo-fuzz',)
              .request)
              .toEqual({ pinned: '0.13.2' },);
            expect(toolById('zig',)
              .request)
              .toEqual({ pinned: '0.15.2' },);
            expect(toolById('llama.cpp',)
              .request)
              .toEqual({ pinned: 'b9829' },);
          },
        },),
        it({
          name: 'records the two channels exactly',
          fn: async () => {
            expect(toolById('rust',)
              .request)
              .toEqual({ channel: 'nightly' },);
            expect(toolById('java',)
              .request)
              .toEqual({ channel: 'temurin-21' },);
          },
        },),
        it({
          name: 'uses only declared route values',
          fn: async () => {
            for (const entry of ALL_TOOLS)
              expect(TOOL_ROUTES.includes(entry.route,),)
                .toBe(true,);
          },
        },),
        it({
          name: 'declares every binary name at most once across the plan',
          fn: async () => {
            /**
             Names seen so far.
             */
            const seen = new Set<string>();
            for (const entry of ALL_TOOLS) {
              for (const name of entry.bins) {
                expect(seen.has(name,),)
                  .toBe(false,);
                seen.add(name,);
              }
            }
          },
        },),
        it({
          name: 'carries OS filters only on the linux-only tools',
          fn: async () => {
            /**
             Tool ids with an OS filter.
             */
            const filtered = TOOLS
              .filter(
              (entry,) => entry.os !== undefined,
              )
              .map(
              (entry,) => entry.id,
              )
              .toSorted();
            expect(filtered,)
              .toEqual(
              [
                'cargo:monochromatic-git-policy-cli',
                'cargo:monochromatic-search-shadow',
                'conda:bubblewrap',
              ],
              );
          },
        },),
        it({
          name: 'records rust components and Android targets as options',
          fn: async () => {
            expect(toolById('rust',)
              .options.rustComponents,)
              .toEqual(
              ['clippy', 'rust-src', 'llvm-tools', 'rustc-dev'],
              );
            expect(toolById('rust',)
              .options.rustTargets,)
              .toEqual(
              ['aarch64-linux-android', 'x86_64-linux-android'],
              );
          },
        },),
        it({
          name: 'records the cargo features and the postinstall link',
          fn: async () => {
            expect(toolById('cargo:coreutils',)
              .options.cargoFeatures,)
              .toEqual(['unix'],);
            expect(toolById('cargo:monochromatic-git-policy-cli',)
              .options.postinstall,)
              .toEqual({
                when: 'always',
                link: {
                  sourceBin: 'cli-git-native',
                  linkName: 'git',
                  linkDir: '.cache/cli-git-bin',
                },
              },);
            expect(toolById('cargo:monochromatic-search-shadow',)
              .options.postinstall,)
              .toEqual({
                when: 'always',
                link: {
                  sourceBin: 'search-shadow',
                  linkName: 'rg',
                  linkDir: '.cache/search-shadow-bin',
                },
              },);
          },
        },),
        it({
          name: 'records the Android SDK package pins in android CLI ids',
          fn: async () => {
            expect(toolById('android-sdk',)
              .options.androidPackages,)
              .toEqual(
              [
                'platforms/android-37.0',
                'build-tools/37.0.0',
                'ndk/29.0.13846066',
                'platform-tools',
              ],
              );
          },
        },),
        it({
          name: 'records tool env exports without machine-absolute paths',
          fn: async () => {
            expect(toolById('rust',)
              .env,)
              .toEqual({ RUSTUP_TOOLCHAIN: toolById('rust',).version, },);
            expect(toolById('java',)
              .env,)
              .toEqual({ JAVA_HOME: 'java/temurin-21.0.13+9.0.LTS' },);
            for (const entry of ALL_TOOLS) {
              for (const value of Object.values(entry.env,))
                expect(value.startsWith('/',),)
                  .toBe(false,);
            }
          },
        },),
        it({
          name: 'records artifact URLs with 64-hex SHA-256 digests',
          fn: async () => {
            for (const entry of ALL_TOOLS) {
              for (const row of Object.values(entry.artifacts,)) {
                expect(isHttpsUrlWithoutWhitespace(row.url,),)
                  .toBe(true,);
                expect(isSha256Hex(row.sha256,),)
                  .toBe(true,);
              }
            }
          },
        },),
        it({
          name: 'leaves macOS and Windows artifact rows absent',
          fn: async () => {
            for (const entry of TOOLS) {
              for (const platform of Object.keys(entry.artifacts,))
                expect(platform.startsWith('linux-',),)
                  .toBe(true,);
            }
          },
        },),
      ],
    },),
    describe({
      name: 'bootstraps',
      children: [
        it({
          name: 'pins the bin CLI standalone binary with a digest',
          fn: async () => {
            expect(BIN_BOOTSTRAP.id,)
              .toBe('bin',);
            expect(BIN_BOOTSTRAP.options.bootstrap,)
              .toEqual({
                distribution: 'standalone-binary',
              },);
            expect(BIN_BOOTSTRAP.artifacts['linux-x64']?.sha256,)
              .toBe(
              'f7a179a813675bde2d8cf0ad3b4ae1d7bb1c6732daf623fb8deeba9f5f4782a7',
              );
          },
        },),
        it({
          name: 'pins mpm and names uv as its staging prerequisite',
          fn: async () => {
            expect(MPM_BOOTSTRAP.id,)
              .toBe('mpm',);
            expect(MPM_BOOTSTRAP.route,)
              .toBe('mpm-uvx',);
            expect(MPM_BOOTSTRAP.options.bootstrap?.requiresTool,)
              .toBe('uv',);
          },
        },),
        it({
          name: 'puts both bootstraps first in ALL_TOOLS',
          fn: async () => {
            expect(ALL_TOOLS.map(
              (entry,) => entry.id,
            )
              .slice(
              0,
              2,
              ),)
              .toEqual(['bin', 'mpm'],);
          },
        },),
      ],
    },),
    describe({
      name: 'lookups',
      children: [
        it({
          name: 'indexes every entry by id',
          fn: async () => {
            expect(TOOLS_BY_ID.size,)
              .toBe(ALL_TOOLS.length,);
            for (const entry of ALL_TOOLS) {
              expect(TOOLS_BY_ID.get(entry.id,),)
                .toBe(entry,);
            }
          },
        },),
        it({
          name: 'throws on an unknown tool id',
          fn: async () => {
            expect(
              () => toolById('no-such-tool',),
            )
              .toThrow();
          },
        },),
        it({
          name: 'exposes installable names for the generate-index filter',
          fn: async () => {
            /**
             Installable names derived from the plan.
             */
            const names = toolPlanInstallableNames();
            expect(names.has('bun',),)
              .toBe(true,);
            expect(names.has('fd-find',),)
              .toBe(true,);
            expect(names.has('yaml-language-server',),)
              .toBe(true,);
            expect(names.has('markdown-oxide',),)
              .toBe(true,);
            expect(names.has('meta-package-manager',),)
              .toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
