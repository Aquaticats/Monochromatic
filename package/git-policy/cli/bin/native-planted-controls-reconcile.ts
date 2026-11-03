#!/usr/bin/env node
/**
 Look up the planted-controls obligation of every campaign that deferred its controls to a controls unit.

 Prints one line per deferred campaign directory: the record that validates it, or "no record yet" with the controls unit,
 its invocation and its log. Exits nonzero unless every one is validated, so a person or a script can tell provisional
 mutation results from proved ones after the fact. A controls invocation that ended without a matching record is
 `unvalidated`, which is a failure.
 */
import { readdir } from 'node:fs/promises';
import { join } from 'node:path';

import { campaignPrefix } from './native-gate-proof.ts';
import {
  describeValidation,
  validatePlanted,
} from './native-planted-controls-obligation.ts';

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
 The validations of every campaign's directories that deferred.
 */
const validations = (await Promise.all(names.map(function validateCampaign(name) {
  return validatePlanted({
    evidence: join(
      evidenceRoot,
      name,
    ),
    evidenceRoot,
    evidencePrefix: campaignPrefix,
  });
}))).flat();
for (const validation of validations)
  console.log(describeValidation(validation));
if (validations.length === 0)
  console.log('No campaign under target/verification deferred its planted controls.');
// A script reads the status of the results from the exit code: zero only when nothing is provisional, unproved or unreadable.
if (validations.some(function isNotValidated(validation) {
  return validation.state !== 'validated';
}))
  process.exitCode = 1;
