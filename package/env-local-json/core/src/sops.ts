/**
 Sops process execution: decryption, stdin value setting, and the interactive
 edit subprocess.

 Nothing here forwards sops stdout or stderr into an error or log; failures
 carry only a class, a numeric exit code, and the store path.

 @module
 */

import {
  execFile,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  MAX_SOPS_OUTPUT_BYTES,
  SOPS_TIMEOUT_MS,
} from './constants.ts';
import {
  EnvLocalJsonDecryptError,
  EnvLocalJsonWriteError,
  type EnvLocalJsonDecryptFailureKind,
  type EnvLocalJsonWriteFailureReason,
} from './errors.ts';

//region Constants

/**
 Logger for subprocess lifecycle decisions; names store paths only.
 */
const sopsLogger = tagged({ tag: 'envLocalJsonSops', },);

//endregion Constants



//region Types

/**
 Outcome of one interactive `sops edit` subprocess.
 */
export type SopsEditOutcome =
  | {
    /**
     Normal exit with a numeric status.
     */
    readonly kind: 'exited';

    /**
     Exit status to pass through to the caller.
     */
    readonly code: number;
  }
    | {
    /**
     Termination by a signal.
     */
      readonly kind: 'signaled';

    /**
     Signal name reported by the child process.
     */
      readonly signal: string;
    };

/**
 Failure class and optional numeric exit code.
 */
type SopsFailure = {
  /**
   Reader-facing failure class.
   */
  readonly kind: EnvLocalJsonDecryptFailureKind;

  /**
   Numeric exit code when sops exited normally.
   */
  readonly exitCode?: number;
};

/**
 Captured stdout state for one sops invocation.
 */
type CapturedOutput = {
  /**
   Text chunks received so far.
   */
  readonly chunks: string[];

  /**
   Number of characters received so far.
   */
  size: number;

  /**
   Whether the output bound was exceeded and the child killed.
   */
  overflow: boolean;
};

//endregion Types

//region Failure classification

/**
 Narrowing shape of a Node child-process failure.

 @param value - rejection value

 @returns `true` for Error values that may carry process fields

 @example
 ```ts
 isProcessError(error);
 ```
 */
function isProcessError(value: unknown,): value is Error & {
  /**
   Exit code or spawn error code.
   */
  readonly code?: number | string;

  /**
   Whether the child was killed after a timeout.
   */
  readonly killed?: boolean;
} {
  return Error.isError(value,);
}

/**
 Explains why a child process terminated without a normal exit status.

 @param timeoutSignal - timeout signal owned by this invocation

 @param signal - caller abort signal, when present

 @returns timeout, caller abort, or generic signal failure

 @example
 ```ts
 terminationFailure({ timeoutSignal });
 ```
 */
function terminationFailure({
  timeoutSignal,
  signal,
}: {
  readonly timeoutSignal: AbortSignal;
  readonly signal?: AbortSignal;
},): SopsFailure {
  if (timeoutSignal.aborted)
    return { kind: 'timeout', };
  if (signal?.aborted === true)
    return { kind: 'aborted', };
  return { kind: 'signal', };
}

/**
 Classifies a child-process failure without reading its message.

 @param error - rejection value

 @param timeoutSignal - timeout signal owned by this invocation

 @param signal - caller abort signal, when present

 @returns failure class and numeric exit code

 @example
 ```ts
 classifyProcessFailure({ error, timeoutSignal });
 ```
 */
function classifyProcessFailure({
  error,
  timeoutSignal,
  signal,
}: {
  readonly error: unknown;
  readonly timeoutSignal: AbortSignal;
  readonly signal?: AbortSignal;
},): SopsFailure {
  if (!isProcessError(error,))
    return { kind: 'unknown', };
  if (error.name === 'AbortError')
    return terminationFailure({
      ...(signal === undefined ? {} : { signal, }),
      timeoutSignal,
    },);
  if (error.code === 'ERR_CHILD_PROCESS_STDIO_MAXBUFFER')
    return { kind: 'output-limit', };
  if (error.killed === true)
    return terminationFailure({
      ...(signal === undefined ? {} : { signal, }),
      timeoutSignal,
    },);
  if ((typeof error.code) === 'number')
    return {
      kind: 'exit',
      exitCode: error.code,
    };
  if ((typeof error.code) === 'string')
    return { kind: 'spawn', };
  return { kind: 'unknown', };
}

/**
 Combines the caller's abort signal with the per-invocation timeout.

 @param signal - caller abort signal, when present

 @param timeoutSignal - timeout signal owned by this invocation

 @returns abort signal covering both cancellation sources

 @example
 ```ts
 const combined = combineSignals({ timeoutSignal });
 ```
 */
function combineSignals({
  signal,
  timeoutSignal,
}: {
  readonly signal?: AbortSignal;
  readonly timeoutSignal: AbortSignal;
},): AbortSignal {
  return signal === undefined
    ? timeoutSignal
    : AbortSignal.any([
      signal,
      timeoutSignal,
    ],);
}

