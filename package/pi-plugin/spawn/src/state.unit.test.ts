import {
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  byPidDir,
  reportedStatePath,
  spawnsDir,
  spawnStatePath,
  type SpawnState,
} from './paths.ts';
import {
  checkCompletedChildren,
  claimSpawn,
  completeSpawn,
  formatSpawnResult,
  isSpawnJsonFilename,
  NOTHING_TO_REPORT,
  spawnIdFromJsonFilename,
  writeInitialSpawnState,
  writePidMapping,
} from './state.ts';
import {
  envVar,
  tempDir,
} from './test-support.ts';

/**
 Fields tests may override on default spawn state fixture.
 */
type SpawnStateFixtureOverrides = {
  /**
   Spawn identifier for fixture.
   */
  readonly spawnId?: SpawnState['spawnId'];
  /**
   Child session identifier for fixture.
   */
  readonly sessionId?: SpawnState['sessionId'];
  /**
   Child session file for fixture.
   */
  readonly sessionFile?: SpawnState['sessionFile'];
  /**
   Parent session identifier for fixture.
   */
  readonly parentSessionId?: SpawnState['parentSessionId'];
  /**
   Parent session file for fixture.
   */
  readonly parentSessionFile?: SpawnState['parentSessionFile'];
  /**
   Working directory for fixture.
   */
  readonly cwd?: SpawnState['cwd'];
  /**
   Lifecycle status for fixture.
   */
  readonly status?: SpawnState['status'];
  /**
   Last assistant message for fixture.
   */
  readonly lastMessage?: SpawnState['lastMessage'];
};

/**
 Builds spawn state fixtures for tests.
 
 @param overrides - fields to override from default stopped state.
 
 @returns spawn state fixture.
 */
