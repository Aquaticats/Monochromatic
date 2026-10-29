#!/usr/bin/env node

/**
 CLI wrapper for `pnpm` that filters out known-benign warnings.
 
 Runs `pnpm` with all provided arguments, captures output,
 and removes allowed cycle warnings from both stdout and stderr
 via {@link filterPnpmOutput}.
 pnpm writes the cycle warning to stdout (verified empirically),
 but both streams are filtered for robustness.
 
 @example
 ```bash
 task-pnpm install
 task-pnpm install --frozen-lockfile
 ```
 */

import spawn from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { filterPnpmOutput, } from './pnpm-output-filter.ts';

//region Main execution

/**
 Arguments forwarded to pnpm.
 */
const pnpmArgs = process.argv
  .slice(2,);

/**
 Options for {@link writeFiltered}.
 
 @example
 ```ts
 const options: WriteFilteredOptions = {
   raw: ' WARN  cycle: a, b\nother\n',
   stream: process.stdout,
 };
 ```
 */
type WriteFilteredOptions = {
  /**
   Raw pnpm output
   */
  readonly raw: string;
  /**
   Target writable stream
   */
  stream: NodeJS.WriteStream;
};

/**
 Filters output via {@link filterPnpmOutput} and writes to the given stream
 if non-empty after filtering.
 
 @param raw - Raw pnpm output
 
 @param stream - Target writable stream
 
 @mutates stream - `stream.write` sends non-empty filtered output to caller-owned stream state.
 
 @example
 ```ts
 writeFiltered({ raw: 'some output\n', stream: process.stdout });
 ```
 */
function writeFiltered({
  raw,
  stream,
}: WriteFilteredOptions,): void {
  if (raw.length
    === 0)
    return;
  /**
   Output with allowed pnpm cycle warnings stripped; may be empty if every line was a known-benign warning.
   */
  const filtered = filterPnpmOutput(raw,);
  if (filtered.length
    > 0)
    stream.write(filtered,);
}

try {
  /**
   Captured pnpm subprocess result; both streams are filtered before forwarding to the parent process.
   */
  const result = await spawn(
    'pnpm',
    [...pnpmArgs,],
  );

  writeFiltered({
    raw: result.stdout,
    stream: process.stdout,
  },);
  writeFiltered({
    raw: result.stderr,
    stream: process.stderr,
  },);
}
catch (error) {
  if ((error !== null) && ((typeof error) === 'object')
    && ('exitCode' in error)) {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- 'exitCode' in check above narrows error to the captured-subprocess shape */
    /**
     Re-typed thrown error so its captured stdout, stderr, and exit fields can be forwarded after filtering.
     */
    const subprocessError = error as {
      stdout?: string;
      stderr?: string;
      exitCode?: number;
      signalName?: string;
    };
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    writeFiltered({
      raw: subprocessError.stdout
        ?? '',
      stream: process.stdout,
    },);
    writeFiltered({
      raw: subprocessError.stderr
        ?? '',
      stream: process.stderr,
    },);

    process.exitCode = subprocessError.exitCode
      ?? 1;

    if ((subprocessError.signalName
      !== undefined)
      && (subprocessError.signalName
        !== ''))
      {
      console.error(
        `[task-pnpm] pnpm terminated by signal: ${subprocessError.signalName}`,
      );
      process.exitCode = 1;
    }
  }
  else {
    console.error(
      `[task-pnpm] failed to execute pnpm: ${
        caughtValueText(error,)
      }`,
    );
    process.exitCode = 1;
  }
}

//endregion Main execution
