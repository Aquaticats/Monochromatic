import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { keepIdArguments } from './container-user.ts';

await describe({
  name: keepIdArguments.name,
  children: [
    it({
      name: 'maps the host user onto the numeric user and group of the image',
      fn: async () => {
        expect(keepIdArguments('1000:1001')).toEqual(['--userns=keep-id:uid=1000,gid=1001']);
      },
    }),
    it({
      name: 'maps the host user onto root for an image with no user, and keeps the image running as root',
      fn: async () => {
        expect(keepIdArguments('')).toEqual(['--userns=keep-id:uid=0,gid=0']);
        expect(keepIdArguments('\n')).toEqual(['--userns=keep-id:uid=0,gid=0']);
      },
    }),
    it({
      name: 'gives a user without a group the root group',
      fn: async () => {
        expect(keepIdArguments('1000')).toEqual(['--userns=keep-id:uid=1000,gid=0']);
      },
    }),
    it({
      name: 'adds nothing for a named user, a named group or extra parts, which only the image can resolve',
      fn: async () => {
        expect(keepIdArguments('tester')).toEqual([]);
        expect(keepIdArguments('1000:staff')).toEqual([]);
        expect(keepIdArguments('1000:1000:1000')).toEqual([]);
        expect(keepIdArguments('-1')).toEqual([]);
      },
    }),
  ],
});