/**
 Extracts the reported exit code from an `exit` event tuple.

 @param exitEvent - raw event arguments

 @returns numeric exit code, or `undefined` when the child was signaled

 @example
 ```ts
 readExitCode({ exitEvent });
 ```
 */
function readExitCode({ exitEvent, }: { readonly exitEvent: unknown; },): unknown {
  return Array.isArray(exitEvent,) ? exitEvent[0] : undefined;
}

/**
 Extracts the reported terminating signal from an `exit` event tuple.

 @param exitEvent - raw event arguments

 @returns signal name, or `undefined` when the child exited normally

 @example
 ```ts
 readExitSignal({ exitEvent });
 ```
 */
function readExitSignal({ exitEvent, }: { readonly exitEvent: unknown; },): unknown {
  return Array.isArray(exitEvent,) ? exitEvent[1] : undefined;
}

//endregion Failure classification

//region Decryption

/**
 Captures one `sops --decrypt --output-type json` invocation.

 @param sopsPath - resolved sops executable

 @param storePath - encrypted store path

 @param childEnv - curated sops child environment

 @param signal - caller abort signal

 @returns decrypted JSON text

 @throws EnvLocalJsonDecryptError when sops fails, times out, is aborted, or
 exceeds the output bound

 @example
 ```ts
 const json = await runSopsDecrypt({ sopsPath, storePath, childEnv });
 ```
 */
export async function runSopsDecrypt({
  sopsPath,
  storePath,
  childEnv,
  signal,
}: {
  readonly sopsPath: string;
  readonly storePath: string;
  readonly childEnv: Readonly<Record<string, string>>;
  readonly signal?: AbortSignal;
},): Promise<string> {
  /**
   Timeout signal owned by this sops invocation.
   */
  const timeoutSignal = AbortSignal.timeout(SOPS_TIMEOUT_MS,);
  /**
   Stdout capture accumulated while sops runs.
   */
  const captured: CapturedOutput = {
    chunks: [],
    overflow: false,
    size: 0,
  };
  try {
    /**
     Decryption child process with captured stdout.
     */
    const child = execFile(
      sopsPath,
      [
        '--decrypt',
        '--output-type',
        'json',
        storePath,
      ],
      {
        encoding: 'utf8',
        env: childEnv,
        maxBuffer: MAX_SOPS_OUTPUT_BYTES,
        timeout: SOPS_TIMEOUT_MS,
        ...(signal === undefined ? {} : { signal, }),
      },
    );
    child.stdout
      ?.setEncoding('utf8',);
    child.stdout
      ?.on(
      'data',
      function captureChunk(chunk: string,): void {
        captured.chunks
          .push(chunk,);
        captured.size += chunk.length;
        if (captured.size > MAX_SOPS_OUTPUT_BYTES) {
          captured.overflow = true;
          child.kill();
        }
      },
      );
    /**
     Raw exit event of the sops child process.
     */
    const exitEvent: unknown = await once(
      child,
      'exit',
    );
    /**
     Reported exit code from the event tuple.
     */
    const code = readExitCode({ exitEvent, });
    if (captured.overflow) {
      throw new EnvLocalJsonDecryptError({
        path: storePath,
        kind: 'output-limit',
      },);
    }
    if ((typeof code) !== 'number') {
      throw new EnvLocalJsonDecryptError({
        path: storePath,
        ...terminationFailure({
          ...(signal === undefined ? {} : { signal, }),
          timeoutSignal,
        },),
      },);
    }
    if (code !== 0) {
      throw new EnvLocalJsonDecryptError({
        path: storePath,
        kind: 'exit',
        exitCode: code,
      },);
    }
    return captured.chunks
      .join('',);
  }
  catch (error) {
    if (error instanceof EnvLocalJsonDecryptError)
      throw error;
    /**
     Failure class derived from the rejected child-process operation.
     */
    const failure = classifyProcessFailure({
      error,
      ...(signal === undefined ? {} : { signal, }),
      timeoutSignal,
    },);
    sopsLogger.debug(
      `sops decrypt failed for ${storePath}: ${failure.kind}${
        failure.exitCode === undefined ? '' : ` ${String(failure.exitCode,)}`
      }`,
    );
    throw new EnvLocalJsonDecryptError({
      path: storePath,
      ...failure,
    },);
  }
}

//endregion Decryption

//region Value setting

/**
 Sets one value in the encrypted store through stdin.

 The JSON-encoded value travels on stdin, never argv, so the process list
 never shows it. sops output is discarded and never forwarded.

 @param sopsPath - resolved sops executable

 @param storePath - encrypted store path to rewrite

 @param key - environment variable name to set

 @param value - string value to encrypt

 @param childEnv - curated sops child environment

 @throws EnvLocalJsonWriteError when sops fails, times out, or cannot start

 @example
 ```ts
 await runSopsSet({ sopsPath, storePath, key: 'A', value: 'x', childEnv });
 ```
 */
