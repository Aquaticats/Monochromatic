/**
 The plan schema's vocabulary: kinds, actions, key tables, and shapes.

 Keeping the tables apart from the readers
 lets a schema change and the code that enforces it sit beside each other.

 @module
 */

import type {
  CheckKind,
  IssueAction,
} from './plan-model.ts';

/**
 Check kinds a plan may name.
 */
export const CHECK_KINDS: readonly CheckKind[] = [
  'absent',
  'present',
  'exactly',
  'atLeast',
  'exists',
  'gone',
  'fileContains',
  'fileLacks',
  'pathMatching',
  'onBranch',
] as const;

/**
 Actions an issue entry may name.
 */
export const ISSUE_ACTIONS: readonly IssueAction[] = [
  'close',
  'comment',
] as const;

/**
 Keys a plan may hold,
 so a misspelled key fails instead of leaving its work undone.
 */
export const PLAN_KEYS: readonly string[] = [
  'repository',
  'checkout',
  'branch',
  'issues',
  'commitCorrections',
] as const;

/**
 Keys an issue entry may hold.
 */
export const ISSUE_KEYS: readonly string[] = [
  'issue',
  'action',
  'checks',
  'comment',
] as const;

/**
 Keys a commit correction may hold.
 */
export const CORRECTION_KEYS: readonly string[] = [
  'commit',
  'body',
] as const;

/**
 Keys each check kind may hold.
 */
export const CHECK_KEYS: Readonly<Record<CheckKind, readonly string[]>> = {
  absent: [
    'kind',
    'pattern',
    'paths',
  ],
  present: [
    'kind',
    'pattern',
    'paths',
  ],
  exactly: [
    'kind',
    'pattern',
    'paths',
    'count',
  ],
  atLeast: [
    'kind',
    'pattern',
    'paths',
    'count',
  ],
  exists: [
    'kind',
    'path',
  ],
  gone: [
    'kind',
    'path',
  ],
  fileContains: [
    'kind',
    'path',
    'text',
  ],
  fileLacks: [
    'kind',
    'path',
    'text',
  ],
  pathMatching: [
    'kind',
    'pattern',
  ],
  onBranch: [
    'kind',
    'commit',
  ],
};

/**
 Smallest issue number a plan may name.
 */
export const LOWEST_ISSUE_NUMBER: number = 1;

/**
 Smallest match count a count check may name.
 */
export const LOWEST_COUNT: number = 0;

/**
 Longest inspected value quoted back in a plan error,
 so a whole pasted body never floods the terminal.
 */
export const ECHO_LIMIT: number = 80;

/**
 Answers whether a value names a check kind this command implements.
 
 @param value - Value as the plan spelled it.
 
 @returns True for one of the check kinds.
 
 @example
 ```ts
 const known = isCheckKind('exists');
 ```
 */
export function isCheckKind(value: string,): value is CheckKind {
  return (CHECK_KINDS as readonly string[]).includes(value,);
}

/**
 Answers whether a value names an issue action this command implements.
 
 @param value - Value as the plan spelled it.
 
 @returns True for one of the actions.
 
 @example
 ```ts
 const known = isIssueAction('close');
 ```
 */
export function isIssueAction(value: string,): value is IssueAction {
  return (ISSUE_ACTIONS as readonly string[]).includes(value,);
}
