// Runs the whole measurement on this host: four container invocations (one per
// experiment) followed by the three host-side analyses, all writing into
// results/. The container is run exactly as rule HXR prescribes and mounts
// nothing, so every result leaves on stdout.
//
// Host-side driver; run with: node scripts/measure.ts

import { spawnSync, } from 'node:child_process';
import {
  mkdirSync,
  writeFileSync,
} from 'node:fs';

//region Config

/**
 * Image tag this measurement builds.
 */
const IMAGE = 'localhost/optique-measure:2026-11-12';

/**
 * Directory that receives every raw result file.
 */
const RESULTS = 'results';

/**
 * Container limits per rule HXR. The parse is single-threaded, so two CPUs
 * leave room for V8's helper threads without inviting scheduling noise.
 */
const CONTAINER_LIMITS: readonly string[] = [
  '--rm',
  '--cpus',
  '2',
  '--memory',
  '4g',
  '--pids-limit',
  '512',
  '--blkio-weight',
  '10',
];

/**
 * One container invocation writing one raw result file.
 */
type Experiment = {
  readonly name: string;
  readonly script: string;
  readonly arguments: readonly string[];
};

/**
 * Round settings shared by every experiment: warmup parses, then timed parses,
 * seven rounds so versions alternate while host load drifts.
 */
const ROUNDS = 7;

/**
 * Warmup parses inside each fresh process.
 */
const WARMUP = 2;

/**
 * Timed parses inside each fresh process.
 */
const TIMED = 7;

/**
 * Every experiment, in the order it runs. Behaviour and profiling are cheap
 * and run first; spread runs before the matrix so its bound is known first.
 */
const EXPERIMENTS: readonly Experiment[] = [
  {
    name: 'behavior',
    script: 'scripts/behavior-all.ts',
    arguments: [],
  },
  {
    name: 'profiles',
    script: 'scripts/profile-run.ts',
    arguments: [],
  },
  {
    name: 'spread',
    script: 'scripts/run-all.ts',
    arguments: [
      '--mode=spread',
      `--rounds=${6}`,
      `--reps=${8}`,
      `--warmup=${WARMUP}`,
      `--timed=${TIMED}`,
    ],
  },
  {
    name: 'runs',
    script: 'scripts/run-all.ts',
    arguments: [
      '--mode=matrix',
      `--rounds=${ROUNDS}`,
      `--warmup=${WARMUP}`,
      `--timed=${TIMED}`,
    ],
  },
];

/**
 * Host-side analyses run after the container finishes.
 */
const ANALYSES: readonly string[] = [
  'scripts/summarise.ts',
  'scripts/spread.ts',
  'scripts/compare-behavior.ts',
  'scripts/report-tables.ts',
];

//endregion Config

//region Helpers

/**
 * Runs one command and returns its streams, failing loudly on a non-zero exit.
 *
 * @param command - Executable name.
 *
 * @param argumentsForCommand - Arguments after the executable.
 *
 * @returns Captured stdout and stderr.
 */
function run(command: string, argumentsForCommand: readonly string[],): {
  stdout: string;
  stderr: string;
} {
  /** Child process outcome. */
  const child = spawnSync(
    command,
    argumentsForCommand,
    {
      encoding: 'utf8',
      maxBuffer: 256 * 1024 * 1024,
    },
  );

  if (child.error !== undefined) {
    throw child.error;
  }
  if (child.status !== 0) {
    process.stderr.write(child.stderr ?? '',);
    throw new Error(`command failed: ${command} ${argumentsForCommand.join(' ')}`,);
  }

  return {
    stdout: child.stdout,
    stderr: child.stderr,
  };
}

//endregion Helpers

//region Entry point

/**
 * Runs every experiment, records the raw output, then runs the analyses.
 */
function main(): void {
  mkdirSync(
    RESULTS,
    { recursive: true, },
  );

  /** Wall-clock start of the whole measurement. */
  const startedAt = new Date();

  for (const experiment of EXPERIMENTS) {
    /** Wall-clock start of this experiment. */
    const experimentStart = performance.now();

    process.stderr.write(`[measure] starting ${experiment.name}\n`,);

    /** Container output for this experiment. */
    const output = run(
      'podman',
      [
        'run',
        ...CONTAINER_LIMITS,
        IMAGE,
        experiment.script,
        ...experiment.arguments,
      ],
    );

    writeFileSync(
      `${RESULTS}/${experiment.name}.jsonl`,
      output.stdout,
    );
    writeFileSync(
      `${RESULTS}/${experiment.name}.log`,
      output.stderr,
    );

    process.stderr.write(
      `[measure] ${experiment.name} finished in ${
        ((performance.now() - experimentStart) / 1000).toFixed(1)
      }s, records=${output.stdout.split('\n',).filter(function nonEmpty(line,) {
        return line !== '';
      },).length}\n`,
    );
  }

  for (const analysis of ANALYSES) {
    process.stderr.write(`[measure] analysing with ${analysis}\n`,);

    /** Analysis output. */
    const output = run(
      process.execPath,
      [analysis,],
    );

    writeFileSync(
      `${RESULTS}/${analysis.replace(
        'scripts/',
        '',
      ).replace(
        '.ts',
        '',
      )}.txt`,
      output.stdout,
    );

    if (output.stderr !== '') {
      process.stderr.write(output.stderr,);
    }
  }

  /** Image identity, so a report can name the exact artefact that ran. */
  const imageInspect = run(
    'podman',
    [
      'image',
      'inspect',
      '--format',
      '{{.Id}}',
      IMAGE,
    ],
  );

  writeFileSync(
    `${RESULTS}/run-meta.json`,
    `${JSON.stringify(
      {
        image: IMAGE,
        imageId: imageInspect.stdout.trim(),
        startedAt: startedAt.toISOString(),
        finishedAt: new Date().toISOString(),
        rounds: ROUNDS,
        warmup: WARMUP,
        timed: TIMED,
        containerLimits: CONTAINER_LIMITS,
        experiments: EXPERIMENTS,
      },
      null,
      2,
    )}\n`,
  );

  process.stderr.write('[measure] done\n',);
}

main();

//endregion Entry point
