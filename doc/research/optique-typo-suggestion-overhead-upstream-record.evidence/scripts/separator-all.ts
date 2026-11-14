// Runs the separator shape: a matrix over the three arms the report names, or a
// spread run that repeats unchanged configurations to bound the noise first.
// One fresh process per arm and round.

import { spawnSync, } from 'node:child_process';
import { cpus, loadavg, } from 'node:os';

//region Config

/**
 * Arms measured on the separator shape.
 */
const SEPARATOR_ARMS: readonly string[] = [
  '1.1.1',
  '1.3.6',
  '1.4.0-dev.2664',
  '1.4.0-dev.2971',
];

/**
 * Arms repeated in spread mode.
 */
const SPREAD_ARMS: readonly string[] = [
  '1.3.6',
  '1.4.0-dev.2971',
];

/**
 * Paths after the separator.
 */
const TOKEN_COUNT = 1000;

//endregion Config

//region Helpers

/**
 * Rotates a list by an offset.
 *
 * @param list - Items to rotate.
 *
 * @param offset - How far the first item moves back.
 *
 * @returns Rotated copy.
 */
function rotate<T>(list: readonly T[], offset: number,): readonly T[] {
  /** Normalised offset. */
  const step = ((offset % list.length) + list.length) % list.length;

  return [
    ...list.slice(step,),
    ...list.slice(
      0,
      step,
    ),
  ];
}

/**
 * Runs one separator process and forwards its record.
 *
 * @param arm - Arm label.
 *
 * @param round - Round index.
 *
 * @param tag - Copy label, or empty for the matrix.
 *
 * @param warmup - Warmup parses per fresh process.
 *
 * @param timed - Timed parses per fresh process.
 */
function runOne(arm: string, round: number, tag: string, warmup: number, timed: number,): void {
  /** Child process outcome. */
  const child = spawnSync(
    process.execPath,
    [
      '--no-warnings',
      '/bench/scripts/separator-run.ts',
      `--arm=${arm}`,
      `--tokenCount=${TOKEN_COUNT}`,
      `--warmup=${warmup}`,
      `--timed=${timed}`,
      `--round=${round}`,
      `--tag=${tag}`,
    ],
    {
      encoding: 'utf8',
      maxBuffer: 32 * 1024 * 1024,
    },
  );

  if (child.stderr !== '' && child.stderr !== null) {
    process.stderr.write(child.stderr,);
  }
  if (child.status !== 0) {
    throw new Error(
      `separator-run failed for ${arm} round ${round}: status ${child.status}`,
    );
  }

  process.stdout.write(child.stdout,);
}

//endregion Helpers

//region Entry point

/**
 * Runs the separator matrix or spread experiment.
 */
function main(): void {
  /** All `--key=value` arguments. */
  const flags = new Map<string, string>(
    process.argv
      .slice(2,)
      .filter(function isFlag(argument,) {
        return argument.startsWith('--',);
      },)
      .map(function asPair(argument,) {
        /** Position of the key/value separator. */
        const separatorIndex = argument.indexOf('=',);

        return [
          argument.slice(
            2,
            separatorIndex,
          ),
          argument.slice(separatorIndex + 1,),
        ] as [string, string];
      },),
  );
  /** Mode, `matrix` or `spread`. */
  const mode = flags.get('mode',) ?? 'matrix';
  /** Warmup parses per fresh process. */
  const warmup = Number.parseInt(
    flags.get('warmup',) ?? '2',
    10,
  );
  /** Timed parses per fresh process. */
  const timed = Number.parseInt(
    flags.get('timed',) ?? '5',
    10,
  );
  /** Copies per repeated arm in spread mode. */
  const reps = Number.parseInt(
    flags.get('reps',) ?? '8',
    10,
  );
  /** Rounds to run. */
  const rounds = Number.parseInt(
    flags.get('rounds',) ?? (mode === 'spread' ? '6' : '7'),
    10,
  );

  process.stderr.write(`${
    JSON.stringify({
      kind: 'plan',
      experiment: 'separator',
      mode,
      rounds,
      warmup,
      timed,
      reps,
      arms: mode === 'spread' ? SPREAD_ARMS : SEPARATOR_ARMS,
      tokenCount: TOKEN_COUNT,
      node: process.version,
      cpuCount: cpus().length,
      load1: loadavg(),
      startedAt: new Date().toISOString(),
    },)
  }\n`,);

  if (mode === 'spread') {
    for (let round = 0; round < rounds; round += 1) {
      /** One task per copy of each repeated arm. */
      const tasks = SPREAD_ARMS.flatMap(function copies(arm,) {
        return Array.from(
          { length: reps, },
          function copy(_value, rep,) {
            return {
              arm,
              tag: `rep${rep}`,
            };
          },
        );
      },);

      for (const task of rotate(
        tasks,
        round * 5,
      )) {
        runOne(
          task.arm,
          round,
          task.tag,
          warmup,
          timed,
        );
      }

      process.stderr.write(`[separator] spread round ${round} done\n`,);
    }

    return;
  }

  for (let round = 0; round < rounds; round += 1) {
    for (const arm of rotate(
      SEPARATOR_ARMS,
      round,
    )) {
      runOne(
        arm,
        round,
        '',
        warmup,
        timed,
      );
    }

    process.stderr.write(`[separator] round ${round} done\n`,);
  }
}

main();

//endregion Entry point
