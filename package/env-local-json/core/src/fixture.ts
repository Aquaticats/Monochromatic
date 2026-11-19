/**
 Real sops/age fixtures for the env-local-json tests.

 Every fixture lives in a disposable `mktemp` directory with a freshly
 generated age identity. No repository store, credential, or committed key
 material is read or written. A missing `sops` or `age-keygen` fails with the
 install command to run.

 @module
 */

import { Buffer, } from 'node:buffer';
import {
  spawn,
  type ChildProcessByStdio,
} from 'node:child_process';
import { once, } from 'node:events';
import type { Readable, } from 'node:stream';
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
} from 'node:path';
import { randomUUID, } from 'node:crypto';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

//region Constants

/**
 Prefix for every freshly generated fixture directory.
 */
const FIXTURE_PREFIX = 'env-local-json-test-';

/**
 File name of the encrypted store inside a fixture root.
 */
const STORE_FILE_NAME = '.env.local.json';

/**
 File name of the generated age identity inside a fixture root.
 */
const IDENTITY_FILE_NAME = 'identity.txt';

/**
 Marker preceding the public key in an age identity file.
 */
const PUBLIC_KEY_MARKER = 'public key: ';

/**
 Marker starting the comment line that holds the public key.
 */
const PUBLIC_KEY_COMMENT_START = '#';

/**
 Fixture-only deadline, including the time to drain both output streams.
 In the 2026-11-18T21:06:47-05:00 probe on this machine, `age-keygen`, `sops --encrypt`, and `sops --decrypt` with
 disposable fixture inputs took 6 to 21 ms each through process close. Two seconds leaves roughly 95 times
 the slowest observed call for scheduling and slower test hosts, without waiting out the review's four-second holder.
 */
export const FIXTURE_COMMAND_TIMEOUT_MS = 2_000;

/**
 Retained bytes from standard error. The same fixture measurement collected 75 bytes from `age-keygen`, none from
 successful sops commands, and 935 bytes from a sops decrypt with the wrong throwaway identity. Four KiB preserves
 several times that diagnostic text while bounding what a noisy tool can add to every failing test report.
 */
export const MAX_FIXTURE_COMMAND_STDERR_BYTES: number = 4 * (2 ** 10);

/**
 Largest delay accepted by Node's `setTimeout` without reducing it to 1 ms.
 */
const MAX_TIMER_DELAY_MS = (2 ** 31) - 1;

//endregion Constants

//region Types

/**
 Disposable real-toolchain fixture.
 */
