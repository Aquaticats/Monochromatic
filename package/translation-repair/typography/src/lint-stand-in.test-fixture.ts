/**
 A stand-in for the linter program, for the cases that need it to report
 another version, exit with a chosen status, or record what it was asked.

 A stand-in is a file a case writes into a directory it owns and makes
 runnable, started the way the real program is: by name or path, through the
 operating system. It is a Node script, so it stands wherever Node does.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  chmod,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { readTextOrEmptyIfMissing, } from '../dist/final/node/index.mjs';

/**
 Permission bits of a runnable stand-in: read and execute for everyone, write for the owner.
 */
const RUNNABLE_MODE = 0o755;

/**
 What a stand-in does once it has answered the version question.
 */
type StandInBehaviour = 'echo' | 'fail-two' | 'kill-itself';

/**
 A written stand-in.
 */
type StandIn = {
  /**
   Path of the program, runnable.
   */
  readonly program: string;

  /**
   Path of the log the program appends its arguments to.
   */
  readonly log: string;
};

/**
 Writes a stand-in for the program into a directory and makes it runnable.

 THE STAND-IN LOGS BEFORE IT ANSWERS: every start appends its arguments, as
 one JSON line, to a log beside it, so a case reads what was asked of it
 whatever the answer was.

 @param dir - directory the case owns

 @param version - version the stand-in reports for `--version`

 @param behaviour - `echo` writes the passage back unchanged; `fail-two`
 writes the passage to standard error and exits with status 2; `kill-itself`
 stops itself with SIGKILL

 @returns Path of the stand-in and path of the log it appends to

 @example
 ```ts
 const { program, log, } = await standIn({ dir, version: '0.2.0', behaviour: 'echo', },);
 ```
 */
export async function standIn(
  {
    dir,
    version,
    behaviour,
  }: {
    readonly dir: string;
    readonly version: string;
    readonly behaviour: StandInBehaviour;
  },
): Promise<StandIn> {
  /**
   Path of the stand-in.
   */
  const program = join(
    dir,
    'purr-lint',
  );
  /**
   Path of the log it appends to.
   */
  const log = join(
    dir,
    'purr-lint.log',
  );
  /**
   What it does with a passage after the version question.
   */
  const afterVersion = (behaviour === 'echo')
    ? 'process.stdout.write(input);'
    : (behaviour === 'fail-two')
      ? 'process.stderr.write(input); process.exit(2);'
      : 'process.kill(process.pid, \'SIGKILL\');';
  await writeFile(
    program,
    [
      '#!/usr/bin/env node',
      'const { appendFileSync, readFileSync } = require(\'node:fs\');',
      'const args = process.argv.slice(2);',
      `appendFileSync(${JSON.stringify(log,)}, JSON.stringify(args) + '\\n');`,
      'if (args[0] === \'--version\') {',
      `  process.stdout.write(${JSON.stringify(`monochromatic-lint ${version}\n`,)});`,
      '  process.exit(0);',
      '}',
      'const input = readFileSync(0, \'utf8\');',
      afterVersion,
      '',
    ].join('\n',),
  );
  await chmod(
    program,
    RUNNABLE_MODE,
  );
  return {
    program,
    log,
  };
}

/**
 Runs a body and hands back what it threw.

 @param body - what to run

 @returns The thrown value

 @throws {@link Error} when the body throws nothing, since every case that
 calls this expects a refusal

 @example
 ```ts
 const thrown = caughtFrom({ body: () => wrapWithLintBinary({ binary, text, },), },);
 ```
 */
export function caughtFrom({ body, }: { readonly body: () => unknown; },): unknown {
  try {
    body();
  }
  catch (error) {
    return error;
  }
  throw new Error('the body threw nothing',);
}

/**
 Lines a stand-in logged, one parsed argument list per start.

 @param log - path of the stand-in's log

 @returns Argument lists in start order; empty where it never started

 @example
 ```ts
 const starts = await startsOf({ log, },);
 ```
 */
export async function startsOf({ log, }: { readonly log: string; },): Promise<readonly (readonly string[])[]> {
  /**
   The log's text, empty where the stand-in never started.
   */
  const text = await readTextOrEmptyIfMissing({ path: log, },);
  return text.split('\n',)
    .filter(function isEntry(line,): boolean {
      return line !== '';
    },)
    .map(function parsed(line,): readonly string[] {
      /**
       One logged argument list, whose shape this fixture's own script wrote.
       */
      const arguments_: unknown = JSON.parse(line,);
      if (!Array.isArray(arguments_,))
        throw new Error('unreachable: the stand-in logs one JSON array per start',);
      return arguments_.map(String,);
    },);
}
