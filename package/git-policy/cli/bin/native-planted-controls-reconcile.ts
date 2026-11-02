#!/usr/bin/env node
/**
 Settle the planted-controls status of every campaign that deferred its controls to a controls unit.

 Prints one line per deferred campaign (`validated`, `pending` while the controls unit still runs, or `unvalidated` when
 it ended without a matching record) and exits nonzero unless every one is validated, so a person or a script can tell
 provisional mutation results from proved ones after the fact.
 */
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';

import { campaignPrefix } from './native-gate-proof.ts';
import {
  type SettledDirectory,
  settlePlantedStatus,
} from './native-planted-controls-status.ts';

/**
 Directory that keeps the evidence of the gate and of every campaign.
 */
const evidenceRoot = join(
  process.cwd(),
  'target',
  'verification',
);
/**
 Names of the campaigns' evidence directories, in name order.
 */
const names = (await readdir(evidenceRoot)).filter(function isCampaign(name) {
  return name.startsWith(campaignPrefix);
})
  .toSorted();
/**
 One line about a settled directory.

 @param located - the campaign directory name and how its directory settled

 @returns the line to print

 @example
 ```ts
 lineOf({ name: 'native-mutation-abc', outcome });
 ```
 */
function lineOf(located: LocatedOutcome): string {
  /**
   How the directory settled.
   */
  const { outcome } = located;
  if (outcome.state === 'invalid')
    return `${located.name}: invalid, ${outcome.reason}`;
  if (outcome.status
    .status
    === 'retired')
    return `${located.name}: retired (${outcome.status
      .retiredBy})`;
  return `${located.name}: ${outcome.state}${outcome.status
    .record
    === undefined ? '' : ` by ${outcome.status
      .record}`} (unit ${outcome.status
        .unit}, image ${outcome.status
          .image})`;
}

/**
 A settled directory with the campaign directory it was found in.
 */
type LocatedOutcome = {
  readonly name: string;
  readonly outcome: SettledDirectory;
};

/**
 The settled directories of every campaign, each with the campaign it belongs to.
 */
const settled = (await Promise.all(names.map(async function settleCampaign(name) {
  return (await settlePlantedStatus({
    evidence: join(
      evidenceRoot,
      name,
    ),
    evidenceRoot,
    evidencePrefix: campaignPrefix,
  })).map(function located(outcome): LocatedOutcome {
    return {
      name,
      outcome,
    };
  });
}))).flat();
for (const located of settled)
  console.log(lineOf(located));
if (settled.length === 0)
  console.log('No campaign under target/verification deferred its planted controls.');
// A script reads the status of the results from the exit code: zero only when nothing is provisional, unproved or unreadable.
if (settled.some(function isNotSettledWell(located: LocatedOutcome) {
  return (located.outcome
    .state
    !== 'validated') && (located.outcome
      .state
      !== 'retired');
}))
  process.exitCode = 1;
