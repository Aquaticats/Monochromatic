/**
 Run the real Pi SDK and the real Pi command against stand-in members, without inherited credentials,
 user config, or project extensions.
 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { mkdtempDisposable, } from 'node:fs/promises';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 Byte streams one verifier left behind.
 */
type VerifierRun = Readonly<{
  stdout: string;
  stderr: string;
  exitCode: number
}>;

/**
 Temporary HOME and working directory for a credential-free Pi process.
 */
await using home = await mkdtempDisposable(join(
  scratchRoot(),
  'pi-rate-limit-fallback-host-',
),);

/**
 Run one verifier in the disposable home and keep its output.

 @param verifier - file name of the verifier under `src/`

 @returns its standard output, standard error and exit status

 @example
 ```ts
 await runVerifier('verify-pi-session.ts');
 ```
 */
async function runVerifier(verifier: string,): Promise<VerifierRun> {
  /**
   Verifier child with only the disposable home and the parent's Node policy.
   */
  const child = spawn(
    process.execPath,
    [fileURLToPath(new URL(
      verifier,
      import.meta.url,
    ),),],
    {
      cwd: home.path,
      env: {
        PATH: process.env
          .PATH,
        NODE_OPTIONS: process.env
          .NODE_OPTIONS,
        HOME: home.path,
        XDG_CONFIG_HOME: home.path,
        XDG_CACHE_HOME: home.path,
        XDG_DATA_HOME: home.path,
        XDG_RUNTIME_DIR: home.path,
        TMPDIR: home.path,
        PI_OFFLINE: '1',
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Exit status, standard output and standard error, drained together and kept until the end.
   */
  const [
    ,
    stdout,
    stderr,
  ] = await Promise.all([
    once(
      child,
      'close',
    ),
    text(child.stdout,),
    text(child.stderr,),
  ],);
  return {
    stdout,
    stderr,
    exitCode: child.exitCode ?? 1,
  };
}

/**
 Print one verifier's output and fail the task when it did not pass.

 @param verifier - file name of the verifier under `src/`

 @param run - what that verifier wrote and returned

 @example
 ```ts
 report({ verifier: 'verify-pi-session.ts', run: await runVerifier('verify-pi-session.ts') });
 ```
 */
function report({
  verifier,
  run,
}: {
  readonly verifier: string;
  readonly run: VerifierRun;
},): void {
  process.stdout
    .write(run.stdout,);
  process.stderr
    .write(run.stderr,);
  if (run.exitCode !== 0)
    throw new Error(
      `${verifier} did not exit successfully.`,
      {
        cause: {
          exitCode: run.exitCode,
          stderr: run.stderr,
        },
      },
    );
}

/**
 Pi's SDK-driven verifier, which loads the built extension and drives it through a real session.
 */
const SDK_VERIFIER = 'verify-pi-session.ts';

/**
 Pi's command-line verifier, which starts real `pi` processes with the package installed from a
 `packages` list, the way `~/.pi/agent/settings.json` lists it.
 */
const COMMAND_LINE_VERIFIER = 'verify-hybrid-sessions.ts';

report({
  verifier: SDK_VERIFIER,
  run: await runVerifier(SDK_VERIFIER,),
},);
// The command line's verifier writes its own scratch state under the same disposable home, so it
// starts after the SDK verifier's session has closed.
report({
  verifier: COMMAND_LINE_VERIFIER,
  run: await runVerifier(COMMAND_LINE_VERIFIER,),
},);
