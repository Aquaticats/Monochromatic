// Measures the separator shape in a fresh process: warmup parses then timed
// parses through the arm's own Optique. Prints one JSON record on stdout.

import { cpus, loadavg, } from 'node:os';
import { pathToFileURL, } from 'node:url';

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
 * Runs the separator shape once and prints its record.
 */
async function main(): Promise<void> {
  /** Parsed command line. */
  const flags = readFlags(process.argv.slice(2,),);
  /** Arm label. */
  const arm = flags.get('arm',);

  if (arm === undefined) {
    throw new Error('missing required flag --arm',);
  }

  /** Paths after the separator. */
  const tokenCount = readInteger(
    flags,
    'tokenCount',
  );
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
  /** Copy label, used by the spread run. */
  const tag = flags.get('tag',) ?? '';
  /** Arm-local separator module. */
  const module = await import(
    pathToFileURL(`/bench/arms/${arm}/bench/separator-parser.ts`,).href,
  ) as {
    createSeparatorRunner: (tokenCountIn: number,) => () => {
      readonly success: boolean;
      readonly tokens: number;
    };
  };
  /** Runner for this arm. */
  const runner = module.createSeparatorRunner(tokenCount,);

  for (let warmupIndex = 0; warmupIndex < warmup; warmupIndex += 1) {
    runner();
  }

  /** Timed durations in milliseconds. */
  const samples: number[] = [];
  /** Outcome of the most recent parse. */
  let lastOutcome: { readonly success: boolean; readonly tokens: number; } | undefined;

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
      experiment: 'separator',
      arm,
      tag,
      tokenCount,
      round,
      warmup,
      timed,
      ms: samples,
      tokens: lastOutcome.tokens,
      ok: lastOutcome.success,
      node: process.version,
      cpuCount: cpus().length,
      load1: loadavg()[0],
    },)
  }\n`,);
}

await main();

//endregion Entry point
