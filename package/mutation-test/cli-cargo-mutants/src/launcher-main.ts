import { CargoMutantsToolingError } from './error.ts';

/**
 Exit status of a launch that a tooling error stopped.
 */
const toolingErrorExitCode = 1;

/**
 Run a launcher's main function and turn the errors a person caused or can fix into plain output.

 A `CargoMutantsToolingError` carries a message written for the person at the terminal (a bad option, a missing image, a
 refused launch), so only its message is printed and the status is nonzero. Anything else is a defect of the launcher or a
 failure of the host, and keeps its stack trace: the error is thrown on, so Node prints it and exits nonzero.

 @param main - the launch; answers the exit status

 @param report - prints one text to standard error; defaults to `console.error`

 @returns the exit status

 @throws Error whatever `main` threw that is not a `CargoMutantsToolingError`

 @example
 ```ts
 process.exitCode = await runLauncherMain({ main });
 ```
 */
export async function runLauncherMain({
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
    if (!(error instanceof CargoMutantsToolingError))
      throw error;
    report(error.message);
    return toolingErrorExitCode;
  }
}
