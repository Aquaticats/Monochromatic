import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  access,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import type {
  SessionStartInput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';

import {
  cleanRootSentinelArtifacts,
  ROOT_SENTINEL_ARTIFACTS,
  sessionStartHousekeepingHandler,
} from './session-start-housekeeping.ts';

/**
 Temporary workspace handle that removes itself at the end of an `await using`
 scope.
 */
type TempWorkspace = {
  /** Absolute path to the temporary workspace root. */
  readonly path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates an isolated temporary workspace for a housekeeping test.
 
 @returns disposable temporary workspace handle
 */
async function makeWorkspace(): Promise<TempWorkspace> {
  const path = await mkdtemp(join(scratchRoot(), 'session-start-housekeeping-',),);

  return {
    path,
    [Symbol.asyncDispose]: async function cleanup() {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Returns whether a path currently exists.
 
 @param path - absolute path to check
 
 @returns true when `access` succeeds, false when it fails
 */
async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (_error: unknown) {
    return false;
  }
}

await describe({
  name: '',
  children: [
    describe({
      name: cleanRootSentinelArtifacts.name,
      children: [
        it({
          name: 'removes root zero-byte sentinel files, including read-only files',
          fn: async () => {
            await using workspace = await makeWorkspace();
            await Promise.all(
              ROOT_SENTINEL_ARTIFACTS.map(
                function writeSentinel(artifact,): Promise<void> {
                  return writeFile(
                    join(workspace.path, artifact,),
                    '',
                    { mode: 0o444, },
                  );
                },
              ),
            );
            await writeFile(join(workspace.path, 'ordinary-file',), '',);

            await cleanRootSentinelArtifacts(workspace.path,);

            const sentinelExists = await Promise.all(
              ROOT_SENTINEL_ARTIFACTS.map(
                function checkSentinel(artifact,): Promise<boolean> {
                  return pathExists(join(workspace.path, artifact,),);
                },
              ),
            );
            expect(sentinelExists,).toEqual(
              ROOT_SENTINEL_ARTIFACTS.map(function expectedMissing() {
                return false;
              },),
            );
            expect(
              await pathExists(join(workspace.path, 'ordinary-file',),),
            ).toBe(
              true,
            );
          },
        },),
        it({
          name: 'keeps non-empty root files with sentinel names',
          fn: async () => {
            await using workspace = await makeWorkspace();
            const headPath = join(workspace.path, 'HEAD',);
            await writeFile(headPath, 'ref: refs/heads/main\n',);

            await cleanRootSentinelArtifacts(workspace.path,);

            expect(await readFile(headPath, 'utf8',),).toBe('ref: refs/heads/main\n',);
          },
        },),
        it({
          name: 'keeps directories and symlinks with sentinel names',
          skip: process.platform === 'win32'
            ? 'Windows symlink permissions differ from the supported hook environment'
            : false,
          fn: async () => {
            await using workspace = await makeWorkspace();
            const targetPath = join(workspace.path, 'target',);
            const symlinkPath = join(workspace.path, 'config',);
            await mkdir(join(workspace.path, 'HEAD',),);
            await writeFile(targetPath, '',);
            await symlink(targetPath, symlinkPath,);

            await cleanRootSentinelArtifacts(workspace.path,);

            expect((await lstat(join(workspace.path, 'HEAD',),)).isDirectory(),).toBe(
              true,
            );
            expect((await lstat(symlinkPath,)).isSymbolicLink(),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: sessionStartHousekeepingHandler.name,
      children: [
        it({
          name: 'runs root sentinel cleanup during session start housekeeping',
          fn: async () => {
            await using workspace = await makeWorkspace();
            const headPath = join(workspace.path, 'HEAD',);
            await writeFile(headPath, '',);

            const event = {
              session_id: 'session-start-housekeeping-test',
              transcript_path: join(workspace.path, 'transcript.jsonl',),
              cwd: workspace.path,
              permission_mode: 'default',
              hook_event_name: 'SessionStart',
              source: 'startup',
              model: 'test-model',
            } satisfies SessionStartInput;

            await sessionStartHousekeepingHandler(event,);

            expect(await pathExists(headPath,),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
