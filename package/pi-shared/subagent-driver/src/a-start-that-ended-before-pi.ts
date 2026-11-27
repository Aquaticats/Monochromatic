/**
 The line a start leaves when it ends before Pi is ever started: the store read that masks this machine's secret environment, the
 folder a run gives Pi, and a session this run cannot mask.

 These three endings write no summary, because each stops the run before the record it would summarise exists. What is left is the
 line in the run folder, printed where the ending is printed, so a launcher can say how the start ended. Issue 6556 is why it is
 here: four starts of 2026-11-26 ended on the store read, and the unit's status was all any reader had.

 @module
 */

import { writeAStartThatEndedWithoutARun, } from './a-start-that-ended-without-a-run.ts';
import { SOPS_TIMEOUT_MS, } from '@monochromatic-dev/env-local-json-core/ts/constants.ts';

/**
 Writes the line a start left and prints it, which is what a start that cannot write a summary leaves instead.

 @param runFolder - the run's own folder

 @param start - what the start has to say

 @example
 ```ts
 await leaveTheLine({ runFolder: '/runs/a-1', start, },);
 ```
 */
async function leaveTheLine({
  runFolder,
  start,
}: {
  readonly runFolder: string;
  readonly start: Parameters<typeof writeAStartThatEndedWithoutARun>[0]['start'];
},): Promise<void> {
  /**
   The line, as the run folder keeps it.
   */
  const line = await writeAStartThatEndedWithoutARun({
    runFolder,
    start,
  },);
  // The unit's own record carries the same line: a launcher reads the folder, and a reader of the journal reads this.
  console.error(line,);
}

/**
 Writes the line a start leaves when the checkout's store, which holds this machine's secret environment, cannot be read.

 @param runFolder - the run's own folder

 @param source - what could not be read, as the failure names it

 @param cause - why it could not be read, which never holds a value

 @param waitedMilliseconds - how long the read had been waiting

 @example
 ```ts
 await noteTheStoreThatCannotBeRead({ runFolder: '/runs/a-1', source: 'the .env.local.json of /repo', cause: 'sops timed out', waitedMilliseconds: 15_002, },);
 ```
 */
export async function noteTheStoreThatCannotBeRead({
  runFolder,
  source,
  cause,
  waitedMilliseconds,
}: {
  readonly runFolder: string;
  readonly source: string;
  readonly cause: string;
  readonly waitedMilliseconds: number;
},): Promise<void> {
  await leaveTheLine({
    runFolder,
    start: {
      waits: [{
        wait: `the read of this machine's secret environment, whose ${source} is read through the sops reader this package asks for the store`,
        bound: `${String(SOPS_TIMEOUT_MS,)} ms`,
        waitedMilliseconds,
      },],
      what: `nothing could be read from it, so no value of this machine's secret environment could be masked before it reached a model, and Pi was never started: ${cause}`,
      next: `Start the run again when the host is quieter: the sops decrypt that reads the store is given ${String(
        SOPS_TIMEOUT_MS,
      )} ms by the reader in @monochromatic-dev/env-local-json-core, and a loaded host can spend longer. If it fails again on a quiet host, run \`sops --decrypt --output-type json <the store named above>\` by hand and fix what it says, and check that the age identity the reader resolves (SOPS_AGE_KEY_FILE, or ~/.config/mise/age.txt) is the one the store was encrypted to. doc/troubleshooting/pi-neuralwatt-encrypted-store-timeout.md walks the same read.`,
    },
  },);
}

/**
 Writes the line a start leaves when the folder a run gives Pi for its temporary files cannot be made.

 @param runFolder - the run's own folder

 @param detail - what the operating system answered, which the failure's message holds

 @example
 ```ts
 await noteTheTemporaryFolderThatCannotBeMade({ runFolder: '/runs/a-1', detail: 'EACCES', },);
 ```
 */
export async function noteTheTemporaryFolderThatCannotBeMade({
  runFolder,
  detail,
}: {
  readonly runFolder: string;
  readonly detail: string;
},): Promise<void> {
  await leaveTheLine({
    runFolder,
    start: {
      waits: [{
        wait: 'the folder a run gives Pi for its temporary files, which the driver makes before it starts anything',
        waitedMilliseconds: 0,
      },],
      what: `the folder could not be made, so Pi was never started: ${detail}`,
      next: 'Read the failure named above: it holds what the operating system answered. A folder that cannot be made is a path that is not there, a permission the account does not have, or a disk with no room, and the line names which. Make the folder reachable, or give the run a `--run-folder` its account may write, then start it again.',
    },
  },);
}

/**
 Writes the line a start leaves when the session it was told to resume cannot be masked into this run's copy of it.

 @param runFolder - the run's own folder

 @param session - the session file the caller named

 @param problem - why the session could not be masked

 @example
 ```ts
 await noteTheSessionThatCannotBeMasked({ runFolder: '/runs/a-1', session: '/s/a.jsonl', problem: 'one entry holds a value', },);
 ```
 */
export async function noteTheSessionThatCannotBeMasked({
  runFolder,
  session,
  problem,
}: {
  readonly runFolder: string;
  readonly session: string;
  readonly problem: string;
},): Promise<void> {
  await leaveTheLine({
    runFolder,
    start: {
      waits: [{
        wait: `the masking of the session ${session} into this run's copy of it, which happens before Pi reads the session`,
        waitedMilliseconds: 0,
      },],
      what: `a value of this machine's secret environment could not be taken out of that session, so no value of it would be masked when Pi read it, and Pi was never started: ${problem}`,
      next: 'Read the failure named above, which names the entry that could not be masked, and fix the session file or start the run without `--session`: a session this code cannot mask is one whose values would reach a model.',
    },
  },);
}
