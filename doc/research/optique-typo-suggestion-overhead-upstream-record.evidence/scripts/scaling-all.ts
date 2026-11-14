// Runs the scaling experiment: three one-factor sweeps, four arms, alternating
// rounds, one fresh process per arm and point. Probe counts are deterministic,
// so they are collected once per arm and point, in the first round only.

import { spawnSync, } from 'node:child_process';
import { cpus, loadavg, } from 'node:os';

import { SCALING_ARMS, } from './workloads.ts';
import {
  pointAt,
  SWEEPS,
} from './scaling-grid.ts';

//region Config

/**
 * Rounds to run. Enough for a per-round slope and a spread estimate without
 * making the slow points dominate.
 */
const ROUNDS = 5;

/**
 * Warmup parses per fresh process.
 */
const WARMUP = 2;

/**
 * Timed parses per fresh process.
 */
const TIMED = 5;

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
 * Runs one scaling process and forwards its record.
 *
 * @param arm - Arm label.
 *
 * @param sweep - Sweep name.
 *
 * @param level - Varied factor's value.
 *
 * @param shape - Point to measure.
 *
 * @param round - Round index.
 *
 * @param mode - `timed` or `count`.
 */
function runOne(
  arm: string,
  sweep: string,
  level: number,
  shape: {
    readonly optionCount: number;
    readonly tokenLength: number;
    readonly tokenCount: number;
  },
  round: number,
  mode: 'timed' | 'count',
): void {
  /** Child process outcome. */
  const child = spawnSync(
    process.execPath,
    [
      '--no-warnings',
      '/bench/scripts/scaling-run.ts',
      `--arm=${arm}`,
      `--sweep=${sweep}`,
      `--level=${level}`,
      `--optionCount=${shape.optionCount}`,
      `--tokenLength=${shape.tokenLength}`,
      `--tokenCount=${shape.tokenCount}`,
      `--warmup=${WARMUP}`,
      `--timed=${TIMED}`,
      `--round=${round}`,
      `--mode=${mode}`,
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
      `scaling-run failed for ${arm} ${sweep}=${level} (${mode}): status ${child.status}`,
    );
  }

  process.stdout.write(child.stdout,);
}

//endregion Helpers

//region Entry point

/**
 * Runs every sweep point for every arm over the configured rounds.
 */
function main(): void {
  /** Optional sweep filter, comma separated. */
  const sweepFilter = process.argv
    .slice(2,)
    .find(function isSweeps(argument,) {
      return argument.startsWith('--sweeps=',);
    },)?.slice('--sweeps='.length,);
  /** Sweeps selected for this run. */
  const selectedSweeps = sweepFilter === undefined
    ? SWEEPS
    : SWEEPS.filter(function selected(sweep,) {
      return sweepFilter
        .split(',',)
        .includes(sweep.name,);
    },);

  process.stderr.write(`${
    JSON.stringify({
      kind: 'plan',
      experiment: 'scaling',
      rounds: ROUNDS,
      warmup: WARMUP,
      timed: TIMED,
      arms: SCALING_ARMS,
      sweeps: selectedSweeps.map(function summary(sweep,) {
        return {
          name: sweep.name,
          levels: sweep.levels,
          fixedOptionCount: sweep.fixedOptionCount,
          fixedTokenLength: sweep.fixedTokenLength,
          fixedTokenCount: sweep.fixedTokenCount,
        };
      },),
      node: process.version,
      cpuCount: cpus().length,
      load1: loadavg(),
      startedAt: new Date().toISOString(),
    },)
  }\n`,);

  /** Wall-clock start. */
  const startedAt = performance.now();

  for (let round = 0; round < ROUNDS; round += 1) {
    for (const sweep of selectedSweeps) {
      for (const level of sweep.levels) {
        /** Point at this level. */
        const shape = pointAt(
          sweep,
          level,
        );

        for (const arm of rotate(
          SCALING_ARMS,
          round * 3,
        )) {
          runOne(
            arm,
            sweep.name,
            level,
            shape,
            round,
            'timed',
          );

          if (round === 0) {
            runOne(
              arm,
              sweep.name,
              level,
              shape,
              round,
              'count',
            );
          }
        }
      }
    }

    process.stderr.write(
      `[scaling] round ${round} done in ${
        ((performance.now() - startedAt) / 1000).toFixed(1)
      }s\n`,
    );
  }
}

main();

//endregion Entry point
