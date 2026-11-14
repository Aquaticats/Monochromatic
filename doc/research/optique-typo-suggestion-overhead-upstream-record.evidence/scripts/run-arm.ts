// Times one arm on one cell in a fresh process: warmup parses, then timed
// parses, then a validation of the last parse. Prints one JSON record on
// stdout so the container needs no writable mount for results.

import { cpus, loadavg, } from 'node:os';

import type { Cell, } from './workloads.ts';
import { buildHarness, } from './harness.ts';
import { loadPaths, } from './workloads.ts';

//region Flags

/**
 * Options parsed from the command line.
 */
type ArmOptions = {
  readonly arm: string;
  readonly tag: string;
  readonly round: number;
  readonly warmup: number;
  readonly timed: number;
  readonly cell: Cell;
};

/**
 * Parses `--key=value` pairs into a lookup, rejecting unknown keys.
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
    /** Flag value as written. */
    const value = argument.slice(separatorIndex + 1,);

    if (!allowedKeys.includes(key,)) {
      throw new Error(`unknown flag --${key}`,);
    }
    if (collected.has(key,)) {
      throw new Error(`duplicate flag --${key}`,);
    }
    collected.set(
      key,
      value,
    );
  }

  return collected;
}

/**
 * Reads a required string flag.
 *
 * @param flags - Parsed flags.
 *
 * @param key - Flag name.
 *
 * @returns The flag value.
 */
function readString(flags: Map<string, string>, key: string,): string {
  /** Raw flag text. */
  const raw = flags.get(key,);

  if (raw === undefined) {
    throw new Error(`missing required flag --${key}`,);
  }

  return raw;
}

/**
 * Reads a non-negative integer flag.
 *
 * @param flags - Parsed flags.
 *
 * @param key - Flag name.
 *
 * @returns The parsed integer.
 */
function readInteger(flags: Map<string, string>, key: string,): number {
  /** Raw flag text. */
  const raw = readString(
    flags,
    key,
  );
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
 * Reads the measured cell from flags.
 *
 * @param flags - Parsed flags.
 *
 * @returns The cell to measure.
 */
function readCell(flags: Map<string, string>,): Cell {
  return {
    workload: readString(
      flags,
      'workload',
    ) as Cell['workload'],
    pathSet: readString(
      flags,
      'pathset',
    ) as Cell['pathSet'],
    count: readInteger(
      flags,
      'count',
    ),
  };
}

//endregion Flags

//region Entry point

/** Flags this script accepts. */
const ALLOWED_KEYS: readonly string[] = [
  'arm',
  'workload',
  'pathset',
  'count',
  'warmup',
  'timed',
  'round',
  'tag',
];

/**
 * Measures one arm on one cell and prints the run record.
 */
async function main(): Promise<void> {
  /** Parsed command line. */
  const flags = readFlags(
    process.argv.slice(2,),
    ALLOWED_KEYS,
  );

  /** Arm and cell to measure. */
  const options: ArmOptions = {
    arm: readString(
      flags,
      'arm',
    ),
    tag: flags.get('tag',) ?? '',
    round: readInteger(
      flags,
      'round',
    ),
    warmup: readInteger(
      flags,
      'warmup',
    ),
    timed: readInteger(
      flags,
      'timed',
    ),
    cell: readCell(flags,),
  };
  /** Paths this cell supplies. */
  const paths = loadPaths(options.cell.pathSet,)
    .slice(
      0,
      options.cell.count,
    );

  if (paths.length !== options.cell.count) {
    throw new Error(
      `path set ${options.cell.pathSet} is shorter than count ${options.cell.count}`,
    );
  }

  /** Harness for this arm and cell. */
  const harness = await buildHarness(
    options.arm,
    options.cell,
    paths,
  );

  for (let warmupIndex = 0; warmupIndex < options.warmup; warmupIndex += 1) {
    harness.run();
  }

  /** Wall-clock duration of every timed parse, in milliseconds. */
  const samples: number[] = [];
  /** Result of the most recent timed parse. */
  let lastResult: unknown = undefined;
  /** Error thrown by a timed parse, if the arm rejects this argv. */
  let thrown: { name: string; message: string; } | undefined;

  for (let timedIndex = 0; timedIndex < options.timed; timedIndex += 1) {
    /** Monotonic start of this parse. */
    const startedAt = performance.now();

    try {
      lastResult = harness.run();
    }
    catch (error) {
      /** Thrown value. */
      const failure = error as Error;

      thrown = {
        name: failure.name,
        message: failure.message,
      };
      break;
    }

    samples.push(performance.now() - startedAt,);
  }

  /** Validation of the last timed parse, skipped when the arm rejected argv. */
  const inspection = thrown === undefined
    ? harness.inspect(lastResult,)
    : {
      ok: false,
      digest: '',
      handled: 0,
    };

  process.stdout.write(`${
    JSON.stringify({
      kind: 'run',
      arm: options.arm,
      version: options.arm,
      tag: options.tag,
      round: options.round,
      warmup: options.warmup,
      timed: options.timed,
      workload: options.cell.workload,
      pathSet: options.cell.pathSet,
      count: options.cell.count,
      ms: samples,
      ok: inspection.ok,
      digest: inspection.digest,
      handled: inspection.handled,
      thrown,
      node: process.version,
      cpuCount: cpus().length,
      load1: loadavg()[0],
    },)
  }\n`,);
}

await main();

//endregion Entry point
