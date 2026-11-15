/**
 What a review of a change reads, and what it leaves out with the reason, by the kinds the range holds.

 The lists of `which-files-a-review-reads.ts` say what each file is, and this table says what a range of those files is: the kinds
 a range can hold, what kind of change it then makes, the kinds a review reads, and the kinds its brief names as left out with the
 reason. One letter stands for each kind: T a test, F a fixture, C code or a build decision, U a file of a kind no extension list
 names, M material a test may read, and P prose.

 A test is read whatever else the range holds, which is the project's decision that a change to tests alone is reviewed; a file a
 review cannot judge never decides for the files beside it, so it is named in the brief as left out and nothing else changes. A
 range that holds only notes, only material, or nothing at all is answered as nothing to review, and the line the writer prints
 says which files it changed and why a review reads none of them.

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
 - TM: tests-and-fixtures; reads TM; leaves out M
 - FM: tests-and-fixtures; reads FM; leaves out M
 - TFM: tests-and-fixtures; reads TFM; leaves out M
 - CM: code-and-build-decisions; reads C; leaves out M
 - TCM: tests-and-fixtures; reads TCM; leaves out M
 - FCM: tests-and-fixtures; reads FCM; leaves out M
 - TFCM: tests-and-fixtures; reads TFCM; leaves out M
 - UM: code-and-build-decisions; reads U; leaves out M
 - TUM: tests-and-fixtures; reads TUM; leaves out M
 - FUM: tests-and-fixtures; reads FUM; leaves out M
 - TFUM: tests-and-fixtures; reads TFUM; leaves out M
 - CUM: code-and-build-decisions; reads CU; leaves out M
 - TCUM: tests-and-fixtures; reads TCUM; leaves out M
 - FCUM: tests-and-fixtures; reads FCUM; leaves out M
 - TFCUM: tests-and-fixtures; reads TFCUM; leaves out M
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
 - TMP: tests-and-fixtures; reads TM; leaves out MP
 - FMP: tests-and-fixtures; reads FM; leaves out MP
 - TFMP: tests-and-fixtures; reads TFM; leaves out MP
 - CMP: code-and-build-decisions; reads C; leaves out MP
 - TCMP: tests-and-fixtures; reads TCM; leaves out MP
 - FCMP: tests-and-fixtures; reads FCM; leaves out MP
 - TFCMP: tests-and-fixtures; reads TFCM; leaves out MP
 - UMP: code-and-build-decisions; reads U; leaves out MP
 - TUMP: tests-and-fixtures; reads TUM; leaves out MP
 - FUMP: tests-and-fixtures; reads FUM; leaves out MP
 - TFUMP: tests-and-fixtures; reads TFUM; leaves out MP
 - CUMP: code-and-build-decisions; reads CU; leaves out MP
 - TCUMP: tests-and-fixtures; reads TCUM; leaves out MP
 - FCUMP: tests-and-fixtures; reads FCUM; leaves out MP
 - TFCUMP: tests-and-fixtures; reads TFCUM; leaves out MP

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
 What a review of a change does: what kind of change it is, the files it reads, the code and the files of a kind no list names
 among them, and the files it leaves out with the reason.
 */
export type ReviewPlan = {
  /**
   The kind of change the kinds of its files make it.
   */
  readonly kind: 'code-and-build-decisions' | 'tests-and-fixtures' | 'nothing-a-review-reads';

  /**
   The files the reviewer is given, in the order git named them, without the files it leaves out.
   */
  readonly files: readonly string[];

  /**
   The code among those files, in the order given: what makes a change to tests also ask the project's requirements.
   */
  readonly filesOfCode: readonly string[];

  /**
   The files among those whose extension is in no list of `which-files-a-review-reads.ts`, in the order given.
   */
  readonly filesOfAnUnlistedKind: readonly string[];

  /**
   The files the change touched that this review does not read, each with the reason its brief gives, in the order git named
   them.
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
  kinds
}: {
  readonly paths: readonly string[];
  readonly kinds: readonly KindOfAFileAReviewReads[];
},): ReviewPlan {
  /**
   The paths of the kinds given, in the order git named them.

   @param wanted - the kinds to keep

   @returns the paths of those kinds
   */
  function pathsOf(wanted: readonly KindOfAFileAReviewReads[],): readonly string[] {
    return paths.filter(function isWanted(
      _path,
      index,
    ) {
      return wanted.includes(kinds[index] ?? 'material-a-test-may-read',);
    },);
  }
  /**
   The files this review leaves out, each with its reason, which a kind a review reads has none of.
   */
  const filesLeftOut = paths.flatMap(function leftOut(
    path,
    index,
  ): readonly LeftOutFile[] {
    /**
     Why this kind of file is left out, which no kind a review reads has.
     */
    const why = WHY_A_FILE_IS_LEFT_OUT.get(kinds[index] ?? 'material-a-test-may-read',);
    return ((why === undefined) ? [] : [
      {
        path,
        why,
      },
    ]);
  },);
  if ((pathsOf([
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
      filesOfCode: pathsOf(['code',],),
      filesOfAnUnlistedKind: [],
      filesLeftOut,
    };
  }
  if ((pathsOf([
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
      filesOfCode: [],
      filesOfAnUnlistedKind: pathsOf(['a-unlisted-kind',],),
      filesLeftOut,
    };
  }
  return {
    kind: 'nothing-a-review-reads',
    files: [],
    filesOfCode: [],
    filesOfAnUnlistedKind: [],
    filesLeftOut: [],
  };
}
