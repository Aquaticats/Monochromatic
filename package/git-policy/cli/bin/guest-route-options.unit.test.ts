/**
 Guards for the command line the four ZFS guest drivers read: the step, the guest name, the log folder and the flag that
 keeps a guest, including the lines that give a flag no word at all.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { guestRouteOptions, } from './guest-route-options.ts';

/** The guest name a line that names none asks for, which every case below either keeps or replaces. */
const DEFAULT_NAME = 'forced-act';

/** The log folder a line that names none asks for, which every case below either keeps or replaces. */
const DEFAULT_LOG_DIR = '/repo/package/git-policy/cli/target/forced-act-guest-logs';

/**
 Read one command line the way a driver reads it.

 @param words - the words after the script's own path, which this adds the interpreter's two words to

 @returns the step and the values the line gave
 */
function readLine(words: readonly string[],): ReturnType<typeof guestRouteOptions> {
  return guestRouteOptions({
    argv: [
      '/usr/bin/node',
      'e2e/forced-act-guest.ts',
      ...words,
    ],
    defaultName: DEFAULT_NAME,
    defaultLogDir: DEFAULT_LOG_DIR,
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: guestRouteOptions.name,
      children: [
        it({
          name: 'reads the step word and the defaults when the line names nothing else',
          fn: async () => {
            const chosen = readLine(['run',],);
            expect(chosen.step,).toBe('run',);
            expect(chosen.name,).toBe(DEFAULT_NAME,);
            expect(chosen.logDir,).toBe(DEFAULT_LOG_DIR,);
            expect(chosen.keepGuest,).toBe(false,);
          },
        },),
        it({
          name: 'reads the guest name and the log folder the line gave',
          fn: async () => {
            const chosen = readLine([
              'prepare',
              '--name',
              'issue-6581-part3-guest',
              '--log-dir',
              '/home/person/temp/guest-logs',
            ],);
            expect(chosen.step,).toBe('prepare',);
            expect(chosen.name,).toBe('issue-6581-part3-guest',);
            expect(chosen.logDir,).toBe('/home/person/temp/guest-logs',);
          },
        },),
        it({
          name: 'keeps the default when the line ends at the flag it was giving',
          fn: async () => {
            const chosen = readLine([
              'run',
              '--name',
            ],);
            expect(chosen.name,).toBe(DEFAULT_NAME,);
          },
        },),
        it({
          name: 'keeps the empty word a line gave a flag, as the drivers read the word after the flag',
          fn: async () => {
            const chosen = readLine([
              'run',
              '--name',
              '',
              '--log-dir',
              '',
            ],);
            expect(chosen.name,).toBe('',);
            expect(chosen.logDir,).toBe('',);
          },
        },),
        it({
          name: 'reads the flags in any order after the step',
          fn: async () => {
            const chosen = readLine([
              'all',
              '--log-dir',
              '/home/person/logs',
              '--name',
              'named-guest',
            ],);
            expect(chosen.name,).toBe('named-guest',);
            expect(chosen.logDir,).toBe('/home/person/logs',);
          },
        },),
        it({
          name: 'keeps the guest when the line asks for it, wherever the flag stands',
          fn: async () => {
            expect(readLine([
              'all',
              '--keep-guest',
            ],).keepGuest,).toBe(true,);
            expect(readLine([
              'all',
              '--name',
              'named-guest',
              '--keep-guest',
            ],).keepGuest,).toBe(true,);
          },
        },),
        it({
          name: 'ignores a flag no driver knows, and the word after it',
          fn: async () => {
            const chosen = readLine([
              'run',
              '--unknown',
              'its-word',
            ],);
            expect(chosen.step,).toBe('run',);
            expect(chosen.name,).toBe(DEFAULT_NAME,);
          },
        },),
        it({
          name: 'reads the first word as the step even when a flag came first, and so reads no name at all',
          fn: async () => {
            const chosen = readLine([
              '--name',
              'named-guest',
              'run',
            ],);
            expect(chosen.step,).toBe('--name',);
            expect(chosen.name,).toBe(DEFAULT_NAME,);
          },
        },),
        it({
          name: 'reads the name the first time the flag names it, as the word search always did',
          fn: async () => {
            const chosen = readLine([
              'run',
              '--name',
              'first-guest',
              '--name',
              'second-guest',
            ],);
            expect(chosen.name,).toBe('first-guest',);
          },
        },),
        it({
          name: 'reads no step at all as the empty step',
          fn: async () => {
            expect(readLine([],).step,).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
