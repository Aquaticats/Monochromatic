import { spawn, } from 'node:child_process';
import {
  addAbortListener,
  once,
} from 'node:events';

//region Real helper process

/**
 Maximum lifetime of a fixture child, not a production request deadline.
 */
const FIXTURE_DEADLINE_MS = 10_000;

/**
 Runs production argv and captures both streams, including lifecycle shutdown noise.

 @param command - detached terminal command without shell parsing

 @param onOutput - optional fixture trigger, such as cancelling when an editor starts

 @returns complete output after a successful helper exit

 @throws when the helper fails or exceeds the fixture deadline

 @example
 ```ts
 const output = await runHelperCommand({ command });
 ```
 */
export async function runHelperCommand({
  command,
  onOutput,
}: {
  readonly command: readonly string[];
  readonly onOutput?: (output: string) => void;
},): Promise<{
  stdout: string;
  stderr: string
}> {
  /**
   Runtime and independent argument tokens supplied by the requester.
   */
  const [executable, ...args] = command;
  if (executable === undefined)
    throw new Error('Missing detached helper command.',);
  /**
   Child cannot inherit terminal input or access the test runner's stdin.
   */
  const child = spawn(
    executable,
    args,
    { stdio: [
      'ignore',
      'pipe',
      'pipe',
    ], },
  );
  /**
   Test owns and terminates its process on every outcome.
   */
  using cleanup = {
    [Symbol.dispose](): void {
      child.kill();
    },
  };
  /**
   Bounds only the fixture, never production editing or desktop startup.
   */
  const deadline = AbortSignal.timeout(FIXTURE_DEADLINE_MS,);
  /**
   Terminate a hung fixture without leaving a child after assertion failure.
   */
  using abortSubscription = addAbortListener(
    deadline,
    function stopTimedOutFixture(): void {
      child.kill();
    },
  );
  /**
   Retained streams distinguish clean cancellation from a swallowed child error.
   */
  const output = {
    stdout: '',
    stderr: '',
  };
  child.stdout
    .setEncoding('utf8',);
  child.stderr
    .setEncoding('utf8',);
  child.stdout
    .on(
    'data',
    function captureOutput(chunk: string,): void {
      output.stdout += chunk;
      onOutput?.(output.stdout,);
    },
    );
  child.stderr
    .on(
    'data',
    function captureError(chunk: string,): void {
      output.stderr += chunk;
    },
    );
  /**
   Close drains both streams before evaluating exit status.
   */
  const exit: readonly unknown[] = await once(
    child,
    'close',
  );
  deadline.throwIfAborted();
  /**
   Exit code narrowed independently of the event helper's untyped tuple.
   */
  const [code,] = exit;
  if (code !== 0)
    throw new Error(`Detached helper exited ${String(code,)}:\n${output.stderr}`,);
  return output;
}

//endregion Real helper process
