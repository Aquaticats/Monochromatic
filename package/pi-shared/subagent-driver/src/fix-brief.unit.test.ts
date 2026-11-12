/**
 The brief a fix agent gets: every placeholder filled, none left, the rules file first, the ban on synthetic load quoted, and no
 owner, hurry or due date in it. Nothing here starts a run or reads a machine.

 @module
 */

import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  FIX_BRIEF_PLACEHOLDERS,
  FIX_BRIEF_WALL_SECONDS,
  NO_SYNTHETIC_HOST_LOAD,
  TemplatePlaceholderError,
  branchOfAFixFor,
  branchOfAReview,
  briefOfAFix,
  fixBriefValues,
  fixRunNameOf,
  placeholdersIn,
  placeholdersInTheFixBrief,
  runNameOfAReview,
} from '../dist/final/node/index.mjs';

/**
 A review's name, as a round writes it.

 @returns the review's name

 @example
 ```ts
 const review = aReview();
 ```
 */
function aReview(): string {
  return 'rewrite-the-gate-round-133';
}

/**
 What the review read, as its own brief words it.

 @returns what was reviewed

 @example
 ```ts
 const subject = aSubject();
 ```
 */
function aSubject(): string {
  return `main...${aReview()} at commit ${aBase()}`;
}

/**
 The commit the review read and the fix starts at.

 @returns the commit's id

 @example
 ```ts
 const base = aBase();
 ```
 */
function aBase(): string {
  return 'abc123';
}

/**
 The namespace a fix branch lives under.

 @returns the namespace

 @example
 ```ts
 const prefix = aBranchPrefix();
 ```
 */
function aBranchPrefix(): string {
  return 'fix/';
}

/**
 The package a fix changes.

 @returns the package's path from the top of the checkout

 @example
 ```ts
 const packageFolder = aPackageFolder();
 ```
 */
function aPackageFolder(): string {
  return 'package/git-policy/cli';
}

/**
 The top of the checkout a fix works in.

 @returns the checkout's path

 @example
 ```ts
 const checkout = aCheckoutFolder();
 ```
 */
