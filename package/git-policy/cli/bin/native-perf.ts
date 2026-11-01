#!/usr/bin/env node
/**
 What:
  Measures the latency the release native wrapper adds to forwarded Git commands and to explicit-path commits.
 Why:
  `SPEC.md` ("Benchmark method") requires a paired direct-Git baseline for every scenario, the median, p95 and median
  absolute deviation of each, the wrapper-added delta, and the raw samples; the added latency of a forwarded
  command must stay strictly under two seconds.

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
  rm,
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

import { repository } from './native-perf-git.ts';
import {
  measure,
  type SampleFailure,
  type ScenarioMeasurement,
  type ScenarioReport,
} from './native-perf-measure.ts';

/**
 The strict upper bound for wrapper-added latency of a forwarded command, in milliseconds.
 */
const BUDGET_MILLISECONDS = 2_000;

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
 Width of the scenario column in the printed lines, so the figures align.
 */
const SCENARIO_COLUMN_WIDTH = 22;

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
      enforced: true,
      samples,
      root,
      directRepository,
      wrappedRepository,
    });
  }
  // An explicit-path commit runs the whole transaction, so it is reported but not held to the forwarded-command budget.
  yield measure({
    name: 'commit-explicit-path',
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
    enforced: false,
    samples,
    root,
    directRepository,
    wrappedRepository,
  });
}

for await (const measurement of measureScenarios()) {
  record(measurement);
}

await rm(
  root,
  {
    recursive: true,
    force: true,
  },
);

/**
 Enforced scenarios whose added p95 reached the budget.
 */
const overBudget = report.filter(function overBudgetEntry(entry) {
  return entry.enforced && (entry.addedP95Milliseconds >= BUDGET_MILLISECONDS);
});
/**
 The JSON document written when `--output` is given.
 */
const document = {
  schemaVersion: 1,
  date: new Date().toISOString(),
  binary: resolve(values.binary),
  samplesPerScenario: samples,
  budgetMilliseconds: BUDGET_MILLISECONDS,
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
  /**
   One printed line of figures for a scenario.
   */
  const line = [
    entry.scenario
      .padEnd(SCENARIO_COLUMN_WIDTH),
    `direct median ${entry.direct
      .median
      .toFixed(1)} ms`,
    `wrapped median ${entry.wrapped
      .median
      .toFixed(1)} ms`,
    `added median ${entry.addedMedianMilliseconds
      .toFixed(1)} ms`,
    `added p95 ${entry.addedP95Milliseconds
      .toFixed(1)} ms`,
  ].join('  ');
  console.log(line);
}
if (failures.length > 0) {
  console.log(`FAILED samples: ${String(failures.length)}; first: ${JSON.stringify(failures[0])}`);
  process.exitCode = 1;
}
if (overBudget.length > 0) {
  /**
   Names of the scenarios that exceeded the budget.
   */
  const overBudgetNames = overBudget.map(function scenarioName(entry) {
    return entry.scenario;
  });
  console.log(`OVER BUDGET (added p95 >= ${String(BUDGET_MILLISECONDS)} ms): ${overBudgetNames.join(', ')}`);
  process.exitCode = 1;
}