function spawnStateFixture(overrides: SpawnStateFixtureOverrides = {},): SpawnState {
  return {
    spawnId: 'spawn-1',
    sessionId: 'child-session',
    sessionFile: '/tmp/child.jsonl',
    parentSessionId: 'parent-session',
    parentSessionFile: '/tmp/parent.jsonl',
    cwd: '/repo',
    status: 'stopped',
    lastMessage: 'child done',
    ...overrides,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: writePidMapping.name,
      children: [
        it({
          name: 'writes mapping under by-pid directory',
          fn: async function testWritePidMapping() {
            await using dir = await tempDir({ prefix: 'spawn-pi-state-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            await writePidMapping({
              pid: 123,
              mapping: {
                sessionId: 'parent-session',
                sessionFile: '/tmp/parent.jsonl',
                cwd: '/repo',
                extensionPath: '/pkg/index.mjs',
              },
            },);

            expect(readFileSync(join(
              byPidDir(),
              '123',
            ), 'utf8',),).toContain('parent-session',);
          },
        },),
      ],
    },),
    describe({
      name: claimSpawn.name,
      children: [
        it({
          name: 'claims only unclaimed spawn state',
          fn: async function testClaimSpawn() {
            await using dir = await tempDir({ prefix: 'spawn-pi-claim-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            await writeInitialSpawnState({
              state: spawnStateFixture({
                sessionId: '',
                sessionFile: '',
                status: 'running',
                lastMessage: '',
              },),
            },);
            await claimSpawn({
              spawnId: 'spawn-1',
              sessionId: 'child-session',
              sessionFile: '/tmp/child.jsonl',
            },);

            expect(readFileSync(spawnStatePath({ spawnId: 'spawn-1', },), 'utf8',),)
              .toContain('child-session',);
          },
        },),
      ],
    },),
    describe({
      name: completeSpawn.name,
      children: [
        it({
          name: 'marks owned spawn as stopped with last message',
          fn: async function testCompleteSpawn() {
            await using dir = await tempDir({ prefix: 'spawn-pi-complete-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            await writeInitialSpawnState({
              state: spawnStateFixture({
                status: 'running',
                lastMessage: '',
              },),
            },);
            await completeSpawn({
              spawnId: 'spawn-1',
              sessionId: 'child-session',
              lastMessage: 'finished',
            },);

            /**
             Updated state JSON after completion.
             */
            const raw = readFileSync(spawnStatePath({ spawnId: 'spawn-1', },), 'utf8',);
            expect(raw,).toContain('"status":"stopped"',);
            expect(raw,).toContain('finished',);
          },
        },),
        it({
          name: 'ignores completion from non-owner session',
          fn: async function testWrongOwnerComplete() {
            await using dir = await tempDir({ prefix: 'spawn-pi-wrong-owner-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            await writeInitialSpawnState({
              state: spawnStateFixture({
                status: 'running',
                lastMessage: '',
              },),
            },);
            await completeSpawn({
              spawnId: 'spawn-1',
              sessionId: 'other-child',
              lastMessage: 'finished',
            },);

            /**
             Unchanged state JSON after rejected completion.
             */
            const raw = readFileSync(spawnStatePath({ spawnId: 'spawn-1', },), 'utf8',);
            expect(raw,).toContain('"status":"running"',);
            expect(raw,).not.toContain('finished',);
          },
        },),
      ],
    },),
    describe({
      name: formatSpawnResult.name,
      children: [
        it({
          name: 'formats completed child result for parent context',
          fn: async function testFormatResult() {
            /**
             Formatted child result for default fixture.
             */
            const result = formatSpawnResult(spawnStateFixture(),);

            expect(result,).toContain(
              'Spawned Pi session completed (spawnId: spawn-1):',
            );
            expect(result,).toContain('child done',);
          },
        },),
      ],
    },),
    describe({
      name: isSpawnJsonFilename.name,
      children: [
        it({
          name: 'detects active json state files only',
          fn: async function testJsonFilename() {
            expect(isSpawnJsonFilename('abc.json',),).toBe(true,);
            expect(isSpawnJsonFilename('abc.reported',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: spawnIdFromJsonFilename.name,
      children: [
        it({
          name: 'drops json suffix',
          fn: async function testSpawnIdFromFilename() {
            expect(spawnIdFromJsonFilename('abc.json',),).toBe('abc',);
          },
        },),
      ],
    },),
    describe({
      name: checkCompletedChildren.name,
      children: [
        it({
          name: 'returns sentinel when no spawn directory exists',
          fn: async function testNoSpawnsDir() {
            await using dir = await tempDir({ prefix: 'spawn-pi-empty-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            expect(await checkCompletedChildren({
              parentSessionId: 'parent-session',
              consume: true,
            },),).toBe(NOTHING_TO_REPORT,);
          },
        },),
        it({
          name: 'filters by parent and stopped status before consuming matches',
          fn: async function testConsumeCompletedChildren() {
            await using dir = await tempDir({ prefix: 'spawn-pi-consume-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            mkdirSync(
              spawnsDir(),
              { recursive: true, },
            );
            writeFileSync(
              spawnStatePath({ spawnId: 'spawn-1', },),
              JSON.stringify(spawnStateFixture(),),
            );
            writeFileSync(
              spawnStatePath({ spawnId: 'spawn-2', },),
              JSON.stringify(spawnStateFixture({
                spawnId: 'spawn-2',
                parentSessionId: 'other-parent',
              },),),
            );
            writeFileSync(
              spawnStatePath({ spawnId: 'spawn-3', },),
              JSON.stringify(spawnStateFixture({
                spawnId: 'spawn-3',
                status: 'running',
              },),),
            );

            const result = await checkCompletedChildren({
              parentSessionId: 'parent-session',
              consume: true,
            },);

            expect(result,).not.toBe(NOTHING_TO_REPORT,);
            expect(String(result,),).toContain('spawn-1',);
            expect(String(result,),).not.toContain('spawn-2',);
            /**
             Reported marker for consumed spawn.
             */
            const reportedSpawnPath = reportedStatePath({ spawnId: 'spawn-1', },);
            /**
             Active state path for consumed spawn.
             */
            const consumedSpawnPath = spawnStatePath({ spawnId: 'spawn-1', },);
            /**
             Active state path for non-matching parent spawn.
             */
            const otherParentSpawnPath = spawnStatePath({ spawnId: 'spawn-2', },);
            /**
             Active state path for still-running spawn.
             */
            const runningSpawnPath = spawnStatePath({ spawnId: 'spawn-3', },);

            expect(existsSync(reportedSpawnPath,),).toBe(true,);
            expect(existsSync(consumedSpawnPath,),).toBe(false,);
            expect(existsSync(otherParentSpawnPath,),).toBe(true,);
            expect(existsSync(runningSpawnPath,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
