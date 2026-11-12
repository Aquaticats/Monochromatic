/**
 Reading the fix brief writer's command line: which reviews, the commit a fix starts at, its branch namespace, its package, the
 batch list, and the refusals of an option that cannot be used.

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
  FIX_BRIEF_WALL_SECONDS,
  FixBriefOptionsError,
  parseFixBriefOptions,
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

/**
 The words every command line in this file carries.

 @returns the required words

 @example
 ```ts
 const words = requiredWords();
 ```
 */
function requiredWords(): readonly string[] {
  return [
    '--review=a-round-1',
    '--base=abc123',
    '--branch-prefix=fix/',
    '--package=package/a',
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: parseFixBriefOptions.name,
      children: [
        it({
          name: 'reads each review in the order given, the base, the branch namespace, the package, the folders a coordinator has always had, and a wall the scratch writers gave a fix',
          fn: async () => {
            expect(parseFixBriefOptions({
              commandLine: [
                ...requiredWords(),
                '--review=b-round-2',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toEqual({
              reviews: ['a-round-1', 'b-round-2',],
              base: 'abc123',
              branchPrefix: 'fix/',
              packageFolder: 'package/a',
              reviewsFolder: join(homedir(), ...DEFAULT_REVIEWS_FOLDER_IN_HOME,),
              runsFolder: join(homedir(), ...DEFAULT_RUNS_FOLDER_IN_HOME,),
              wallSeconds: FIX_BRIEF_WALL_SECONDS,
              checkoutFolder: aCheckoutFolder(),
            },);
          },
        },),
        it({
          name: 'reads a batch list, a wall and folders a caller names',
          fn: async () => {
            /**
             What the caller asked for.
             */
            const options = parseFixBriefOptions({
              commandLine: [
                ...requiredWords(),
                '--batch-file=/runs/batch.json',
                '--wall-seconds=600',
                '--reviews-folder=/reviews',
                '--runs-folder=/runs',
              ],
              checkoutFolder: aCheckoutFolder(),
            },);
            expect(options.batchFile,).toBe('/runs/batch.json',);
            expect(options.wallSeconds,).toBe(600,);
            expect(options.reviewsFolder,).toBe('/reviews',);
            expect(options.runsFolder,).toBe('/runs',);
          },
        },),
        it({
          name: 'names no batch list when the caller names none, so nothing is written beside the briefs',
          fn: async () => {
            expect(parseFixBriefOptions({
              commandLine: requiredWords(),
              checkoutFolder: aCheckoutFolder(),
            },).batchFile,).toBeUndefined();
          },
        },),
        it({
          name: 'refuses a command line naming no review, no base, no branch namespace or no package, and a wall that is not a number above zero',
          fn: async () => {
            expect(() => parseFixBriefOptions({
              commandLine: [],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow(FixBriefOptionsError,);
            expect(() => parseFixBriefOptions({
              commandLine: [
                '--base=abc123',
                '--branch-prefix=fix/',
                '--package=package/a',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--review is required',);
            expect(() => parseFixBriefOptions({
              commandLine: [
                '--review=a',
                '--branch-prefix=fix/',
                '--package=package/a',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--base is required',);
            expect(() => parseFixBriefOptions({
              commandLine: [
                '--review=a',
                '--base=abc123',
                '--package=package/a',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--branch-prefix is required',);
            expect(() => parseFixBriefOptions({
              commandLine: [
                '--review=a',
                '--base=abc123',
                '--branch-prefix=fix/',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--package is required',);
            expect(() => parseFixBriefOptions({
              commandLine: [
                ...requiredWords(),
                '--wall-seconds=soon',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--wall-seconds must be a number above zero',);
          },
        },),
      ],
    },),
  ],
},);
