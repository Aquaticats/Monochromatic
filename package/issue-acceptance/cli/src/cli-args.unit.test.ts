/**
 Tests for the command-line grammar.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CliInvocationError,
  parseArguments,
} from '../dist/final/node/index.mjs';

/**
 Returns the error a call throws.
 
 @param action - Call expected to refuse.
 
 @returns Whatever the call threw.
 
 @throws {@link Error} when the call returns, which means it refused nothing.
 
 @example
 ```ts
 const refused = failureOf(() => parseArguments({ tokens: [] }));
 ```
 */
function failureOf(action: () => unknown,): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('The call returned where the test expects a refusal.',);
}

await describe({
  name: parseArguments.name,
  children: [
    it({
      name: 'reads --help and the help subcommand as the usage request',
      fn: async () => {
        expect(parseArguments({ tokens: ['--help',], },),).toEqual({ kind: 'help', },);
        expect(parseArguments({ tokens: ['help',], },),).toEqual({ kind: 'help', },);
      },
    },),
    it({
      name: 'reads settle with a plan and an optional apply',
      fn: async () => {
        expect(parseArguments({
          tokens: [
            'settle',
            '--plan',
            'plan.json',
          ],
        },),).toEqual({
          kind: 'settle',
          planPath: 'plan.json',
          apply: false,
        },);
        expect(parseArguments({
          tokens: [
            'settle',
            '--plan',
            'plan.json',
            '--apply',
          ],
        },),).toEqual({
          kind: 'settle',
          planPath: 'plan.json',
          apply: true,
        },);
      },
    },),
    it({
      name: 'refuses settle without a plan file',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({ tokens: ['settle',], },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('settle needs --plan FILE',);
      },
    },),
    it({
      name: 'refuses a flag that belongs to another subcommand',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({
          tokens: [
            'settle',
            '--plan',
            'plan.json',
            '--repo',
            'OWNER/NAME',
          ],
        },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('settle does not take --repo.',);
      },
    },),
    it({
      name: 'refuses a positional argument on settle',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({
          tokens: [
            'settle',
            'plan.json',
          ],
        },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('settle takes no positional argument',);
      },
    },),
    it({
      name: 'reads show with numbers, a repository, and the acceptance mode',
      fn: async () => {
        expect(parseArguments({
          tokens: [
            'show',
            '--repo',
            'OWNER/NAME',
            '7',
            '8',
            '--acceptance',
          ],
        },),).toEqual({
          kind: 'show',
          repository: 'OWNER/NAME',
          numbers: [7, 8,],
          mode: 'acceptance',
        },);
      },
    },),
    it({
      name: 'refuses show without a repository',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({ tokens: ['show', '7',], },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('show needs --repo OWNER/NAME',);
      },
    },),
    it({
      name: 'refuses show without an issue number',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({
          tokens: [
            'show',
            '--repo',
            'OWNER/NAME',
          ],
        },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('show needs at least one issue number',);
      },
    },),
    it({
      name: 'refuses a number that is not an issue number',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({
          tokens: [
            'show',
            '--repo',
            'OWNER/NAME',
            'seven',
          ],
        },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('is not one',);
      },
    },),
    it({
      name: 'reads ready-bugs with and without an output file',
      fn: async () => {
        expect(parseArguments({
          tokens: [
            'ready-bugs',
            '--repo',
            'OWNER/NAME',
          ],
        },),).toEqual({
          kind: 'ready-bugs',
          repository: 'OWNER/NAME',
        },);
        expect(parseArguments({
          tokens: [
            'ready-bugs',
            '--repo',
            'OWNER/NAME',
            '--out',
            'grouping.json',
          ],
        },),).toEqual({
          kind: 'ready-bugs',
          repository: 'OWNER/NAME',
          outPath: 'grouping.json',
        },);
      },
    },),
    it({
      name: 'refuses an unknown subcommand without a command',
      fn: async () => {
        expect(failureOf(() => parseArguments({ tokens: [], },),),).toBeInstanceOf(CliInvocationError,);
        /**
         Value the parse of an unknown command threw.
         */
        const refused = failureOf(() => parseArguments({ tokens: ['close',], },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('is not a subcommand',);
      },
    },),
    it({
      name: 'refuses an unknown flag with the usage text',
      fn: async () => {
        /**
         Value the parse threw.
         */
        const refused = failureOf(() => parseArguments({ tokens: ['settle', '--aply',], },),);
        expect(refused,).toBeInstanceOf(CliInvocationError,);
        expect((refused as Error).message,).toContain('Usage:',);
      },
    },),
  ],
},);
