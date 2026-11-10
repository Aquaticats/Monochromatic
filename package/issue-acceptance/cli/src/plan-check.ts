/**
 Reads one check of a plan.

 Each kind reads only the fields it needs,
 and an unknown key fails rather than sitting in the plan unread.

 @module
 */

import { PlanError, } from './errors.ts';
import type {
  Check,
  CheckKind,
} from './plan-model.ts';
import {
  CHECK_KEYS,
  CHECK_KINDS,
  isCheckKind,
  LOWEST_COUNT,
} from './plan-schema.ts';
import {
  asCommit,
  asInteger,
  asRecord,
  asString,
  asStringArray,
  refuse,
  refuseUnknownKeys,
} from './plan-value.ts';

/**
 Reads one check.
 
 @param value - Value read from the plan.
 
 @param at - Path of the check inside the plan.
 
 @returns The check as a typed value.
 
 @throws {@link PlanError} when the check breaks the schema.
 
 @example
 ```ts
 const check = readCheck({ value: { kind: 'exists', path: 'README.md' }, at: 'issues[0].checks[0]' });
 ```
 */
export function readCheck({
  value,
  at,
}: {
  readonly value: unknown;
  readonly at: string;
},): Check {
  /**
   Check as a keyed record, before its kind is read.
   */
  const record = asRecord({
    value,
    at,
  },);
  /**
   Check kind as the plan spelled it.
   */
  const kind = asString({
    value: record.kind,
    at: `${at}.kind`,
  },);
  if (!isCheckKind(kind,)) {
    refuse({
      at: `${at}.kind`,
      expects: `one of ${CHECK_KINDS.join(', ')}`,
      found: JSON.stringify(kind,),
    },);
  }
  refuseUnknownKeys({
    record,
    allowed: CHECK_KEYS[kind],
    at,
  },);
  if ((kind === 'absent') || (kind === 'present')) {
    return {
      kind,
      pattern: asString({
        value: record.pattern,
        at: `${at}.pattern`,
      },),
      paths: asStringArray({
        value: record.paths,
        at: `${at}.paths`,
      },),
    };
  }
  if ((kind === 'exactly') || (kind === 'atLeast')) {
    return {
      kind,
      pattern: asString({
        value: record.pattern,
        at: `${at}.pattern`,
      },),
      paths: asStringArray({
        value: record.paths,
        at: `${at}.paths`,
      },),
      count: asInteger({
        value: record.count,
        at: `${at}.count`,
        floor: LOWEST_COUNT,
      },),
    };
  }
  if ((kind === 'exists') || (kind === 'gone')) {
    return {
      kind,
      path: asString({
        value: record.path,
        at: `${at}.path`,
      },),
    };
  }
  if ((kind === 'fileContains') || (kind === 'fileLacks')) {
    return {
      kind,
      path: asString({
        value: record.path,
        at: `${at}.path`,
      },),
      text: asString({
        value: record.text,
        at: `${at}.text`,
      },),
    };
  }
  if (kind === 'pathMatching') {
    return {
      kind,
      pattern: asString({
        value: record.pattern,
        at: `${at}.pattern`,
      },),
    };
  }
  return {
    kind: 'onBranch',
    commit: asCommit({
      value: record.commit,
      at: `${at}.commit`,
    },),
  };
}
