// Measures one point of the scaling experiment in a fresh process: warmup
// parses then timed parses on the default error path, or a single parse with a
// counting mismatch callback. Prints one JSON record on stdout.

import { cpus, loadavg, } from 'node:os';
import { pathToFileURL, } from 'node:url';

import type { ScalingOutcome, } from './scaling-parser.ts';

//region Flags

/**
 * Parses `--key=value` pairs.
 *
 * @param argv - Raw arguments after the script path.
 *
 * @returns Value per accepted key.
 */
function readFlags(argv: readonly string[],): Map<string, string> {
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
    collected.set(
      argument.slice(
        2,
        separatorIndex,
      ),
      argument.slice(separatorIndex + 1,),
    );
  }

  return collected;
}

/**
 * Reads a required integer flag.
 *
 * @param flags - Parsed flags.
 *
 * @param key - Flag name.
 *
 * @returns The parsed integer.
 */
function readInteger(flags: Map<string, string>, key: string,): number {
  /** Raw flag text. */
  const raw = flags.get(key,);

  if (raw === undefined) {
    throw new Error(`missing required flag --${key}`,);
  }

  /** Parsed integer. */
  const parsed = Number.parseInt(
    raw,
    10,
  );

  if (!Number.isSafeInteger(parsed,)) {
    throw new Error(`--${key} is not an integer: ${raw}`,);
  }

  return parsed;
}

//endregion Flags

//region Entry point

/**
 * Runs one point and prints its record.
 */
async function main(): Promise<void> {
  /** Parsed command line. */
  const flags = readFlags(process.argv.slice(2,),);
  /** Arm label. */
  const arm = flags.get('arm',);
  /** Measurement mode: `timed` for the default error path, `count` for probes. */
  const mode = flags.get('mode',) ?? 'timed';

  if (arm === undefined) {
    throw new Error('missing required flag --arm',);
  }

  /** Point of the sweep. */
  const shape = {
    optionCount: readInteger(
      flags,
      'optionCount',
    ),
    tokenLength: readInteger(
      flags,
      'tokenLength',
    ),
    tokenCount: readInteger(
      flags,
      'tokenCount',
    ),
  };
  /** Arm-local scaling module. */
  const module = await import(
    pathToFileURL(`/bench/arms/${arm}/bench/scaling-parser.ts`,).href,
  ) as {
    createScalingRunner: (
      shapeIn: typeof shape,
      countProbes: boolean,
    ) => () => ScalingOutcome;
  };
  /** Warmup parses. */
  const warmup = readInteger(
    flags,
    'warmup',
  );
  /** Timed parses. */
  const timed = readInteger(
    flags,
    'timed',
  );
  /** Round index. */
  const round = readInteger(
    flags,
    'round',
  );
  /** Sweep name, carried through for the host-side fits. */
  const sweep = flags.get('sweep',) ?? '';
  /** Level of the varied factor, carried through for the fits. */
  const level = readInteger(
    flags,
    'level',
  );

  if (mode === 'count') {
    /** Runner that counts every mismatch probe. */
    const countingRunner = module.createScalingRunner(
      shape,
      true,
    );
    /** Single counting parse. */
    const outcome = countingRunner();

    process.stdout.write(`${
      JSON.stringify({
        kind: 'count',
        arm,
        sweep,
        level,
        shape,
        round,
        probes: outcome.probes,
        tokens: outcome.tokens,
        ok: outcome.success,
        node: process.version,
      },)
    }\n`,);

    return;
  }

  /** Runner on the default error path. */
  const runner = module.createScalingRunner(
    shape,
    false,
  );

  for (let warmupIndex = 0; warmupIndex < warmup; warmupIndex += 1) {
    runner();
  }

  /** Timed durations in milliseconds. */
  const samples: number[] = [];
  /** Outcome of the most recent parse. */
  let lastOutcome: ScalingOutcome | undefined;

  for (let timedIndex = 0; timedIndex < timed; timedIndex += 1) {
    /** Monotonic start of this parse. */
    const startedAt = performance.now();
    lastOutcome = runner();
    samples.push(performance.now() - startedAt,);
  }

  if (lastOutcome === undefined) {
    lastOutcome = runner();
  }

  process.stdout.write(`${
    JSON.stringify({
      kind: 'run',
      arm,
      sweep,
      level,
      shape,
      round,
      warmup,
      timed,
      ms: samples,
      tokens: lastOutcome.tokens,
      ok: lastOutcome.success,
      probes: lastOutcome.probes,
      node: process.version,
      cpuCount: cpus().length,
      load1: loadavg()[0],
    },)
  }\n`,);
}

await main();

//endregion Entry point
