/**
 What the driver accepts on its command line.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  DriverOptionsError,
  parseDriverOptions,
} from './parse-driver-options.ts';

const COMPLETE = [
  '--run-folder=/run',
  '--working-folder=/work',
  '--prompt-file=/prompt',
  '--provider=p',
  '--model=m',
  '--wall-seconds=60',
  '--idle-seconds=1.5',
  '--turn-cap=80',
] as const;

/**
 Reads the error a command line gives.

 @param commandLine - arguments

 @returns the error's message
 */
function errorOf(commandLine: readonly string[],): string {
  try {
    parseDriverOptions(commandLine,);
  }
  catch (error) {
    expect(error,).toBeInstanceOf(DriverOptionsError,);
    return (error as DriverOptionsError).message;
  }
  throw new Error('the command line was accepted',);
}

await describe({
  name: 'parseDriverOptions',
  children: [
    it({
      name: 'reads a complete command line, with no session and no extra words by default',
      fn: () => {
        expect(parseDriverOptions(COMPLETE,),).toEqual({
          runFolder: '/run',
          workingFolder: '/work',
          promptFile: '/prompt',
          provider: 'p',
          model: 'm',
          session: null,
          piArguments: [],
          wallSeconds: 60,
          idleSeconds: 1.5,
          turnCap: 80,
        },);
      },
    },),
    it({
      name: 'keeps a session path with spaces and each extra word whole, in order',
      fn: () => {
        const options = parseDriverOptions([...COMPLETE, '--session=/a b/c.jsonl', '--pi-argument=--x', '--pi-argument=y z',],);
        expect(options.session,).toBe('/a b/c.jsonl',);
        expect(options.piArguments,).toEqual(['--x', 'y z',],);
      },
    },),
    it({
      name: 'requires every limit, so a run is always bounded',
      fn: () => {
        for (const name of ['wall-seconds', 'idle-seconds', 'turn-cap',]) {
          expect(errorOf(COMPLETE.filter((option,) => !option.startsWith(`--${name}`,)),),).toContain(`--${name} is required`,);
        }
      },
    },),
    it({
      name: 'refuses a limit that is not a number above zero',
      fn: () => {
        for (const value of ['0', '-1', 'soon', 'Infinity',]) {
          expect(errorOf([...COMPLETE.filter((option,) => !option.startsWith('--wall-seconds',)), `--wall-seconds=${value}`,],),)
            .toContain('must be a number above zero',);
        }
      },
    },),
    it({
      name: 'refuses an empty required value',
      fn: () => {
        expect(errorOf([...COMPLETE.filter((option,) => !option.startsWith('--model',)), '--model=',],),).toContain('--model is required',);
      },
    },),
    it({
      name: 'refuses an unknown option and a stray word, showing the usage',
      fn: () => {
        expect(errorOf([...COMPLETE, '--nope=1',],),).toContain('Usage:',);
        expect(errorOf([...COMPLETE, 'stray',],),).toContain('Usage:',);
      },
    },),
  ],
},);
