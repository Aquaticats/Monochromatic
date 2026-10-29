import {
  mkdirSync,
  writeFileSync,
  utimesSync,
} from 'node:fs';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  byPidDir,
  type Environment,
} from './paths.ts';
import {
  findByMostRecent,
  findByProcessTree,
  findCallingSession,
  SESSION_NOT_FOUND,
} from './session-finder.ts';

/**
 Older fixture modification time.
 */
const OLDER_MTIME = new Date(1_000,);

/**
 Newer fixture modification time.
 */
const NEWER_MTIME = new Date(2_000,);

/**
 Temporary HOME handle that removes itself at the end of an `await using`
 scope.
 */
type TempHome = {
  /**
   Absolute path to temporary HOME root.
   */
  readonly path: string;
  /**
   Removes temporary HOME root recursively.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates an isolated HOME directory for spawn-claude session-finder tests.
 
 @returns disposable temporary HOME handle
 
 @example
 ```ts
 await using home = await makeTempHome();
 ```
 */
async function makeTempHome(): Promise<TempHome> {
  /**
   Temporary HOME root path.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'spawn-claude-session-finder-',
  ),);

  return {
    path,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
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
 Builds environment override for a temporary HOME.
 
 @param home - temporary HOME path
 
 @returns environment override used by path helpers
 
 @example
 ```ts
 testEnv('/tmp/home');
 ```
 */
function testEnv(home: string,): Environment {
  return { HOME: home, };
}

/**
 Writes mapping fixture directly to PID mapping directory.
 
 @param env - environment controlling mapping directory
 
 @param pid - process id filename to write
 
 @param sessionId - session id stored in mapping
 
 @param mtime - file modification time to apply
 
 @example
 ```ts
 writeMappingFixture({ env, pid: 123, sessionId: 's', mtime: NEWER_MTIME });
 ```
 */
function writeMappingFixture(
  {
    env,
    pid,
    sessionId,
    mtime,
  }: {
    readonly env: Environment;
    readonly pid: number;
    readonly sessionId: string;
    readonly mtime: Date;
  },
): void {
  mkdirSync(
    byPidDir(env,),
    { recursive: true, },
  );
  /**
   Mapping fixture path.
   */
  const path = join(
    byPidDir(env,),
    String(pid,),
  );
  writeFileSync(
    path,
    JSON.stringify({
      sessionId,
      transcriptPath: `/tmp/${sessionId}.jsonl`,
    },),
  );
  utimesSync(
    path,
    mtime,
    mtime,
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: findByMostRecent.name,
      children: [
        it({
          name: 'returns newest readable mapping from env-injected HOME',
          fn: async function testNewestMapping(): Promise<void> {
            await using home = await makeTempHome();
            /**
             Environment override for this test.
             */
            const env = testEnv(home.path,);

            writeMappingFixture({
              env,
              pid: 101,
              sessionId: 'older-session',
              mtime: OLDER_MTIME,
            },);
            writeMappingFixture({
              env,
              pid: 202,
              sessionId: 'newer-session',
              mtime: NEWER_MTIME,
            },);

            /**
             Mapping resolved by newest-file fallback.
             */
            const mapping = await findByMostRecent(env,);
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected newest mapping',);
            expect(mapping.sessionId,).toBe('newer-session',);
          },
        },),
      ],
    },),
    describe({
      name: findByProcessTree.name,
      children: [
        it({
          name: 'uses direct parent process mapping from env-injected HOME',
          fn: async function testProcessTreeMapping(): Promise<void> {
            await using home = await makeTempHome();
            /**
             Environment override for this test.
             */
            const env = testEnv(home.path,);

            writeMappingFixture({
              env,
              pid: process.ppid,
              sessionId: 'parent-session',
              mtime: NEWER_MTIME,
            },);

            /**
             Mapping resolved by process-tree lookup.
             */
            const mapping = await findByProcessTree(env,);
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected process-tree mapping',);
            expect(mapping.sessionId,).toBe('parent-session',);
          },
        },),
      ],
    },),
    describe({
      name: findCallingSession.name,
      children: [
        it({
          name: 'falls back to newest mapping when process tree has no env mapping',
          fn: async function testFindCallingSessionFallback(): Promise<void> {
            await using home = await makeTempHome();
            /**
             Environment override for this test.
             */
            const env = testEnv(home.path,);

            writeMappingFixture({
              env,
              pid: 303,
              sessionId: 'fallback-session',
              mtime: NEWER_MTIME,
            },);

            /**
             Mapping resolved by full tree-then-newest lookup.
             */
            const mapping = await findCallingSession(env,);
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected fallback mapping',);
            expect(mapping.sessionId,).toBe('fallback-session',);
          },
        },),
      ],
    },),
  ],
},);
