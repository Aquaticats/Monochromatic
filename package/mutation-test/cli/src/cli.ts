#!/usr/bin/env node

/**
 Host CLI entrypoint for the yuku-based mutation tester.
 
 @example
 ```bash
 mutation-test --package package/module/fs-path
 mutation-test --package package/module/fs-path --dry-run src/trim.ts
 ```
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { cwd, } from 'node:process';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  parseCliOptions,
  type CliOptions,
} from './host/cli-options.ts';
import { enumeratePackage, } from './host/enumerate-package.ts';
import { orchestrateRun, } from './host/orchestrate.ts';
import {
  buildRunReport,
  formatTerminalSummary,
} from './host/report.ts';

/**
 Module logger for the CLI entrypoint.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Runs the CLI flow for parsed options.
 
 @param options - Parsed CLI options.
 
 @throws Error on infra failure (red baseline, unresolved mutants).
 
 @mutates options - `JSON.stringify` may invoke hooks on report data derived from options.
 
 @example
 ```ts
 await runCli(parseCliOptions(['--package', 'package/module/fs-path']));
 ```
 */
export async function runCli(options: CliOptions,): Promise<void> {
  /**
   Logger scoped to this invocation.
   */
  const rl = tagged({
    tag: runCli.name,
    l,
  },);

  if (options.dryRun) {
    /**
     Enumeration-only view for the dry run.
     */
    const {
      groups,
      ignored,
    } = await enumeratePackage(options,);

    for (const group of groups) {
      // Raw console: user-facing CLI summary output.
      console.log(
        `${group.file}: ${String(group.mutants
          .length,)} mutants, tests: ${
          group.tests
            .join(', ',)
        }`,
      );
    }

    console.log(`Ignored by suppression: ${String(ignored.length,)}`,);
    return;
  }

  /**
   Full orchestration outcome.
   */
  const outcome = await orchestrateRun(options,);
  /**
   Native JSON run report.
   */
  const report = buildRunReport({
    outcome,
    packagePath: options.packagePath,
  },);
  /**
   Report destination, defaulting beside the current directory.
   */
  const reportFile = options.reportFile === ''
    ? join(
      cwd(),
      'mutation-report.json',
    )
    : options.reportFile;
  await writeFile(
    reportFile,
    `${JSON.stringify(
      report,
      null,
      2,
    )}\n`,
    'utf8',
  );
  rl.info(`report written to ${reportFile}`,);

  // Raw console: user-facing CLI summary output.
  console.log(formatTerminalSummary(report,),);

  if (report.infraErrors
    .length
    > 0)
    throw new Error(
      `${String(report.infraErrors
        .length,)} infrastructure errors; see report ${reportFile}`,
    );
}

await runCli(
  parseCliOptions(process.argv
    .slice(2,),),
);
