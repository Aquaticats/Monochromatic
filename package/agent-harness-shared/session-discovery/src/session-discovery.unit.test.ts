import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findByMostRecent,
  findCallingSession,
  SESSION_NOT_FOUND,
  walkProcessTreeFrom,
} from './index.ts';
import {
  BY_PID_DIR,
  fakeIo,
  mappingJson,
  parseTestMapping,
} from './test-support.ts';

await describe({
  name: 'session-discovery traversal',
  children: [
    describe({
      name: walkProcessTreeFrom.name,
      children: [
        it({
          name: 'walks parents until a mapping is found',
          fn: async function testWalkProcessTree() {
            const mapping = await walkProcessTreeFrom({
              pid: 300,
              byPidDir: BY_PID_DIR,
              parseMapping: parseTestMapping,
              io: fakeIo({
                files: { [`${BY_PID_DIR}/200`]: mappingJson('session-200'), },
                mtimes: {},
                parents: new Map([
                  [300, 250,],
                  [250, 200,],
                ],),
              },),
            },);

            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected ancestor mapping',);
            expect(mapping.sessionId,).toBe('session-200',);
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
            const mapping = await findByMostRecent({
              byPidDir: BY_PID_DIR,
              parseMapping: parseTestMapping,
              io: fakeIo({
                files: {
                  [`${BY_PID_DIR}/100`]: mappingJson('older'),
                  [`${BY_PID_DIR}/200`]: mappingJson('newer'),
                },
                mtimes: {
                  [`${BY_PID_DIR}/100`]: 1,
                  [`${BY_PID_DIR}/200`]: 2,
                },
                parents: new Map(),
              },),
            },);

            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected newest mapping',);
            expect(mapping.sessionId,).toBe('newer',);
          },
        },),
      ],
    },),
    describe({
      name: findCallingSession.name,
      children: [
        it({
          name: 'uses process tree before newest fallback',
          fn: async function testFindCallingSessionFromTree() {
            const mapping = await findCallingSession({
              startPid: 300,
              byPidDir: BY_PID_DIR,
              parseMapping: parseTestMapping,
              io: fakeIo({
                files: {
                  [`${BY_PID_DIR}/200`]: mappingJson('from-tree'),
                  [`${BY_PID_DIR}/500`]: mappingJson('newest'),
                },
                mtimes: {
                  [`${BY_PID_DIR}/200`]: 1,
                  [`${BY_PID_DIR}/500`]: 2,
                },
                parents: new Map([[300, 200,],],),
              },),
            },);

            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected tree mapping',);
            expect(mapping.sessionId,).toBe('from-tree',);
          },
        },),
        it({
          name: 'falls back to newest mapping when tree lookup fails',
          fn: async function testFindCallingSessionFallback() {
            const mapping = await findCallingSession({
              startPid: 300,
              byPidDir: BY_PID_DIR,
              parseMapping: parseTestMapping,
              io: fakeIo({
                files: { [`${BY_PID_DIR}/500`]: mappingJson('newest'), },
                mtimes: { [`${BY_PID_DIR}/500`]: 2, },
                parents: new Map(),
              },),
            },);

            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected fallback mapping',);
            expect(mapping.sessionId,).toBe('newest',);
          },
        },),
      ],
    },),
  ],
},);
