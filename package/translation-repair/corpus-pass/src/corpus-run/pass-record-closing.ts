import type { ClosingReport, } from '@monochromatic-dev/translation-repair-run/ts';
import { recordRunLine, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Pass record closing
// WHAT THE RUN SAYS AT ITS END BELONGS TO ITS RECORD (ledger B1111). The seat report (`SEAT ...`, `SEATS DARK:`) and the
// end reports (NeuralWatt's plan energy) are printed by the command when the pass returns, which is after the pass's
// run log is closed. The pass takes the same lines from the command before the log closes and records them, and the
// command prints them as it always did: they are built once (`ClosingReport`), so a provider is asked for a reading once.

/**
 Records the closing lines of the run in the installed run log when the scope ends, and prints nothing.

 @param closing - the lines the command says at its end, absent where nothing prints them

 @returns Async disposable recording the lines on dispose, so it is declared after the run log and disposed before it

 @example
 ```ts
 await using _closing = recordingClosing({ closing: input.closing, },);
 ```
 */
export function recordingClosing({ closing, }: { readonly closing?: ClosingReport; },): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      if (closing === undefined)
        return;
      for (const line of await closing.lines()) {
        recordRunLine({
          level: 'info',
          line,
        },);
      }
    },
  };
}

//endregion Pass record closing
