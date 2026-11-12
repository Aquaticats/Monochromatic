/**
 The brief a review of a change reads: every placeholder filled, none left, and the sentences the project requires in it, which
 are the rules file first, the requirements as the project's own, and the review file written at the end. Nothing here starts a
 run or reads a machine.

 @module
 */

import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  FOCUS_OF_A_CHANGE_REVIEW,
  NO_SYNTHETIC_HOST_LOAD,
  REVIEW_BRIEF_PLACEHOLDERS,
  TemplatePlaceholderError,
  briefOfAReview,
  placeholdersIn,
  placeholdersInTheReviewBrief,
  reviewBriefValues,
} from '../dist/final/node/index.mjs';

/**
 The change a review reads.

 @returns the change's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'rewrite-the-gate';
}

/**
 The commit under review.

 @returns the commit's id

 @example
 ```ts
 const commit = aCommit();
 ```
 */
function aCommit(): string {
  return 'abc123';
}

/**
 The folder whose changes are reviewed.

 @returns the folder's path

 @example
 ```ts
 const source = aSourceFolder();
 ```
 */
function aSourceFolder(): string {
  return 'package/git-policy/cli/src/native';
}

/**
 A file the reviewer reads.

 @returns the file's path

 @example
 ```ts
 const file = aReviewedFile();
 ```
 */
function aReviewedFile(): string {
  return `${aSourceFolder()}/gate.rs`;
}

/**
 A folder that holds one folder per review.

 @returns the folder's path

 @example
 ```ts
 const folder = aReviewsFolder();
 ```
 */
function aReviewsFolder(): string {
  return '/reviews';
}

/**
 A folder the driver writes its runs into.

 @returns the folder's path

 @example
 ```ts
 const folder = aRunsFolder();
 ```
 */
function aRunsFolder(): string {
  return '/runs';
}

/**
 The rules file the brief names first.

 @returns the file's path

 @example
 ```ts
 const rules = aRulesFile();
 ```
 */
function aRulesFile(): string {
  return '/repo/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md';
}

/**
 The values every expectation in this file fills its briefs with.

 @returns the request every brief here is filled from

 @example
 ```ts
 const request = aBriefRequest();
 ```
 */
function aBriefRequest(): {
  readonly review: string;
  readonly commit: string;
  readonly source: string;
  readonly files: readonly string[];
  readonly focus: string;
  readonly reviewsFolder: string;
  readonly runsFolder: string;
  readonly rulesFile: string;
} {
  return {
    review: aReview(),
    commit: aCommit(),
    source: aSourceFolder(),
    files: [aReviewedFile(),],
    focus: FOCUS_OF_A_CHANGE_REVIEW,
    reviewsFolder: aReviewsFolder(),
    runsFolder: aRunsFolder(),
    rulesFile: aRulesFile(),
  };
}

/**
 The brief this file's expectations are about.

 @returns the brief's text

 @example
 ```ts
 const brief = aBrief();
 ```
 */
function aBrief(): string {
  return briefOfAReview(aBriefRequest(),);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the review brief\'s placeholders',
      children: [
        it({
          name: 'are the ones every brief fills, so a placeholder added to the text is a failing test rather than a word a reviewer reads as its task',
          fn: async () => {
            expect([...placeholdersInTheReviewBrief(),],)
              .toEqual([...REVIEW_BRIEF_PLACEHOLDERS,],);
            /**
             The names the brief's values fill.
             */
            const filledNames = Object.keys(
              reviewBriefValues(aBriefRequest(),),
            );
            expect(filledNames,).toEqual([...REVIEW_BRIEF_PLACEHOLDERS,],);
          },
        },),
      ],
    },),
    describe({
      name: briefOfAReview.name,
      children: [
        it({
          name: 'fills every placeholder, so no reviewer reads one as its task',
          fn: async () => {
            expect(placeholdersIn({ template: aBrief(), },),).toEqual([],);
          },
        },),
        it({
          name: 'names the rules file first, and quotes the ban on synthetic host load',
          fn: async () => {
            expect(aBrief().startsWith(`Read \`${aRulesFile()}\` first and follow it.`,),).toBe(true,);
            expect(aBrief(),).toContain(NO_SYNTHETIC_HOST_LOAD,);
          },
        },),
        it({
          name: 'states the requirements as the project\'s own, names this machine\'s session bus, and never names an owner',
          fn: async () => {
            expect(aBrief(),).toContain('It must meet these requirements:',);
            expect(aBrief(),).toContain('this machine\'s session bus',);
            expect(aBrief().toLowerCase(),).not.toContain('owner',);
          },
        },),
        it({
          name: 'asks for the review file as soon as the reading is done, marked not final, because a review that ends with no file is lost',
          fn: async () => {
            expect(aBrief(),).toContain('marked at its top as not final',);
            expect(aBrief(),).toContain('A review that ends with no file is lost',);
            expect(aBrief(),).toContain('every command\'s working folder stays inside',);
          },
        },),
        it({
          name: 'names the commit, each file to read, the focus and the review file, and asks for the review file before the end',
          fn: async () => {
            expect(aBrief(),).toContain(aCommit(),);
            expect(aBrief(),).toContain(aReviewedFile(),);
            expect(aBrief(),).toContain(FOCUS_OF_A_CHANGE_REVIEW,);
            expect(aBrief(),).toContain(join(aReviewsFolder(), aReview(), 'review.md',),);
            expect(aBrief(),).toContain('write your review file before anything else you do at the end',);
          },
        },),
        it({
          name: 'lists the folders of earlier reviews, and the reviews folder outside the review\'s own, among the answers it must not read',
          fn: async () => {
            expect(aBrief(),).toContain('/var/home/user/temp/agent/kimi-reviews',);
            expect(aBrief(),).toContain('/var/home/user/temp/agent/qwen-trials',);
            expect(aBrief(),).toContain(`\`${aReviewsFolder()}\` outside your own review folder`,);
          },
        },),
        it({
          name: 'refuses a request whose values leave a placeholder unfilled',
          fn: async () => {
            expect(() => briefOfAReview({
              ...aBriefRequest(),
              reviewsFolder: `${aReviewsFolder()}/@FOCUS@`,
            },),).toThrow(TemplatePlaceholderError,);
          },
        },),
      ],
    },),
  ],
},);
