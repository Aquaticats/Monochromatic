/**
 Per-mutant TypeScript check via the native tsgo binary.
 
 Chosen over a watch daemon on measured evidence: tsgo 7.0.1-rc watch
 output has no completion terminator, while a warm one-shot `--noEmit`
 check measured 0.125 s on the module-test package. Incrementality
 comes from each package tsconfig's own build-info settings, warmed by
 the baseline check; a CLI `--incremental` flag would conflict with
 project-file configuration.
 
 @example
 ```ts
 await tsgoCheck({ cwd: '/work/packages/module/fs-path' });
 ```
 */

import spawn from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { isRecord, } from '../is-record.ts';

/**
 Cap on captured diagnostic text per failure, keeping shard reports
 readable when tsc emits hundreds of errors.
 */
const DIAGNOSTIC_TEXT_LIMIT = 4_000;

/**
 Module logger for container-side type checking.
 */
const l = tagged({ tag: 'mutation-test-container', },);

/**
 Outcome of one tsgo project check.
 */
export type TsgoOutcome = {
  readonly clean: boolean;
  readonly durationMs: number;
  readonly detail: string;
};

/**
 Extracts compiler diagnostics from a spawn failure.
 
 nano-spawn attaches captured stdout/stderr to its error; without them
 a red-baseline report only says "exit code 1", which is undebuggable.
 
 @param error - Caught spawn error.
 
 @returns Combined diagnostic text, possibly empty.
 
 @example
 ```ts
 diagnosticsFromError(caught);
 ```
 */
export function diagnosticsFromError(error: unknown,): string {
  if (!isRecord(error,))
    return '';

  return [
    error.stdout,
    error.stderr,
  ]
    .filter(function isText(value,): value is string {
      return ((typeof value) === 'string') && (value !== '');
    },)
    .join('\n',)
    .slice(
    0,
    DIAGNOSTIC_TEXT_LIMIT,
    );
}

/**
 Runs one incremental project check in the package directory.
 
 Uses the repo-root tsc bin (tsgo under typescript 7) resolved through
 the baked workspace instead of a package-local dependency.
 
 @param options - Package working directory inside the work tree.
 
 @returns Check outcome; `clean` false means the mutant cannot compile.
 
 @example
 ```ts
 const outcome = await tsgoCheck({ cwd: packageCwd });
 ```
 */
export async function tsgoCheck(options: {
  readonly cwd: string;
},): Promise<TsgoOutcome> {
  /**
   Logger scoped to this check invocation.
   */
  const rl = tagged({
    tag: tsgoCheck.name,
    l,
  },);
  /**
   Start timestamp for duration measurement.
   */
  const startedAt = performance.now();

  try {
    await spawn(
      'node',
      [
        '/baked/node_modules/typescript/lib/tsc.js',
        '--noEmit',
        '--project',
        '.',
      ],
      { cwd: options.cwd, },
    );
    return {
      clean: true,
      durationMs: performance.now() - startedAt,
      detail: '',
    };
  }
  catch (error) {
    rl.debug(`type check rejected mutant: ${String(error,)}`,);
    return {
      clean: false,
      durationMs: performance.now() - startedAt,
      detail: `${String(error,)} ${diagnosticsFromError(error,)}`.trim(),
    };
  }
}