export async function runSopsSet({
  sopsPath,
  storePath,
  key,
  value,
  childEnv,
}: {
  readonly sopsPath: string;
  readonly storePath: string;
  readonly key: string;
  readonly value: string;
  readonly childEnv: Readonly<Record<string, string>>;
},): Promise<void> {
  /**
   Timeout signal owned by this one sops invocation.
   */
  const timeoutSignal = AbortSignal.timeout(SOPS_TIMEOUT_MS,);
  try {
    /**
     sops `set` child process; its output is never surfaced.
     */
    const child = spawn(
      sopsPath,
      [
        'set',
        '--value-stdin',
        storePath,
        `[${JSON.stringify(key,)}]`,
      ],
      {
        env: childEnv,
        signal: timeoutSignal,
        stdio: [
          'pipe',
          'ignore',
          'ignore',
        ],
      },
    );
    child.stdin
      .on(
      'error',
      function ignoreStdinError(): void {
        // A child that exits before reading stdin reports EPIPE; the exit
        // status below is the authoritative failure signal.
      },
      );
    child.stdin
      .end(JSON.stringify(value,),);
    /**
     Raw exit event of the sops child process.
     */
    const exitEvent: unknown = await once(
      child,
      'exit',
    );
    /**
     Reported exit code from the event tuple.
     */
    const code = readExitCode({ exitEvent, });
    if ((typeof code) !== 'number') {
      throw new EnvLocalJsonWriteError({
        key,
        path: storePath,
        reason: timeoutSignal.aborted ? 'timeout' : 'terminated',
      },);
    }
    if (code !== 0) {
      throw new EnvLocalJsonWriteError({
        key,
        path: storePath,
        reason: 'sops-exit',
      },);
    }
  }
  catch (error) {
    if (error instanceof EnvLocalJsonWriteError)
      throw error;
    /**
     Failure class derived from the rejected child-process operation.
     */
    const failure = classifyProcessFailure({
      error,
      timeoutSignal,
    },);
    sopsLogger.debug(
      `sops set failed for ${storePath} key ${key}: ${failure.kind}`,
    );
    throw new EnvLocalJsonWriteError({
      key,
      path: storePath,
      reason: writeFailureReason({
        failure,
        timeoutSignal,
      },),
    },);
  }
}

/**
 Maps a process failure to the writer's failure reason union.

 @param failure - classified process failure

 @param timeoutSignal - timeout signal owned by this invocation

 @returns writer failure reason

 @example
 ```ts
 writeFailureReason({ failure, timeoutSignal });
 ```
 */
function writeFailureReason({
  failure,
  timeoutSignal,
}: {
  readonly failure: SopsFailure;
  readonly timeoutSignal: AbortSignal;
},): EnvLocalJsonWriteFailureReason {
  if (failure.kind === 'timeout')
    return 'timeout';
  if (failure.kind === 'output-limit')
    return 'output-limit';
  if ((failure.kind === 'spawn') || (failure.kind === 'aborted'))
    return 'spawn';
  if (timeoutSignal.aborted)
    return 'timeout';
  return 'sops-exit';
}

//endregion Value setting

//region Interactive editing

/**
 Runs `sops edit` with inherited stdio for the CLI subcommand.

 @param sopsPath - resolved sops executable

 @param storePath - encrypted store to edit

 @param childEnv - curated sops child environment

 @returns exit status to pass through, or the terminating signal

 @example
 ```ts
 const outcome = await runSopsEdit({ sopsPath, storePath, childEnv });
 ```
 */
export async function runSopsEdit({
  sopsPath,
  storePath,
  childEnv,
}: {
  readonly sopsPath: string;
  readonly storePath: string;
  readonly childEnv: Readonly<Record<string, string>>;
},): Promise<SopsEditOutcome> {
  /**
   Interactive sops child process attached to the caller's terminal.
   */
  const child = spawn(
    sopsPath,
    [
      'edit',
      storePath,
    ],
    {
      env: childEnv,
      stdio: 'inherit',
    },
  );
  /**
   Raw exit event of the interactive editor session.
   */
  const exitEvent: unknown = await once(
    child,
    'exit',
  );
  /**
   Reported exit code from the event tuple.
   */
  const code = readExitCode({ exitEvent, });
  /**
   Reported terminating signal from the event tuple.
   */
  const signal = readExitSignal({ exitEvent, });
  if ((typeof code) === 'number')
    return {
      kind: 'exited',
      code,
    };
  if ((typeof signal) === 'string')
    return {
      kind: 'signaled',
      signal,
    };
  return {
    kind: 'exited',
    code: 1,
  };
}

//endregion Interactive editing
