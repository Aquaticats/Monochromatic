#!/usr/bin/env node
/**
 Prove panic guards with real executable and public-loader controls in disposable source variants.
 */
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  run,
  type RunResult,
  ScannerVerificationError,
  snapshot,
  type SnapshotContext,
} from './container-snapshot.ts';
import {
  guardVariants,
  inject,
} from './guard-injection.ts';
import { removalOf } from './removal-of.ts';

/**
 Cargo arguments of the panic-contract test, run in every variant.
 */
const contractCommand: readonly string[] = [
  'cargo',
  'test',
  '--offline',
  '--locked',
  '--test',
  'panic_contract',
  '--',
  '--nocapture',
  '--test-threads=1',
];

/**
 Exit status of a Rust test binary that panicked, which is what a disabled guard must produce.
 */
const rustPanicStatus = 101;

/**
 What a control run showed.
 */
type Verdict = {
  readonly variant: string;
  readonly status?: number;
  readonly expected: boolean;
  readonly protectedPass: boolean;
  readonly disabledFailure: boolean;
};

/**
 Judge a control run: the protected scanner passes, and each disabled variant fails the assertion that names it.

 @param variant - the control variant

 @param result - how the container ended

 @param stdout - the container's standard output

 @param stderr - the container's standard error

 @returns whether the run proved the expected boundary, with the two outcomes it was judged by

 @example
 ```ts
 judge({ variant: 'protected', result, stdout, stderr });
 ```
 */
function judge({
  variant,
  result,
  stdout,
  stderr,
}: {
  readonly variant: string;
  readonly result: RunResult;
  readonly stdout: string;
  readonly stderr: string;
}): Verdict {
  /**
   Standard output and error together.
   */
  const output = stdout + stderr;
  /**
   Whether the protected scanner passed all three contract tests.
   */
  const protectedPass = (result.status === 0) && stdout.includes('3 passed')
    && stderr.includes('fixture-host-hook-called');
  /**
   Whether the run failed a previously passing assertion rather than failing to compile.
   */
  const disabledFailure = (result.status === rustPanicStatus) && stdout.includes('FAILED')
    && output.includes('test result: FAILED');
  /**
   The text the disabled variant's failure must also show.
   */
  const named: Readonly<Record<string, boolean>> = {
    'without-output-hook': output.includes('SYNTHETIC_PRIVATE_STARTUP'),
    'without-process-catch': output.includes('actual CLI exit must be two'),
    'without-load-catch': stdout.includes('public_loader_discards_partial_rules_and_preserves_host_hook ... FAILED'),
    'without-content-catch': stdout.includes('public_scan_catches_faults_without_retaining_partial_hits ... FAILED'),
    'without-name-catch': stdout.includes('public_scan_catches_faults_without_retaining_partial_hits ... FAILED'),
  };
  return {
    variant,
    ...(result.status === undefined ? {} : { status: result.status }),
    expected: variant === 'protected' ? protectedPass : disabledFailure && (named[variant] ?? true),
    protectedPass,
    disabledFailure,
  };
}

/**
 Run one control variant in its own snapshot and keep its verdict as evidence.

 @param variant - the control variant

 @param test - the frozen consumer test

 @throws ScannerVerificationError when the control did not prove the expected boundary

 @example
 ```ts
 await verifyVariant({ variant: 'protected', test });
 ```
 */
async function verifyVariant({
  variant,
  test,
}: {
  readonly variant: string;
  readonly test: string;
}): Promise<void> {
  /**
   The snapshot image and its evidence directory.
   */
  const fixture = await snapshot({
    name: `guard-${variant}`,
    command: contractCommand,
    async transform({
      context,
      scanner,
    }: SnapshotContext): Promise<void> {
      await inject({
        context,
        scanner,
        variant,
        test,
      });
    },
  });
  /**
   Removal of the build context when the variant ends.
   */
  await using cleanup = removalOf(fixture.context);
  void cleanup;
  /**
   What the control container left behind.
   */
  const result = await run({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...fixture.limits,
      '--env',
      'SCANNER_GUARD_HOME=/home/tester',
      '--env',
      'FORBIDDEN_STRINGS_CACHE_DIR=/home/tester/cache',
      fixture.image,
    ],
    allowFailure: true,
    transcript: fixture.evidence,
  });
  /**
   The verdict of this run.
   */
  const verdict = judge({
    variant,
    result,
    stdout: await readFile(
      join(
        fixture.evidence,
        'stdout.log',
      ),
      'utf8',
    ),
    stderr: await readFile(
      join(
        fixture.evidence,
        'stderr.log',
      ),
      'utf8',
    ),
  });
  await writeFile(
    join(
      fixture.evidence,
      'control.json',
    ),
    `${JSON.stringify(
      {
        variant,
        status: verdict.status ?? null,
        expected: verdict.expected,
        protectedPass: verdict.protectedPass,
        disabledFailure: verdict.disabledFailure,
      },
      null,
      2,
    )}\n`,
  );
  if (!verdict.expected)
    throw new ScannerVerificationError(`Panic control did not prove the expected boundary: ${variant}; inspect ${fixture.evidence}.`);
}

/**
 Every disabled variant must fail the previously passing assertion, not merely fail compilation.

 @throws ScannerVerificationError when arguments are given or a control does not prove its boundary

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  if (process.argv
    .length
    !== 2) throw new ScannerVerificationError('This task accepts no arguments.');
  /**
   The committed consumer, frozen once: edits during a multi-variant campaign must not change its assertions.
   */
  const test = await readFile(
    join(
      import.meta.dirname,
      'guard-fixture.rs.txt',
    ),
    'utf8',
  );
  await mapInSequence({
    items: guardVariants,
    run: function verifyOf(variant) {
      return verifyVariant({
        variant,
        test,
      });
    },
  });
}

await main();
