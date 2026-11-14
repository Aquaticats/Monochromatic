// Parses one arm's workload repeatedly under `node --cpu-prof`, so the V8
// sampling profiler records the parse itself. Iteration count is set per arm
// so a slow version and a fast version both produce enough samples.

import { buildHarness, } from './harness.ts';
import { loadPaths, } from './workloads.ts';

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

//endregion Flags

//region Entry point

/**
 * Parses the workload `--repeat` times and prints the validation record, so a
 * profile can be tied to a run that actually did the work.
 */
async function main(): Promise<void> {
  /** Parsed command line. */
  const flags = readFlags(process.argv.slice(2,),);
  /** Arm label, which is also the npm version. */
  const arm = flags.get('arm',);
  /** Iteration count. */
  const repeatRaw = flags.get('repeat',);

  if (arm === undefined || repeatRaw === undefined) {
    throw new Error('required: --arm=<version> --repeat=<count>',);
  }

  /** Iterations to parse. */
  const repeat = Number.parseInt(
    repeatRaw,
    10,
  );
  /** Path set to parse. */
  const pathSet = flags.get('pathset',) ?? 'real';
  /** Path count to parse. */
  const countRaw = flags.get('count',) ?? '1000';
  /** Path count as a number. */
  const count = Number.parseInt(
    countRaw,
    10,
  );
  /** Paths for this run. */
  const paths = loadPaths(pathSet as Parameters<typeof loadPaths>[0],)
    .slice(
      0,
      count,
    );
  /** Harness for this arm and shape. */
  const harness = await buildHarness(
    arm,
    {
      workload: 'no-separator',
      pathSet,
      count,
    },
    paths,
  );

  for (let index = 0; index < repeat; index += 1) {
    harness.run();
  }

  process.stderr.write(
    `[profile] arm=${arm} repeat=${repeat} count=${count} ok=${
      harness.inspect(harness.run(),).handled === count
    }\n`,
  );
}

await main();

//endregion Entry point
