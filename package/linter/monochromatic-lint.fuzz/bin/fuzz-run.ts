/**
 The mount-free fuzz run: a run image with the binaries, seeds and dictionary, then one bounded container per target.
 */
import {
  cp,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  createContainer,
  runCommand,
} from '../../../git-policy/cli/bin/native-verification-process.ts';
import {
  baseImage,
  buildBounds,
  FuzzVerificationError,
  limits,
  runImage,
  targets,
} from './fuzz-constants.ts';

/**
 Libfuzzer's count of executed inputs, as it prints it in the final statistics.
 */
const executedUnitsLabel = 'stat::number_of_executed_units:';

/**
 Characters of a decimal number.
 */
const decimalDigits = '0123456789';

/**
 Take the decimal digits a text starts with.

 @param text - the text after a label

 @returns the leading digits, or the empty text when the text does not start with a digit

 @example
 ```ts
 leadingDigits('12 units');
 ```
 */
function leadingDigits(text: string): string {
  /**
   Position of the first character that is not a digit.
   */
  const end = Array.from(text)
    .findIndex(function isNotDigit(character) {
      return !decimalDigits.includes(character);
    });
  return end === (-1) ? text : text.slice(
    0,
    end,
  );
}

/**
 Read the executed-input count out of the fuzzer's output.

 @param output - standard output and error of the fuzz container

 @returns the count after the first label that is followed by whitespace and digits, or zero when there is none

 @example
 ```ts
 executedUnitsIn('stat::number_of_executed_units: 12');
 ```
 */
function executedUnitsIn(output: string): number {
  /**
   The digits after each label that whitespace separates from them.
   */
  const counts = output.split(executedUnitsLabel)
    .slice(1)
    .flatMap(function countOf(segment) {
      /**
       The text after the label without the whitespace between.
       */
      const afterSpace = segment.trimStart();
      /**
       The digits that follow, if whitespace came first.
       */
      const digits = afterSpace.length < segment.length ? leadingDigits(afterSpace) : '';
      return digits === '' ? [] : [Number(digits)];
    });
  return counts[0] ?? 0;
}

/**
 Containerfile of the mount-free run image.

 @returns the text of the Containerfile

 @example
 ```ts
 const text = runContainerfile();
 ```
 */
function runContainerfile(): string {
  return [
    '# Fuzz execution has no compiler, registry, repository, home, or credential mounts.',
    `FROM ${baseImage}`,
    'COPY bin /fuzz/bin',
    'COPY corpus /fuzz/corpus',
    'COPY dictionary /fuzz/dictionary',
    `RUN ${JSON.stringify([
      'mkdir',
      '--parents',
      ...targets.map(function artifactsOf(target) {
        return `/fuzz/artifacts/${target}`;
      }),
    ])}`,
    'WORKDIR /fuzz',
    '',
  ].join('\n');
}

/**
 Build the run image from the instrumented binaries, the seeds and the dictionaries.

 @param context - the build context that holds the binaries

 @param source - the fuzz sidecar directory that holds the seeds and dictionaries

 @example
 ```ts
 await buildRunImage({ context, source });
 ```
 */
export async function buildRunImage({
  context,
  source,
}: {
  readonly context: string;
  readonly source: string;
}): Promise<void> {
  await cp(
    join(
      source,
      'seed',
    ),
    join(
      context,
      'corpus',
    ),
    { recursive: true },
  );
  await cp(
    join(
      source,
      'dictionary',
    ),
    join(
      context,
      'dictionary',
    ),
    { recursive: true },
  );
  await writeFile(
    join(
      context,
      'Run.Containerfile',
    ),
    runContainerfile(),
  );
  await runCommand({
    command: 'podman',
    args: [
      'build',
      ...buildBounds,
      '--tag',
      runImage,
      '--file',
      join(
        context,
        'Run.Containerfile',
      ),
      context,
    ],
  });
}

/**
 Fuzz one target in its own container and keep its logs, corpus and artifacts.

 @param target - the fuzz target

 @param evidence - the evidence directory

 @throws FuzzVerificationError when the target exits nonzero or executes no input

 @example
 ```ts
 await fuzzTarget({ target, evidence });
 ```
 */
async function fuzzTarget({
  target,
  evidence,
}: {
  readonly target: string;
  readonly evidence: string;
}): Promise<void> {
  /**
   The fuzz container, removed when this function ends.
   */
  await using container = await createContainer({
    args: [
      'create',
      ...limits,
      runImage,
      `/fuzz/bin/${target}`,
      `/fuzz/corpus/${target}`,
      '-max_total_time=30',
      '-max_len=8192',
      '-rss_limit_mb=1536',
      '-timeout=5',
      '-print_final_stats=1',
      `-artifact_prefix=/fuzz/artifacts/${target}/`,
      ...(target === 'configuration' ? ['-dict=/fuzz/dictionary/configuration.dict'] : []),
    ],
  });
  /**
   How the fuzz run ended.
   */
  const result = await runCommand({
    command: 'podman',
    args: [
      'start',
      '--attach',
      container.id,
    ],
    capture: true,
    allowFailure: true,
  });
  /**
   Evidence directory of this target.
   */
  const targetEvidence = join(
    evidence,
    target,
  );
  await mkdir(
    targetEvidence,
    { recursive: true },
  );
  await writeFile(
    join(
      targetEvidence,
      'stdout.log',
    ),
    result.stdout,
  );
  await writeFile(
    join(
      targetEvidence,
      'stderr.log',
    ),
    result.stderr,
  );
  await runCommand({
    command: 'podman',
    args: [
      'cp',
      `${container.id}:/fuzz/corpus/${target}`,
      join(
        targetEvidence,
        'corpus',
      ),
    ],
  });
  await runCommand({
    command: 'podman',
    args: [
      'cp',
      `${container.id}:/fuzz/artifacts/${target}`,
      join(
        targetEvidence,
        'artifacts',
      ),
    ],
  });
  /**
   How many inputs the fuzzer executed.
   */
  const executedUnits = executedUnitsIn(`${result.stdout}\n${result.stderr}`);
  await writeFile(
    join(
      targetEvidence,
      'exit.json',
    ),
    `${JSON.stringify({
      status: result.status ?? null,
      signal: result.signal ?? null,
      executedUnits,
    })}\n`,
  );
  if ((result.status !== 0) || (executedUnits < 1))
    throw new FuzzVerificationError(`${target} exited ${result.status ?? 'without a status'} after ${executedUnits} verified executions; retained inputs and logs are in ${evidence}.`);
  console.log(`${target}: ${executedUnits} executions, exit 0; evidence ${targetEvidence}`);
}

/**
 Fuzz every target, one after another.

 @param evidence - the evidence directory

 @throws FuzzVerificationError when a target exits nonzero or executes no input

 @example
 ```ts
 await fuzzTargets(evidence);
 ```
 */
export async function fuzzTargets(evidence: string): Promise<void> {
  await mapInSequence({
    items: targets,
    run: function fuzzOf(target) {
      return fuzzTarget({
        target,
        evidence,
      });
    },
  });
}
