#!/usr/bin/env node
/**
 Remove concrete processor safety guards in a mount-free bounded immutable snapshot.

 The campaign mutates the three processor modules, keeps only the mutants that replace a guard with a constant, and
 requires all five of them to be caught.
 The campaign flow is the shared one in `@monochromatic-dev/mutation-test-cli-cargo-mutants`.
 */
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  campaignCommand,
  CargoMutantsToolingError,
  parseContainerLimitOptions,
  runCampaign,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

/**
 Concurrent worktrees set MONOCHROMATIC_LINT_IMAGE_TAG so one snapshot never runs another's image.
 */
const imageTag = process.env
  .MONOCHROMATIC_LINT_IMAGE_TAG
  ?? 'development';

/**
 Report directory inside the container.
 */
const containerReport = '/work/processor-mutation-report';

/**
 Per-mutant test limit, in seconds.
 */
const timeoutSeconds = 180;

/**
 Guard replacements the campaign tries: a processor guard turned into a constant.
 */
const guardFilter = String.raw`replace (has_main|verify|encoded|rewrite).* with (true|false|Ok\(\(\)\)|String::new\(\)|Ok\(String::new\(\)\))$`;

/**
 Mutants the campaign must catch: the five safety guard controls.
 */
const expectedMutants = 5;

/**
 Source files the campaign mutates.
 */
const processorFiles: readonly string[] = [
  'src/processors_prepare.rs',
  'src/processors_projection.rs',
  'src/processors_rewrite.rs',
];

/**
 Check that a parsed report holds the two counts the campaign needs.

 @param value - parsed `outcomes.json`
 
 @returns true when `total_mutants` and `caught` are both numbers
 
 @example
 ```ts
 hasCounts({ total_mutants: 5, caught: 5 });
 ```
 */
function hasCounts(value: unknown): value is {
  readonly total_mutants: number;
  readonly caught: number
} {
  return ((typeof value) === 'object') && (value !== null)
    && ('total_mutants' in value)
    && ((typeof value.total_mutants) === 'number')
    && ('caught' in value)
    && ((typeof value.caught) === 'number');
}

/**
 Memory, CPUs and processes of the build and the container; this task takes no other option.
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
if (rest.length > 0)
  throw new CargoMutantsToolingError(`Only --memory, --cpus and --pids-limit are accepted, got ${rest.join(' ')}.`);

await runCampaign({
  testImage: `localhost/monochromatic-lint-processors:${imageTag}`,
  mutationImage: `localhost/monochromatic-lint-processors-mutation:${imageTag}`,
  testImageHint: '',
  // Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
  // whose backing device is unknowable.
  scratchBase: join(
    import.meta.dirname,
    '..',
    'node_modules',
    '.monochromatic',
    'monochromatic-lint',
  ),
  contextPrefix: 'processor-mutants-',
  evidenceRoot: join(
    process.cwd(),
    'target',
    'verification',
  ),
  evidencePrefix: 'processors-mutation-',
  limits,
  containerReport,
  command: campaignCommand({
    outputDirectory: containerReport,
    timeoutSeconds,
    buildTimeoutSeconds: 300,
    additionalExclusions: [],
    tail: [
      '--jobserver-tasks',
      '2',
      ...processorFiles.flatMap(function fileArguments(file) {
        return [
          '--file',
          file
        ];
      }),
      '--re',
      guardFilter,
      '--cargo-test-arg=processors',
      '--caught',
    ],
  }),
  manifest: function processorsManifest({
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
    };
  },
  afterReport: async function requireAllCaught(evidence) {
    /**
     Counts of the finished campaign.
     */
    const outcomes: unknown = JSON.parse(await readFile(
      join(
        evidence,
        'mutants.out',
        'outcomes.json',
      ),
      'utf8',
    ));
    if ((!hasCounts(outcomes)) || (outcomes.total_mutants !== expectedMutants)
      || (outcomes.caught !== expectedMutants))
      throw new CargoMutantsToolingError(`Expected all five safety guard controls to be caught; inspect ${evidence}`);
  },
});
