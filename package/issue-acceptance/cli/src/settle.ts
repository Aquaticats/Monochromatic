/**
 Settles a plan: decides each issue on the branch, then records the verdict.

 A run without `apply` only decides and prints,
 so a session can read every verdict before anything is written on GitHub.

 @module
 */

/* oxlint-disable eslint/no-await-in-loop -- one plan entry at a time: a failure cannot interleave two verdicts, and GitHub receives one request in flight; see doc/lint/sequential-awaits.md */

import {
  ChildProcessError,
  IssueAcceptanceError,
} from './errors.ts';
import type { GitReader } from './git-reader.ts';
import type { IssueTracker, } from './issue-tracker.ts';
import type {
  IssueEntry,
  Plan,
} from './plan-model.ts';
import {
  evaluateChecks,
  type Verdict,
} from './verdict.ts';

/**
 State GitHub reports for an issue that can still receive a closing comment.
 */
const OPEN_STATE = 'OPEN';

/**
 What became of one issue.
 */
export type SettlementKind =
  | 'does-not-hold'
    | 'would-settle'
    | 'settled'
    | 'left-open'
    | 'already-closed'
    | 'failed';

/**
 One issue's fate in this run.
 */
export type IssueSettlement = {
  readonly issue: number;
  readonly kind: SettlementKind;
  readonly detail: string;
  readonly verdict: Verdict;
};

/**
 What became of one commit correction.
 */
export type CorrectionSettlement = {
  readonly commit: string;
  readonly kind: 'would-comment' | 'commented' | 'failed';
  readonly detail: string;
};

/**
 Everything one run decided and did.
 */
export type SettleReport = {
  readonly issues: readonly IssueSettlement[];
  readonly corrections: readonly CorrectionSettlement[];
  readonly failed: boolean;
};

/**
 One settlement run.
 */
export type SettleRequest = {
  readonly plan: Plan;
  readonly reader: GitReader;
  readonly tracker: IssueTracker;
  readonly apply: boolean;
  readonly write: (line: string,) => void;
};

/**
 One issue inside a run.
 */
export type SettleIssueRequest = {
  readonly request: SettleRequest;
  readonly entry: IssueEntry;
};

/**
 One commit correction inside a run.
 */
export type SettleCorrectionRequest = {
  readonly request: SettleRequest;
  readonly commit: string;
  readonly body: string;
};

/**
 Joins the details of the checks that failed.
 
 @param verdict - One issue's verdict.
 
 @returns Semicolon-separated failure details, or an empty string when every check held.
 
 @example
 ```ts
 const failures = failedDetails({ holds: true, outcomes: [] });
 ```
 */
function failedDetails(verdict: Verdict,): string {
  return verdict.outcomes
    .filter(function failed(outcome,): boolean {
      return !outcome.holds;
    },)
    .map(function toDetail(outcome,): string {
      return outcome.detail;
    },)
    .join('; ',);
}

/**
 Settles one issue and reports what happened.

 The checks run before the state is read,
 so an issue whose acceptance no longer holds is never touched.
 
 @param request - Run settings, shared by every issue.
 
 @param entry - One issue of the plan.
 
 @returns One issue's settlement.
 
 @example
 ```ts
 const settlement = await settleIssue({ request, entry: plan.issues[0] });
 ```
 */
async function settleIssue({
  request,
  entry,
}: SettleIssueRequest,): Promise<IssueSettlement> {
  /**
   Decision of every check of this issue.
   */
  const verdict = await evaluateChecks({
    checks: entry.checks,
    reader: request.reader,
  },);
  if (!verdict.holds) {
    /**
     Text naming every check that failed.
     */
    const detail = failedDetails(verdict,);
    request.write(`#${String(entry.issue,)} DOES NOT HOLD: ${detail}`,);
    return {
      issue: entry.issue,
      kind: 'does-not-hold',
      detail,
      verdict,
    };
  }
  if (!request.apply) {
    request.write(`#${String(entry.issue,)} holds on ${request.plan
      .branch}: would ${entry.action}`,);
    return {
      issue: entry.issue,
      kind: 'would-settle',
      detail: `holds on ${request.plan
        .branch}`,
      verdict,
    };
  }
  try {
    /**
     State GitHub reports for this issue now.
     */
    const state = await request.tracker
      .state(entry.issue,);
    if (state !== OPEN_STATE) {
      request.write(`#${String(entry.issue,)} is already ${state}, nothing done`,);
      return {
        issue: entry.issue,
        kind: 'already-closed',
        detail: state,
        verdict,
      };
    }
    if (entry.action === 'close') {
      await request.tracker
        .close({
          number: entry.issue,
          comment: entry.comment,
        },);
      request.write(`#${String(entry.issue,)} closed`,);
      return {
        issue: entry.issue,
        kind: 'settled',
        detail: 'closed',
        verdict,
      };
    }
    await request.tracker
      .comment({
        number: entry.issue,
        body: entry.comment,
      },);
    request.write(`#${String(entry.issue,)} commented, left open`,);
    return {
      issue: entry.issue,
      kind: 'left-open',
      detail: 'commented',
      verdict,
    };
  }
  catch (error) {
    if (!(error instanceof ChildProcessError)) {
      throw error;
    }
    request.write(`#${String(entry.issue,)} FAILED: ${error.message}`,);
    return {
      issue: entry.issue,
      kind: 'failed',
      detail: error.message,
      verdict,
    };
  }
}

