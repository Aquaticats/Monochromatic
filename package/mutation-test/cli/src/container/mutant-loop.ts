/**
 Sequential per-mutant execution loop inside one shard container.
 
 Applies each mutant by string-offset splice, type-checks, runs the
 selected tests, restores the original file, and classifies the result.
 The first anomaly (timeout, spawn failure, restore failure) aborts the
 remainder: the container is tainted and the host reshards what's left.
 
 @example
 ```ts
 await runMutantLoop({ packageCwd, manifest, baselineTestsMs: 900 });
 ```
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { spliceReplacement, } from '../engine/splice.ts';
import { runBuildStep, } from './build-step.ts';
import { runTests, } from './test-run.ts';
import { tsgoCheck, } from './tsgo-check.ts';
import type { MutantStatus, } from '../engine/types.ts';
import type {
  ShardManifest,
  ShardMutantResult,
} from '../shard-schema.ts';

/**
 Module logger for the container-side mutant loop.
 */
const l = tagged({ tag: 'mutation-test-container', },);

/**
 Loop output: per-mutant results plus the taint-abandoned remainder.
 */
export type MutantLoopOutput = {
  readonly results: readonly ShardMutantResult[];
  readonly unrun: readonly string[];
  readonly anomaly: string;
};

/**
 Statuses that taint the container for every later mutant.
 
 A timeout means a mutant's process tree had to be killed; a runtime
 error means infrastructure misbehaved. Either way later results in
 this container are untrustworthy.
 */
const TAINTING_STATUSES: ReadonlySet<MutantStatus> = new Set([
  'timeout',
  'runtimeError',
],);

/**
 Computes the effective per-mutant test timeout.
 
 @param options - Manifest limits and measured baseline duration.
 
 @returns Milliseconds allowed for one mutant's test run.
 
 @example
 ```ts
 effectiveTimeoutMs({ floorMs: 5000, factor: 3, baselineMs: 900 });
 // 5000
 ```
 */
export function effectiveTimeoutMs(options: {
  readonly floorMs: number;
  readonly factor: number;
  readonly baselineMs: number;
},): number {
  return Math.max(
    options.floorMs,
    Math.ceil(options.factor * options.baselineMs,),
  );
}

/**
 Runs every mutant in the shard sequentially.
 
 @param options - Package cwd, shard manifest, and baseline test time.
 
 @returns Results, unrun remainder, and anomaly description.
 
 @example
 ```ts
 const output = await runMutantLoop({ packageCwd, manifest, baselineTestsMs: 900 });
 ```
 */
export async function runMutantLoop(options: {
  readonly packageCwd: string;
  readonly manifest: Readonly<ShardManifest>;
  readonly baselineTestsMs: number;
},): Promise<MutantLoopOutput> {
  /**
   Logger scoped to this shard's loop.
   */
  const rl = tagged({
    tag: runMutantLoop.name,
    l,
  },);
  /**
   Effective test timeout for every mutant in this shard.
   */
  const timeoutMs = effectiveTimeoutMs({
    floorMs: options.manifest
      .timeoutFloorMs,
    factor: options.manifest
      .timeoutFactor,
    baselineMs: options.baselineTestsMs,
  },);
  /**
   Accumulated per-mutant results.
   */
  const results: ShardMutantResult[] = [];

  rl.info(
    `running ${String(options.manifest
      .mutants
      .length,)} mutants, timeout ${String(timeoutMs,)}ms`,
  );

  /* oxlint-disable no-await-in-loop */
  // Mutants share one work tree, so execution is sequential by design;
  // parallelism lives at the shard-container level on the host.
  for (const [
    index,
    mutant,
  ] of options.manifest
    .mutants
    .entries()) {
    /**
     Absolute path of the file under mutation in the work tree.
     */
    const filePath = join(
      options.packageCwd,
      mutant.file,
    );
    /**
     Pristine file text restored after the mutant's verdict.
     */
    const original = await readFile(
      filePath,
      'utf8',
    );
    /**
     Start timestamp for this mutant.
     */
    const startedAt = performance.now();
    /**
     Scope guard restoring pristine file text when this iteration
     exits, whatever the verdict; replaces try/finally per house style.
     */
    await using restoreOriginal = {
      async [Symbol.asyncDispose](): Promise<void> {
        await writeFile(
          filePath,
          original,
          'utf8',
        );
      },
    };

    try {
      await writeFile(
        filePath,
        spliceReplacement({
          source: original,
          start: mutant.start,
          end: mutant.end,
          text: mutant.replacement,
        },),
        'utf8',
      );

      /**
       Build verdict for the spliced mutant; tests exercise built
       output, so the build must precede them, and its declaration
       emit must precede the type gate.
       */
      const build = await runBuildStep({ packageCwd: options.packageCwd, },);
      /**
       Type-check verdict for the spliced mutant.
       */
      const typeCheck = build.clean
        ? await tsgoCheck({ cwd: options.packageCwd, },)
        : {
          clean: false,
          durationMs: 0,
          detail: build.detail,
        };
      /**
       Final status for this mutant.
       */
      const status: MutantStatus = await (async function classify(): Promise<MutantStatus> {
        if (!typeCheck.clean)
          return 'compileError';

        /**
         Test verdict for the compiling mutant.
         */
        const testRun = await runTests({
          cwd: options.packageCwd,
          tests: options.manifest
            .tests,
          timeoutMs,
        },);

        if (testRun.kind === 'passed')
          return 'survived';

        if (testRun.kind === 'timeout')
          return 'timeout';

        return 'killed';
      })();

      results.push({
        id: mutant.id,
        status,
        position: index + 1,
        durationMs: performance.now() - startedAt,
        detail: mutant.description,
      },);

      if (TAINTING_STATUSES.has(status,)) {
        rl.warn(`tainting status ${status} at position ${String(index + 1,)}; aborting remainder`,);
        // Expected taint, not an infrastructure failure: the result is
        // recorded and the unrun remainder reshards; anomaly stays empty
        // so the host does not flag the run as infra-failed.
        return {
          results,
          unrun: options.manifest
            .mutants
            .slice(index + 1,)
            .map(function toId(remaining,): string {
              return remaining.id;
            },),
          anomaly: '',
        };
      }
    }
    catch (error) {
      rl.error(`infrastructure failure at position ${String(index + 1,)}: ${String(error,)}`,);
      results.push({
        id: mutant.id,
        status: 'runtimeError',
        position: index + 1,
        durationMs: performance.now() - startedAt,
        detail: String(error,),
      },);
      return {
        results,
        unrun: options.manifest
          .mutants
          .slice(index + 1,)
          .map(function toId(remaining,): string {
            return remaining.id;
          },),
        anomaly: `infrastructure failure on mutant ${mutant.id}: ${String(error,)}`,
      };
    }
  }
  /* oxlint-enable no-await-in-loop */

  return {
    results,
    unrun: [],
    anomaly: '',
  };
}
