/**
 Reads an issue entry and a commit correction from a plan.

 @module
 */

import { PlanError, } from './errors.ts';
import { readCheck, } from './plan-check.ts';
import type {
  Check,
  CommitCorrection,
  IssueEntry,
} from './plan-model.ts';
import {
  CORRECTION_KEYS,
  isIssueAction,
  ISSUE_ACTIONS,
  ISSUE_KEYS,
  LOWEST_ISSUE_NUMBER,
} from './plan-schema.ts';
import {
  asArray,
  asCommit,
  asInteger,
  asRecord,
  asString,
  refuse,
  refuseUnknownKeys,
} from './plan-value.ts';

/**
 Reads one issue entry.
 
 @param value - Value read from the plan.
 
 @param at - Path of the entry inside the plan.
 
 @returns The entry as a typed value.
 
 @throws {@link PlanError} when the entry breaks the schema.
 
 @example
 ```ts
 const entry = readIssueEntry({ value: { issue: 1, action: 'close', checks: [], comment: 'Fixed.' }, at: 'issues[0]' });
 ```
 */
export function readIssueEntry({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): IssueEntry {
  /**
   Entry as a keyed record, before its fields are read.
   */
  const record = asRecord({
    value,
    at,
  },);
  refuseUnknownKeys({
    record,
    allowed: ISSUE_KEYS,
    at,
  },);
  /**
   Action as the plan spelled it.
   */
  const action = asString({
    value: record.action,
    at: `${at}.action`,
  },);
  if (!isIssueAction(action,)) {
    refuse({
      at: `${at}.action`,
      expects: `one of ${ISSUE_ACTIONS.join(', ')}`,
      found: JSON.stringify(action,),
    },);
  }
  return {
    issue: asInteger({
      value: record.issue,
      at: `${at}.issue`,
      floor: LOWEST_ISSUE_NUMBER,
    },),
    action,
    checks: asArray({
      value: record.checks,
      at: `${at}.checks`,
    },)
      .map(function readOneCheck(
        check,
        index,
      ): Check {
        return readCheck({
          value: check,
          at: `${at}.checks[${String(index,)}]`,
        },);
      },),
    comment: asString({
      value: record.comment,
      at: `${at}.comment`,
    },),
  };
}

/**
 Reads one commit correction.
 
 @param value - Value read from the plan.
 
 @param at - Path of the correction inside the plan.
 
 @returns The correction as a typed value.
 
 @throws {@link PlanError} when the correction breaks the schema.
 
 @example
 ```ts
 const correction = readCommitCorrection({ value: { commit: '2b309f8f25', body: 'Correction.' }, at: 'commitCorrections[0]' });
 ```
 */
export function readCommitCorrection({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): CommitCorrection {
  /**
   Correction as a keyed record, before its fields are read.
   */
  const record = asRecord({
    value,
    at,
  },);
  refuseUnknownKeys({
    record,
    allowed: CORRECTION_KEYS,
    at,
  },);
  return {
    commit: asCommit({
      value: record.commit,
      at: `${at}.commit`,
    },),
    body: asString({
      value: record.body,
      at: `${at}.body`,
    },),
  };
}

/**
 Refuses two entries for one issue,
 because the second would comment on an issue the first closed.
 
 @param entries - Entries read so far.
 
 @throws {@link PlanError} naming the repeated issue.
 
 @example
 ```ts
 refuseRepeatedIssues([{ issue: 1, action: 'close', checks: [], comment: 'Fixed.' }]);
 ```
 */
export function refuseRepeatedIssues(entries: readonly IssueEntry[],): void {
  /**
   Issue numbers already named.
   */
  const seen = new Set<number>();
  for (const entry of entries) {
    if (seen.has(entry.issue,)) {
      throw new PlanError(`The plan names issue #${String(entry.issue,)} twice. Settle it in one entry, or remove the other.`,);
    }
    seen.add(entry.issue,);
  }
}