/**
 Records one commit correction, or says it would have.
 
 @param request - Run settings.
 
 @param commit - Commit whose message is corrected.
 
 @param body - Correction text.
 
 @returns One correction's settlement.
 
 @example
 ```ts
 const correction = await settleCorrection({ request, commit: '2b309f8f25', body: 'Correction.' });
 ```
 */
async function settleCorrection({
  request,
  commit,
  body,
}: SettleCorrectionRequest,): Promise<CorrectionSettlement> {
  if (!request.apply) {
    request.write(`commit ${commit}: would record a corrective comment`,);
    return {
      commit,
      kind: 'would-comment',
      detail: 'not applied',
    };
  }
  try {
    /**
     Id of the comment GitHub recorded on the commit.
     */
    const id = await request.tracker
      .commentOnCommit({
        commit,
        body,
      },);
    request.write(`commit ${commit}: corrective comment ${id}`,);
    return {
      commit,
      kind: 'commented',
      detail: id,
    };
  }
  catch (error) {
    if (!(error instanceof ChildProcessError)) {
      throw error;
    }
    request.write(`commit ${commit}: FAILED: ${error.message}`,);
    return {
      commit,
      kind: 'failed',
      detail: error.message,
    };
  }
}

/**
 Answers whether one settlement means the round is not finished.
 
 @param settlement - One issue's settlement.
 
 @returns True when a check failed or a call did.
 
 @example
 ```ts
 const bad = isUnfinished({ issue: 1, kind: 'failed', detail: 'x', verdict });
 ```
 */
function isUnfinished(settlement: IssueSettlement,): boolean {
  return (settlement.kind === 'does-not-hold') || (settlement.kind === 'failed');
}

/**
 Settles every issue of one plan.

 Issues are settled one after another,
 so the operator reads one line per issue as the round runs
 and GitHub receives one request at a time.
 
 @param request - Plan, branch reader, tracker, and whether to write.
 
 @returns Every verdict and what each mutation did.
 
 @throws {@link IssueAcceptanceError} when the branch tracks no path, which means the checkout or branch is wrong.
 
 @example
 ```ts
 const report = await settlePlan({ plan, reader, tracker, apply: false, write });
 ```
 */
export async function settlePlan(request: SettleRequest,): Promise<SettleReport> {
  /**
   Every path the plan's branch tracks, which proves the checkout and branch resolve.
   */
  const tracked = await request.reader
    .trackedPaths();
  if (tracked.length === 0) {
    throw new IssueAcceptanceError(`Branch ${request.plan
      .branch} of ${request.plan
        .checkout} tracks no path, so the checkout or the branch name is wrong.`,);
  }
  request.write(`${request.plan
    .branch}: ${String(tracked.length,)} tracked paths in ${request.plan
      .checkout}`,);
  /**
   Settlements collected so far.
   */
  const issues: IssueSettlement[] = [];
  for (const entry of request.plan
    .issues) {
    /**
     What happened to this issue.
     */
    const settlement = await settleIssue({
      request,
      entry,
    },);
    issues.push(settlement,);
  }
  /**
   Corrections collected so far.
   */
  const corrections: CorrectionSettlement[] = [];
  for (const correction of request.plan
    .commitCorrections) {
    /**
     What happened to this correction.
     */
    const settlement = await settleCorrection({
      request,
      commit: correction.commit,
      body: correction.body,
    },);
    corrections.push(settlement,);
  }
  return {
    issues,
    corrections,
    failed: issues.some(isUnfinished,)
      || corrections.some(function isFailedCorrection(settlement,): boolean {
        return settlement.kind === 'failed';
      },),
  };
}
