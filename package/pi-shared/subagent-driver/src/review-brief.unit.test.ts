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
  type RemovedOrRenamedPath,
  FOCUS_OF_A_CHANGE_REVIEW,
  NO_SYNTHETIC_HOST_LOAD,
  REVIEW_BRIEF_PLACEHOLDERS,
  TemplatePlaceholderError,
  aPathAsABriefShowsIt,
  briefOfAChangeToTests,
  briefOfAReview,
  placeholdersIn,
  placeholdersInTheBriefOfAChangeToTests,
  placeholdersInTheReviewBrief,
  reviewBriefValues,
  whatTheReviewReads,
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
  readonly removedOrRenamed: readonly RemovedOrRenamedPath[];
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
    removedOrRenamed: [],
    focus: FOCUS_OF_A_CHANGE_REVIEW,
    reviewsFolder: aReviewsFolder(),
    runsFolder: aRunsFolder(),
    rulesFile: aRulesFile(),
  };
}

/**
 A path the change deleted.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const deleted = aDeletedPath();
 ```
 */
function aDeletedPath(): string {
  return `${aSourceFolder()}/old_gate.rs`;
}

/**
 The name a renamed path had before the change.

 @returns the path, from the top of the checkout

 @example
 ```ts
 const renamed = aRenamedPath();
 ```
 */
function aRenamedPath(): string {
  return `${aSourceFolder()}/old_name.rs`;
}

/**
 The paths this file's briefs say the change removed or renamed.

 @returns the removals and renames

 @example
 ```ts
 const removed = theRemovedPaths();
 ```
 */
