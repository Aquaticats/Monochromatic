/**
 Reading the check's command line: which reviews to check, the folders they live in, the rules file and focus a restart's brief
 takes, and whether the restarts it writes are started.

 @module
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CHECK_REVIEW_USAGE,
  CheckReviewOptionsError,
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
  FOCUS_OF_A_CHANGE_REVIEW,
  RULES_FILE_IN_THE_CHECKOUT,
  START_RESTARTS_FLAG,
  parseCheckReviewOptions,
} from '../dist/final/node/index.mjs';

/**
 The top of the checkout the options are read in.

 @returns the checkout's path

 @example
 ```ts
 const folder = aCheckoutFolder();
 ```
 */
function aCheckoutFolder(): string {
  return '/repo';
}

await describe({
  name: '',
  children: [
    describe({
      name: parseCheckReviewOptions.name,
      children: [
        it({
          name: 'reads each review in the order given, with the folders, rules file and focus a coordinator has always had',
          fn: async () => {
            expect(parseCheckReviewOptions({
              commandLine: ['--review=a', '--review=b',],
              checkoutFolder: aCheckoutFolder(),
            },),).toEqual({
              reviews: ['a', 'b',],
              reviewsFolder: join(homedir(), ...DEFAULT_REVIEWS_FOLDER_IN_HOME,),
              runsFolder: join(homedir(), ...DEFAULT_RUNS_FOLDER_IN_HOME,),
              rulesFile: join(aCheckoutFolder(), RULES_FILE_IN_THE_CHECKOUT,),
              focus: FOCUS_OF_A_CHANGE_REVIEW,
              start: false,
            },);
          },
        },),
        it({
          name: 'reads the folders, the rules file and the focus the caller names, and the flag that starts what it writes',
          fn: async () => {
            const options = parseCheckReviewOptions({
              commandLine: [
                '--review=a',
                '--reviews-folder=/reviews',
                '--runs-folder=/runs',
                '--rules-file=/rules.md',
                '--focus=data loss',
                START_RESTARTS_FLAG,
              ],
              checkoutFolder: aCheckoutFolder(),
            },);
            expect(options.reviewsFolder,).toBe('/reviews',);
            expect(options.runsFolder,).toBe('/runs',);
            expect(options.rulesFile,).toBe('/rules.md',);
            expect(options.focus,).toBe('data loss',);
            expect(options.start,).toBe(true,);
          },
        },),
        it({
          name: 'refuses a command line naming no review, and one naming an empty review, with the usage',
          fn: async () => {
            try {
              parseCheckReviewOptions({
                commandLine: [],
                checkoutFolder: aCheckoutFolder(),
              },);
              expect('it read',).toBe('it refused',);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(CheckReviewOptionsError,);
              expect((error as Error).message,).toContain('--review is required at least once',);
              expect((error as Error).message,).toContain(CHECK_REVIEW_USAGE,);
            }
            try {
              parseCheckReviewOptions({
                commandLine: ['--review=',],
                checkoutFolder: aCheckoutFolder(),
              },);
              expect('it read',).toBe('it refused',);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(CheckReviewOptionsError,);
              expect((error as Error).message,).toContain('--review is required',);
            }
          },
        },),
      ],
    },),
  ],
},);
