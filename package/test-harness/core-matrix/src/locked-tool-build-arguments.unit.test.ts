/**
 Tests for reading the build arguments of a locked tool from `toolchain.lock.json`.
 Lock files are written into disposable directories, so no test reads the repository's own lock.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  LOCK_FILE_NAME,
  LockedToolBuildArgumentsError,
  lockedToolBuildArgumentsFromText,
  readLockedToolBuildArguments,
  ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE,
} from '../dist/final/node/index.mjs';

/**
 Lock platform the fixture archives are filed under.
 */
const FIXTURE_PLATFORM = ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE.x64;

/**
 Lock platform the fixture has no archive for.
 */
const FIXTURE_PLATFORM_WITHOUT_ARCHIVE = ZIG_LOCK_PLATFORM_BY_HOST_ARCHITECTURE.arm64;

/**
 Archive address the fixture lock records for Zig.
 */
const FIXTURE_ZIG_URL = 'https://example.invalid/zig-fixture.tar.xz';

/**
 Archive checksum the fixture lock records for Zig.
 */
const FIXTURE_ZIG_SHA256 = 'fixture-sha256';

/**
 Version the fixture lock records for `cargo-about`.
 */
const FIXTURE_CARGO_ABOUT_VERSION = '1.2.3';

/**
 Lock text with a Zig archive for one platform and a `cargo-about` version.
 */
const FIXTURE_LOCK = JSON.stringify({
  tools: {
    'zig': {
      version: '9.9.9',
      artifacts: { [FIXTURE_PLATFORM]: { url: FIXTURE_ZIG_URL, sha256: FIXTURE_ZIG_SHA256, }, },
    },
    'cargo:cargo-about': { version: FIXTURE_CARGO_ABOUT_VERSION, artifacts: {}, },
  },
},);

/**
 Creates a disposable directory that is removed when the `await using` binding ends.

 @returns the directory path and its disposer
 */
async function createScratchDirectory(): Promise<{ readonly path: string; [Symbol.asyncDispose]: () => Promise<void>; }> {
  /**
   Disposable directory under the repository's scratch root.
   */
  const path = await mkdtemp(join(scratchRoot(), 'locked-tool-build-arguments-',),);
  return {
    path,
    [Symbol.asyncDispose]: async function cleanup() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'locked tool build arguments',
  children: [
    describe({
      name: lockedToolBuildArgumentsFromText.name,
      children: [
        it({
          name: 'returns the archive address and checksum of the platform for zig',
          fn: async () => {
            expect(lockedToolBuildArgumentsFromText({
              lockText: FIXTURE_LOCK,
              tool: 'zig',
              platform: FIXTURE_PLATFORM,
            },),).toEqual([`ZIG_ARCHIVE_URL=${FIXTURE_ZIG_URL}`, `ZIG_ARCHIVE_SHA256=${FIXTURE_ZIG_SHA256}`,],);
          },
        },),
        it({
          name: 'returns only the version for cargo-about, whatever the platform',
          fn: async () => {
            expect(lockedToolBuildArgumentsFromText({
              lockText: FIXTURE_LOCK,
              tool: 'cargo-about',
              platform: undefined,
            },),).toEqual([`CARGO_ABOUT_VERSION=${FIXTURE_CARGO_ABOUT_VERSION}`,],);
          },
        },),
        it({
          name: 'rejects text that is not JSON',
          fn: async () => {
            expect(() =>
              lockedToolBuildArgumentsFromText({
                lockText: '{',
                tool: 'zig',
                platform: FIXTURE_PLATFORM,
              },)
            ).toThrow(LockedToolBuildArgumentsError,);
          },
        },),
        it({
          name: 'rejects a zig lock without an archive for the platform, without a platform, and with an empty checksum',
          fn: async () => {
            expect(() =>
              lockedToolBuildArgumentsFromText({
                lockText: FIXTURE_LOCK,
                tool: 'zig',
                platform: FIXTURE_PLATFORM_WITHOUT_ARCHIVE,
              },)
            ).toThrow(LockedToolBuildArgumentsError,);
            expect(() =>
              lockedToolBuildArgumentsFromText({
                lockText: FIXTURE_LOCK,
                tool: 'zig',
                platform: undefined,
              },)
            ).toThrow(LockedToolBuildArgumentsError,);
            expect(() =>
              lockedToolBuildArgumentsFromText({
                lockText: JSON.stringify({
                  tools: { zig: { artifacts: { [FIXTURE_PLATFORM]: { url: FIXTURE_ZIG_URL, sha256: '', }, }, }, },
                },),
                tool: 'zig',
                platform: FIXTURE_PLATFORM,
              },)
            ).toThrow(LockedToolBuildArgumentsError,);
          },
        },),
        it({
          name: 'rejects a cargo-about lock without a row, with an empty version, and with a null document',
          fn: async () => {
            for (const lockText of ['{"tools":{}}', '{"tools":{"cargo:cargo-about":{"version":""}}}', 'null',]) {
              expect(() =>
                lockedToolBuildArgumentsFromText({
                  lockText,
                  tool: 'cargo-about',
                  platform: undefined,
                },)
              ).toThrow(LockedToolBuildArgumentsError,);
            }
          },
        },),
      ],
    },),
    describe({
      name: readLockedToolBuildArguments.name,
      children: [
        it({
          name: 'finds the lock in an ancestor directory',
          fn: async () => {
            await using directory = await createScratchDirectory();
            await writeFile(join(directory.path, LOCK_FILE_NAME,), FIXTURE_LOCK,);
            /**
             Directory two levels below the lock.
             */
            const nested = join(directory.path, 'one', 'two',);
            await mkdir(nested, { recursive: true, },);
            expect(await readLockedToolBuildArguments({
              startDirectory: nested,
              tool: 'cargo-about',
              platform: undefined,
            },),).toEqual([`CARGO_ABOUT_VERSION=${FIXTURE_CARGO_ABOUT_VERSION}`,],);
          },
        },),
        it({
          name: 'reports a lock without the tool instead of looking further up',
          fn: async () => {
            await using directory = await createScratchDirectory();
            await writeFile(join(directory.path, LOCK_FILE_NAME,), '{"tools":{}}',);
            await expect(readLockedToolBuildArguments({
              startDirectory: directory.path,
              tool: 'zig',
              platform: FIXTURE_PLATFORM,
            },),).rejects.toThrow(LockedToolBuildArgumentsError,);
          },
        },),
      ],
    },),
  ],
},);
