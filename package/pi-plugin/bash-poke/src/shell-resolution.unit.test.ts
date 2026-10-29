/**
 Tests for shell resolution in the built bash-poke artifact.

 @module
 */

import { mkdtemp, rm, writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  constants,
  executableExists,
  executablesOnPath,
  resolveShell,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Probe accepting only the paths listed, so resolution is deterministic.
 
 @param allowed - paths treated as executable
 
 @returns probe suitable for injection
 
 @example
 ```ts
 probeAllowing(['/usr/bin/bash']);
 ```
 */
function probeAllowing(allowed: readonly string[], ) {
  return async function probe(path: string, ): Promise<boolean> {
    return allowed.includes(path, );
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region executableExists

    describe({
      name: executableExists.name,
      children: [
        it({
          name: 'accepts the running Node executable',
          fn: async () => {
            expect(await executableExists(process.execPath, ), ).toBe(true);
          },
        }, ),
        it({
          name: 'rejects a path that does not exist',
          fn: async () => {
            const root = await mkdtemp(join(scratchRoot(), 'bash-poke-shell-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(root, { recursive: true, force: true, }, );
              },
            };
            expect(
              await executableExists(join(root, 'absent', ), ),
            ).toBe(false);
          },
        }, ),
        it({
          name: 'rejects a file that exists but is not executable',
          fn: async () => {
            const root = await mkdtemp(join(scratchRoot(), 'bash-poke-shell-', ), );
            await using cleanup = {
              async [Symbol.asyncDispose](): Promise<void> {
                await rm(root, { recursive: true, force: true, }, );
              },
            };
            const path = join(root, 'plain.txt', );
            await writeFile(path, 'text', { mode: 0o644, }, );
            expect(await executableExists(path, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    //endregion executableExists

    //region executablesOnPath

    describe({
      name: executablesOnPath.name,
      children: [
        it({
          name: 'keeps PATH order and skips empty entries',
          fn: async () => {
            const found = await executablesOnPath({
              name: 'bash',
              pathValue: `/first::/second`,
              exists: probeAllowing(['/second/bash', '/first/bash', ], ),
            }, );
            expect(found, ).toEqual(['/first/bash', '/second/bash', ]);
          },
        }, ),
        it({
          name: 'returns nothing when no candidate is executable',
          fn: async () => {
            const found = await executablesOnPath({
              name: 'bash',
              pathValue: '/first:/second',
              exists: probeAllowing([], ),
            }, );
            expect(found, ).toEqual([]);
          },
        }, ),
      ],
    }, ),

    //endregion executablesOnPath

    //region resolveShell

    describe({
      name: resolveShell.name,
      children: [
        it({
          name: 'prefers the absolute bash when it is executable',
          fn: async () => {
            const shell = await resolveShell({
              pathValue: '/usr/bin',
              exists: probeAllowing([constants.ABSOLUTE_BASH_PATH, '/usr/bin/bash', ], ),
            }, );
            expect(shell.command, ).toBe(constants.ABSOLUTE_BASH_PATH);
            expect(shell.args, ).toEqual([constants.SHELL_COMMAND_FLAG, ]);
          },
        }, ),
        it({
          name: 'falls back to bash on PATH',
          fn: async () => {
            const shell = await resolveShell({
              pathValue: '/usr/bin:/bin',
              exists: probeAllowing(['/usr/bin/bash', ], ),
            }, );
            expect(shell.command, ).toBe('/usr/bin/bash');
          },
        }, ),
        it({
          name: 'falls back to sh when bash is absent',
          fn: async () => {
            const shell = await resolveShell({
              pathValue: '/usr/bin',
              exists: probeAllowing(['/usr/bin/sh', ], ),
            }, );
            expect(shell.command, ).toBe('/usr/bin/sh');
          },
        }, ),
        it({
          name: 'returns the bare shell name when nothing is executable',
          fn: async () => {
            const shell = await resolveShell({
              pathValue: '/usr/bin',
              exists: probeAllowing([], ),
            }, );
            expect(shell.command, ).toBe(constants.SH_COMMAND_NAME);
            expect(shell.args, ).toEqual([constants.SHELL_COMMAND_FLAG, ]);
          },
        }, ),
        it({
          name: 'resolves a real shell on this machine by default',
          fn: async () => {
            const shell = await resolveShell({ pathValue: process.env.PATH ?? '', }, );
            expect(shell.command.length > 0, ).toBe(true);
            expect(shell.args, ).toEqual([constants.SHELL_COMMAND_FLAG, ]);
          },
        }, ),
      ],
    }, ),

    //endregion resolveShell
  ],
}, );
