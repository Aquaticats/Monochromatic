/**
 The exit path the three gate commands share: a refusal prints its own sentence and nothing else, and an unexpected error
 keeps its stack.

 A coordinator reads these commands' output beside a gate log, so a refusal names the input that was refused and what to
 do next, and a defect of this package is the only thing that prints a stack trace.
 */
import { NativeGateError } from './native-gate-error.ts';

/**
 Exit status of a refusal a person can act on.
 */
export const gateRefusalExitCode = 1;

/**
 Prints one line of a gate command's own output, which is the product here and not a log record.

 @param line - the line to print

 @example
 ```ts
 printLine('gpc-gate-a.service=/home/a/temp/agent/gate/gpc-gate-a.log');
 ```
 */
export function printLine(line: string): void {
  console.log(line);
}

/**
 Runs one gate command's main and turns a refusal into an exit status.

 @param main - the command's work, answering the status it earns

 @param report - writes one line of a refusal, which defaults to standard error

 @returns the status `main` answered, or {@link gateRefusalExitCode} for a refusal

 @throws the caught value when it is not a refusal, so a defect never reads as a gate that refused

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
export async function runGateMain({
  main,
  report = console.error,
}: {
  readonly main: () => Promise<number>;
  readonly report?: (text: string) => void;
}): Promise<number> {
  try {
    return await main();
  }
  catch (error) {
    if (!(error instanceof NativeGateError))
      throw error;
    report(error.message);
    return gateRefusalExitCode;
  }
}