export type SopsFixture = {
  /**
   Disposable fixture root.
   */
  readonly root: string;

  /**
   Path of the generated age identity.
   */
  readonly identityPath: string;

  /**
   Public age recipient matching the identity.
   */
  readonly recipient: string;

  /**
   Path of the default encrypted store.
   */
  readonly storePath: string;

  /**
   Values encrypted into the default store.
   */
  readonly values: Readonly<Record<string, string>>;

  /**
   Removes the fixture tree after the test.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Disposable temporary directory owned by one test.
 */
export type TempRoot = {
  /**
   Absolute temporary directory path.
   */
  readonly root: string;

  /**
   Removes the directory tree after the test.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Captured result of one fixture subprocess.
 */
type CommandResult = {
  /**
   Accumulated standard output.
   */
  readonly stdout: string;
};

/**
 Options for {@link runCommand}.
 */
type RunCommandOptions = {
  /**
   Executable to run.
   */
  readonly command: string;

  /**
   Arguments passed without a shell.
   */
  readonly args: readonly string[];

  /**
   Extra environment values layered over the test process environment.
   */
  readonly extraEnv?: Readonly<Record<string, string>>;

  /**
   Maximum time to wait for the tool and all its output, in milliseconds.
   */
  readonly timeoutMilliseconds?: number;
};

/**
 A fixture tool call that did not complete successfully.
 */
export class FixtureCommandError extends Error {
  /**
   Distinguishes fixture failures in test reports.
   */
  override readonly name = 'FixtureCommandError';
}

//endregion Types

//region Command execution

/**
 Runs one real tool and captures its whole standard output.

 The capture ends at the tool's `close` event, never at its `exit` event: a
 tool can end while its standard output pipe still holds unread data, and a
 join there yields empty or cut text that then lands in a fixture store.

 The capture stays here instead of going through `collectChildStdout` in
 `./sops-stdout-capture.ts` because this package's lint refuses a fixture
 module that imports its package's source, and the built artifact that lint
 allows would make every suite importing this fixture need this package built
 first. `fixture.unit.test.ts` guards the event this join waits for.

 Exported so a test can drive a stand-in tool the fixture's own helpers cannot
 name, such as one whose standard output arrives after its exit event.

 @param command - executable to run

 @param args - arguments passed without a shell

 @param extraEnv - extra environment values layered over the process environment

 @param timeoutMilliseconds - deadline for tool execution and pipe closure, defaulting to {@link FIXTURE_COMMAND_TIMEOUT_MS}

 @returns captured standard output

 @throws {@link FixtureCommandError} if the deadline is invalid, the tool cannot start, either output cannot be read,
 the tool exits non-zero or by signal, or the deadline passes. Failures may include bounded standard error, never
 standard output or the inherited environment.

 @example
 ```ts
 const result = await runCommand({ command: 'age-keygen', args: ['--version'] });
 ```
 */
export async function runCommand({
  command,
  args,
  extraEnv,
  timeoutMilliseconds = FIXTURE_COMMAND_TIMEOUT_MS,
}: RunCommandOptions,): Promise<CommandResult> {
  /**
   Tool and argument boundaries as passed to spawn, not a shell command.
   */
  const invocation = JSON.stringify([
    command,
    ...args,
  ],);
  if ((!Number.isInteger(timeoutMilliseconds,)) || (timeoutMilliseconds < 1)
    || (timeoutMilliseconds > MAX_TIMER_DELAY_MS)) {
    throw new FixtureCommandError(
      `Fixture command ${invocation} has an invalid deadline (${String(timeoutMilliseconds,)} ms). `
        + `Use a whole number of milliseconds from 1 to ${String(MAX_TIMER_DELAY_MS,)}.`,
    );
  }

  /**
   Standard output chunks, kept separate from diagnostics because they can contain plaintext.
   */
  const captured: string[] = [];
  /**
   Prefix of standard error, measured before decoding so the limit is in bytes.
   */
  const diagnostic = {
    bytes: Buffer.alloc(MAX_FIXTURE_COMMAND_STDERR_BYTES,),
    length: 0,
    truncated: false,
  };

  /**
   Names the invocation and attaches only the retained tool diagnostic.

   @param explanation - result and what to do about it

   @param cause - underlying process or stream error, when available

   @returns failure ready to throw or reject
   */
  function failure({
    explanation,
    cause,
  }: {
    readonly explanation: string;
    readonly cause?: unknown;
  },): FixtureCommandError {
    /**
     Visible warning when the captured diagnostic is only a prefix.
     */
    const note = diagnostic.truncated
      ? `\n[Standard error truncated after ${String(MAX_FIXTURE_COMMAND_STDERR_BYTES,)} bytes]`
      : '';
    /**
     Diagnostic omitted entirely when the tool wrote nothing to standard error.
     */
    const detail = diagnostic.length === 0
      ? ''
      : `\nStandard error:\n${diagnostic.bytes
        .toString(
          'utf8',
          0,
          diagnostic.length,
        )}${note}`;
    return new FixtureCommandError(
      `Fixture command ${invocation} ${explanation}${detail}`,
      { cause, },
    );
  }

  /**
   Distinguishes a missing executable or interpreter from other launch failures.

   @param error - cause reported by Node

   @returns failure with remediation specific to the launch attempt
   */
  function launchFailure(error: unknown,): FixtureCommandError {
    /**
     A missing executable or interpreter has different remediation from a denied or invalid launch.
     */
    const missing = Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT');
    /**
     Installation guidance applies only to the fixture's own tools.
     */
    const install = (command === 'sops') || (command === 'age-keygen')
      ? " If the fixture tools are not installed, run 'mise install sops age'."
      : '';
    /**
     Actions that may fix the observed launch failure.
     */
    const advice = missing
      ? `Check the executable path or PATH and any interpreter the tool requires.${install} Then retry.`
      : 'Check that the tool is executable and the arguments are valid, then retry.';
    return failure({
      explanation: `could not start: ${caughtValueText(error,)}. ${advice}`,
      cause: error,
    },);
  }

  /**
   Starts the child, preserving the same error contract for synchronous launch failures.

   @returns child from which the runner reads both streams

   @throws {@link FixtureCommandError} when Node rejects the launch before starting the tool
   */
  function startTool(): ChildProcessByStdio<null, Readable, Readable> {
    try {
      return spawn(
        command,
        [...args,],
        {
          env: {
            ...process.env,
            ...extraEnv,
          },
          stdio: [
            'ignore',
            'pipe',
            'pipe',
          ],
        },
      );
    }
    catch (error) {
      throw launchFailure(error,);
    }
  }

  /**
   Child to wait on and stop if necessary.
   */
  const child = startTool();
  /**
   Pipes owned by the runner rather than inherited by the test process.
   */
  const {
    stdout,
    stderr,
  } = child;
  /**
   Interrupts the close wait on an expired deadline or failed pipe read.
   */
  const stopWaiting = new AbortController();

  /**
   Checks whether the direct child has reported any kind of exit.

   @returns whether stopping the direct child could still affect it
   */
  function isRunning(): boolean {
    return (child.exitCode === null) && (child.signalCode === null);
  }

  /**
   Records failed pipe reads as command failures instead of allowing an uncaught stream error.

   @param pipeName - stream named in diagnostic

   @returns error listener for the named stream
   */
  function pipeErrorListener(pipeName: string,): (error: Error) => void {
    return function reportPipeError(error: Error,): void {
      stopWaiting.abort(failure({
        cause: error,
        explanation: `could not read ${pipeName}: ${caughtValueText(error,)}. Check the tool and retry.`,
      },),);
    };
  }

  stdout?.on(
    'error',
    pipeErrorListener('standard output',),
  );
  stderr?.on(
    'error',
    pipeErrorListener('standard error',),
  );
  stdout?.setEncoding('utf8',);
  stdout?.on(
    'data',
    function captureOutput(chunk: string,): void {
      captured.push(chunk,);
    },
  );
  stderr?.on(
    'data',
    function captureDiagnostic(chunk: Buffer,): void {
    /**
     Bytes that fit; keep draining after the cap so the writer cannot become blocked.
     */
      const copied = chunk.copy(
        diagnostic.bytes,
        diagnostic.length,
      );
      diagnostic.length += copied;
      if (copied < chunk.length)
        diagnostic.truncated = true;
    },
  );

  /**
   Disposed on every return or throw, including a failed launch.
   */
  using timer = setTimeout(
    function reportDeadline(): void {
      /**
       The direct child can have exited while a helper still owns either pipe.
       */
      const detail = isRunning()
        ? 'Check why the tool did not finish.'
        : 'The tool exited, but its output did not close. Check whether a helper still holds its output open.';
      stopWaiting.abort(failure({
        explanation: `did not complete within ${String(timeoutMilliseconds,)} ms. ${detail} `
          + 'If the delay is expected, increase timeoutMilliseconds for this test and retry.',
      },),);
    },
    timeoutMilliseconds,
  );
  try {
    await once(
      child,
      'close',
      { signal: stopWaiting.signal, },
    );
  }
  catch (error) {
    /**
     The abort reason is our deadline or pipe error; otherwise `once` rejected with the child's launch error.
     */
    const reason: unknown = stopWaiting.signal
      .reason;
    /**
     Report the original failure even when stopping the child runs into a problem of its own.
     */
    const reported = reason instanceof FixtureCommandError ? reason : launchFailure(error,);
    stdout?.destroy();
    stderr?.destroy();
    if ((child.pid !== undefined) && isRunning()) {
      child.unref();
      try {
        child.kill('SIGKILL',);
      }
      catch (killError) {
        throw new FixtureCommandError(
          `${reported.message}\nCould not stop the tool: ${caughtValueText(killError,)}. Check whether it is still running.`,
          { cause: reported, },
        );
      }
    }
    throw reported;
  }
  if (child.signalCode !== null) {
    throw failure({
      explanation: `was terminated by ${child.signalCode}. Check what sent the signal before retrying.`,
    },);
  }
  if (child.exitCode !== 0) {
    throw failure({
      explanation: `exited with status ${String(child.exitCode,)}. Check the tool's diagnostic and fixture inputs, then retry.`,
    },);
  }
  return { stdout: captured.join('',), };
}

//endregion Command execution

//region Fixture helpers

/**
 Extracts the age public key from an identity file's comment line.

 @param identityText - full text of the generated identity file

 @returns age recipient string

 @throws Error when the identity file has no public-key comment

 @example
 ```ts
 const recipient = publicKeyFromIdentity({ identityText });
 ```
 */
function publicKeyFromIdentity({
  identityText,
}: {
  readonly identityText: string;
},): string {
  /**
   Comment line holding the public key, or the whole file when absent.
   */
  const lines = identityText.split('\n',);
  for (const line of lines) {
    if (!line.startsWith(PUBLIC_KEY_COMMENT_START,))
      continue;
    /**
     Offset of the public-key marker on this comment line.
     */
    const markerStart = line.indexOf(PUBLIC_KEY_MARKER,);
    if (markerStart === (-1))
      continue;
    return line.slice(markerStart + PUBLIC_KEY_MARKER.length,)
      .trim();
  }
  throw new Error('generated age identity has no public-key comment',);
}

/**
 Creates one disposable temporary directory.

 @param prefix - directory-name prefix

 @returns disposable temporary root

 @example
 ```ts
 await using temp = await createTempRoot({ prefix: 'env-local-json-' });
 ```
 */
export async function createTempRoot({
  prefix,
}: {
  readonly prefix?: string;
} = {},): Promise<TempRoot> {
  /**
   Fresh temporary directory.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    prefix ?? FIXTURE_PREFIX,
  ),);
  await chmod(
    root,
    0o700,
  );
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 A repository whose link to the 860 drive dangles: a link at `<root>/repolink` to a folder on a drive that is not mounted,
 and a home whose repository link identity file (phase `after`) names that link.

 @param root - disposable root that holds the link, the mount point stand-in and the home

 @returns the link path, the store path through it, and the home

 @example
 ```ts
 const gone = await createRepositoryOnGoneDrive({ root: temp.root });
 ```
 */
export async function createRepositoryOnGoneDrive({
  root,
}: {
  readonly root: string;
},): Promise<{
  readonly home: string;
  readonly link: string;
  readonly store: string;
  readonly mountPoint: string
}> {
  /**
   Stand-in for the mount point of a drive that is not mounted.
   */
  const mountPoint = join(
    root,
    'not-a-mount',
  );
  /**
   Where the link points: a folder that is not there.
   */
  const target = join(
    mountPoint,
    'repository',
    'Monochromatic',
  );
  /**
   The repository's old path.
   */
  const link = join(
    root,
    'repolink',
  );
  /**
   Home that holds the identity file.
   */
  const home = join(
    root,
    'identity-home',
  );
  await mkdir(
    mountPoint,
    { recursive: true, },
  );
  await symlink(
    target,
    link,
  );
  await mkdir(
    join(
      home,
      '.config',
      'monochromatic-dev',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      home,
      '.config',
      'monochromatic-dev',
      'repository-link.json',
    ),
    JSON.stringify({
      linkPath: link,
      linkText: target,
      canonicalRoot: target,
      mountPoint,
      driveUuid: 'DRIVE',
      nvmeUuid: 'NVME',
      rootCommit: 'abc123',
      phase: 'after',
    },),
  );
  return {
    home,
    link,
    mountPoint,
    store: join(
      link,
      '.env.local.json',
    ),
  };
}

/**
 Encrypts one JSON document with the fixture's age recipient.

 @param fixture - fixture owning the recipient and root

 @param json - plaintext JSON document

 @param fileName - store file name inside the fixture root

 @returns path of the encrypted store

 @example
 ```ts
 const storePath = await encryptJson({ fixture, json: '{"A":"x"}', fileName: 'store.json' });
 ```
 */
export async function encryptJson({
  fixture,
  json,
  fileName,
}: {
  readonly fixture: SopsFixture;
  readonly json: string;
  readonly fileName: string;
},): Promise<string> {
  /**
   Temporary plaintext file removed after encryption.
   */
  const plaintextPath = join(
    fixture.root,
    `plaintext-${randomUUID()}.json`,
  );
  /**
   Encrypted output path.
   */
  const storePath = join(
    fixture.root,
    fileName,
  );
  await writeFile(
    plaintextPath,
    json,
    'utf8',
  );
  /**
   Disposable guard that removes the plaintext file on every exit path.
   */
  await using _plaintext = {
    [Symbol.asyncDispose]: async function removePlaintext(): Promise<void> {
      await rm(
        plaintextPath,
        { force: true, },
      );
    },
  };
  /**
   Encryption result carrying the ciphertext on stdout.
   */
  const result = await runCommand({
    args: [
      '--encrypt',
      '--age',
      fixture.recipient,
      '--input-type',
      'json',
      '--output-type',
      'json',
      plaintextPath,
    ],
    command: 'sops',
  },);
  await writeFile(
    storePath,
    result.stdout,
    {
      encoding: 'utf8',
      mode: 0o600,
    },
  );
  return storePath;
}

/**
 Decrypts an encrypted store with the fixture identity.

 @param fixture - fixture owning the identity

 @param storePath - encrypted store path

 @returns validated top-level string values

 @throws Error when the store is not a JSON object of strings

 @example
 ```ts
 const values = await decryptStore({ fixture, storePath: fixture.storePath });
 ```
 */
export async function decryptStore({
  fixture,
  storePath,
}: {
  readonly fixture: SopsFixture;
  readonly storePath: string;
},): Promise<Readonly<Record<string, string>>> {
  /**
   Decryption result carrying plaintext JSON on stdout.
   */
  const result = await runCommand({
    args: [
      '--decrypt',
      '--output-type',
      'json',
      storePath,
    ],
    command: 'sops',
    extraEnv: { SOPS_AGE_KEY_FILE: fixture.identityPath, },
  },);
  /**
   Parsed decrypted document.
   */
  const parsed: unknown = JSON.parse(
    result.stdout,
  );
  if (((typeof parsed) !== 'object') || (parsed === null)
    || Array.isArray(parsed,)) {
    throw new Error('decrypted fixture store is not a JSON object',);
  }
  /**
   Validated string values collected from the document.
   */
  const values: Record<string, string> = {};
  for (const [key, value,] of Object.entries(parsed,)) {
    if ((typeof value) !== 'string')
      throw new Error(`decrypted fixture key ${key} is not a string`,);
    values[key] = value;
  }
  return values;
}

/**
 Creates a disposable fixture with a generated identity and encrypted store.

 @param values - values encrypted into the default store

 @returns disposable fixture

 @throws {@link FixtureCommandError} when sops or age-keygen cannot run successfully

 @example
 ```ts
 await using fixture = await createSopsFixture({ values: { A: 'x' } });
 ```
 */
export async function createSopsFixture({
  values,
}: {
  readonly values: Readonly<Record<string, string>>;
},): Promise<SopsFixture> {
  /**
   Disposable fixture root.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    FIXTURE_PREFIX,
  ),);
  await chmod(
    root,
    0o700,
  );
  /**
   Generated age identity path.
   */
  const identityPath = join(
    root,
    IDENTITY_FILE_NAME,
  );
  await runCommand({
    args: [
      '--output',
      identityPath,
    ],
    command: 'age-keygen',
  },);
  await chmod(
    identityPath,
    0o600,
  );
  /**
   Generated identity text.
   */
  const identityText = await readFile(
    identityPath,
    'utf8',
  );
  /**
   Fixture under construction; the store path is filled in after encryption.
   */
  const fixture: SopsFixture = {
    identityPath,
    recipient: publicKeyFromIdentity({ identityText, },),
    root,
    storePath: join(
      root,
      STORE_FILE_NAME,
    ),
    values,
    [Symbol.asyncDispose]: async function removeFixture(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
  await encryptJson({
    fileName: STORE_FILE_NAME,
    fixture,
    json: JSON.stringify(values,),
  },);
  return fixture;
}

/**
 Generates an additional age identity inside a fixture root.

 @param fixture - fixture owning the root

 @param fileName - identity file name inside the fixture root

 @returns path of the generated identity

 @example
 ```ts
 const wrongIdentity = await createIdentityFile({ fixture, fileName: 'wrong.txt' });
 ```
 */
export async function createIdentityFile({
  fixture,
  fileName,
}: {
  readonly fixture: SopsFixture;
  readonly fileName: string;
},): Promise<string> {
  /**
   Generated identity path.
   */
  const identityPath = join(
    fixture.root,
    fileName,
  );
  await runCommand({
    args: [
      '--output',
      identityPath,
    ],
    command: 'age-keygen',
  },);
  await chmod(
    identityPath,
    0o600,
  );
  return identityPath;
}

/**
 Builds a reader environment that cannot see the developer's identity
 variables or real store configuration.

 @param home - fixture home directory

 @param extra - extra environment values to layer in

 @returns environment ready for `readEnvLocalJson`

 @example
 ```ts
 const env = readerEnvironment({ home: fixture.root });
 ```
 */
export function readerEnvironment({
  home,
  extra,
}: {
  readonly home: string;
  readonly extra?: Readonly<Record<string, string>>;
},): Readonly<Record<string, string>> {
  return {
    PATH: process.env
      .PATH
      ?? '',
    HOME: home,
    TMPDIR: scratchRoot(),
    ...extra,
  };
}

//endregion Fixture helpers
