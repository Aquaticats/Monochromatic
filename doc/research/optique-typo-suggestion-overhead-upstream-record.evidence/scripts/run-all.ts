// Runs the whole measurement: every arm on every cell, one fresh process per
// arm-and-cell, ordered in rounds so versions alternate instead of running
// back to back. Prints every run record on stdout, progress on stderr.
//
// Modes:
//   matrix  - arms x cells, rounds rotated so drift hits every arm equally
//   spread  - one unchanged configuration repeated under different labels, to
//             measure the run-to-run spread before any version comparison

import { spawnSync, } from 'node:child_process';
import { cpus, loadavg, } from 'node:os';

import type { Cell, } from './workloads.ts';
import {
  cellIsSupportedByArm,
  CELLS,
  MEASURED_ARMS,
} from './workloads.ts';

//region Config

/**
 * One fixed configuration repeated in spread mode.
 */
type SpreadConfig = {
  readonly arm: string;
  readonly cell: Cell;
};

/**
 * Configurations whose run-to-run spread bounds every version difference: the
 * slow-but-not-largest parse, the fastest dev parse, and a tiny parse whose
 * absolute differences are sub-millisecond.
 */
const SPREAD_CONFIGS: readonly SpreadConfig[] = [
  {
    arm: '1.3.6',
    cell: {
      workload: 'no-separator',
      pathSet: 'real',
      count: 1000,
    },
  },
  {
    arm: '1.4.0-dev.2971',
    cell: {
      workload: 'no-separator',
      pathSet: 'real',
      count: 1000,
    },
  },
  {
    arm: '1.3.6',
    cell: {
      workload: 'no-separator',
      pathSet: 'real',
      count: 1,
    },
  },
  {
    arm: '1.3.6',
    cell: {
      workload: 'no-separator',
      pathSet: 'real',
      count: 10,
    },
  },
  {
    arm: '1.3.6',
    cell: {
      workload: 'no-separator',
      pathSet: 'real',
      count: 100,
    },
  },
  {
    arm: '1.3.6',
    cell: {
      workload: 'no-separator',
      pathSet: 'short',
      count: 1000,
    },
  },
  {
    arm: '1.3.6',
    cell: {
      workload: 'issue988',
      pathSet: 'issue988',
      count: 1000,
    },
  },
  {
    arm: 'owned-4879a44e',
    cell: {
      workload: 'no-separator',
      pathSet: 'real',
      count: 1000,
    },
  },
];

//endregion Config

//region Helpers

/**
 * Rotates a list by an offset, wrapping around.
 *
 * @param list - Items to rotate.
 *
 * @param offset - How many positions the first item moves back.
 *
 * @returns Rotated copy.
 */
