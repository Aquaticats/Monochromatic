/**
 Reading the review starter's command line: which branches, the integration their changes are read against, the source folder, the
 folders a coordinator has always had, and the refusals of an option that cannot be used.

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
  DEFAULT_REVIEWS_FOLDER_IN_HOME,
  DEFAULT_RUNS_FOLDER_IN_HOME,
  StartReviewsOptionsError,
  parseStartReviewsOptions,
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
      name: parseStartReviewsOptions.name,
      children: [
        it({
          name: 'reads each branch in the order given, the integration, the source, the folders a coordinator has always had, and the checkout it was read in',
          fn: async () => {
            expect(parseStartReviewsOptions({
              commandLine: [
                '--branch=fix/a',
                '--branch=fix/b',
                '--integration=main',
                '--source=package/a/src',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toEqual({
              branches: ['fix/a', 'fix/b',],
              integration: 'main',
              source: 'package/a/src',
              reviewsFolder: join(homedir(), ...DEFAULT_REVIEWS_FOLDER_IN_HOME,),
              runsFolder: join(homedir(), ...DEFAULT_RUNS_FOLDER_IN_HOME,),
              checkoutFolder: aCheckoutFolder(),
            },);
          },
        },),
        it({
          name: 'reads the folders a caller names instead of the defaults',
          fn: async () => {
            /**
             What the caller asked for.
             */
            const options = parseStartReviewsOptions({
              commandLine: [
                '--branch=fix/a',
                '--integration=main',
                '--source=package/a/src',
                '--reviews-folder=/reviews',
                '--runs-folder=/runs',
              ],
              checkoutFolder: aCheckoutFolder(),
            },);
            expect(options.reviewsFolder,).toBe('/reviews',);
            expect(options.runsFolder,).toBe('/runs',);
          },
        },),
        it({
          name: 'refuses a command line naming no branch, no integration or no source',
          fn: async () => {
            expect(() => parseStartReviewsOptions({
              commandLine: [],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow(StartReviewsOptionsError,);
            expect(() => parseStartReviewsOptions({
              commandLine: [
                '--integration=main',
                '--source=package/a/src',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--branch is required',);
            expect(() => parseStartReviewsOptions({
              commandLine: [
                '--branch=fix/a',
                '--source=package/a/src',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--integration is required',);
            expect(() => parseStartReviewsOptions({
              commandLine: [
                '--branch=fix/a',
                '--integration=main',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--source is required',);
          },
        },),
      ],
    },),
  ],
},);
