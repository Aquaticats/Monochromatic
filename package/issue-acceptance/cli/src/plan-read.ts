/**
 Reads and validates a settlement plan file.

 Validation happens before any child process runs,
 so a plan typo fails with the field's path instead of a half-settled round.

 @module
 */

import { readFile, } from 'node:fs/promises';

import { PlanError, } from './errors.ts';
import {
  readCommitCorrection,
  readIssueEntry,
  refuseRepeatedIssues,
} from './plan-entry.ts';
import type {
  CommitCorrection,
  IssueEntry,
  Plan,
} from './plan-model.ts';
import {
  ECHO_LIMIT,
  PLAN_KEYS,
} from './plan-schema.ts';
import { looksLikeRepository, } from './plan-shape.ts';
import {
  asArray,
  asRecord,
  asString,
  refuse,
  refuseUnknownKeys,
} from './plan-value.ts';

/**
 Parses plan text, naming the source when it is not JSON.
 
 @param text - Plan file contents.
 
 @param at - How failures name the source.
 
 @returns The parsed value, before the schema is checked.
 
 @throws {@link PlanError} when the text is not JSON.
 
 @example
 ```ts
 const parsed = parseText({ text: '{}', at: 'plan.json' });
 ```
 */
function parseText({
  text,
  at,
}: {
  readonly text: string;
  readonly at: string;
},): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new PlanError(`${at} is not valid JSON: ${error.message}`,);
  }
}

/**
 Parses and validates plan text.
 
 @param text - Plan file contents.
 
 @param at - How failures name the source, usually the file path.
 
 @returns The validated plan.
 
 @throws {@link PlanError} when the text is not a valid plan.
 
 @example
 ```ts
 const plan = parsePlan({ text: '{"repository":"OWNER/NAME","checkout":"/repo","branch":"main","issues":[]}', at: 'plan.json' });
 ```
 */
export function parsePlan({
  text,
  at,
}: {
  readonly text: string;
  readonly at: string;
},): Plan {
  /**
   Plan as a keyed record, before its fields are read.
   */
  const record = asRecord({
    value: parseText({
      text,
      at,
    },),
    at,
  },);
  refuseUnknownKeys({
    record,
    allowed: PLAN_KEYS,
    at,
  },);
  /**
   Repository slug the plan settles issues in.
   */
  const repository = asString({
    value: record.repository,
    at: `${at}.repository`,
  },);
  if (!looksLikeRepository(repository,)) {
    refuse({
      at: `${at}.repository`,
      expects: 'an OWNER/NAME repository slug',
      found: JSON.stringify(repository.slice(
        0,
        ECHO_LIMIT,
      ),),
    },);
  }
  /**
   Issue entries, each already checked against the entry schema.
   */
  const issues = asArray({
    value: record.issues,
    at: `${at}.issues`,
  },)
    .map(function readOneEntry(
      entry,
      index,
    ): IssueEntry {
      return readIssueEntry({
        value: entry,
        at: `${at}.issues[${String(index,)}]`,
      },);
    },);
  refuseRepeatedIssues(issues,);
  return {
    repository,
    checkout: asString({
      value: record.checkout,
      at: `${at}.checkout`,
    },),
    branch: asString({
      value: record.branch,
      at: `${at}.branch`,
    },),
    issues,
    commitCorrections: asArray({
      value: record.commitCorrections ?? [],
      at: `${at}.commitCorrections`,
    },)
      .map(function readOneCorrection(
        correction,
        index,
      ): CommitCorrection {
        return readCommitCorrection({
          value: correction,
          at: `${at}.commitCorrections[${String(index,)}]`,
        },);
      },),
  };
}

/**
 Reads and validates a plan file.
 
 @param path - Plan file path.
 
 @returns The validated plan.
 
 @throws {@link PlanError} when the file cannot be read or is not a valid plan.
 
 @example
 ```ts
 const plan = await readPlan({ path: 'plan.json' });
 ```
 */
/**
 Reads the plan file's text, naming the file when it cannot be read.
 
 @param path - Plan file path.
 
 @returns The file's contents.
 
 @throws {@link PlanError} when the file cannot be read.
 
 @example
 ```ts
 const text = await readText({ path: 'plan.json' });
 ```
 */
async function readText({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new PlanError(`The plan file ${path} could not be read: ${error.message}`,);
  }
}

/**
 Reads and validates a plan file.
 
 @param path - Plan file path.
 
 @returns The validated plan.
 
 @throws {@link PlanError} when the file cannot be read or is not a valid plan.
 
 @example
 ```ts
 const plan = await readPlan({ path: 'plan.json' });
 ```
 */
export async function readPlan({ path, }: { readonly path: string; },): Promise<Plan> {
  return parsePlan({
    text: await readText({ path, },),
    at: path,
  },);
}
