// Runs Parts 1 to 3 on this host: the scaling sweeps, the separator matrix and
// spread, the Part 3 profiles, and the 0.7.0 search-cost probe, each in a
// fresh mount-free container, then the host-side fits and summaries.
//
// Run with: node scripts/measure-scaling.ts

import { spawnSync, } from 'node:child_process';
import {
  mkdirSync,
  writeFileSync,
} from 'node:fs';

//region Config

/**
 * Image this measurement uses.
 */
const IMAGE = 'localhost/optique-measure:2026-11-12';

/**
 * Result folder for this round of experiments.
 */
const RESULTS = 'results-scaling';

/**
 * Container limits per rule HXR.
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
  /** How many fresh processes this experiment runs. Defaults to one. */
  readonly repeats?: number;
};

/**
 * Every container experiment, in order.
 */
const EXPERIMENTS: readonly Experiment[] = [
  {
    name: 'scaling',
    script: 'scripts/scaling-all.ts',
    arguments: [],
  },
  {
    name: 'separator-spread',
    script: 'scripts/separator-all.ts',
    arguments: [
      '--mode=spread',
      '--rounds=6',
      '--reps=8',
      '--warmup=2',
      '--timed=5',
    ],
  },
  {
    name: 'separator',
    script: 'scripts/separator-all.ts',
    arguments: [
      '--mode=matrix',
      '--rounds=7',
      '--warmup=2',
      '--timed=5',
    ],
  },
  {
    name: 'profiles',
    script: 'scripts/profile-run2.ts',
    arguments: [],
  },
  {
    name: 'search',
    script: 'arms/0.7.0/bench/search-cost.ts',
    arguments: [],
    repeats: 3,
  },
  {
    name: 'token-probe',
    script: 'scripts/scaling-token-probe.ts',
    arguments: [],
  },
  {
    name: 'profile-sizes',
    script: 'scripts/profile-separator-sizes.ts',
    arguments: [],
  },
];

/**
 * Host-side analyses, in dependency order.
 */
const ANALYSES: readonly string[] = [
  'scripts/fit-scaling.ts',
  'scripts/summarise-separator.ts',
  'scripts/summarise-search.ts',
  'scripts/summarise-probes.ts',
];

//endregion Config

//region Helpers

/**
 * Runs one command and captures its streams.
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
 * Runs every experiment, writes the raw output, then runs the analyses.
 */
function main(): void {
  mkdirSync(
    RESULTS,
    { recursive: true, },
  );

  /** Wall-clock start. */
  const startedAt = new Date();

  for (const experiment of EXPERIMENTS) {
    /** Wall-clock start of this experiment. */
    const experimentStart = performance.now();
    /** How many times to repeat this experiment as fresh processes. */
    const repeats = experiment.repeats ?? 1;

    process.stderr.write(`[measure-scaling] starting ${experiment.name}\n`,);

    /** Combined stdout of every repeat. */
    let stdout = '';
    /** Combined stderr of every repeat. */
    let stderr = '';

    for (let repeat = 0; repeat < repeats; repeat += 1) {
      /** One container run. */
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

      stdout += output.stdout;
      stderr += output.stderr;
    }

    writeFileSync(
      `${RESULTS}/${experiment.name}.jsonl`,
      stdout,
    );
    writeFileSync(
      `${RESULTS}/${experiment.name}.log`,
      stderr,
    );

    process.stderr.write(
      `[measure-scaling] ${experiment.name} finished in ${
        ((performance.now() - experimentStart) / 1000).toFixed(1)
      }s, records=${stdout.split('\n',).filter(function nonEmpty(line,) {
        return line !== '';
      },).length}\n`,
    );
  }

  for (const analysis of ANALYSES) {
    process.stderr.write(`[measure-scaling] analysing with ${analysis}\n`,);

    /** Analysis output. */
    const output = run(
      process.execPath,
      [analysis,],
    );

    writeFileSync(
      `${RESULTS}/${
        analysis.replace(
          'scripts/',
          '',
        ).replace(
          '.ts',
          '',
        )
      }.txt`,
      output.stdout,
    );

    if (output.stderr !== '') {
      process.stderr.write(output.stderr,);
    }
  }

  /** Image identity. */
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
        containerLimits: CONTAINER_LIMITS,
        experiments: EXPERIMENTS,
      },
      null,
      2,
    )}\n`,
  );

  process.stderr.write('[measure-scaling] done\n',);
}

main();

//endregion Entry point
