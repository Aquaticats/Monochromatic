/**
 The folder Pi keeps its session files in, whose room a new run needs: Pi's own folder under the home directory, or the one a
 session sends Pi to. No test here reads this machine's home directory or its disk.

 @module
 */

import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { sessionFolderOf, } from '../dist/final/node/index.mjs';

/**
 The home directory every folder in this file is under.

 @returns the home directory's path

 @example
 ```ts
 const home = aHomePath();
 ```
 */
function aHomePath(): string {
  return '/home/a-person-of-the-test';
}

await describe({
  name: '',
  children: [
    describe({
      name: sessionFolderOf.name,
      children: [
        it({
          name: 'answers the folder Pi keeps its session files under the home directory while no session names another',
          fn: async () => {
            expect(sessionFolderOf({ home: aHomePath(), },),).toBe(join(
              aHomePath(),
              '.pi',
              'agent',
              'sessions',
            ),);
            expect(sessionFolderOf({
              home: aHomePath(),
              sessionFolder: '',
            },),).toBe(join(
              aHomePath(),
              '.pi',
              'agent',
              'sessions',
            ),);
          },
        },),
        it({
          name: 'answers the folder a session sends Pi to when it names one, because that is the drive the write fills',
          fn: async () => {
            /**
             The folder the environment sends Pi to, which is on another drive than the home directory.
             */
            const sessionFolder = '/mnt/encrypted/pi-sessions';
            expect(sessionFolderOf({
              home: aHomePath(),
              sessionFolder,
            },),).toBe(sessionFolder,);
          },
        },),
      ],
    },),
  ],
},);