function rotate<T>(list: readonly T[], offset: number,): readonly T[] {
  /** Normalised offset inside the list. */
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
 * Parses `--key=value` pairs.
 *
 * @param argv - Raw arguments after the script path.
 *
 * @param allowedKeys - Keys this script accepts.
 *
 * @returns Value per accepted key.
 */
function readFlags(argv: readonly string[], allowedKeys: readonly string[],): Map<string, string> {
  /** Collected key/value pairs. */
  const collected = new Map<string, string>();

  for (const argument of argv) {
    if (!argument.startsWith('--',)) {
      throw new Error(`unexpected positional argument: ${argument}`,);
    }
    /** Position of the key/value separator. */
    const separatorIndex = argument.indexOf('=',);

    if (separatorIndex === -1) {
      throw new Error(`expected --key=value, got ${argument}`,);
    }

    /** Flag name without the leading dashes. */
    const key = argument.slice(
      2,
      separatorIndex,
    );

    if (!allowedKeys.includes(key,)) {
      throw new Error(`unknown flag --${key}`,);
    }
    collected.set(
      key,
      argument.slice(separatorIndex + 1,),
    );
  }

  return collected;
}

/**
 * Reads a non-negative integer flag with a default.
 *
 * @param flags - Parsed flags.
 *
 * @param key - Flag name.
 *
 * @param fallback - Value used when the flag is absent.
 *
 * @returns The parsed integer.
 */
function readInteger(flags: Map<string, string>, key: string, fallback: number,): number {
  /** Raw flag text. */
  const raw = flags.get(key,);

  if (raw === undefined) {
    return fallback;
  }

  /** Parsed integer. */
  const parsed = Number.parseInt(
    raw,
    10,
  );

  if (!Number.isSafeInteger(parsed,) || parsed < 0) {
    throw new Error(`--${key} is not a non-negative integer: ${raw}`,);
  }

  return parsed;
}

/**
 * Describes a cell as a single key, used for filtering.
 *
 * @param cell - Cell to describe.
 *
 * @returns `workload:pathSet:count`.
 */
function cellKey(cell: Cell,): string {
  return `${cell.workload}:${cell.pathSet}:${cell.count}`;
}

/**
 * Runs one arm on one cell in a fresh process and returns its run record.
 *
 * @param arm - Arm label.
 *
 * @param cell - Cell to measure.
 *
 * @param round - Round index this run belongs to.
 *
 * @param tag - Optional label distinguishing repeated copies of one arm.
 *
 * @param warmup - Warmup parses inside the fresh process.
 *
 * @param timed - Timed parses inside the fresh process.
 *
 * @returns The parsed run record.
 */
function runOne(
  arm: string,
  cell: Cell,
  round: number,
  tag: string,
  warmup: number,
  timed: number,
): {
  readonly ms: readonly number[];
  readonly ok: boolean;
} {
  /** Arguments for the arm runner. */
  const argumentsForArm = [
    '--no-warnings',
    '/bench/scripts/run-arm.ts',
    `--arm=${arm}`,
    `--workload=${cell.workload}`,
    `--pathset=${cell.pathSet}`,
    `--count=${cell.count}`,
    `--warmup=${warmup}`,
    `--timed=${timed}`,
    `--round=${round}`,
    `--tag=${tag}`,
  ];
  /** Child process outcome. */
  const child = spawnSync(
    process.execPath,
    argumentsForArm,
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
      `run-arm failed for ${arm} ${cellKey(cell,)} round ${round}: status ${child.status}`,
    );
  }

  /** Run records printed by the child. */
  const records = child.stdout
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },)
    .map(function parse(line,) {
      return JSON.parse(line,) as {
        readonly ms: readonly number[];
        readonly ok: boolean;
      };
    },);

  if (records.length !== 1) {
    throw new Error(`expected one record, got ${records.length}`,);
  }

  process.stdout.write(child.stdout,);

  /** Single record for this run. */
  const [record] = records;

  if (record === undefined) {
    throw new Error('missing run record',);
  }

  return record;
}

/**
 * Averages nothing but reports a median, used only for progress output.
 *
 * @param samples - Durations in milliseconds.
 *
 * @returns Middle duration.
 */
function medianOf(samples: readonly number[],): number {
  /** Sorted copy of the samples. */
  const sorted = [...samples].sort(function ascending(left, right,) {
    return left - right;
  },);
  /** Index of the middle sample. */
  const middleIndex = Math.floor(sorted.length / 2,);

  return sorted[middleIndex] ?? Number.NaN;
}

//endregion Helpers

//region Entry point

/** Flags this script accepts. */
const ALLOWED_KEYS: readonly string[] = [
  'mode',
  'rounds',
  'warmup',
  'timed',
  'reps',
  'cells',
];

/**
 * Runs the measurement and prints every record.
 */
