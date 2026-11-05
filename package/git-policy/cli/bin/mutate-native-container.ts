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
import {
  campaignPrefix,
  passingGateEvidence,
} from './native-gate-proof.ts';
import { passOneFor } from './native-pass-one.ts';
import {
  plantedControls,
  runPlantedControls,
} from './native-planted-controls.ts';
import { ensurePlantedControls } from './native-planted-controls-ensure.ts';
import { campaignKeyParts } from './native-planted-controls-key-parts.ts';
import { plantedHarnessId } from './native-planted-controls-harness.ts';
import {
  assertNotUnmet,
  describeValidation,
  earlierPassOwes,
  validatePlanted,
} from './native-planted-controls-obligation.ts';
import {
  obligationField,
  type PlantedObligation,
} from './native-planted-controls-manifest.ts';
import { controlsKeyOf } from './native-planted-controls-record.ts';
import {
  plantedUnitFor,
  plantedUnitsEnvironmentVariable,
} from './native-planted-controls-units.ts';
import {
  baselineSkipArguments,
  parseScopes,
  parseTimeout,
} from './native-mutation-options.ts';

/**
 The planted-controls obligation the latest container run of this process carries, as the only element, or none: the one
 its manifest holds, and the one finalization must find again.
 */
const outstanding: { current: readonly PlantedObligation[] } = { current: [] };

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
  // Kept outcomes were scored under these controls, this runner and judge, and these limits; changing any rescores.
  keyParts: campaignKeyParts({
    controls: controlsKeyOf(plantedControls),
    harness: plantedHarnessId,
    limits: {
      memory: containerLimits.memory,
      pidsLimit: containerLimits.pidsLimit,
    },
  }),
  contextPrefix: 'monochromatic-git-policy-cli-mutation-',
  evidenceRoot,
  evidencePrefix: campaignPrefix,
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
  // A pass of the controls over this image at these limits is recorded in the campaign's evidence, and a later campaign
  // over the same image with at least those limits skips them. `NATIVE_MUTATION_SKIP_PLANTED=1` skips without a record.
  beforeBuild: async function plantControls({
    baseImage,
    containerArguments,
    contextPath,
    evidence,
  }) {
    /**
     The unit that runs these controls beside this campaign, as the only element, or none; `native:mutation:parallel`
     names it when no record proves them.
     */
    const [beside] = plantedUnitFor({
      image: baseImage,
      text: process.env[plantedUnitsEnvironmentVariable] ?? '',
    });
    /**
     What this attempt decided about the planted controls.
     */
    const decision = await ensurePlantedControls({
      setting: {
        base: baseImage,
        limits: [...containerArguments],
        context: contextPath,
        evidence,
      },
      evidenceRoot,
      evidencePrefix: campaignPrefix,
      skipRequested: process.env
        .NATIVE_MUTATION_SKIP_PLANTED
        === '1',
      ...(beside === undefined ? {} : { beside, }),
      // A later pass that runs the controls itself must publish the record when the first pass is still waiting for one.
      requirePublished: await earlierPassOwes(evidence),
      run: runPlantedControls,
    });
    // The manifest this attempt writes holds the obligation only when it deferred; a proof or a local run owes nothing.
    outstanding.current = decision.kind === 'deferred' ? [decision.obligation] : [];
  },
  // The campaign's own report is complete here; a deferral whose controls unit ended without a record is a failure to say so.
  afterReport: async function validateDeferral(evidence) {
    /**
     How the obligation of this campaign (and of its pass-1 directory) stands against the records that exist now.
     */
    const validations = await validatePlanted({
      evidence,
      evidenceRoot,
      evidencePrefix: campaignPrefix,
    });
    for (const validation of validations)
      console.log(describeValidation(validation));
    assertNotUnmet({
      validations,
      expected: outstanding.current,
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
      // A deferral is a debt: these results hold only once a matching record exists (see afterReport).
      ...(outstanding.current[0] === undefined ? {} : { [obligationField]: outstanding.current[0], }),
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
