import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readParentPid,
  SESSION_NOT_FOUND,
} from './index.ts';

await describe({
  name: readParentPid.name,
  children: [
    it({
      name: 'parses parent pid from procfs status text through fake file IO',
      fn: async function testReadParentPid() {
        const parent = await readParentPid({
          pid: 123,
          io: {
            readFile: function readFile(path,): Promise<string> {
              expect(path,).toBe('/proc/123/status',);
              return Promise.resolve('Name:\tnode\nPPid:\t42\n',);
            },
          },
        },);

        expect(parent,).toBe(42,);
      },
    },),
    it({
      name: 'returns sentinel when parent pid is unavailable',
      fn: async function testMissingParentPid() {
        const parent = await readParentPid({
          pid: 123,
          io: {
            readFile: function readFile(): Promise<string> {
              return Promise.reject(new Error('missing procfs',),);
            },
          },
        },);

        expect(parent,).toBe(SESSION_NOT_FOUND,);
      },
    },),
  ],
},);
