#!/usr/bin/env node

/**
 Runs `pnpm update --recursive --no-save`. When pnpm's strict
 `minimumReleaseAge` gate refuses it without naming a package, finds the
 blocked versions, when each passes the gate, and who pulls it in.

 Rationale: `doc/troubleshooting/pnpm-update-no-save-strict-release-age.md`.

 @example
 ```sh
 mise run deps:update
 ```
 */

import { resolve, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { diagnoseImmaturePicks, } from './diagnose.ts';
import { runPnpm, } from './pnpm.ts';
import { formatImmatureReport, } from './report.ts';
import {
  runUpdate,
  STRICT_REQUIRES_SAVE_CODE,
} from './update.ts';

/**
 Entry logger.
 */
const l = tagged({ tag: 'deps-update', },);

//region Errors

/**
 Thrown when pnpm's age gate blocks the update; message is the full report.
 */
export class ImmaturePicksError extends Error {
  /**
   Builds the error from the rendered report.

   @param report - diagnosis text shown to the user

   @example
   ```ts
   throw new ImmaturePicksError('pnpm update refused 1 version(s) ...');
   ```
   */
  constructor(report: string,) {
    super(report,);
    this.name = 'ImmaturePicksError';
  }
}

/**
 Thrown when pnpm update fails for a reason this task does not diagnose.
 */
export class PnpmUpdateError extends Error {
  /**
   Builds the error with pnpm's exit code; pnpm's own output was already shown.

   @param exit - exit code, or a description when a signal ended pnpm

   @example
   ```ts
   throw new PnpmUpdateError('1');
   ```
   */
  constructor(exit: string,) {
    super(`pnpm update --recursive --no-save failed (exit ${exit}); see pnpm's output above.`,);
    this.name = 'PnpmUpdateError';
  }
}

//endregion Errors

//region Main

/**
 Workspace root: mise runs root tasks from the repository root.
 */
const root = resolve('.',);

/**
 Result of the real update.
 */
const outcome = await runUpdate({
  cwd: root,
  command: 'pnpm',
  commandArgs: [],
},);

if (!outcome.ok) {
  if (!outcome.stderr
    .includes(STRICT_REQUIRES_SAVE_CODE,))
    throw new PnpmUpdateError(outcome.exitCode === undefined ? 'by signal' : String(outcome.exitCode,),);
  l.info(`${STRICT_REQUIRES_SAVE_CODE}: diagnosing which versions are too new`,);
  /**
   Blocked picks and configured age.
   */
  const diagnosis = await diagnoseImmaturePicks({
    root,
    runPnpm,
    fetchImpl: fetch,
  },);
  throw new ImmaturePicksError(formatImmatureReport(diagnosis,),);
}

//endregion Main
