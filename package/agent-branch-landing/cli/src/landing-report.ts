/**
 What a landing reports about the branch it landed: the commit it put on `main`, what is still not on it, and the paths a release
 gate depends on that the branch changed.

 @module
 */

import {
  MERGE_SUBJECT_CHARACTERS,
  A_RELEASE_GATE_IS_INVALIDATED,
} from './constants.ts';
import {
  changedPathsSinceMain,
  commitsOfBranchNotOnMain,
  lastCommitOnMain,
  type Repository,
} from './git-commands.ts';
import { releasePathsAmong, } from './paths-of-a-branch.ts';
import type { Report, } from './report.ts';

/**
 What a landing reported about the branch it landed.
 */
export type LandingReport = {
  /**
   The last commit of `main`, short name and subject.
   */
  readonly lastCommit: string;

  /**
   How many of the branch's commits are still not on `main`, which is zero for a landing that took all of them.
   */
  readonly commitsNotOnMain: number;

  /**
   The paths the branch changed that a release gate depends on.
   */
  readonly releasePaths: readonly string[];
};

/**
 Reads what the landing left: the last commit of `main`, what is still not on it, and the release paths the branch changed.

 @param repository - the checkout to ask

 @param branch - the branch

 @param report - where to report

 @returns the landing's own proof

 @example
 ```ts
 const report = await landingReportOf({ repository, branch: 'fix/a', report, },);
 ```
 */
export async function landingReportOf({
  repository,
  branch,
  report,
}: {
  readonly repository: Repository;
  readonly branch: string;
  readonly report: Report;
},): Promise<LandingReport> {
  /**
   The commit that landed.
   */
  const lastCommit = await lastCommitOnMain(repository,);
  /**
   How many of the branch's commits are still not on `main`.
   */
  const commitsNotOnMain = await commitsOfBranchNotOnMain({
    repository,
    branch,
  },);
  /**
   The paths the branch changed that a release gate depends on.
   */
  const releasePaths = releasePathsAmong(await changedPathsSinceMain({
    repository,
    branch,
  },),);
  report.line(lastCommit.slice(
    0,
    MERGE_SUBJECT_CHARACTERS,
  ),);
  report.line(`${String(commitsNotOnMain,)} commits of the branch not on main`,);
  if (releasePaths.length > 0) {
    report.line(`release paths changed by this branch: ${releasePaths.join(' ',)}`,);
    report.line(A_RELEASE_GATE_IS_INVALIDATED,);
  }
  return {
    lastCommit,
    commitsNotOnMain,
    releasePaths,
  };
}
