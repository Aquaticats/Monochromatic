#!/usr/bin/env node
/**
 Mutation-test the engine inside a mount-free, bounded container.

 Mutated code is arbitrary and unpredictable, so it runs only inside the disposable container, never on the host.
 The campaign layers the installed cargo-mutants over the snapshot image that `bin/build-test-image.ts` bakes, runs the
 baseline and every mutant through nextest, and keeps the report.
 The crate's own `.cargo/mutants.toml` applies: it excludes the mutants that are provably equivalent, each with its
 proof.
 The campaign flow is the shared one in `@monochromatic-dev/mutation-test-cli-cargo-mutants`; one campaign runs one
 container, and several run at once through that package's launcher and slot limiter.
 */
import { join } from 'node:path';

import {
  campaignCommand,
  defaultBuildTimeoutSeconds,
  parseContainerLimitOptions,
  parseFileScopes,
  parseTimeoutAndExamine,
  runCampaign,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Concurrent worktrees set FORBIDDEN_REGEX_IMAGE_TAG so one snapshot never mutates another's image.
 */
const imageTag = process.env
  .FORBIDDEN_REGEX_IMAGE_TAG
  ?? 'development';

/**
 Report directory inside the container.
 */
const containerReport = '/work/mutation-report';

/**
 Container limits parsed from the command line (`--memory`, `--cpus`, `--pids-limit`), and the other options.
 */
const {
  limits,
  rest,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
  leadingOnly: false,
});

/**
 Per-mutant test limit, in seconds, and the mutant-name patterns, from `--timeout` and `--examine`.
 */
const {
  timeoutSeconds,
  examine,
  scopeOptions,
} = parseTimeoutAndExamine({
  options: rest,
  rules: {
    defaultTimeoutSeconds: 120,
    maximumTimeoutSeconds: 3_600,
  },
});

/**
 Validated `--file src/<glob>.rs` scopes, or none for a whole-crate campaign.
 */
const scopes = parseFileScopes({
  options: scopeOptions,
  rules: {
    prefix: 'src/',
    suffix: '.rs',
  },
});

await runCampaign({
  testImage: `localhost/forbidden-regex-test:${imageTag}`,
  mutationImage: `localhost/forbidden-regex-mutation:${imageTag}`,
  testImageHint: '; run test:mutation:image and test:mutation:snapshot first',
  // Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
  // whose backing device is unknowable.
  scratchBase: join(
    import.meta.dirname,
    '..',
    'node_modules',
    '.monochromatic',
    'forbidden-regex',
  ),
  contextPrefix: 'forbidden-regex-mutation-',
  evidenceRoot: join(
    process.cwd(),
    'target',
    'verification',
  ),
  evidencePrefix: 'mutation-',
  limits,
  containerReport,
  command: campaignCommand({
    outputDirectory: containerReport,
    timeoutSeconds,
    buildTimeoutSeconds: defaultBuildTimeoutSeconds,
    additionalExclusions: [],
    useProjectConfig: true,
    tail: [
      '--colors=never',
      ...examine.flatMap(function examineArguments(pattern) {
        return [
          '--re',
          pattern
        ];
      }),
      ...scopes.flatMap(function scopeArguments(scope) {
        return [
          '--file',
          scope
        ];
      }),
    ],
  }),
  manifest: function engineManifest({
    baseImage,
    campaignImage,
    toolSha256,
    container,
    command,
  }) {
    return {
      baseImage,
      campaignImage,
      toolSha256,
      container,
      command,
      scopes,
      limits: {
        memory: limits.memory,
        cpus: limits.cpus,
        pids: limits.pidsLimit,
        buildTimeoutSeconds: defaultBuildTimeoutSeconds,
        testTimeoutSeconds: timeoutSeconds,
      },
    };
  },
});
