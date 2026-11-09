/**
 Tests for the coverage harness's `--verbose` argv scope.

 The console sink decides verbose output from `process.argv`, so the coverage
 scenarios that need the flag take it for a scope. Restoring the arguments the
 scope started with is what keeps the flag out of the scenarios that follow and
 keeps their own arguments in place.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { verboseArgv, } from './coverage-console.ts';

//region Fixtures

/**
 Argument another writer appends while the scope is open.
 */
const APPENDED_ARGUMENT = '--appended-during-scope';

/**
 Reads the arguments of this process.

 @returns the current argv as a plain array
 */
function argv(): readonly string[] {
  return [...process.argv,];
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: verboseArgv.name,
      concurrency: 1,
      children: [
        //region The flag is present only inside the scope

        it({
          name: 'adds the flag inside the scope and removes it on disposal',
          fn: async () => {
            /**
             Arguments from before the scope.
             */
            const before = argv();
            {
              using _flag = verboseArgv();
              expect(argv(),).toContain('--verbose',);
            }
            expect(argv(),).toEqual(before,);
          },
        },),

        //endregion The flag is present only inside the scope

        //region Disposal restores the arguments the scope started with

        it({
          name: 'leaves the arguments as they were when another writer appends during the scope',
          fn: async () => {
            /**
             Arguments from before the scope.
             */
            const before = argv();
            {
              using _flag = verboseArgv();
              process.argv.push(APPENDED_ARGUMENT,);
            }
            expect(argv(),).toEqual(before,);
            expect(argv(),).not.toContain('--verbose',);
          },
        },),

        it({
          name: 'keeps a flag that was already on the arguments before the scope',
          fn: async () => {
            using _callerFlag = {
              [Symbol.dispose](): void {
                process.argv = process.argv
                  .filter(function keepNonVerbose(argument,) {
                    return argument !== '--verbose';
                  },);
              },
            };
            process.argv.push('--verbose',);
            /**
             Arguments carrying the caller's own flag.
             */
            const before = argv();
            {
              using _flag = verboseArgv();
              expect(argv(),).toEqual([
                ...before,
                '--verbose',
              ],);
            }
            expect(argv(),).toEqual(before,);
          },
        },),

        //endregion Disposal restores the arguments the scope started with
      ],
    },),
  ],
},);
