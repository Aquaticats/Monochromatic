#!/usr/bin/env node
/**
 Mutation-test the native wrapper inside disposable, bounded containers.

 Phase 1 plants known guard removals and requires the named control to fail,
 proving the harness can observe a missing guard before any mutation result is trusted.
 Phase 2 runs cargo-mutants over the already tested source snapshot.
 Source mutation and compilation stay inside containers; only reports leave them.
 The campaign flow itself is the shared runner in `@monochromatic-dev/mutation-test-cli-cargo-mutants`;
 this script supplies the native wrapper's names, scopes, limits and manifest.
 */
import {
  join,
  resolve,
} from 'node:path';

import {
  campaignCommand,
  runCampaign,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { parseContainerLimitOptions } from './container-limits.ts';
import { passingGateEvidence } from './native-gate-proof.ts';
import { passOneFor } from './native-pass-one.ts';
import { runPlantedControls } from './native-planted-controls.ts';
import {
  baselineSkipArguments,
  parseScopes,
  parseTimeout,
} from './native-mutation-options.ts';

/**
 Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never mutates another's image.
 */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';
/**
 Report directory inside the container; the tester's home is the only writable place outside the package.
 */
const containerReport = '/home/tester/mutation-report';
/**
 Container limits parsed from the command line (`--memory`, `--cpus`, `--pids-limit`), and the other options.
 */
const {
  limits: containerLimits,
  rest: campaignOptions,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
});
/**
 Campaign options parsed from the command line: the per-mutant test and build timeouts, the mutant-name patterns and
 the rest.
 */
const {
  timeoutSeconds,
  buildTimeoutSeconds,
  examine,
  scopeOptions,
  passes,
} = parseTimeout(campaignOptions);
/**
 Validated `--file` scopes, or none for a whole-package campaign.
 */
const scopes = parseScopes(scopeOptions);
/**
 Directory that keeps the evidence of the gate and of every campaign.
 */
const evidenceRoot = join(
  process.cwd(),
  'target',
  'verification',
);

/**
 The campaign command: the whole suite for every mutant, the baseline run, and the scopes.
 */
const command = campaignCommand({
  outputDirectory: containerReport,
  timeoutSeconds,
  buildTimeoutSeconds,
  additionalExclusions: [],
  tail: [
    ...examine.flatMap(function examineArguments(pattern) {
      return [
        '--re',
        pattern,
      ];
    }),
    // Scoped runs are evidence for the named files only. The unmutated baseline still runs over everything.
    ...scopes.flatMap(function scopeArguments(scope) {
      return [
        '--file',
        scope,
      ];
    }),
    ...baselineSkipArguments(),
  ],
});
/**
 Pass 1 of a two-pass campaign (`--passes 2`): the tests that can reach the mutated files. Absent for one pass.
 */
const passOne = passes === 2
  ? await passOneFor({
    // NATIVE_MUTATION_CRATE_DIRECTORY (relative to the working directory) names another crate, for the pinned scenarios.
    crateDirectory: resolve(
      process.cwd(),
      process.env
        .NATIVE_MUTATION_CRATE_DIRECTORY
        ?? join(
          import.meta.dirname,
          '..',
        ),
    ),
    scopes,
    command,
  })
  : undefined;
if (passOne !== undefined) {
  console.log(`Pass 1 selects the tests of ${passOne.scope
    .libraryModules
    .length} of ${passOne.scope
      .libraryModuleCount} library modules${
     passOne.scope
       .testTargets
       .length
       === 0 ? '' : ` and the test targets ${passOne.scope
         .testTargets
         .join(', ')}`
     } (the closure of the importers of ${passOne.scope
       .mutatedModules
       .join(', ')}); pass 2 runs the whole suite on its survivors.`);
}

await runCampaign({
  testImage: `localhost/git-policy-native-test:${imageTag}`,
  mutationImage: `localhost/git-policy-native-mutation:${imageTag}`,
  testImageHint: '; run native:test:container first',
  // Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
  // whose backing device is unknowable.
  scratchBase: join(
    import.meta.dirname,
    '..',
    'node_modules',
    '.monochromatic',
    'git-policy',
  ),
  contextPrefix: 'cli-git-native-mutation-',
  evidenceRoot,
  evidencePrefix: 'native-mutation-',
  limits: containerLimits,
  containerReport,
  command,
  ...(passOne === undefined ? {} : { passOne: { command: passOne.command } }),
  // A gate that passed on this exact image ran the same suite, so the campaign's own baseline adds only time.
  baselineProof: async function gateProof({ baseImage }) {
    return await passingGateEvidence({
      evidenceRoot,
      image: baseImage,
      limits: {
        memory: containerLimits.memory,
        pidsLimit: containerLimits.pidsLimit,
      },
    });
  },
  // Parallel campaigns over one tested snapshot prove the harness once; `NATIVE_MUTATION_SKIP_PLANTED=1` skips the repeat.
  beforeBuild: async function plantControls({
    baseImage,
    containerArguments,
    contextPath,
    evidence,
    scoredMutants,
  }) {
    // A resumed campaign scored mutants only after an earlier attempt passed the controls over this same snapshot.
    if (scoredMutants > 0) {
      console.log('Planted controls skipped: an earlier attempt of this campaign passed them before scoring.');
      return;
    }
    if (process.env
      .NATIVE_MUTATION_SKIP_PLANTED
      === '1') {
      console.log('Planted controls skipped: another campaign over this snapshot ran them.');
      return;
    }
    await runPlantedControls({
      base: baseImage,
      limits: [...containerArguments],
      context: contextPath,
      evidence,
    });
  },
  manifest: function nativeManifest({
    baseImage,
    toolSha256,
    container,
    command: containerCommand,
    baselineProof,
    pass,
  }) {
    return {
      baseImage,
      toolSha256,
      container,
      command: containerCommand,
      ...(baselineProof === undefined ? {} : { baselineProof }),
      ...(pass === undefined ? {} : { pass }),
      ...(passOne === undefined ? {} : {
        passOne: {
          mutatedModules: passOne.scope
            .mutatedModules,
          selectedModules: passOne.scope
            .libraryModules
            .length,
          libraryModuleCount: passOne.scope
            .libraryModuleCount,
          testTargets: passOne.scope
            .testTargets,
          filterset: passOne.filterset,
        },
      }),
      scopes,
      limits: {
        memory: containerLimits.memory,
        cpus: containerLimits.cpus,
        pids: containerLimits.pidsLimit,
        buildTimeoutSeconds,
        testTimeoutSeconds: timeoutSeconds,
      },
    };
  },
});
