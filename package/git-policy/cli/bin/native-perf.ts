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
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import { homedir } from 'node:os';
import { parseArgs } from 'node:util';

import { enableSshSigning } from './native-perf-git.ts';
import { makeRoot } from './native-perf-root.ts';
import {
  makeFixture,
  sampleEditsOf,
} from './native-perf-shape.ts';
import {
  assertServiceAnswers,
  makeService,
  makeSigningKey,
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
import { collectEvidence } from './native-perf-evidence.ts';
import { assertPerfVerdict } from './native-perf-verdict.ts';
import { removeScratchRoot } from './native-perf-scratch.ts';
import { forwarded } from './native-perf-scenarios.ts';
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
    // A prepared repository the fixtures are reflink copies of, so the commits run at the shape of the person's own main
    // worktree (tens of thousands of tracked files, a multi-megabyte index) instead of thirty files.
    shape: { type: 'string' },
    // Git directory whose policy hooks the shaped fixtures get.
    'hooks-from': { type: 'string' },
    // Executables the hooks need on PATH (for example `git-lfs`), linked into the scratch `tools` directory.
    tool: {
      type: 'string',
      multiple: true,
    },
    // Comma-separated scenario names; absent measures every scenario.
    only: { type: 'string' },
    // Directory that receives an `strace` record of every sample and of each commit service.
    'trace-directory': { type: 'string' },
    // Drop each repository's pack, index and packed-refs pages from the page cache before every sample, on both sides.
    cold: { type: 'boolean' },
    // How many tracked text files every commit sample edits and commits by name (the size of the commit), default one.
    files: { type: 'string', },
    'new-files': { type: 'string', },
    'change-rules': { type: 'boolean', },
    // Milliseconds to wait after each round, so the work a commit leaves running after its answer is over before the next one.
    pause: { type: 'string' },
  },
});
/**
 Names of the scenarios to measure, or absent for all of them.
 */
const only = values.only === undefined
  ? undefined
  : new Set(values.only
    .split(','));
/**
 Whether a scenario is measured in this run.

 @param name - scenario name

 @returns true when no `--only` was given or it names the scenario
 */
function wanted(name: string): boolean {
  return (only === undefined) || only.has(name);
}
/**
 Directory that receives the traces, made before the first sample.
 */
const traceDirectory = values['trace-directory'] === undefined
  ? undefined
  : resolve(values['trace-directory']);
