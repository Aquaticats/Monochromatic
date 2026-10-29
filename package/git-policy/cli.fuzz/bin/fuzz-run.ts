/**
 Fuzz one target in a mount-free container and keep its logs, corpus and artifacts as evidence.

 A target may print on every execution, so both streams go straight to files instead of a bounded buffer.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed.
/// <reference types="node" />
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
} from '../../cli/bin/native-verification-process.mjs';
import {
  limits,
  maxInputBytes,
  runImage,
} from './fuzz-images.mjs';

/** How much of the end of each stream is searched for libFuzzer's final statistics: 256 KiB. */
const statisticsTailBytes = 262_144;
/** The line libFuzzer's `-print_final_stats=1` prints before the executed-unit count. */
const statisticsMarker = 'stat::number_of_executed_units:';

/**
 Last bytes of a possibly very large log, so a long campaign never loads the whole stream.

 @param {string} file - log path
 @returns {Promise<string>} at most the last 256 KiB as text
 */
async function readTail(file) {
  await using handle = await open(
    file,
    'r',
  );
  const { size } = await handle.stat();
  const length = Math.min(
    size,
    statisticsTailBytes,
  );
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

 @param {string} text - the tails of both streams
 @returns {number} the count, or 0 when the line is absent or has no number after whitespace
 */
export function executedUnitsIn(text) {
  const markerAt = text.indexOf(statisticsMarker);
  if (markerAt === (-1))
    return 0;
  const afterMarker = text.slice(markerAt + statisticsMarker.length);
  const afterSpace = afterMarker.trimStart();
  if (afterSpace.length === afterMarker.length)
    return 0;
  const nonDigitAt = afterSpace.split('')
    .findIndex(function isNotDigit(character) {
      return !'0123456789'.includes(character);
    });
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
 Run the target's fuzzer to its time limit.

 @param {{ target: import('./fuzz-options.mjs').FuzzTarget, evidence: string, secondsPerTarget: number }} request -
   the target, the evidence directory and the seconds to fuzz
 @returns {Promise<string | undefined>} what went wrong, or nothing when the target ran and executed inputs
 */
export async function fuzzTarget({
  target,
  evidence,
  secondsPerTarget,
}) {
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
  const directory = join(
    evidence,
    target.name,
  );
  await mkdir(
    directory,
    { recursive: true },
  );
  const stdoutLog = join(
    directory,
    'stdout.log',
  );
  const stderrLog = join(
    directory,
    'stderr.log',
  );
  await using stdoutFile = await open(
    stdoutLog,
    'w',
  );
  await using stderrFile = await open(
    stderrLog,
    'w',
  );
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
    return `${target.name} exited ${String(child.exitCode)} after ${executedUnits} verified executions`;
  }
  console.log(`${target.name}: ${executedUnits} executions, exit 0; evidence ${directory}`);
  return undefined;
}
