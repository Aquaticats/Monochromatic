import {
  existsSync,
  readFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import spawnPi, {
  deliverCompletedChildren,
} from './index.ts';
import {
  SPAWN_EXTENSION_PATH_ENV,
  SPAWN_ID_ENV,
  SPAWN_PI_CUSTOM_TYPE,
} from './constants.ts';
import {
  byPidDir,
  reportedStatePath,
  spawnStatePath,
  type SpawnState,
} from './paths.ts';
import {
  createAgentEndEvent,
  createExtensionContext,
  createSessionShutdownEvent,
  createSessionStartEvent,
  fakePiApi,
  onlyHandler,
} from './pi-test-harness.ts';
import {
  writeInitialSpawnState,
} from './state.ts';
import {
  CLEAR_ENV,
  envVar,
  tempDir,
} from './test-support.ts';

/**
 Fields tests may override on default extension spawn state fixture.
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
 Builds spawn state fixture for extension tests.
 
 @param overrides - fields to override from default state.
 
 @returns spawn state fixture.
 */
function spawnStateFixture(overrides: SpawnStateFixtureOverrides = {},): SpawnState {
  return {
    spawnId: 'spawn-1',
    sessionId: '',
    sessionFile: '',
    parentSessionId: 'parent-session',
    parentSessionFile: '/tmp/parent.jsonl',
    cwd: '/repo',
    status: 'running',
    lastMessage: '',
    ...overrides,
  };
}

await describe({
  name: spawnPi.name,
  concurrency: 1,
  children: [
    it({
      name: 'registers coordination event handlers',
      fn: async function testRegistrations() {
        const harness = fakePiApi();
        spawnPi(harness.api,);

        expect(harness.registrations,).toContain('event:session_start',);
        expect(harness.registrations,).toContain('event:session_shutdown',);
        expect(harness.registrations,).toContain('event:agent_end',);
      },
    },),
    it({
      name: 'session_start writes pid mapping, extension env, and child claim',
      fn: async function testSessionStart() {
        await using dir = await tempDir({ prefix: 'spawn-pi-index-start-', },);
        using _agentDir = envVar({
          name: 'PI_CODING_AGENT_DIR',
          value: dir.path,
        },);
        using _spawnId = envVar({
          name: SPAWN_ID_ENV,
          value: 'spawn-1',
        },);
        using _extensionPath = envVar({
          name: SPAWN_EXTENSION_PATH_ENV,
          value: CLEAR_ENV,
        },);

        await writeInitialSpawnState({ state: spawnStateFixture(), },);
        const harness = fakePiApi();
        spawnPi(harness.api,);

        await onlyHandler(harness.handlers.sessionStart)(
          createSessionStartEvent(),
          createExtensionContext({
            sessionId: 'child-session',
            sessionFile: '/tmp/child.jsonl',
            cwd: '/repo',
          },),
        );

        expect(process.env[SPAWN_EXTENSION_PATH_ENV]?.endsWith('/index.ts',),).toBe(true,);
        expect(readFileSync(join(
          byPidDir(),
          String(process.pid,),
        ), 'utf8',),).toContain('child-session',);
        expect(readFileSync(spawnStatePath({ spawnId: 'spawn-1', },), 'utf8',),)
          .toContain('child-session',);

        await onlyHandler(harness.handlers.sessionShutdown)(
          createSessionShutdownEvent(),
          createExtensionContext({ sessionId: 'child-session', },),
        );
      },
    },),
    it({
      name: 'agent_end writes child completion into claimed spawn state',
      fn: async function testAgentEndCompletion() {
        await using dir = await tempDir({ prefix: 'spawn-pi-index-end-', },);
        using _agentDir = envVar({
          name: 'PI_CODING_AGENT_DIR',
          value: dir.path,
        },);
        using _spawnId = envVar({
          name: SPAWN_ID_ENV,
          value: 'spawn-1',
        },);

        await writeInitialSpawnState({
          state: spawnStateFixture({
            sessionId: 'child-session',
            sessionFile: '/tmp/child.jsonl',
          },),
        },);
        const harness = fakePiApi();
        spawnPi(harness.api,);

        await onlyHandler(harness.handlers.agentEnd)(
          createAgentEndEvent('child result',),
          createExtensionContext({ sessionId: 'child-session', },),
        );

        /**
         Completed state JSON after child agent_end.
         */
        const raw = readFileSync(spawnStatePath({ spawnId: 'spawn-1', },), 'utf8',);
        expect(raw,).toContain('"status":"stopped"',);
        expect(raw,).toContain('child result',);
      },
    },),
    it({
      name: 'session_shutdown stops completed-child monitor',
      fn: async function testSessionShutdown() {
        const harness = fakePiApi();
        spawnPi(harness.api,);

        await onlyHandler(harness.handlers.sessionShutdown)(
          createSessionShutdownEvent(),
          createExtensionContext({ sessionId: 'parent-session', },),
        );

        expect(harness.registrations,).toContain('event:session_shutdown',);
      },
    },),
    it({
      name: 'deliverCompletedChildren sends completed child message and consumes state',
      fn: async function testDeliverCompletedChildren() {
        await using dir = await tempDir({ prefix: 'spawn-pi-index-deliver-', },);
        using _agentDir = envVar({
          name: 'PI_CODING_AGENT_DIR',
          value: dir.path,
        },);
        using _spawnId = envVar({
          name: SPAWN_ID_ENV,
          value: CLEAR_ENV,
        },);

        await writeInitialSpawnState({
          state: spawnStateFixture({
            sessionId: 'child-session',
            sessionFile: '/tmp/child.jsonl',
            status: 'stopped',
            lastMessage: 'child result',
          },),
        },);
        const harness = fakePiApi();
        const delivered = await deliverCompletedChildren({
          pi: harness.api,
          ctx: createExtensionContext({ sessionId: 'parent-session', },),
        },);

        expect(delivered,).toBe(true,);
        expect(harness.sentMessages,).toHaveLength(1,);
        expect(harness.sentMessages[0]?.message.customType,).toBe(SPAWN_PI_CUSTOM_TYPE,);
        expect(String(harness.sentMessages[0]?.message.content,),).toContain('child result',);
        expect(harness.sentMessages[0]?.message.display,).toBe(true,);
        /**
         Reported marker path after delivery consumes state.
         */
        const reportedSpawnPath = reportedStatePath({ spawnId: 'spawn-1', },);

        expect(existsSync(reportedSpawnPath,),).toBe(true,);
      },
    },),
  ],
},);
