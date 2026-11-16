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
  type LeftOutFile,
  NO_SYNTHETIC_HOST_LOAD,
  REVIEW_BRIEF_PLACEHOLDERS,
  TemplatePlaceholderError,
  WHAT_TESTS_AND_THEIR_CODE_MUST_MEET,
  briefOfAChangeToTests,
  briefOfAReview,
  THE_REMOVED_TESTS_TO_READ,
  breaksParagraphOf,
  filesToListItems,
  leftOutParagraphOf,
  removedParagraphOf,
  placeholdersIn,
  placeholdersInTheBriefOfAChangeToTests,
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
 The changed tests and fixtures a review of a change to tests alone reads.

 @returns their paths, from the top of the checkout

 @example
 ```ts
 const files = theChangedTests();
 ```
 */
function theChangedTests(): readonly string[] {
  return [
    `${aSourceFolder()}/gate_tests.rs`,
    `${aSourceFolder()}/stand-in-git.ts`,
  ];
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
  readonly filesLeftOut: readonly LeftOutFile[];
  readonly filesRemoved: readonly string[];
  readonly filesOfTests: readonly string[];
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
    filesLeftOut: [],
    filesRemoved: [],
    filesOfTests: [],
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

/**
 The brief of a change to tests alone this file's expectations are about.

 @returns the brief's text

 @example
 ```ts
 const brief = aBriefOfAChangeToTests();
 ```
 */
function aBriefOfAChangeToTests(): string {
  return briefOfAChangeToTests({
    ...aBriefRequest(),
    files: theChangedTests(),
    filesOfCode: [],
    filesOfTests: theChangedTests(),
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: filesToListItems.name,
      children: [
        it({
          name: 'lists a file a reviewer reads as a path, and says what an escape means only when a name holds one, because Git names are bytes and a brief is text',
          fn: async () => {
            expect(filesToListItems({ files: [aReviewedFile(),], },),).toBe(`- \`${aReviewedFile()}\``,);
            expect(filesToListItems({ files: [aReviewedFile(),], },),).not.toContain('escapes',);
            /**
             A name whose bytes are not UTF-8, as the material shows it.
             */
            const escaped = `${aSourceFolder()}/\\xe9.unit.test.ts`;
            expect(filesToListItems({ files: [escaped,], },),).toContain(`- \`${escaped}\``,);
            expect(filesToListItems({ files: [escaped,], },),).toContain("a shell reads as a `$'...'` word",);
          },
        },),
      ],
    },),
    describe({
      name: breaksParagraphOf.name,
      children: [
        it({
          name: 'asks for a break under each test the reviewed commit holds, and asks instead that a removed test be read in the diff when the change removed them all, because nothing is left to run',
          fn: async () => {
            expect(breaksParagraphOf({ filesOfTests: [`${aSourceFolder()}/gate_tests.rs`,], },),).toContain(
              'For each changed test above, plant a break under it',
            );
            expect(breaksParagraphOf({ filesOfTests: [], },),).toContain(THE_REMOVED_TESTS_TO_READ,);
            expect([breaksParagraphOf({ filesOfTests: [], },).includes('plant a break',),],).toEqual([false,],);
          },
        },),
      ],
    },),
    describe({
      name: removedParagraphOf.name,
      children: [
        it({
          name: 'names every file the change removed and says the change diff is where it is read, and says nothing when the change removed no file the review reads',
          fn: async () => {
            expect(removedParagraphOf({ filesRemoved: [], },),).toBe('',);
            expect(removedParagraphOf({ filesRemoved: [`${aSourceFolder()}/prod.ts`,], },),).toContain(
              `- \`${aSourceFolder()}/prod.ts\``,
            );
            expect(removedParagraphOf({ filesRemoved: [`${aSourceFolder()}/prod.ts`,], },),).toContain(
              'the change diff is where they are read',
            );
          },
        },),
      ],
    },),
    describe({
      name: leftOutParagraphOf.name,
      children: [
        it({
          name: 'says every changed file is read when none is left out, and names each file it leaves out with the reason, so a note beside a test is named rather than taking the review away',
          fn: async () => {
            expect(leftOutParagraphOf({ filesLeftOut: [], },),).toBe('No file the change touched is left out.',);
            /**
             The note beside the changed test.
             */
            const note: LeftOutFile = { path: `${aSourceFolder()}/notes.md`, why: 'a note, which no test reads', };
            expect(leftOutParagraphOf({ filesLeftOut: [note,], },),).toContain(
              `- \`${note.path}\` is left out: a note, which no test reads.`,
            );
            expect(leftOutParagraphOf({ filesLeftOut: [note,], },),).toContain('These files are left out, with the reason:',);
          },
        },),
      ],
    },),
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
              reviewBriefValues({
                ...aBriefRequest(),
                requirements: WHAT_TESTS_AND_THEIR_CODE_MUST_MEET,
                breaks: '',
              },),
            );
            expect(filledNames,).toEqual([...REVIEW_BRIEF_PLACEHOLDERS,],);
          },
        },),
        it({
          name: 'are the same in the brief of a change to tests alone, so the two briefs can never want a value the other fills',
          fn: async () => {
            expect([...placeholdersInTheBriefOfAChangeToTests(),],)
              .toEqual([...REVIEW_BRIEF_PLACEHOLDERS,],);
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
    describe({
      name: briefOfAChangeToTests.name,
      children: [
        it({
          name: 'fills every placeholder, so no reviewer reads one as its task',
          fn: async () => {
            expect(placeholdersIn({ template: aBriefOfAChangeToTests(), },),).toEqual([],);
          },
        },),
        it({
          name: 'names the rules file first, quotes the ban on synthetic host load, and keeps the folders whose answers are another reviewer\'s out',
          fn: async () => {
            expect(aBriefOfAChangeToTests().startsWith(`Read \`${aRulesFile()}\` first and follow it.`,),).toBe(true,);
            expect(aBriefOfAChangeToTests(),).toContain(NO_SYNTHETIC_HOST_LOAD,);
            expect(aBriefOfAChangeToTests(),).toContain('this machine\'s session bus',);
            expect(aBriefOfAChangeToTests(),).toContain('/var/home/user/temp/agent/kimi-reviews',);
            expect(aBriefOfAChangeToTests(),).toContain(`\`${aReviewsFolder()}\` outside your own review folder`,);
          },
        },),
        it({
          name: 'says in plain words that the change is to tests, reads every changed test and fixture in full, and names the commit',
          fn: async () => {
            expect(aBriefOfAChangeToTests(),)
              .toContain(`Review a change to the tests under \`${aSourceFolder()}\`.`,);
            expect(aBriefOfAChangeToTests(),).toContain('the change is to them alone',);
            expect(aBriefOfAChangeToTests(),).toContain(aCommit(),);
            for (const file of theChangedTests()) {
              expect(aBriefOfAChangeToTests(),).toContain(`- \`${file}\``,);
            }
          },
        },),
        it({
          name: 'asks for a break planted under each changed test, that test run by name, and whether it fails, because a test that still passes with the break in is a finding',
          fn: async () => {
            expect(aBriefOfAChangeToTests(),).toContain('For each changed test above, plant a break under it',);
            expect(aBriefOfAChangeToTests(),).toContain('in a copy of the code it tests or in the value it expects',);
            expect(aBriefOfAChangeToTests(),).toContain('run that test by name, and say whether it fails',);
            expect(aBriefOfAChangeToTests(),).toContain('a test that still passes with the break in is a finding',);
            expect(aBriefOfAChangeToTests(),).toContain('Say what you read and what you ran',);
          },
        },),
        it({
          name: 'asks for the review file as soon as the reading is done, marked not final, and lists only concrete defects',
          fn: async () => {
            expect(aBriefOfAChangeToTests(),).toContain('marked at its top as not final',);
            expect(aBriefOfAChangeToTests(),).toContain('write your review file before anything else you do at the end',);
            expect(aBriefOfAChangeToTests(),).toContain('Leave out style, naming, and anything you cannot state as a sequence and a wrong result',);
            expect(aBriefOfAChangeToTests(),).toContain(join(aReviewsFolder(), aReview(), 'review.md',),);
          },
        },),
        it({
          name: 'names the files a change of code leaves out, each with the reason, so data beside code is said to be left out rather than dropped',
          fn: async () => {
            /**
             The data file beside the code.
             */
            const data: LeftOutFile = {
              path: `${aSourceFolder()}/catalog.json`,
              why: 'data, a picture or what a tool wrote, which is no code and which no test in this change reads',
            };
            /**
             The brief, filled with that file left out.
             */
            const brief = briefOfAReview({
              ...aBriefRequest(),
              filesLeftOut: [data,],
            },);
            expect(brief,).toContain(`- \`${data.path}\` is left out: ${data.why}.`,);
            expect(brief,).not.toContain('Every file the change touched is read above.',);
          },
        },),
        it({
          name: 'leaves the planted-break demand out of a change of code, whose reviewer judges the code as the change itself',
          fn: async () => {
            expect(aBrief(),).not.toContain('plant a break',);
            expect(aBrief(),).toContain(`Review a change under \`${aSourceFolder()}\`.`,);
          },
        },),
        it({
          name: 'refuses a request whose values leave a placeholder unfilled',
          fn: async () => {
            expect(() => briefOfAChangeToTests({
              ...aBriefRequest(),
              filesOfCode: [],
              reviewsFolder: `${aReviewsFolder()}/@FOCUS@`,
            },),).toThrow(TemplatePlaceholderError,);
          },
        },),
        it({
          name: 'asks the code requirements as well when the change holds code beside its tests, because its reviewer reads that code too',
          fn: async () => {
            /**
             The brief of a change whose tests come with the code they check.
             */
            const brief = briefOfAChangeToTests({
              ...aBriefRequest(),
              files: [...theChangedTests(), aReviewedFile(),],
              filesOfCode: [aReviewedFile(),],
              filesOfTests: theChangedTests(),
            },);
            expect(brief,).toContain(WHAT_TESTS_AND_THEIR_CODE_MUST_MEET,);
            expect(brief,).toContain('For each changed test above, plant a break under it',);
            expect(aBriefOfAChangeToTests(),).not.toContain('The code must meet these requirements',);
          },
        },),
        it({
          name: 'names a note beside the changed tests as left out, so a line of prose never takes a test review away',
          fn: async () => {
            /**
             The note that changed beside the tests.
             */
            const note: LeftOutFile = { path: `${aSourceFolder()}/notes.md`, why: 'a note, which no test reads', };
            /**
             The brief of that change.
             */
            const brief = briefOfAChangeToTests({
              ...aBriefRequest(),
              files: theChangedTests(),
              filesOfCode: [],
              filesLeftOut: [note,],
              filesOfTests: theChangedTests(),
            },);
            expect(brief,).toContain(`- \`${note.path}\` is left out: a note, which no test reads.`,);
            expect(brief,).toContain('For each changed test above, plant a break under it',);
          },
        },),
      ],
    },),
  ],
},);
