#!/usr/bin/env node
/**
 What:
  Measures the latency the release native wrapper adds to forwarded Git commands and to explicit-path commits.
 Why:
  `SPEC.md` ("Benchmark method") requires a paired direct-Git baseline for every scenario, the median, p95 and median
  absolute deviation of each, the wrapper-added delta, and the raw samples; the added latency of a forwarded
  command must stay strictly under two seconds, and (owner's measure of 2026-11-01) a commit through the wrapper must
  not take five seconds or more longer than the same commit with native Git.

 In TS you'd write (pseudocode):
 ```ts
 for (const scenario of scenarios) report(await paired(scenario, samples));
 ```
 */
import {
  access,
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { parseArgs } from 'node:util';

import {
  enableSshSigning,
  repository,
} from './native-perf-git.ts';
import {
  makeSigningKey,
  startService,
} from './native-perf-service.ts';
import {
  COMMIT_DELTA_BOUND_MILLISECONDS,
  FORWARDED_BOUND_MILLISECONDS,
  judge,
} from './native-perf-judge.ts';
import {
  reportLine,
  violationLine,
} from './native-perf-lines.ts';
import { removeScratchRoot } from './native-perf-scratch.ts';
import {
  measure,
  type SampleFailure,
  type ScenarioMeasurement,
  type ScenarioReport,
} from './native-perf-measure.ts';

/**
 The fewest samples per scenario that make a median, a p95 and a deviation meaningful.
 */
const MINIMUM_SAMPLES = 5;

/**
 Permission bits of the copied wrapper binary: executable by everyone, writable by its owner.
 */
const EXECUTABLE_MODE = 0o755;

/**
 Permission bits of the scratch parent directory: private to the current user.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 Parsed command line: the binary to measure, the sample count and the optional report path.
 */
const { values } = parseArgs({
  options: {
    binary: { type: 'string' },
    samples: {
      type: 'string',
      default: '30',
    },
    output: { type: 'string' },
  },
});
if (values.binary === undefined) {
  throw new Error('--binary <path to the release cli-git-native> is required');
}
/**
 Samples per scenario, validated below as an integer of at least the minimum.
 */
const samples = Number(values.samples);
if ((!Number.isInteger(samples)) || (samples < MINIMUM_SAMPLES)) {
  throw new Error(`--samples must be an integer of at least ${String(MINIMUM_SAMPLES)}`);
}

if (values.output !== undefined) {
  /**
   Directory that must already hold the report, checked before any measurement starts.
   */
  const outputDirectory = dirname(resolve(values.output));
  try {
    await access(outputDirectory);
  }
  catch (error) {
    throw new Error(
      `the directory of --output does not exist: ${outputDirectory}`,
      { cause: error },
    );
  }
}

/**
 Private parent of the scratch root, outside the repository so no fixture is ever committed.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);
await mkdir(
  scratchParent,
  {
    recursive: true,
    mode: PRIVATE_DIRECTORY_MODE,
  },
);
/**
 Scratch root holding the wrapper `bin`, the empty `home` and both fixture repositories.
 */
const root = await mkdtemp(join(
  scratchParent,
  'native-perf-',
));
await mkdir(join(
  root,
  'bin',
));
await mkdir(join(
  root,
  'home',
));
/**
 Copy of the release wrapper, so the measurement never races a rebuild of the original.
 */
const wrapperBinary = join(
  root,
  'bin',
  'cli-git-native',
);
await copyFile(
  resolve(values.binary),
  wrapperBinary,
);
await chmod(
  wrapperBinary,
  EXECUTABLE_MODE,
);
await symlink(
  wrapperBinary,
  join(
    root,
    'bin',
    'git',
  ),
);

/**
 Fixture repository that the direct samples run in.
 */
const directRepository = await repository({
  root,
  name: 'direct-repo',
});
/**
 Fixture repository that the wrapped samples run in.
 */
const wrappedRepository = await repository({
  root,
  name: 'wrapped-repo',
});

/**
 Forwarded read commands: the wrapper only classifies them and runs real Git.
 */
const forwarded = [
  {
    name: 'rev-parse',
    arguments_: [
      'rev-parse',
      'HEAD',
    ],
  },
  {
    name: 'status',
    arguments_: [
      'status',
      '--porcelain',
    ],
  },
  {
    name: 'log',
    arguments_: [
      'log',
      '-1',
      '--format=%H',
    ],
  },
  {
    name: 'diff-stat',
    arguments_: [
      'diff',
      '--stat',
    ],
  },
];

/**
 Throwaway SSH signing key of this run, made in the scratch root; never the person's key.
 */
const signing = await makeSigningKey({ root, });
/**
 Fixture repositories whose commits are SSH-signed, as the person's commits are, so the signed scenarios measure what the
 person runs.
 */
const directSignedRepository = await repository({
  root,
  name: 'direct-signed-repo',
});
const wrappedSignedRepository = await repository({
  root,
  name: 'wrapped-signed-repo',
});
for (const directory of [
  directSignedRepository,
  wrappedSignedRepository,
]) {
  await enableSshSigning({
    root,
    directory,
    ...signing,
  });
}
/**
 The commit service this run starts for the service-route scenarios, in a runtime directory of its own. The wrapped samples of
 the other scenarios get no runtime directory, so they take the route used before the service existed and nothing in the
 person's own runtime directory or session is read or changed.
 */
const service = await startService({
  program: join(
    dirname(resolve(values.binary)),
    'cli-git-service',
  ),
  root,
  scratchParent,
});

/**
 Failed samples of every scenario, in the order they ran.
 */
const failures: SampleFailure[] = [];
/**
 Report of every scenario that had a successful sample on both sides.
 */
const report: ScenarioReport[] = [];

/**
 Keep a scenario's failed samples and its report, in the order the scenarios ran.

 @param measurement - what one scenario measured
 */
function record(measurement: ScenarioMeasurement): void {
  failures.push(...measurement.failures);
  if (measurement.entry !== undefined) {
    report.push(measurement.entry);
  }
}

/**
 Measure every scenario one at a time, forwarded commands first, because concurrent scenarios would distort each other.

 @returns one measurement per scenario,
   in the order the scenarios ran
 */
async function* measureScenarios(): AsyncGenerator<ScenarioMeasurement> {
  for (const scenario of forwarded) {
    yield measure({
      name: scenario.name,
      command() {
        return scenario.arguments_;
      },
      kind: 'forwarded',
      samples,
      root,
      directRepository,
      wrappedRepository,
    });
  }
  // A commit is held to the paired-delta bound, not to the forwarded-command bound. Each of the four arms differs from the previous in one
  // thing: the route (the transaction used before the service, or the service) and whether the repository signs.
  for (const arm of [
    {
      name: 'commit-explicit-path',
      direct: directRepository,
      wrapped: wrappedRepository,
      runtime: undefined,
    },
    {
      name: 'commit-service',
      direct: directRepository,
      wrapped: wrappedRepository,
      runtime: service.runtime,
    },
    {
      name: 'commit-signed',
      direct: directSignedRepository,
      wrapped: wrappedSignedRepository,
      runtime: undefined,
    },
    {
      name: 'commit-signed-service',
      direct: directSignedRepository,
      wrapped: wrappedSignedRepository,
      runtime: service.runtime,
    },
  ] as const) {
    yield measure({
      name: arm.name,
      async prepare({
        directory,
        sample,
      }) {
        await writeFile(
          join(
            directory,
            'file-0.txt',
          ),
          `edit ${String(sample)} ${String(Math.random())}\n`,
        );
      },
      command({ sample }) {
        return [
          'commit',
          '--quiet',
          `--message=perf ${String(sample)}`,
          '--',
          'file-0.txt',
        ];
      },
      kind: 'commit',
      samples,
      root,
      directRepository: arm.direct,
      wrappedRepository: arm.wrapped,
      runtime: arm.runtime,
    });
  }
}

try {
  for await (const measurement of measureScenarios()) {
    record(measurement);
  }
}
finally {
  // The service writes into the scratch tree, so it stops before the tree is removed.
  await service.stop();
  // Git detaches maintenance after a commit; cleanup waits for every process that still touches the tree.
  await removeScratchRoot({ root, });
}

/**
 Scenarios whose judged statistic reached its bound: the added p95 of a forwarded command, the paired delta p95 of a
 commit.
 */
const violations = judge(report);
/**
 The JSON document written when `--output` is given.
 */
const document = {
  schemaVersion: 2,
  date: new Date().toISOString(),
  binary: resolve(values.binary),
  samplesPerScenario: samples,
  forwardedBoundMilliseconds: FORWARDED_BOUND_MILLISECONDS,
  commitDeltaBoundMilliseconds: COMMIT_DELTA_BOUND_MILLISECONDS,
  platform: process.platform,
  failures,
  report,
};
if (values.output !== undefined) {
  await writeFile(
    values.output,
    `${JSON.stringify(
      document,
      undefined,
      2,
    )}\n`,
  );
}
for (const entry of report) {
  console.log(reportLine(entry));
}
if (failures.length > 0) {
  console.log(`FAILED samples: ${String(failures.length)}; first: ${JSON.stringify(failures[0])}`);
  process.exitCode = 1;
}
for (const violation of violations) {
  console.log(violationLine(violation));
  process.exitCode = 1;
}
