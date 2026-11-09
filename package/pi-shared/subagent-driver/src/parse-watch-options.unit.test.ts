/**
 What `watch-overdue-pi-runs` accepts on its command line, and what it refuses.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  LOOK_INTERVAL_MILLISECONDS,
  RECHECK_MILLISECONDS,
  WatchOptionsError,
  parseWatchOptions,
} from '../dist/final/node/index.mjs';

/**
 The least a command line can say and still bound the wait.
 */
const COMPLETE = ['--runs-folder=/runs', '--deadline-seconds=60',] as const;

/**
 Reads the error a command line gives.

 @param commandLine - arguments

 @returns the error's message

 @example
 ```ts
 const message = errorOf(['--runs-folder=/runs',],);
 ```
 */
function errorOf(commandLine: readonly string[],): string {
  try {
    parseWatchOptions(commandLine,);
  }
  catch (error) {
    expect(error,).toBeInstanceOf(WatchOptionsError,);
    return (error as WatchOptionsError).message;
  }
  throw new Error('the command line was accepted',);
}

await describe({
  name: parseWatchOptions.name,
  children: [
    it({
      name: 'reads a command line that names only where the runs are and how long to look, taking the interval defaults',
      fn: async () => {
        /**
         What the run was asked for.
         */
        const options = parseWatchOptions(COMPLETE,);
        expect(options,).toEqual({
          runsFolder: '/runs',
          deadlineSeconds: 60,
          intervalSeconds: LOOK_INTERVAL_MILLISECONDS / 1_000,
          recheckSeconds: RECHECK_MILLISECONDS / 1_000,
          once: false,
        },);
      },
    },),
    it({
      name: 'reads every option, including a limit of its own and the file that keeps what was reported',
      fn: async () => {
        expect(parseWatchOptions([
          ...COMPLETE,
          '--interval-seconds=30',
          '--recheck-seconds=90',
          '--limit-minutes=5',
          '--state-file=/state/reported.json',
          '--once',
        ],),).toEqual({
          runsFolder: '/runs',
          deadlineSeconds: 60,
          intervalSeconds: 30,
          recheckSeconds: 90,
          limitMinutes: 5,
          stateFile: '/state/reported.json',
          once: true,
        },);
      },
    },),
    it({
      name: 'requires the runs folder and the deadline, so no look is taken without either',
      fn: async () => {
        for (const name of ['runs-folder', 'deadline-seconds',]) {
          expect(errorOf(COMPLETE.filter(function named(option,) {
            return !option.startsWith(`--${name}`,);
          },),),).toContain(`--${name} is required`,);
        }
      },
    },),
    it({
      name: 'refuses a duration that is not a number above zero or whose milliseconds could not be finite',
      fn: async () => {
        for (const value of ['0', '-1', 'soon', 'Infinity', '1e308',]) {
          expect(errorOf(['--runs-folder=/runs', `--deadline-seconds=${value}`,],),).toContain('--deadline-seconds',);
        }
      },
    },),
    it({
      name: 'refuses an unknown option and a stray word, showing the usage',
      fn: async () => {
        expect(errorOf([...COMPLETE, '--nope=1',],),).toContain('Usage:',);
        expect(errorOf([...COMPLETE, 'stray',],),).toContain('Usage:',);
      },
    },),
  ],
},);
