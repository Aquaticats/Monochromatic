import {
  mkdirSync,
  writeFileSync,
  utimesSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { byPidDir, } from './paths.ts';
import {
  findByMostRecent,
  findCallingSession,
  readByPidDir,
  readParentPid,
  readPidMapping,
  SESSION_NOT_FOUND,
  walkProcessTreeFrom,
} from './session-finder.ts';
import {
  envVar,
  tempDir,
} from './test-support.ts';
import { writePidMapping, } from './state.ts';

/**
 Writes mapping fixture directly to PID mapping directory.
 
 @param pid - process id filename to write.
 
 @param sessionId - session id stored in mapping.
 */
function writeMappingFixture(
  {
    pid,
    sessionId,
  }: {
    readonly pid: number;
    readonly sessionId: string;
  },
): void {
  mkdirSync(
    byPidDir(),
    { recursive: true, },
  );
  writeFileSync(
    join(
      byPidDir(),
      String(pid,),
    ),
    JSON.stringify({
      sessionId,
      sessionFile: `/tmp/${sessionId}.jsonl`,
      cwd: '/repo',
      extensionPath: '/pkg/index.mjs',
    },),
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readParentPid.name,
      children: [
        it({
          name: 'returns a numeric parent pid for current process on procfs hosts',
          skip: process.platform !== 'linux',
          fn: async function testReadParentPid() {
            expect(await readParentPid(process.pid,),).not.toBe(SESSION_NOT_FOUND,);
          },
        },),
      ],
    },),
    describe({
      name: readPidMapping.name,
      children: [
        it({
          name: 'reads mapping for pid file',
          fn: async function testReadPidMapping() {
            await using dir = await tempDir({ prefix: 'spawn-pi-pid-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            await writePidMapping({
              pid: 456,
              mapping: {
                sessionId: 'session-456',
                sessionFile: '/tmp/session-456.jsonl',
                cwd: '/repo',
                extensionPath: '/pkg/index.mjs',
              },
            },);

            const mapping = await readPidMapping({ pid: 456, },);
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected mapping for pid 456',);
            expect(mapping.sessionId,).toBe('session-456',);
          },
        },),
        it({
          name: 'returns sentinel for missing pid file',
          fn: async function testMissingPidMapping() {
            await using dir = await tempDir({ prefix: 'spawn-pi-missing-pid-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            expect(await readPidMapping({ pid: 999, },),).toBe(SESSION_NOT_FOUND,);
          },
        },),
      ],
    },),
    describe({
      name: walkProcessTreeFrom.name,
      children: [
        it({
          name: 'returns direct mapping before walking procfs',
          fn: async function testDirectWalkMapping() {
            await using dir = await tempDir({ prefix: 'spawn-pi-walk-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            writeMappingFixture({
              pid: 789,
              sessionId: 'session-789',
            },);

            const mapping = await walkProcessTreeFrom({ pid: 789, },);
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected direct process tree mapping',);
            expect(mapping.sessionId,).toBe('session-789',);
          },
        },),
      ],
    },),
    describe({
      name: readByPidDir.name,
      children: [
        it({
          name: 'returns sentinel when mapping directory is absent',
          fn: async function testMissingByPidDir() {
            await using dir = await tempDir({ prefix: 'spawn-pi-no-by-pid-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            expect(await readByPidDir(),).toBe(SESSION_NOT_FOUND,);
          },
        },),
      ],
    },),
    describe({
      name: findByMostRecent.name,
      children: [
        it({
          name: 'returns newest readable mapping',
          fn: async function testNewestMapping() {
            await using dir = await tempDir({ prefix: 'spawn-pi-newest-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            writeMappingFixture({
              pid: 101,
              sessionId: 'older-session',
            },);
            writeMappingFixture({
              pid: 202,
              sessionId: 'newer-session',
            },);

            /**
             Timestamp for older fixture.
             */
            const olderDate = new Date(1_000,);
            /**
             Timestamp for newer fixture.
             */
            const newerDate = new Date(2_000,);
            utimesSync(
              join(
                byPidDir(),
                '101',
              ),
              olderDate,
              olderDate,
            );
            utimesSync(
              join(
                byPidDir(),
                '202',
              ),
              newerDate,
              newerDate,
            );

            const mapping = await findByMostRecent();
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected newest mapping',);
            expect(mapping.sessionId,).toBe('newer-session',);
          },
        },),
      ],
    },),
    describe({
      name: findCallingSession.name,
      children: [
        it({
          name: 'uses direct parent process mapping when available',
          fn: async function testFindCallingSession() {
            await using dir = await tempDir({ prefix: 'spawn-pi-calling-', },);
            using _env = envVar({
              name: 'PI_CODING_AGENT_DIR',
              value: dir.path,
            },);

            writeMappingFixture({
              pid: process.ppid,
              sessionId: 'parent-process-session',
            },);

            const mapping = await findCallingSession();
            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected calling session mapping',);
            expect(mapping.sessionId,).toBe('parent-process-session',);
          },
        },),
      ],
    },),
  ],
},);
