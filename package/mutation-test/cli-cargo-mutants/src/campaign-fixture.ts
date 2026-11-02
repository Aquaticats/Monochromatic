#!/usr/bin/env node
/**
 Smallest product campaign script, run by the resume tests against the stand-in programs.
 It supplies what a product adapter supplies, with fixed names, and starts `runCampaign`.
 */
import { join } from 'node:path';

import { campaignCommand } from './campaign-files.ts';
import { runCampaign } from './campaign-runner.ts';
import { defaultContainerLimits } from './container-limits.ts';

/**
 Report directory inside the container.
 */
const containerReport = '/report';

/**
 Proof text the fixture's product offers that its tested image passed the whole suite; absent means no proof.
 */
const baselineProof = process.env.FIXTURE_BASELINE_PROOF;

await runCampaign({
  testImage: 'localhost/fixture-test:1',
  mutationImage: 'localhost/fixture-mutation:1',
  testImageHint: '',
  scratchBase: join(
    process.cwd(),
    'scratch',
  ),
  contextPrefix: 'fixture-context-',
  evidenceRoot: join(
    process.cwd(),
    'target',
    'verification',
  ),
  evidencePrefix: 'fixture-',
  limits: defaultContainerLimits,
  containerReport,
  command: campaignCommand({
    outputDirectory: containerReport,
    timeoutSeconds: 1,
    buildTimeoutSeconds: 1,
    additionalExclusions: [],
    tail: [],
  }),
  ...(baselineProof === undefined ? {} : {
    baselineProof: async function fixtureProof() {
      return baselineProof;
    },
  }),
  manifest: function fixtureManifest(facts) {
    return facts;
  },
});
