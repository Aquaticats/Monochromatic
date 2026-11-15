/**
 What a review of a change reads, and what it leaves out with the reason, by the kinds the range holds.

 The lists of `which-files-a-review-reads.ts` say what each file is, and this table says what a range of those files is: the kinds
 a range can hold, what kind of change it then makes, the kinds a review reads, and the kinds its brief names as left out with the
 reason. One letter stands for each kind: T a test, F a fixture, C code or a build decision, U a file of a kind no extension list
 names, M material a test may read, and P prose.

 A test is read whatever else the range holds, which is the project's decision that a change to tests alone is reviewed; a file a
 review cannot judge never decides for the files beside it, so the row leaves it out by name and with the reason, and nothing else
 changes. A range that holds only notes, only material, or nothing at all is answered as nothing to review, and the line the
 writer prints says which files it changed and why a review reads none of them.

 Every path of the range is in exactly one of `files`, `filesRemoved` and `filesLeftOut`, which is what makes a brief list a file
 and never also say it is left out. A file of a kind no extension list names is code wherever it is read, beside a test or not: it
 is in `filesOfCode`, so the brief asks the project's requirements for it, and in `filesOfAnUnlistedKind`, so the line prints the
 notice. The brief and the line take those answers and add nothing of their own.

 A path the reviewed commit no longer holds is still a path the range touched, and its kind is the kind its name says: a removed
 code file makes the change one to code, and the brief names it as removed and says the change's diff is where it is read, because
 a reviewer cannot open it. So a row's read kinds are in `files` where a reviewer can open them and in `filesRemoved` where the
 change removed them, while its left-out kinds are left out either way. A rename is two paths, the old name removed and the new one
 read, because the listing asks git not to fold a rename. A path whose type changed, a file become a symlink for instance, is held
 by the reviewed commit, so it is read like any other file of its kind.

 - no file: nothing-a-review-reads; reads none; leaves out none
 - T: tests-and-fixtures; reads T; leaves out none
 - F: tests-and-fixtures; reads F; leaves out none
 - TF: tests-and-fixtures; reads TF; leaves out none
 - C: code-and-build-decisions; reads C; leaves out none
 - TC: tests-and-fixtures; reads TC; leaves out none
 - FC: tests-and-fixtures; reads FC; leaves out none
 - TFC: tests-and-fixtures; reads TFC; leaves out none
 - U: code-and-build-decisions; reads U; leaves out none
 - TU: tests-and-fixtures; reads TU; leaves out none
 - FU: tests-and-fixtures; reads FU; leaves out none
 - TFU: tests-and-fixtures; reads TFU; leaves out none
 - CU: code-and-build-decisions; reads CU; leaves out none
 - TCU: tests-and-fixtures; reads TCU; leaves out none
 - FCU: tests-and-fixtures; reads FCU; leaves out none
 - TFCU: tests-and-fixtures; reads TFCU; leaves out none
 - M: nothing-a-review-reads; reads none; leaves out none
 - TM: tests-and-fixtures; reads TM; leaves out none
 - FM: tests-and-fixtures; reads FM; leaves out none
 - TFM: tests-and-fixtures; reads TFM; leaves out none
 - CM: code-and-build-decisions; reads C; leaves out M
 - TCM: tests-and-fixtures; reads TCM; leaves out none
 - FCM: tests-and-fixtures; reads FCM; leaves out none
 - TFCM: tests-and-fixtures; reads TFCM; leaves out none
 - UM: code-and-build-decisions; reads U; leaves out M
 - TUM: tests-and-fixtures; reads TUM; leaves out none
 - FUM: tests-and-fixtures; reads FUM; leaves out none
 - TFUM: tests-and-fixtures; reads TFUM; leaves out none
 - CUM: code-and-build-decisions; reads CU; leaves out M
 - TCUM: tests-and-fixtures; reads TCUM; leaves out none
 - FCUM: tests-and-fixtures; reads FCUM; leaves out none
 - TFCUM: tests-and-fixtures; reads TFCUM; leaves out none
 - P: nothing-a-review-reads; reads none; leaves out none
 - TP: tests-and-fixtures; reads T; leaves out P
 - FP: tests-and-fixtures; reads F; leaves out P
 - TFP: tests-and-fixtures; reads TF; leaves out P
 - CP: code-and-build-decisions; reads C; leaves out P
 - TCP: tests-and-fixtures; reads TC; leaves out P
 - FCP: tests-and-fixtures; reads FC; leaves out P
 - TFCP: tests-and-fixtures; reads TFC; leaves out P
 - UP: code-and-build-decisions; reads U; leaves out P
 - TUP: tests-and-fixtures; reads TU; leaves out P
 - FUP: tests-and-fixtures; reads FU; leaves out P
 - TFUP: tests-and-fixtures; reads TFU; leaves out P
 - CUP: code-and-build-decisions; reads CU; leaves out P
 - TCUP: tests-and-fixtures; reads TCU; leaves out P
 - FCUP: tests-and-fixtures; reads FCU; leaves out P
 - TFCUP: tests-and-fixtures; reads TFCU; leaves out P
 - MP: nothing-a-review-reads; reads none; leaves out none
 - TMP: tests-and-fixtures; reads TM; leaves out P
 - FMP: tests-and-fixtures; reads FM; leaves out P
 - TFMP: tests-and-fixtures; reads TFM; leaves out P
 - CMP: code-and-build-decisions; reads C; leaves out MP
 - TCMP: tests-and-fixtures; reads TCM; leaves out P
 - FCMP: tests-and-fixtures; reads FCM; leaves out P
 - TFCMP: tests-and-fixtures; reads TFCM; leaves out P
 - UMP: code-and-build-decisions; reads U; leaves out MP
 - TUMP: tests-and-fixtures; reads TUM; leaves out P
 - FUMP: tests-and-fixtures; reads FUM; leaves out P
 - TFUMP: tests-and-fixtures; reads TFUM; leaves out P
 - CUMP: code-and-build-decisions; reads CU; leaves out MP
 - TCUMP: tests-and-fixtures; reads TCUM; leaves out P
 - FCUMP: tests-and-fixtures; reads FCUM; leaves out P
 - TFCUMP: tests-and-fixtures; reads TFCUM; leaves out P

 Without a table a writer asks the kinds in the order it thinks of them, and then the first kind present says what the whole change
 is: one file of a kind no extension list names took the code path and dropped the changed test beside it on 2026-11-15. The plan
 asks the kinds first and decides from all of them, so no file of a range decides for the files beside it.

 @module
 */

