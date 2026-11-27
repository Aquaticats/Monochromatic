/**
 The read of this machine's secret environment, which happens once in the driver's own process before Pi is started.

 A store that is there and cannot be read stops the run there, because the values the rules file bans printing are exactly the ones
 this code would then not know. Nothing of such a run reached Pi, so no summary can be written: the line this module leaves in the run
 folder is what says how the start ended, which is what issue 6556's four silent starts of 2026-11-26 had none of.

 @module
 */

import { noteTheStoreThatCannotBeRead, } from './a-start-that-ended-before-pi.ts';
import type { RecordOfPiRun, } from './driver-types.ts';
import { SecretEnvironmentUnreadableError, } from './secret-environment-unreadable-error.ts';
import type { SecretValuesOfThisMachine, } from './secret-values-of-this-machine.ts';

/**
 Reads the values of this machine's secret environment, closing the run's feed and leaving the line that says how the start ended
 before saying why there is no run when the store cannot be read.

 Nothing of this run reached Pi, so no summary is written: the line is what a reader of the run folder, and a launcher that prints
 it, learn the wait and its bound from.

 @param record - the run's feed and counts, which is taken away by this call when the values cannot be read

 @param runFolder - the run's own folder, where the line is left

 @param read - the read itself

 @returns the values, and how long the read had been waiting when it answered

 @throws SecretEnvironmentUnreadableError when the checkout's store is there and cannot be read

 @example
 ```ts
 const { values, waitedMilliseconds } = await secretValuesOrNoRun({ record, runFolder: '/runs/a-1', read, },);
 ```
 */
export async function secretValuesOrNoRun({
  record,
  runFolder,
  read,
}: {
  readonly record: RecordOfPiRun;
  readonly runFolder: string;
  readonly read: () => Promise<SecretValuesOfThisMachine>;
},): Promise<{
  readonly values: SecretValuesOfThisMachine;
  readonly waitedMilliseconds: number;
}> {
  /**
   When the read began, which is how long the wait had been waiting when it ended.
   */
  const startedAt = Date.now();
  try {
    return {
      values: await read(),
      waitedMilliseconds: Date.now() - startedAt,
    };
  }
  catch (error) {
    /**
     How long the read had waited, which the line this start leaves names.
     */
    const waitedMilliseconds = Date.now() - startedAt;
    await record.close();
    if (error instanceof SecretEnvironmentUnreadableError) {
      await noteTheStoreThatCannotBeRead({
        runFolder,
        source: error.source,
        cause: error.message,
        waitedMilliseconds,
      },);
    }
    throw error;
  }
}