function aCheckoutFolder(): string {
  return '/repo';
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
 The rules file the brief names first.

 @returns the file's path

 @example
 ```ts
 const rules = aRulesFile();
 ```
 */
function aRulesFile(): string {
  return `${aCheckoutFolder()}/package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md`;
}

/**
 The request every brief in this file is filled from.

 @returns the request

 @example
 ```ts
 const request = aBriefRequest();
 ```
 */
function aBriefRequest(): {
  readonly review: string;
  readonly branch: string;
  readonly base: string;
  readonly subject: string;
  readonly packageFolder: string;
  readonly reviewsFolder: string;
  readonly checkoutFolder: string;
  readonly rulesFile: string;
} {
  return {
    review: aReview(),
    branch: branchOfAFixFor({ review: aReview(), branchPrefix: aBranchPrefix(), },),
    base: aBase(),
    subject: aSubject(),
    packageFolder: aPackageFolder(),
    reviewsFolder: aReviewsFolder(),
    checkoutFolder: aCheckoutFolder(),
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
  return briefOfAFix(aBriefRequest(),);
}

await describe({
  name: '',
  children: [
    describe({
      name: fixRunNameOf.name,
      children: [
        it({
          name: 'names a fix run of its own, saying which review it fixes and sharing nothing with that review\'s run',
          fn: async () => {
            expect(fixRunNameOf({ review: aReview(), },),).toBe(`fix-of-${aReview()}`,);
            // The review's own run and worktree are what a fix used to collide with.
            expect(fixRunNameOf({ review: aReview(), },),).not.toBe(runNameOfAReview({ review: aReview(), },),);
            expect(fixRunNameOf({ review: aReview(), },),).toContain(aReview(),);
          },
        },),
      ],
    },),
    describe({
      name: branchOfAFixFor.name,
      children: [
        it({
          name: 'puts a fix on a branch of its own under the namespace its caller names, never on the reviewed change\'s branch or the review\'s own',
          fn: async () => {
            expect(branchOfAFixFor({ review: aReview(), branchPrefix: aBranchPrefix(), },),)
              .toBe(`${aBranchPrefix()}${aReview()}`,);
            expect(branchOfAFixFor({ review: aReview(), branchPrefix: aBranchPrefix(), },),)
              .not.toBe(branchOfAReview({ review: aReview(), },),);
            // Two rounds of one change answer on two branches, so the second does not find the first one's commits.
            expect(branchOfAFixFor({ review: `${aReview()}`, branchPrefix: aBranchPrefix(), },),)
              .not.toBe(branchOfAFixFor({ review: 'rewrite-the-gate-round-134', branchPrefix: aBranchPrefix(), },),);
            expect(branchOfAFixFor({ review: aReview(), branchPrefix: aBranchPrefix(), },).toLowerCase(),)
              .not.toContain('kimi',);
            expect(branchOfAFixFor({ review: aReview(), branchPrefix: aBranchPrefix(), },).toLowerCase(),)
              .not.toContain('qwen',);
          },
        },),
      ],
    },),
    describe({
      name: 'the fix brief\'s placeholders',
      children: [
        it({
          name: 'are the ones every brief fills, so a placeholder added to the text is a failing test rather than a word an agent reads as its task',
          fn: async () => {
            expect([...placeholdersInTheFixBrief(),],).toEqual([...FIX_BRIEF_PLACEHOLDERS,],);
            /**
             The names the brief's values fill.
             */
            const filledNames = Object.keys(
              fixBriefValues(aBriefRequest(),),
            );
            expect(filledNames,).toEqual([...FIX_BRIEF_PLACEHOLDERS,],);
          },
        },),
      ],
    },),
    describe({
      name: briefOfAFix.name,
      children: [
        it({
          name: 'fills every placeholder, so no agent reads one as its task',
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
          name: 'names no owner, no hurry and no due date',
          fn: async () => {
            expect(aBrief().toLowerCase(),).not.toContain('owner',);
            expect(aBrief().toLowerCase(),).not.toContain('asap',);
            expect(aBrief().toLowerCase(),).not.toContain('urgent',);
            expect(aBrief().toLowerCase(),).not.toContain('due date',);
          },
        },),
        it({
          name: 'names the package, the branch, the commit, what the review read, the worktree and the review\'s files, so an agent reads the review it answers',
          fn: async () => {
            expect(aBrief(),).toContain(aPackageFolder(),);
            expect(aBrief(),).toContain(branchOfAFixFor({ review: aReview(), branchPrefix: aBranchPrefix(), },),);
            expect(aBrief(),).toContain(aBase(),);
            expect(aBrief(),).toContain(aSubject(),);
            expect(aBrief(),).toContain(join(
              aCheckoutFolder(),
              '.claude',
              'worktrees',
              `pi-${fixRunNameOf({ review: aReview(), },)}`,
            ),);
            expect(aBrief(),).toContain(join(aReviewsFolder(), aReview(), 'review.md',),);
            expect(aBrief(),).toContain(join(aReviewsFolder(), aReview(), 'change.diff',),);
            expect(aBrief(),).toContain(join(aReviewsFolder(), aReview(), 'reproductions',),);
          },
        },),
        it({
          name: 'refuses a request whose values leave a placeholder unfilled',
          fn: async () => {
            expect(() => briefOfAFix({
              ...aBriefRequest(),
              base: '@BASE@',
            },),).toThrow(TemplatePlaceholderError,);
          },
        },),
      ],
    },),
    describe({
      name: 'the wall a fix gets',
      children: [
        it({
          name: 'is longer than no run at all, so a fix that reproduces each defect and runs the package\'s tests has time to finish',
          fn: async () => {
            expect(FIX_BRIEF_WALL_SECONDS,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
  ],
},);
