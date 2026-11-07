/**
 The issues an earlier run could not publish: merged by marker and retried at the start of the next run.

 @module
 */

import { publishIssues, } from './daily-update-issue-publish.ts';
import type {
  CheckEffects,
  RenderedIssue,
} from './daily-update-package-check-types.ts';
import type { DailyUpdateEffects, } from './daily-update-run-types.ts';

/**
 Merges issues waiting for the next run, keeping one per marker.

 @param issues - issues in order of arrival

 @returns the issues without a repeated marker

 @example
 ```ts
 oneIssuePerMarker([issue, issue,],).length; // 1
 ```
 */
export function oneIssuePerMarker(issues: readonly RenderedIssue[],): readonly RenderedIssue[] {
  return [
    ...new Map(issues.map(function keyed(issue,) {
      return [
        issue.marker,
        issue,
      ] as const;
    },),)
      .values(),
  ];
}

/**
 Publishes the issues an earlier run could not, and keeps the ones that still fail.

 @param effects - the outside world of the update

 @param checkEffects - the outside world of the checks

 @param problems - where problems for the owner are collected

 @example
 ```ts
 await retryPendingIssues({ effects, checkEffects, problems: [], },);
 ```
 */
export async function retryPendingIssues({
  effects,
  checkEffects,
  problems,
}: {
  readonly effects: DailyUpdateEffects;
  readonly checkEffects: CheckEffects;
  readonly problems: string[];
},): Promise<void> {
  /**
   Issues left by earlier runs.
   */
  const pending = await checkEffects.readPendingIssues();
  if (pending.length === 0)
    return;
  effects.log(`publishing ${pending.length} issue(s) an earlier run could not publish`,);
  /**
   What came of each.
   */
  const outcomes = await publishIssues({
    gh: checkEffects.gh,
    issues: pending,
  },);
  /**
   Outcomes that failed again.
   */
  const stillFailing = outcomes.filter(function failed(outcome,) {
    return outcome.action === 'failed';
  },);
  for (const outcome of outcomes) {
    effects.log(
      `pending issue ${outcome.issue
        .title}: ${outcome.action}${outcome.reason === '' ? '' : ` (${outcome.reason})`}`,
    );
  }
  for (const outcome of stillFailing) {
    problems.push(
      `An issue from an earlier run is still waiting to be published (${outcome.issue
        .title}): ${outcome.reason}`,
    );
  }
  await checkEffects.writePendingIssues(stillFailing.map(function issueOf(outcome,) {
    return outcome.issue;
  },),);
}
