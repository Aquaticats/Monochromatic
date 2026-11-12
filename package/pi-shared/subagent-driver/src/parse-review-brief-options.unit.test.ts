/**
 Reading the brief writer's command line: what it was asked, the defaults a coordinator's own folders give it, and the refusals of
 an option that cannot be used. Nothing here reads a checkout or starts a run.

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
  FOCUS_OF_A_CHANGE_REVIEW,
  RULES_FILE_IN_THE_CHECKOUT,
  ReviewBriefOptionsError,
  parseReviewBriefOptions,
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
    '--review=a',
    '--commit=abc123',
    '--range=main...fix/a',
    '--source=package/a/src',
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: parseReviewBriefOptions.name,
      children: [
        it({
          name: 'reads the change, the commit, each range in the order given, the source, and the folders a coordinator has always had',
          fn: async () => {
            expect(parseReviewBriefOptions({
              commandLine: [
                ...requiredWords(),
                '--range=main...fix/a~2',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toEqual({
              review: 'a',
              commit: 'abc123',
              ranges: ['main...fix/a', 'main...fix/a~2',],
              source: 'package/a/src',
              focus: FOCUS_OF_A_CHANGE_REVIEW,
              reviewsFolder: join(homedir(), ...DEFAULT_REVIEWS_FOLDER_IN_HOME,),
              runsFolder: join(homedir(), ...DEFAULT_RUNS_FOLDER_IN_HOME,),
              rulesFile: join(aCheckoutFolder(), RULES_FILE_IN_THE_CHECKOUT,),
            },);
          },
        },),
        it({
          name: 'reads a focus and a rules file the caller names instead of the defaults',
          fn: async () => {
            /**
             What the caller asked for.
             */
            const options = parseReviewBriefOptions({
              commandLine: [
                ...requiredWords(),
                '--focus=data loss',
                '--rules-file=/rules.md',
                '--reviews-folder=/reviews',
                '--runs-folder=/runs',
              ],
              checkoutFolder: aCheckoutFolder(),
            },);
            expect(options.focus,).toBe('data loss',);
            expect(options.rulesFile,).toBe('/rules.md',);
            expect(options.reviewsFolder,).toBe('/reviews',);
            expect(options.runsFolder,).toBe('/runs',);
          },
        },),
        it({
          name: 'refuses a command line missing a review, a commit, a range or a source',
          fn: async () => {
            expect(() => parseReviewBriefOptions({
              commandLine: [],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow(ReviewBriefOptionsError,);
            expect(() => parseReviewBriefOptions({
              commandLine: [
                '--review=a',
                '--commit=abc123',
                '--source=package/a/src',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--range is required',);
            expect(() => parseReviewBriefOptions({
              commandLine: [
                '--commit=abc123',
                '--range=main...fix/a',
                '--source=package/a/src',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--review is required',);
            expect(() => parseReviewBriefOptions({
              commandLine: [
                '--review=a',
                '--range=main...fix/a',
                '--source=package/a/src',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--commit is required',);
            expect(() => parseReviewBriefOptions({
              commandLine: [
                '--review=a',
                '--commit=abc123',
                '--range=main...fix/a',
              ],
              checkoutFolder: aCheckoutFolder(),
            },),).toThrow('--source is required',);
          },
        },),
      ],
    },),
  ],
},);
