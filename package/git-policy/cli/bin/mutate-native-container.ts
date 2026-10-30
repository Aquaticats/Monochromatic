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
import { join } from 'node:path';

import {
  campaignCommand,
  defaultBuildTimeoutSeconds,
  runCampaign,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import { parseContainerLimitOptions } from './container-limits.ts';
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
  leadingOnly: false,
});
/**
 Campaign options parsed from the command line: the per-mutant timeout, the mutant-name patterns and the rest.
 */
const {
  timeoutSeconds,
  examine,
  scopeOptions
} = parseTimeout(campaignOptions);
/**
 Validated `--file` scopes, or none for a whole-package campaign.
 */
const scopes = parseScopes(scopeOptions);

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
  evidenceRoot: join(
    process.cwd(),
    'target',
    'verification',
  ),
  evidencePrefix: 'native-mutation-',
  limits: containerLimits,
  containerReport,
  command: campaignCommand({
    outputDirectory: containerReport,
    timeoutSeconds,
    buildTimeoutSeconds: defaultBuildTimeoutSeconds,
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
  }),
  // Parallel campaigns over one tested snapshot prove the harness once; `NATIVE_MUTATION_SKIP_PLANTED=1` skips the repeat.
  beforeBuild: async function plantControls({
    baseImage,
    containerArguments,
    contextPath,
    evidence,
  }) {
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
    command,
  }) {
    return {
      baseImage,
      toolSha256,
      container,
      command,
      scopes,
      limits: {
        memory: containerLimits.memory,
        cpus: containerLimits.cpus,
        pids: containerLimits.pidsLimit,
        buildTimeoutSeconds: defaultBuildTimeoutSeconds,
        testTimeoutSeconds: 90,
      },
    };
  },
});