if (traceDirectory !== undefined) {
  await mkdir(
    traceDirectory,
    { recursive: true, },
  );
}
if (values.binary === undefined) {
  throw new Error('--binary <path to the release monochromatic-git-policy-cli> is required');
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
 Scratch root holding the wrapper `bin`, the empty `home`, the linked `tools` and the fixture repositories.
 */
const root = await makeRoot({
  scratchParent,
  binary: resolve(values.binary),
  tools: values.tool ?? [],
});

/**
 Makes one fixture repository of this run, at the shape the run was asked for.

 @param name - directory name inside the scratch root

 @returns the repository directory
 */
function fixture(name: string): Promise<string> {
  return makeFixture({
    root,
    name,
    ...(values.shape === undefined ? {} : { shape: resolve(values.shape), }),
    ...(values['hooks-from'] === undefined ? {} : { hooksFrom: resolve(values['hooks-from']), }),
  });
}

/**
 Fixture repository that the direct samples run in.
 */
const directRepository = await fixture('direct-repo');
/**
 Fixture repository that the wrapped samples run in.
 */
const wrappedRepository = await fixture('wrapped-repo');

/**
 The tracked files every commit sample edits: the one file of the default run, or the first `--files` tracked text files.
 */
const edits = await sampleEditsOf({
  ...(values.files === undefined ? {} : { files: values.files, }),
  ...(values['new-files'] === undefined ? {} : { added: values['new-files'], }),
  ...(values['change-rules'] === true ? { changeRules: true, } : {}),
  root,
  shaped: values.shape !== undefined,
  directory: directRepository,
});

/**
 Throwaway SSH signing key of this run, made in the scratch root; never the person's key.
 */
const signing = await makeSigningKey({ root, });
/**
 Fixture repository that the direct signed samples run in; the signed repositories sign their commits over SSH, as the
 person's commits are, so the signed scenarios measure what the person runs.
 */
const directSignedRepository = await fixture('direct-signed-repo');
/**
 Fixture repository that the wrapped signed samples run in.
 */
const wrappedSignedRepository = await fixture('wrapped-signed-repo');
// The two repositories are independent setup, so signing is enabled in both at once; no sample is timed yet.
await Promise.all([
  directSignedRepository,
  wrappedSignedRepository,
].map(function enableSigning(directory) {
  return enableSshSigning({
    root,
    directory,
    ...signing,
  });
}));
/**
 The commit services this run starts for the commit scenarios, in a runtime directory of its own. The wrapped samples of
 the forwarded scenarios get no runtime directory, and nothing in the person's own runtime directory or session is read or
 changed.
 */
const service = await makeService({
  program: resolve(values.binary),
  root,
  scratchParent,
  ...(traceDirectory === undefined ? {} : { traceDirectory, }),
});

/**
 What the run must clean up, whatever happens: the services write into the scratch tree, so they stop before the tree is
 removed; Git detaches maintenance after a commit, so removal waits for every process that still touches the tree.
 */
const scratch: AsyncDisposable = {
  async [Symbol.asyncDispose](): Promise<void> {
    await service.stop();
    await removeScratchRoot({ root, });
  },
};

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
  for (const scenario of forwarded.filter(function isWanted(entry) {
    return wanted(entry.name);
  })) {
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
      ...(traceDirectory === undefined ? {} : { traceDirectory, }),
      ...(values.cold === true ? { cold: true, } : {}),
      ...(values.pause === undefined ? {} : { pauseMilliseconds: Number(values.pause), }),
    });
  }
  // A commit is held to the bound on its slowest paired delta, not to the forwarded-command bound. Every commit lands through
  // the service, so the two arms differ in one thing: whether the repository signs.
  /**
   The commit scenarios and the repositories each runs in, as the run was asked to measure them.
   */
  const arms: readonly {
    readonly name: string;
    readonly direct: string;
    readonly wrapped: string;
  }[] = [
    {
      name: 'commit-service',
      direct: directRepository,
      wrapped: wrappedRepository,
    },
    {
      name: 'commit-signed-service',
      direct: directSignedRepository,
      wrapped: wrappedSignedRepository,
    },
  ];
  for (const wantedArm of arms.filter(function isWanted({ name, }) {
    return wanted(name);
  })) {
    yield measure({
      name: wantedArm.name,
      prepare: edits.prepare,
      command({ sample }) {
        return [
          'commit',
          '--quiet',
          `--message=perf ${String(sample)}`,
          '--',
          ...edits.pathsOf(sample),
        ];
      },
      kind: 'commit',
      samples,
      root,
      directRepository: wantedArm.direct,
      wrappedRepository: wantedArm.wrapped,
      runtime: service.runtime,
      ...(traceDirectory === undefined ? {} : { traceDirectory, }),
      ...(values.cold === true ? { cold: true, } : {}),
      ...(values.pause === undefined ? {} : { pauseMilliseconds: Number(values.pause), }),
    });
  }
}

/**
 Serve, check and measure every wanted scenario, and read the evidence before the scratch tree goes.

 @returns the commit timing records and the service's landing lines of the run

 @example
 ```ts
 const evidence = await measureAll();
 ```
 */
async function measureAll(): Promise<Awaited<ReturnType<typeof collectEvidence>>> {
  /**
   Held for the length of the measurement, so the cleanup runs when it ends or throws.
   */
  await using held = scratch;
  // One service per worktree: each wrapped repository a commit scenario lands in gets its own, and a wrapped command that
  // cannot reach it fails here, once, instead of failing every commit sample.
  /**
   The wrapped repository of each commit scenario, whose service is started when the scenario is measured.
   */
  const served: readonly {
    readonly name: string;
    readonly directory: string;
  }[] = [
    {
      name: 'commit-service',
      directory: wrappedRepository,
    },
    {
      name: 'commit-signed-service',
      directory: wrappedSignedRepository,
    },
  ];
  await Promise.all(served
    .filter(function isWanted({ name, }) {
      return wanted(name);
    })
    .map(async function serveAndCheck({ directory, }) {
      await service.serve(join(
        directory,
        '.git',
      ));
      await assertServiceAnswers({
        root,
        runtime: service.runtime,
        directory,
      });
    }));
  for await (const measurement of measureScenarios()) {
    record(measurement);
  }
  // The phases of each wrapped commit and the commit service's own lines are the evidence that says which phase a slow
  // sample spent its time in, and they live in the scratch tree that is removed when this function ends.
  return await collectEvidence({
    root,
    repositories: {
      'commit-service': wrappedRepository,
      'commit-signed-service': wrappedSignedRepository,
    },
  });
}

/**
 What the run kept beyond its timings.
 */
const evidence = await measureAll();

/**
 Scenarios whose judged statistic reached its bound: the added p95 of a forwarded command, the maximum paired delta of a
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
  evidence,
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
for (const violation of violations) {
  console.log(violationLine(violation));
}
// The printed lines are the evidence; the throw is the status, so a sample this run could not measure or a figure past
// its bound reaches the error channel and exits nonzero from there.
assertPerfVerdict({
  failures,
  violations,
},);
