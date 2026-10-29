/**
 Fuzz one target in a mount-free container and keep its logs, corpus and artifacts as evidence.

 A target may print on every execution, so both streams go straight to files instead of a bounded buffer.
 */
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  mkdir,
  open,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  createContainer,
  NativeVerificationError,
  runCommand,
} from '../../cli/bin/native-verification-process.ts';
import {
  limits,
  maxInputBytes,
  runImage,
} from './fuzz-images.ts';
import type { FuzzTarget } from './fuzz-options.ts';

/**
 How much of the end of each stream is searched for libFuzzer's final statistics: 256 KiB.
 */
const statisticsTailBytes = 262_144;
/**
 The line libFuzzer's `-print_final_stats=1` prints before the executed-unit count.
 */
const statisticsMarker = 'stat::number_of_executed_units:';

/**
 Last bytes of a possibly very large log, so a long campaign never loads the whole stream.

 @param file - log path

 @returns at most the last 256 KiB as text
 */
async function readTail(file: string): Promise<string> {
  /**
   The log, closed when the read ends.
   */
  await using handle = await open(
    file,
    'r',
  );
  /**
   Size of the whole log in bytes.
   */
  const { size } = await handle.stat();
  /**
   How many of the last bytes to read.
   */
  const length = Math.min(
    size,
    statisticsTailBytes,
  );
  /**
   Receives the tail.
   */
  const buffer = Buffer.alloc(length);
  await handle.read(
    buffer,
    0,
    length,
    size - length,
  );
  return buffer.toString('utf8');
}

/**
 Read the executed-unit count libFuzzer prints at the end of a run.

 @param text - tails of both streams
 
 @returns the count, or 0 when the line is absent or has no number after whitespace

 @example
 ```ts
 executedUnitsIn('stat::number_of_executed_units: 120');
 ```
 */
export function executedUnitsIn(text: string): number {
  /**
   Where the statistics line starts, negative one when it is absent.
   */
  const markerAt = text.indexOf(statisticsMarker);
  if (markerAt === (-1))
    return 0;
  /**
   Everything after the marker.
   */
  const afterMarker = text.slice(markerAt + statisticsMarker.length);
  /**
   The text after the marker without its leading whitespace.
   */
  const afterSpace = afterMarker.trimStart();
  if (afterSpace.length === afterMarker.length)
    return 0;
  /**
   Where the digits end, negative one when the text is digits only.
   */
  const nonDigitAt = afterSpace.split('')
    .findIndex(function isNotDigit(character) {
      return !'0123456789'.includes(character);
    });
  /**
   How many digits follow the whitespace.
   */
  const digitCount = nonDigitAt === (-1)
    ? afterSpace.length
    : nonDigitAt;
  return digitCount === 0
    ? 0
    : Number(afterSpace.slice(
      0,
      digitCount,
    ));
}

/**
 What one fuzz run found wrong; `failure` is absent when the target ran and executed inputs.
 */
export type FuzzOutcome = {
  readonly failure?: string;
};

/**
 Run the target's fuzzer to its time limit.

 @param target - target to fuzz

 @param evidence - evidence directory

 @param secondsPerTarget - seconds to fuzz

 @returns what went wrong, or nothing when the target ran and executed inputs

 @example
 ```ts
 const { failure } = await fuzzTarget({ target, evidence, secondsPerTarget: 30 });
 ```
 */
export async function fuzzTarget({
  target,
  evidence,
  secondsPerTarget,
}: {
  readonly target: FuzzTarget;
  readonly evidence: string;
  readonly secondsPerTarget: number;
}): Promise<FuzzOutcome> {
  /**
   The container that fuzzes the target, removed when the run ends.
   */
  await using container = await createContainer({
    args: [
      'create',
      ...limits,
      runImage,
      `/fuzz/bin/${target.name}`,
      `/fuzz/corpus/${target.name}`,
      `-max_total_time=${secondsPerTarget}`,
      `-max_len=${maxInputBytes}`,
      '-rss_limit_mb=1536',
      '-timeout=5',
      '-print_final_stats=1',
      `-artifact_prefix=/fuzz/artifacts/${target.name}/`,
      `-dict=/fuzz/dictionary/${target.dictionary}`,
    ],
  });
  /**
   The target's evidence directory.
   */
  const directory = join(
    evidence,
    target.name,
  );
  await mkdir(
    directory,
    { recursive: true },
  );
  /**
   Log of the fuzzer's standard output.
   */
  const stdoutLog = join(
    directory,
    'stdout.log',
  );
  /**
   Log of the fuzzer's standard error.
   */
  const stderrLog = join(
    directory,
    'stderr.log',
  );
  /**
   The standard output log, open while the fuzzer writes to it.
   */
  await using stdoutFile = await open(
    stdoutLog,
    'w',
  );
  /**
   The standard error log, open while the fuzzer writes to it.
   */
  await using stderrFile = await open(
    stderrLog,
    'w',
  );
  /**
   The attached `podman start`, whose streams go straight to the two logs.
   */
  const child = spawn(
    'podman',
    [
      'start',
      '--attach',
      container.id,
    ],
    {
      stdio: [
        'ignore',
        stdoutFile.fd,
        stderrFile.fd,
      ],
    },
  );
  try {
    await once(
      child,
      'close',
    );
  }
  catch (error) {
    throw new NativeVerificationError(
      'podman could not start.',
      { cause: error },
    );
  }
  await runCommand({
    command: 'podman',
    args: [
      'cp',
      `${container.id}:/fuzz/corpus/${target.name}`,
      join(
        directory,
        'corpus',
      ),
    ],
  });
  await runCommand({
    command: 'podman',
    args: [
      'cp',
      `${container.id}:/fuzz/artifacts/${target.name}`,
      join(
        directory,
        'artifacts',
      ),
    ],
  });
  /**
   How many inputs the fuzzer reports having executed.
   */
  const executedUnits = executedUnitsIn(`${await readTail(stdoutLog)}\n${await readTail(stderrLog)}`);
  await writeFile(
    join(
      directory,
      'exit.json',
    ),
    `${JSON.stringify({
      status: child.exitCode,
      signal: child.signalCode,
      executedUnits,
    })}\n`,
  );
  // Exit 0 alone is not evidence: a dictionary or corpus setup failure also ends quickly.
  if ((child.exitCode !== 0) || (executedUnits < 1)) {
    console.log(`${target.name}: FAILED, exit ${String(child.exitCode)}, ${executedUnits} executions; evidence ${directory}`);
    return { failure: `${target.name} exited ${String(child.exitCode)} after ${executedUnits} verified executions`, };
  }
  console.log(`${target.name}: ${executedUnits} executions, exit 0; evidence ${directory}`);
  return {};
}
