/**
 What `check-in-on-pi-runs` accepts on its command line, and what it refuses.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CheckInOptionsError,
  parseCheckInOptions,
} from '../dist/final/node/index.mjs';

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
    parseCheckInOptions(commandLine,);
  }
  catch (error) {
    expect(error,).toBeInstanceOf(CheckInOptionsError,);
    return (error as CheckInOptionsError).message;
  }
  throw new Error('the command line was accepted',);
}

await describe({
  name: parseCheckInOptions.name,
  children: [
    it({
      name: 'reads the runs folder and every unit named, as a unit name or as a run name',
      fn: async () => {
        /**
         What the run was asked for.
         */
        const options = parseCheckInOptions([
          '--runs-folder=/runs',
          '--unit=pi-alpha.service',
          '--unit=beta',
          '--unit=pi-gamma',
        ],);
        expect(options.runsFolder,).toBe('/runs',);
        expect(options.unitNames,).toEqual([
          'pi-alpha.service',
          'pi-beta.service',
          'pi-gamma.service',
        ],);
      },
    },),
    it({
      name: 'requires at least one unit, because a run with none describes nothing',
      fn: async () => {
        expect(errorOf(['--runs-folder=/runs',],),).toContain('--unit is required',);
      },
    },),
    it({
      name: 'requires the runs folder, so it never guesses where the runs are',
      fn: async () => {
        expect(errorOf(['--unit=alpha',],),).toContain('--runs-folder is required',);
      },
    },),
    it({
      name: 'refuses an unknown option and a stray word, showing the usage',
      fn: async () => {
        expect(errorOf(['--runs-folder=/runs', '--unit=alpha', '--nope=1',],),).toContain('Usage:',);
        expect(errorOf(['--runs-folder=/runs', '--unit=alpha', 'stray',],),).toContain('Usage:',);
      },
    },),
  ],
},);
