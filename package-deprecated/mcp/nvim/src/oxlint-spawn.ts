/**
 * Low-level oxlint process spawning.
 *
 * Spawns a single oxlint process with the given config,
 * captures JSON output, and returns parsed diagnostics.
 *
 * @module
 */

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';

import type { Diagnostic, } from './nvim-client.ts';
import { rootLogger, } from './log.ts';
import { parseOxlintOutput, } from './oxlint-parse.ts';
import type { OxlintJsonOutput, } from './oxlint-types.ts';

//region Constants

/**
 * Timeout in milliseconds for the oxlint process.
 */
const OXLINT_TIMEOUT_MS = 10_000;

//endregion Constants

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site so a log line names the call that produced it.
 */
const oxlintSpawnLogger = tagged({
  l: rootLogger,
  tag: 'oxlint-spawn',
},);

//endregion Logging

//region Process spawning: low-level oxlint invocation

/**
 * Spawns a single oxlint process and returns parsed diagnostics.
 *
 * @param cwd - Working directory for the oxlint process.
 *
 * @param files - Absolute file paths to lint.
 *
 * @param typeAware - Whether to pass `--type-aware`.
 *
 * @returns Diagnostics grouped by absolute file path.
 *
 * @example
 * ```ts
 * const diagnostics = await spawnOxlint({ cwd: '/project', files: ['/project/src/index.ts'], typeAware: false });
 * // Map { '/project/src/index.ts' => [{ message: '...', severity: 'warning', ... }] }
 * ```
 */
export async function spawnOxlint({
  cwd,
  files,
  typeAware,
}: {
  cwd: string;
  files: readonly string[];
  typeAware: boolean;
},): Promise<Map<string, Diagnostic[]>> {
  /**
   * CLI argv for oxlint; forces JSON output and optionally enables type-aware rules.
   */
  const args = [
    '--format',
    'json',
    ...(typeAware ? ['--type-aware',] : []),
    ...files,
  ];

  try {
    /**
     * Successful oxlint invocation (exit code 0); means no diagnostics were emitted.
     */
    const result = await spawn(
      'oxlint',
      args,
      {
        cwd,
        timeout: OXLINT_TIMEOUT_MS,
      },
    );
    /**
     * Captured stdout from the zero-exit run; non-empty when diagnostics happen to be present at warning level.
     */
    const { stdout, } = result;

    if (stdout.trim()
      .length
      === 0) {
      tagged({
        tag: spawnOxlint.name,
        l: oxlintSpawnLogger,
      },)
        .error('oxlint produced no output (exit code 0)',);
      return new Map();
    }

    /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- oxlint JSON output conforms to OxlintJsonOutput schema */
    /**
     * Parsed oxlint JSON output; trusted to match {@link OxlintJsonOutput} because oxlint is the producer.
     */
    const parsed = JSON.parse(stdout,) as OxlintJsonOutput;
    /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
    return parseOxlintOutput({
      output: parsed,
      cwd,
    },);
  }
  catch (err: unknown) {
    // oxlint exits non-zero when it finds diagnostics, which is expected
    if ((err !== null)
      && (err !== undefined)
      && ((typeof err) === 'object')
      && ('stdout' in err))
    {
      /**
       * Captured stdout from the failed run; oxlint emits diagnostics here even when exiting non-zero.
       */
      const stdout = String(err.stdout,);
      if (stdout.trim()
        .length
        > 0) {
        /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- oxlint JSON output conforms to OxlintJsonOutput schema */
        /**
         * Parsed oxlint JSON output from the failed run; same schema as the success path.
         */
        const parsed = JSON.parse(stdout,) as OxlintJsonOutput;
        /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
        return parseOxlintOutput({
          output: parsed,
          cwd,
        },);
      }
      /**
       * Exit code surfaced to the log when oxlint produced no diagnostics; `'unknown'` when nano-spawn omits the field.
       */
      const exitCode = 'exitCode' in err ? String(err.exitCode,) : 'unknown';
      tagged({
        tag: spawnOxlint.name,
        l: oxlintSpawnLogger,
      },)
        .error(`oxlint produced no output (exit code ${exitCode})`,);
      return new Map();
    }
    /**
     * Fallback error description used when the caught value isn't an oxlint result; covers spawn failures, timeouts, etc.
     */
    const message = err instanceof Error ? err.message : String(err,);
    tagged({
      tag: spawnOxlint.name,
      l: oxlintSpawnLogger,
    },)
      .error(`Failed to run oxlint: ${message}`,);
    return new Map();
  }
}

//endregion Process spawning
