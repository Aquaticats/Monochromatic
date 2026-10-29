#!/usr/bin/env node

/**
 Integration check: run the built CLI against the fixture package and
 assert the report matches its documented expectations.
 
 Needs podman and (on first run) an image build, so it lives behind the
 dedicated `test:integration` mise task instead of `test:unit`.
 
 @example
 ```bash
 mise run '//package/mutation-test/cli:test:integration'
 ```
 */

import { readFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { cwd, } from 'node:process';

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { isRecord, } from './is-record.ts';
import type { RunReport, } from './host/report.ts';

/**
 Module logger for the integration check.
 */
const l = tagged({ tag: 'mutation-test-integration', },);

/**
 Fixture package path relative to the repo root.
 */
const FIXTURE_PACKAGE = 'package/mutation-test/cli.fixture';

/**
 Asserts one condition, throwing with context when it fails.
 
 @param options - Condition and failure message.
 
 @throws Error when the condition is false.
 
 @example
 ```ts
 check({ that: totals.killed > 0, message: 'expected kills' });
 ```
 */
function check(options: {
  readonly that: boolean;
  readonly message: string;
},): void {
  if (!options.that)
    throw new Error(`integration check failed: ${options.message}`,);
}

/**
 Runs the CLI against the fixture and asserts report expectations.
 
 @example
 ```ts
 await runIntegrationCheck();
 ```
 */
export async function runIntegrationCheck(): Promise<void> {
  /**
   Logger scoped to this check run.
   */
  const rl = tagged({
    tag: runIntegrationCheck.name,
    l,
  },);
  /**
   Report destination for this run.
   */
  const reportFile = join(
    scratchRoot(),
    `mutation-integration-${String(Date.now(),)}.json`,
  );
  rl.info(`running CLI against ${FIXTURE_PACKAGE}`,);
  await spawn(
    'node',
    [
      join(
        cwd(),
        'package/mutation-test/cli/dist/final/node/cli.mjs',
      ),
      '--package',
      FIXTURE_PACKAGE,
      '--report',
      reportFile,
    ],
    {
      stdout: 'inherit',
      stderr: 'inherit',
    },
  );

  /**
   Parsed report before shape validation.
   */
  const parsed: unknown = JSON.parse(await readFile(
    reportFile,
    'utf8',
  ),);

  check({
    that: isRecord(parsed,)
      && isRecord(parsed.totals,)
      && Array.isArray(parsed.mutants,),
    message: 'report JSON must carry totals and mutants',
  },);

  /**
   Parsed run report from the fixture run; shape checked above, and the
   writer is this same package, so deeper validation adds no safety.
   */
  // oxlint-disable-next-line no-unsafe-type-assertion -- shape checked at runtime above; writer is this package's own report builder
  const report = parsed as RunReport;

  check({
    that: report.totals
      .killed
      > 0,
    message: `clampedSum mutants must die, got killed=${String(report.totals
      .killed,)}`,
  },);
  check({
    that: report.totals
      .survived
      > 0,
    message: `describeSign zero-branch and untested.ts mutants must survive, got survived=${
      String(report.totals
        .survived,)
    }`,
  },);
  check({
    that: report.totals
      .runtimeError
      === 0,
    message: `no infra errors expected, got runtimeError=${String(report.totals
      .runtimeError,)}`,
  },);

  /**
   Survivors from the file no test selects.
   */
  const untestedSurvivors = report.mutants
    .filter(function fromUntested(record,): boolean {
      return (record.file === 'src/untested.ts') && (record.status === 'survived');
    },);
  check({
    that: untestedSurvivors.length > 0,
    message: 'untested.ts must produce short-circuit survivors',
  },);
  check({
    that: untestedSurvivors.every(function isConfirmed(record,): boolean {
      return record.confirmed;
    },),
    message: 'short-circuit survivors must be confirmed by construction',
  },);
  check({
    that: report.mutants
      .filter(function actionable(record,): boolean {
        return (record.status === 'survived') || (record.status === 'timeout');
      },)
      .every(function isConfirmed(record,): boolean {
        return record.confirmed;
      },),
    message: 'every survivor and timeout must be confirmed',
  },);

  rl.info(`integration check passed; report at ${reportFile}`,);
}

await runIntegrationCheck();