function theRemovedPaths(): readonly RemovedOrRenamedPath[] {
  return [
    {
      status: 'deleted',
      path: aDeletedPath(),
    },
    {
      status: 'renamed',
      was: aRenamedPath(),
      now: `${aSourceFolder()}/new_name.rs`,
    },
  ];
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
  },);
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
        it({
          name: 'names what a change deleted or renamed when the commit holds no code file there, so a range that only removes code is still reviewed',
          fn: async () => {
            /**
             The brief of a change that deleted its code and edited its test.
             */
            const brief = briefOfAReview({
              ...aBriefRequest(),
              files: [],
              removedOrRenamed: [{ status: 'deleted', path: aDeletedPath() }],
            },);
            expect(brief,).toContain('The reviewed commit holds no file at a path this change touches',);
            expect(brief,).toContain(`- \`${aDeletedPath()}\` (deleted)`,);
            expect(brief,).toContain('Say what depended on each of them, and what does its work now or why nothing does.',);
            expect(brief,).not.toContain('the change is to tests',);
            expect(brief,).not.toContain('plant a break',);
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
            expect(aBriefOfAChangeToTests(),).toContain('The change is to tests and the fixtures tests use, and to nothing else.',);
            expect(aBriefOfAChangeToTests(),).toContain(aCommit(),);
            for (const file of theChangedTests()) {
              expect(aBriefOfAChangeToTests(),).toContain(`- \`${file}\``,);
            }
          },
        },),
        it({
          name: 'asks for a break planted under each changed test, that test run by name, and whether it fails, because a test that still passes with the break in is a finding',
          fn: async () => {
            expect(aBriefOfAChangeToTests(),).toContain('For each changed test the commit still holds, plant a break under it',);
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
              reviewsFolder: `${aReviewsFolder()}/@FOCUS@`,
            },),).toThrow(TemplatePlaceholderError,);
          },
        },),
        it({
          name: 'names a deleted or renamed test and asks what depended on it, while the breaks to plant stay with the tests the commit still holds',
          fn: async () => {
            /**
             The brief of a change to tests that also deleted one.
             */
            const brief = briefOfAChangeToTests({
              ...aBriefRequest(),
              files: theChangedTests(),
              removedOrRenamed: [{ status: 'deleted', path: `${aSourceFolder()}/gone_tests.rs` }],
            },);
            expect(brief,).toContain(`- \`${aSourceFolder()}/gone_tests.rs\` (deleted)`,);
            expect(brief,).toContain('Say what depended on each of them',);
            expect(brief,).toContain('For each changed test the commit still holds, plant a break under it',);
          },
        },),
      ],
    },),
    describe({
      name: whatTheReviewReads.name,
      children: [
        it({
          name: 'names every file the reviewer reads in full, which is the part a brief of a change with nothing removed has always had',
          fn: async () => {
            expect(whatTheReviewReads({
              files: [aReviewedFile(),],
              removedOrRenamed: [],
            },),).toBe(`Read each of these files in full before you answer:
- \`${aReviewedFile()}\``,);
          },
        },),
        it({
          name: 'names each removed or renamed path with what happened to it, and asks what depended on it, beside the files the commit still holds',
          fn: async () => {
            /**
             The part of a brief for a change that removed and renamed paths.
             */
            const reads = whatTheReviewReads({
              files: [aReviewedFile(),],
              removedOrRenamed: theRemovedPaths(),
            },);
            expect(reads,).toContain(`- \`${aReviewedFile()}\``,);
            expect(reads,).toContain('The change removes or renames these paths, and the diff holds each one\'s change:',);
            expect(reads,).toContain(`- \`${aDeletedPath()}\` (deleted)`,);
            expect(reads,).toContain(`- \`${aRenamedPath()}\` (renamed to \`${aSourceFolder()}/new_name.rs\`)`,);
            expect(reads,).toContain('Say what depended on each of them, and what does its work now or why nothing does.',);
          },
        },),
        it({
          name: 'says the diff is what there is to read when the commit holds no file at a path the change touched, which a range that only deletes code does',
          fn: async () => {
            /**
             The part of a brief for a change that deleted a code file and nothing else.
             */
            const reads = whatTheReviewReads({
              files: [],
              removedOrRenamed: theRemovedPaths(),
            },);
            expect(reads,).toContain('The reviewed commit holds no file at a path this change touches, so the diff holds everything there is to read.',);
            expect(reads,).not.toContain('Read each of these files in full',);
            expect(reads,).toContain(`- \`${aDeletedPath()}\` (deleted)`,);
          },
        },),
      ],
    },),
    describe({
      name: aPathAsABriefShowsIt.name,
      children: [
        it({
          name: 'shows an ordinary path as itself in a code span, so every brief written before a name needed another form reads the same',
          fn: async () => {
            expect(aPathAsABriefShowsIt({ path: 'package/a/src/gate.rs' },),).toBe('`package/a/src/gate.rs`',);
          },
        },),
        it({
          name: 'shows a path with a space, a double quote, a backslash, a byte outside ASCII or a leading dash as itself, because a code span carries each of them',
          fn: async () => {
            /**
             The names a code span carries as they are.
             */
            const plain = [
              'package/a/src/a b.ts',
              'package/a/src/a"b.ts',
              String.raw`package/a/src/a\b.ts`,
              'package/a/src/naïve.ts',
              '-dash.ts',
            ];
            for (const path of plain) {
              expect(aPathAsABriefShowsIt({ path, },),).toBe(`\`${path}\``,);
            }
          },
        },),
        it({
          name: 'shows a path with a line break or a tab as a JSON string, because a line of the brief cannot carry the character',
          fn: async () => {
            expect(aPathAsABriefShowsIt({ path: 'package/a/src/a\nb.ts' },),).toBe('`"package/a/src/a\\nb.ts"`',);
            expect(aPathAsABriefShowsIt({ path: 'package/a/src/a\tb.ts' },),).toBe('`"package/a/src/a\\tb.ts"`',);
          },
        },),
        it({
          name: 'shows a path with a backtick as a JSON string whose backtick is escaped, because the backtick would end the code span',
          fn: async () => {
            expect(aPathAsABriefShowsIt({ path: 'package/a/src/a`b.ts' },),).toBe('`"package/a/src/a\\u0060b.ts"`',);
          },
        },),
        it({
          name: 'shows a path that begins and ends with a space as a JSON string, because a code span trims one space from each end',
          fn: async () => {
            expect(aPathAsABriefShowsIt({ path: ' a ' },),).toBe('`" a "`',);
          },
        },),
      ],
    },),
    describe({
      name: whatTheReviewReads.name,
      children: [
        it({
          name: 'names a file whose name holds a line break as a JSON string and says what that form is, so a reviewer can open the file it names',
          fn: async () => {
            /**
             The part of a brief for a change whose only file holds a line break in its name.
             */
            const reads = whatTheReviewReads({
              files: ['package/a/src/a\nb.ts',],
              removedOrRenamed: [],
            },);
            expect(reads,).toContain('- `"package/a/src/a\\nb.ts"`',);
            expect(reads,).toContain('A path written in double quotes is shown as a JSON string',);
            expect(reads,).toContain('open the file it names',);
          },
        },),
        it({
          name: 'says nothing about the JSON form when every path can stand in a line as itself, so an ordinary brief keeps its words',
          fn: async () => {
            /**
             The part of a brief for a change whose files are ordinary names.
             */
            const reads = whatTheReviewReads({
              files: ['package/a/src/gate.rs',],
              removedOrRenamed: [{ status: 'deleted', path: 'package/a/src/old_gate.rs' },],
            },);
            expect(reads,).not.toContain('JSON',);
          },
        },),
      ],
    },),
  ],
},);
