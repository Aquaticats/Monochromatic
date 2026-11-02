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
  type PlantedStatus,
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
 A settled status with the campaign directory it was found in.
 */
type LocatedStatus = {
  readonly name: string;
  readonly status: PlantedStatus;
};

/**
 The settled statuses of every campaign, each with the directory it was found in.
 */
const settled = (await Promise.all(names.map(async function settleCampaign(name) {
  return (await settlePlantedStatus({
    evidence: join(
      evidenceRoot,
      name,
    ),
    evidenceRoot,
    evidencePrefix: campaignPrefix,
  })).map(function located(status): LocatedStatus {
    return {
      name,
      status,
    };
  });
}))).flat();
for (const {
  name,
  status
} of settled)
  console.log(`${name}: ${status.status}${status.record === undefined ? '' : ` by ${status.record}`} (unit ${status.unit}, image ${status.image})`);
if (settled.length === 0)
  console.log('No campaign under target/verification deferred its planted controls.');
// A script reads the status of the results from the exit code: zero only when nothing is provisional or unproved.
if (settled.some(function isNotValidated(located: LocatedStatus) {
  return located.status
    .status
    !== 'validated';
}))
  process.exitCode = 1;