function main(): void {
  /** Parsed command line. */
  const flags = readFlags(
    process.argv.slice(2,),
    ALLOWED_KEYS,
  );
  /** Measurement mode. */
  const mode = flags.get('mode',) ?? 'matrix';
  /** Rounds to run. */
  const rounds = readInteger(
    flags,
    'rounds',
    7,
  );
  /** Warmup parses per fresh process. */
  const warmup = readInteger(
    flags,
    'warmup',
    3,
  );
  /** Timed parses per fresh process. */
  const timed = readInteger(
    flags,
    'timed',
    9,
  );
  /** Copies per configuration in spread mode. */
  const reps = readInteger(
    flags,
    'reps',
    8,
  );
  /** Optional cell filter, comma separated. */
  const cellFilter = flags.get('cells',);

  /** Cells selected for this run. */
  const selectedCells = cellFilter === undefined
    ? CELLS
    : CELLS.filter(function selected(cell,) {
      return cellFilter
        .split(',',)
        .includes(cellKey(cell,),);
    },);

  if (selectedCells.length === 0) {
    throw new Error(`cell filter matched nothing: ${cellFilter ?? ''}`,);
  }

  process.stderr.write(`${
    JSON.stringify({
      kind: 'plan',
      mode,
      rounds,
      warmup,
      timed,
      reps,
      arms: MEASURED_ARMS,
      cells: selectedCells.map(cellKey,),
      node: process.version,
      cpuCount: cpus().length,
      load1: loadavg(),
      startedAt: new Date().toISOString(),
    },)
  }\n`,);

  /** Wall-clock start of the whole run. */
  const startedAt = performance.now();
  /** How many arm-and-cell executions have finished. */
  let completed = 0;
  /** Every task of the spread experiment, one per copy of a configuration. */
  const spreadTasks = SPREAD_CONFIGS.flatMap(function copies(config,) {
    return Array.from(
      { length: reps, },
      function copy(_value, rep,) {
        return {
          arm: config.arm,
          cell: config.cell,
          tag: `rep${rep}`,
        };
      },
    );
  },);
  /** Total executions this plan will perform. */
  const total = mode === 'spread'
    ? rounds * spreadTasks.length
    : rounds * selectedCells.reduce(function countSupported(totalPerRound, cell,) {
      return totalPerRound + MEASURED_ARMS.filter(function supported(arm,) {
        return cellIsSupportedByArm(
          arm,
          cell,
        );
      },).length;
    }, 0,);

  /** Runs one task and reports progress. */
  function runTask(
    task: {
      readonly arm: string;
      readonly cell: Cell;
      readonly tag: string;
    },
    round: number,
  ): void {
    /** One fresh-process run. */
    const record = runOne(
      task.arm,
      task.cell,
      round,
      task.tag,
      warmup,
      timed,
    );

    completed += 1;
    process.stderr.write(
      `[run-all] ${completed}/${total} round=${round} arm=${task.arm}${
        task.tag === '' ? '' : `/${task.tag}`
      } cell=${cellKey(task.cell,)} median=${
        medianOf(record.ms,).toFixed(3)
      }ms ok=${record.ok}\n`,
    );
  }

  if (mode === 'spread') {
    for (let round = 0; round < rounds; round += 1) {
      for (const task of rotate(
        spreadTasks,
        round * 5,
      )) {
        runTask(
          task,
          round,
        );
      }

      process.stderr.write(
        `[run-all] round ${round} done in ${
          ((performance.now() - startedAt) / 1000).toFixed(1)
        }s\n`,
      );
    }
  } else {
    for (let round = 0; round < rounds; round += 1) {
      /** Cells in an order that rotates per round. */
      const orderedCells = rotate(
        selectedCells,
        round,
      );
      /** Arms in an order that rotates per round, coprime with arm count. */
      const orderedArms = rotate(
        MEASURED_ARMS,
        round * 3,
      );

      for (const cell of orderedCells) {
        for (const arm of orderedArms) {
          if (!cellIsSupportedByArm(
            arm,
            cell,
          )) {
            continue;
          }

          runTask(
            {
              arm,
              cell,
              tag: '',
            },
            round,
          );
        }
      }

      process.stderr.write(
        `[run-all] round ${round} done in ${
          ((performance.now() - startedAt) / 1000).toFixed(1)
        }s\n`,
      );
    }
  }

  process.stderr.write(
    `[run-all] finished ${completed} executions in ${
      ((performance.now() - startedAt) / 1000).toFixed(1)
    }s\n`,
  );
}

main();

//endregion Entry point