import type { KindOfAFileAReviewReads, } from './which-files-a-review-reads.ts';

/**
 A file the change touched that a review does not read, with the reason its brief gives for leaving it out.
 */
export type LeftOutFile = {
  /**
   The file's path from the top of the checkout, as the brief shows it.
   */
  readonly path: string;

  /**
   Why a review of this change does not read it, as a clause the brief reads after the path.
   */
  readonly why: string;
};

/**
 Why a review of a change leaves a file out, by the kind of the file: the clause its brief reads after the path.

 A kind a review reads has no clause here, and the absence is the rule: a file of a kind this map names is not read, and a file of
 any other kind is.
 */
const WHY_A_FILE_IS_LEFT_OUT: ReadonlyMap<KindOfAFileAReviewReads, string> = new Map([
  [
    'a-note',
    'a note, which no test reads',
  ],
  [
    'material-a-test-may-read',
    'data, a picture or what a tool wrote, which is no code and which no test in this change reads',
  ],
],);

/**
 One changed path with the kind it was handed in with.
 */
type APathWithItsKind = {
  /**
   The path from the top of the checkout.
   */
  readonly path: string;

  /**
   What kind of file the path is.
   */
  readonly kind: KindOfAFileAReviewReads;
};

/**
 What a review of a change does: what kind of change it is, the files it reads, the code and the files of a kind no list names
 among them, and the files it leaves out with the reason.
 */
export type ReviewPlan = {
  /**
   The kind of change the kinds of its files make it.
   */
  readonly kind: 'code-and-build-decisions' | 'tests-and-fixtures' | 'nothing-a-review-reads';

  /**
   The files the reviewer is given, in the order git named them, without the files it leaves out and without the files the change
   removed, which the reviewed commit no longer holds.
   */
  readonly files: readonly string[];

  /**
   The files the change removed, in the order git named them: a review reads them in the change's diff. They are the kind their
   path says, so a removed code file still makes the change one to code.
   */
  readonly filesRemoved: readonly string[];

  /**
   The code among all the files this review reads, the removed ones included, in the order given, a file of a kind no list names
   being code wherever it is read. This is what decides the requirements the brief asks: the project's own for the code whenever it
   is not empty, and for a change to tests the tests-and-code sentence rather than the tests-alone one.
   */
  readonly filesOfCode: readonly string[];

  /**
   The files among those whose extension is in no list of `which-files-a-review-reads.ts`, in the order given. This is what the
   brief says is read as code of a kind the writer does not know, and the line prints the notice from it.
   */
  readonly filesOfAnUnlistedKind: readonly string[];

  /**
   The files the change touched that this review does not read, each with the reason its brief gives, in the order git named
   them. No path is in both this and `files`.
   */
  readonly filesLeftOut: readonly LeftOutFile[];
};

/**
 Reads the plan a review of a change follows: the module's table, in code, from the kind of every changed path.

 A file a review cannot judge never decides for the files beside it, and a test is read whatever else the change holds, so a range
 that holds a test or a fixture is a change to tests, and a range that holds code or a file of a kind no list names, and no test,
 is a change of code.

 @param paths - the changed paths the reviewed commit holds, in the order git named them

 @param kinds - the kind of each of those paths, in the same order

 @returns what the change is, the files a review reads, and the files it leaves out

 @example
 ```ts
 const plan = planOfAChange({ paths: ['package/a/src/gate.unit.test.ts'], kinds: ['a-test'], },);
 ```
 */
