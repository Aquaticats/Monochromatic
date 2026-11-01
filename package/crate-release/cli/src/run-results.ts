import {
  decimalDigits,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';

import {
  isRecord,
  parseJson,
} from './json-value.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import { hasControlCharacter } from './validate.ts';

/**
 Read the run identifier out of a run address.

 The address must be an `https` address of the form `<host>/<owner>/<repo>/actions/runs/<digits>`, optionally followed by
 a job path, a query or a fragment, and must hold no space or control character, which URL parsing would otherwise
 quietly drop.

 @param address - text under test

 @returns the identifier, or a miss when the text is not such an address

 @example
 ```ts
 runIdOfAddress('https://github.com/o/r/actions/runs/123');
 ```
 */
function runIdOfAddress(address: string): Lookup<string> {
  if (hasControlCharacter(address) || address.includes(' '))
    return missingValue('The address holds a space or control character.');
  /**
   The parsed address, when the text parses.
   */
  const url = URL.parse(address);
  if ((url === null) || (url.protocol !== 'https:'))
    return missingValue('The address is not an https address.');
  /**
   Path segments: an empty one for the leading slash, owner, repository, `actions`, `runs`, the identifier, then
   whatever follows.
   */
  const [, owner, repository, actions, runs, runId] = url.pathname
    .split('/');
  if ((owner === undefined) || (owner === '')
    || (repository === undefined)
    || (repository === '')
    || (actions !== 'actions')
    || (runs !== 'runs')
    || (runId === undefined)
    || (!usesOnly({
      text: runId,
      allowed: decimalDigits
    })))
    return missingValue('The address is not of the form <host>/<owner>/<repo>/actions/runs/<digits>.');
  return foundValue(runId);
}

/**
 Read the numeric identifier of a workflow run from what a person gives: the identifier itself, or the run's address.

 @param reference - all digits, or an `https://<host>/<owner>/<repo>/actions/runs/<digits>` address

 @returns the identifier, or a miss that says which forms are accepted

 @example
 ```ts
 parseRunReference('https://github.com/o/r/actions/runs/123');
 ```
 */
export function parseRunReference(reference: string): Lookup<string> {
  if (usesOnly({
    text: reference,
    allowed: decimalDigits
  }))
    return foundValue(reference);
  /**
   The identifier, when the reference is an address.
   */
  const fromAddress = runIdOfAddress(reference);
  return fromAddress.found
    ? fromAddress
    : missingValue(
      `${JSON.stringify(reference)} is neither a run identifier (digits) nor a run address such as https://github.com/owner/name/actions/runs/123.`,
    );
}

/**
 Find the address of the created run in what `gh workflow run` printed.

 @param output - standard output of `gh workflow run`

 @returns the first line that is a run address, or a miss when `gh` did not print one

 @example
 ```ts
 findRunAddress('Created workflow_dispatch event\nhttps://github.com/o/r/actions/runs/9\n');
 ```
 */
export function findRunAddress(output: string): Lookup<string> {
  /**
   The first line that is a run address, trimmed.
   */
  const address = output.split('\n')
    .map(function trimmed(line) {
      return line.trim();
    })
    .find(function isRunAddress(line) {
      return line.startsWith('https://')
        && runIdOfAddress(line)
          .found;
    });
  return address === undefined ? missingValue('gh did not print the address of the created run.') : foundValue(address);
}

/**
 Read when a run was created from the answer of `gh run view`.

 @param body - JSON text printed by `gh run view --json createdAt,status,conclusion,url`

 @returns the creation time as the API wrote it, or a miss when the answer is not JSON or lacks a time

 @example
 ```ts
 parseRunCreatedAt('{"createdAt":"2026-11-01T10:00:00Z"}');
 ```
 */
export function parseRunCreatedAt(body: string): Lookup<string> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if ((!isRecord(answer.value)) || ((typeof answer.value
    .createdAt) !== 'string')
    || Number.isNaN(Date.parse(answer.value
      .createdAt as string)))
    return missingValue('The run answer holds no createdAt time.');
  return foundValue(answer.value
    .createdAt as string);
}

/**
 The run the publish workflow started: its identifier and the commit it was built from.
 */
export type DispatchedRun = {
  readonly runId: string;
  readonly headSha: string;
};

/**
 Check that a listed run has the three fields the selection reads.

 @param entry - one entry of the run listing

 @returns true when it has a numeric `databaseId`, a text `headSha` and a `createdAt` that is a time

 @example
 ```ts
 isListedRun({ databaseId: 1, headSha: 'a', createdAt: '2026-11-01T10:00:00Z' });
 ```
 */
function isListedRun(entry: unknown): entry is {
  readonly databaseId: number;
  readonly headSha: string;
  readonly createdAt: string;
} {
  return isRecord(entry)
    && ((typeof entry.databaseId) === 'number')
    && ((typeof entry.headSha) === 'string')
    && ((typeof entry.createdAt) === 'string')
    && (!Number.isNaN(Date.parse(entry.createdAt as string)));
}

/**
 Pick the manually started run that the release run started: the earliest one created at or after the release run.

 A release run starts its publish run before it ends, so the publish run is never older than the release run, and a
 manual dry run that someone started earlier cannot be mistaken for it.

 @param listing - JSON text printed by `gh run list --event workflow_dispatch --json databaseId,headSha,createdAt`

 @param notBefore - creation time of the release run, as the API wrote it

 @returns the run, or a miss when the listing is not a list, or holds no run created at or after `notBefore`

 @example
 ```ts
 selectDispatchedRun({ listing: '[]', notBefore: '2026-11-01T10:00:00Z' });
 ```
 */
export function selectDispatchedRun({
  listing,
  notBefore,
}: {
  readonly listing: string;
  readonly notBefore: string;
}): Lookup<DispatchedRun> {
  /**
   Parsed listing.
   */
  const runs = parseJson(listing);
  if (!runs.found)
    return runs;
  if (!Array.isArray(runs.value))
    return missingValue('The run listing is not a list.');
  /**
   Earliest start the run may have, as a time.
   */
  const earliest = Date.parse(notBefore);
  /**
   Listed runs created at or after the release run, oldest first.
   */
  const candidates = (runs.value as readonly unknown[])
    .filter(isListedRun)
    .filter(function isNotOlder(entry) {
      return Date.parse(entry.createdAt) >= earliest;
    })
    .toSorted(function oldestFirst(
      first,
      second
    ) {
      return Date.parse(first.createdAt) - Date.parse(second.createdAt);
    });
  /**
   The oldest candidate.
   */
  const [oldest] = candidates;
  return oldest === undefined
    ? missingValue(`No listed run was created at or after ${notBefore}.`)
    : foundValue({
      runId: String(oldest.databaseId),
      headSha: oldest.headSha
    });
}
