import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readByPidDir,
  readPidMapping,
  SESSION_NOT_FOUND,
} from './index.ts';
import {
  BY_PID_DIR,
  fakeIo,
  mappingJson,
  parseTestMapping,
} from './test-support.ts';

await describe({
  name: 'session-discovery mapping reads',
  children: [
    describe({
      name: readPidMapping.name,
      children: [
        it({
          name: 'reads direct pid mapping',
          fn: async function testReadPidMapping() {
            const mapping = await readPidMapping({
              pid: 200,
              byPidDir: BY_PID_DIR,
              parseMapping: parseTestMapping,
              io: fakeIo({
                files: { [`${BY_PID_DIR}/200`]: mappingJson('session-200'), },
                mtimes: {},
                parents: new Map(),
              },),
            },);

            if (mapping === SESSION_NOT_FOUND)
              throw new Error('expected mapping',);
            expect(mapping.sessionId,).toBe('session-200',);
          },
        },),
        it({
          name: 'returns sentinel for missing pid mapping',
          fn: async function testMissingPidMapping() {
            const mapping = await readPidMapping({
              pid: 404,
              byPidDir: BY_PID_DIR,
              parseMapping: parseTestMapping,
              io: fakeIo({
                files: {},
                mtimes: {},
                parents: new Map(),
              },),
            },);

            expect(mapping,).toBe(SESSION_NOT_FOUND,);
          },
        },),
      ],
    },),
    describe({
      name: readByPidDir.name,
      children: [
        it({
          name: 'returns sentinel when directory read fails',
          fn: async function testMissingByPidDir() {
            const entries = await readByPidDir({
              byPidDir: BY_PID_DIR,
              io: {
                readDir: function readDir(): Promise<readonly string[]> {
                  return Promise.reject(new Error('missing dir',),);
                },
              },
            },);

            expect(entries,).toBe(SESSION_NOT_FOUND,);
          },
        },),
      ],
    },),
  ],
},);