export function planOfAChange({
  paths,
  kinds,
  removed
}: {
  readonly paths: readonly string[];
  readonly kinds: readonly KindOfAFileAReviewReads[];

  /**
   The paths among those the reviewed commit no longer holds, which a review reads only in the change's diff.
   */
  readonly removed: readonly string[];
},): ReviewPlan {
  /**
   Every changed path with the kind it was handed in with, in the order git named them.
   */
  const changed: readonly APathWithItsKind[] = paths.map(function withItsKind(
    path,
    index,
  ): APathWithItsKind {
    return {
      path,
      kind: kinds[index] ?? 'material-a-test-may-read',
    };
  },);
  /**
   The paths of the kinds given that the reviewed commit holds, in the order git named them.

   @param wanted - the kinds to keep

   @returns the paths of those kinds a reviewer can open
   */
  function pathsOf(wanted: readonly KindOfAFileAReviewReads[],): readonly string[] {
    return changed
      .filter(function isWanted(entry,) {
        return wanted.includes(entry.kind,) && (!removed.includes(entry.path,));
      },)
      .map(function pathOf(entry,) {
        return entry.path;
      },);
  }
  /**
   The paths of the kinds given, whether a reviewer can open them or the change removed them, in the order git named them.

   @param wanted - the kinds to keep

   @returns the paths of those kinds
   */
  function readPathsOf(wanted: readonly KindOfAFileAReviewReads[],): readonly string[] {
    return changed
      .filter(function isWanted(entry,) {
        return wanted.includes(entry.kind,);
      },)
      .map(function pathOf(entry,) {
        return entry.path;
      },);
  }
  /**
   The paths of the kinds given that the change removed, in the order git named them.

   @param wanted - the kinds to keep

   @returns the paths of those kinds the reviewed commit no longer holds
   */
  function removedPathsOf(wanted: readonly KindOfAFileAReviewReads[],): readonly string[] {
    return changed
      .filter(function isWanted(entry,) {
        return wanted.includes(entry.kind,) && removed.includes(entry.path,);
      },)
      .map(function pathOf(entry,) {
        return entry.path;
      },);
  }
  /**
   The files a row leaves out, each with its reason: the row's own left-out kinds, which are the kinds it does not read and no
   other, so a file a row reads is never also said to be left out.

   @param leftOutKinds - the kinds the row leaves out

   @returns those files, each with the reason its brief gives

   @throws Error when a kind is left out and no reason is written for it
   */
  function leftOutOf(leftOutKinds: readonly KindOfAFileAReviewReads[],): readonly LeftOutFile[] {
    return changed
      .filter(function isLeftOut(entry,) {
        return leftOutKinds.includes(entry.kind,);
      },)
      .map(function leftOut(entry,) {
        /**
         Why this kind of file is left out, which every kind a row leaves out has.
         */
        const why = WHY_A_FILE_IS_LEFT_OUT.get(entry.kind,);
        if (why === undefined) {
          throw new Error(`no reason is written for leaving out a file of the kind ${entry.kind}`,);
        }
        return {
          path: entry.path,
          why,
        };
      },);
  }
  if ((readPathsOf([
    'a-test',
    'a-fixture',
  ],)
    .length
    > 0)) {
    return {
      kind: 'tests-and-fixtures',
      files: pathsOf([
        'a-test',
        'a-fixture',
        'code',
        'a-unlisted-kind',
        'material-a-test-may-read',
      ],),
      filesRemoved: removedPathsOf([
        'a-test',
        'a-fixture',
        'code',
        'a-unlisted-kind',
        'material-a-test-may-read',
      ],),
      filesOfCode: readPathsOf([
        'code',
        'a-unlisted-kind',
      ],),
      filesOfAnUnlistedKind: readPathsOf(['a-unlisted-kind',],),
      filesLeftOut: leftOutOf(['a-note',],),
    };
  }
  if ((readPathsOf([
    'code',
    'a-unlisted-kind',
  ],)
    .length
    > 0)) {
    return {
      kind: 'code-and-build-decisions',
      files: pathsOf([
        'code',
        'a-unlisted-kind',
      ],),
      filesRemoved: removedPathsOf([
        'code',
        'a-unlisted-kind',
      ],),
      filesOfCode: readPathsOf([
        'code',
        'a-unlisted-kind',
      ],),
      filesOfAnUnlistedKind: readPathsOf(['a-unlisted-kind',],),
      filesLeftOut: leftOutOf([
        'material-a-test-may-read',
        'a-note',
      ],),
    };
  }
  return {
    kind: 'nothing-a-review-reads',
    files: [],
    filesRemoved: [],
    filesOfCode: [],
    filesOfAnUnlistedKind: [],
    filesLeftOut: [],
  };
}
