/**
 A disposable compiler executable exercises the built wrapper's real process boundary.

 @module
 */
import spawn, { SubprocessError } from 'nano-spawn';
import {
  chmod,
  mkdir,
  mkdtemp,
  open,
  readFile,
  rm,
  writeFile
} from 'node:fs/promises';
import { homedir } from 'node:os';
import {
  delimiter,
  join
} from 'node:path';

/**
 Owner-only permissions apply to the private fixture directory and executable.
 */
const PRIVATE_MODE = 0o700;

/**
 Controlled compiler output distinguishes suppressed diagnostics from process failure.
 */
export type CompilerFailureCase = {
  readonly name: string;
  readonly stdout: string;
  readonly stderr: string;
  readonly exitCode: number;
  readonly expectedCode: number;
  readonly expectedStdout: string;
  readonly expectedStderr: string;
};

/**
 Invoke the actual built wrapper against an executable compiler in a disposable working directory.

 @param input - fixed compiler streams and exit status
 
 @returns observed wrapper process result
 
 @throws Error when the wrapper process cannot start or fails to exit normally
 
 @example
 ```ts
 const result = await runCompilerFailureCase(fixtureCase);
 ```
 */
export async function runCompilerFailureCase(input: CompilerFailureCase): Promise<{
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
}> {
  /**
   Private scratch persists across machine restarts while each case owns its directory.
   */
  const scratch = join(
    homedir(),
    'temp',
    'agent'
  );
  await mkdir(
    scratch,
    {
      recursive: true,
      mode: PRIVATE_MODE
    }
  );
  await chmod(
    scratch,
    PRIVATE_MODE
  );
  /**
   Only this test's newly created directory is removed after the child exits.
   */
  const directory = await mkdtemp(join(
    scratch,
    'tsc-filter-failure-'
  ));
  /**
   Lexical cleanup also runs when process startup or an assertion fails.
   */
  await using fixture = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true
        }
      );
    },
  };
  /**
   Compiler output crosses into JavaScript only through JSON string encoding.
   */
  const source = [
    '#!/usr/bin/env node',
    `process.stdout.write(${JSON.stringify(input.stdout)});`,
    `process.stderr.write(${JSON.stringify(input.stderr)});`,
    `process.exitCode = ${JSON.stringify(input.exitCode)};`,
    '',
  ].join('\n');
  await writeFile(
    join(
      directory,
      'tsc'
    ),
    source,
    { mode: PRIVATE_MODE }
  );
  /**
   The emitted CLI is the same entry invoked by package lint tasks.
   */
  const entry = join(
    import.meta.dirname,
    '..',
    'dist',
    'final',
    'node',
    'tsc-filter.mjs'
  );
  /**
   Output files retain exact wrapper bytes independently of subprocess-library normalization.
   */
  const stdoutPath = join(
    directory,
    'stdout.txt'
  );
  /**
   Error output remains separate from ordinary compiler metadata.
   */
  const stderrPath = join(
    directory,
    'stderr.txt'
  );
  /**
   The child borrows these descriptors until its process has completely exited.
   */
  await using stdoutFile = await open(
    stdoutPath,
    'w'
  );
  /**
   Descriptor disposal follows the completed subprocess and output readback.
   */
  await using stderrFile = await open(
    stderrPath,
    'w'
  );
  /**
   A nonzero wrapper status is observed test data, not a fixture startup failure.
   */
  const outcome = { status: 0 };
  try {
    await spawn(
      process.execPath,
      [
        entry,
        '--version'
      ],
      {
        cwd: directory,
        stdout: stdoutFile.fd,
        stderr: stderrFile.fd,
        env: { PATH: `${directory}${delimiter}${process.env
          .PATH
          ?? ''}` },
        timeout: 10_000,
      }
    );
  } catch (error) {
    if ((!(error instanceof SubprocessError)) || (error.exitCode === undefined)) throw error;
    outcome.status = error.exitCode;
  }
  return {
    status: outcome.status,
    stdout: await readFile(
      stdoutPath,
      'utf8'
    ),
    stderr: await readFile(
      stderrPath,
      'utf8'
    )
  };
}
