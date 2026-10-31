import { usageExitCode } from './constants.ts';
import { CrateReleaseError } from './error.ts';
import { wantsHelp } from './option-parsing.ts';

/**
 Run one command of this package with the shared handling of help, missing arguments and refused input.

 @param argv - arguments after the script path

 @param usage - usage text of the command

 @param execute - runs the command on the arguments and returns its exit status; it throws `CrateReleaseError` for
 input it refuses

 @param print - prints a line to standard output

 @param printError - prints a line to standard error

 @returns the exit status: zero for help, the usage status for no arguments or refused input, else what `execute`
 returned

 @throws the error `execute` throws when it is not a `CrateReleaseError`, because that is a defect, not bad input

 @example
 ```ts
 process.exitCode = await runCli({ argv: process.argv.slice(2), usage, execute, print: console.log, printError: console.error });
 ```
 */
export async function runCli({
  argv,
  usage,
  execute,
  print,
  printError,
}: {
  readonly argv: readonly string[];
  readonly usage: string;
  readonly execute: (argv: readonly string[]) => Promise<number>;
  readonly print: (text: string) => void;
  readonly printError: (text: string) => void;
}): Promise<number> {
  if (wantsHelp(argv)) {
    print(usage);
    return 0;
  }
  if (argv.length === 0) {
    printError(usage);
    return usageExitCode;
  }
  try {
    return await execute(argv);
  }
  catch (error) {
    if (!(error instanceof CrateReleaseError))
      throw error;
    printError(error.message);
    return usageExitCode;
  }
}
