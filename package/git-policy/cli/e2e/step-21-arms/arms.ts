/**
 Lay the file-backed pool step 21's arms add worktrees on, and run the arms themselves in the built test target.

 The arms print one `STEP21` line per thing they did, and this step pulls the whole log back into the run's folder, so a
 run that only returned without performing an arm fails the driver's marker check rather than reporting a green suite.

 @example
 ```ts
 const pulled = await runTheArms({ chosen });
 ```
 */

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

import {
  holdsLineStartingWith,
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import {
  awaitDetachedGuestScript,
  pullGuestLog,
} from '../../bin/mvm-guest-detached-script.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import { startTheScript, } from './start-the-script.ts';
import {
  armLog,
  armTimeoutMilliseconds,
  checkout,
  crateDirectory,
  expectedMarkers,
  finishMarginMilliseconds,
  fixtureRoot,
  guestVariable,
  openZfsPathWithTools,
  openZfsSbin,
  pool,
  poolTimeoutMilliseconds,
  poolVariable,
  rootVariable,
  sbinVariable,
} from './step-21-fixture.ts';

/**
 The pool's own log inside the guest's shared folder.
 */
const poolLog = 'step-21-pool.log';

/**
 The launcher's own log, as the host sees it, for the arm step's detached script.
 */
const armStepLog = 'step-21-arms-step.log';

/**
 Lay the file-backed pool the arms' datasets stand on, which lives only inside this throwaway guest.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the status the script ended with

 @example
 ```ts
 await layThePool({ chosen });
 ```
 */
export async function layThePool({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  /**
   The pool, created only when the guest does not already hold it, so a second run reuses the first one's datasets.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPathWithTools}`,
    `mkdir --parents ${fixtureRoot}`,
    `mkdir --parents ${checkout}`,
    `if ! zpool list -H -o name ${pool} > /dev/null 2>&1; then truncate --size 4G ${fixtureRoot}/pool.img; zpool create -f -O mountpoint=none ${pool} ${fixtureRoot}/pool.img; fi`,
    `zpool list -H -o name,size ${pool}`,
  ].join('\n',);
  await startTheScript({
    chosen,
    script,
    log: poolLog,
    timeoutMilliseconds: poolTimeoutMilliseconds,
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: poolLog,
    finishTimeoutMilliseconds: poolTimeoutMilliseconds + finishMarginMilliseconds,
  },);
}

/**
 Pull a detached script's own log into the run's folder, which is what `mvm`'s own copy does for a driver that names no
 reader of its own.

 @param chosen - the run's guest and log folder

 @param file - the file's name inside the guest's shared folder

 @param log - the pull's own host log

 @returns the pulled log's text

 @example
 ```ts
 await pullTheStepLog({ chosen, file: 'step-21-arms-step.log', log: 'host-pull-step-21-arms-step.log' });
 ```
 */
async function pullTheStepLog({
  chosen,
  file,
  log,
}: {
  readonly chosen: GuestFacts;
  readonly file: string;
  readonly log: string;
},): Promise<string> {
  return await pullGuestLog({
    chosen,
    file,
    log,
  },);
}

/**
 How many lines of a failed run's log the refusal quotes.
 */
const TAIL_LINES = 20;

/**
 What the arms step answered: the status line its script wrote, and the reason it refused when it did.

 @example
 ```ts
 const settled: TheStepSettled = { ended: 'DONE=0', refusal: '' };
 ```
 */
type TheStepSettled = {
  /**
   The script's own status line.
   */
  readonly ended: string;
  /**
   Why the step refused, or empty text when it did not.
   */
  readonly refusal: string;
};

/**
 Run the arms step and read what it answered, or the reason it refused.

 @param chosen - the run's guest and log folder

 @returns the status line and the refusal, one of which is empty

 @example
 ```ts
 await theStepSettled({ chosen });
 ```
 */
async function theStepSettled({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<TheStepSettled> {
  try {
    /**
     The status line the script wrote before it ended.
     */
    const ended = await awaitDetachedGuestScript({
      chosen,
      log: armStepLog,
      finishTimeoutMilliseconds: armTimeoutMilliseconds + finishMarginMilliseconds,
      pull: pullTheStepLog,
    },);
    return {
      ended,
      refusal: '',
    };
  }
  catch (caught) {
    return {
      ended: '',
      refusal: caughtValueText(caught,),
    };
  }
}

/**
 The last lines of a log, which say which arm stopped a run.

 @param text - the log's whole text

 @returns its last lines

 @example
 ```ts
 theLogTail({ text: pulled });
 ```
 */
function theLogTail({
  text,
}: {
  readonly text: string;
},): string {
  return text
    .split('\n',)
    .slice(-TAIL_LINES,)
    .join('\n',);
}

/**
 Run the arms in the guest's built test target and read the lines they performed out of the pulled log.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws GuestDriverError naming the marker the pulled log lacks, or the log that skipped

 @example
 ```ts
 const pulled = await runTheArms({ chosen });
 ```
 */
export async function runTheArms({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The arms themselves: the built test target's step 21 arms, run where the guest's own pool is and never on a machine
   somebody uses.
   */
  const script = [
    'set -eu',
    `export PATH=${openZfsPathWithTools}`,
    `export ${guestVariable}=${chosen.name}`,
    `export ${poolVariable}=${pool}`,
    `export ${sbinVariable}=${openZfsSbin}`,
    `export ${rootVariable}=${checkout}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'if cargo nextest run --release --locked --test native_binary --no-capture --status-level all '
      + `--filterset "test(the_step_21_guest_arms)" > /root/${armLog} 2>&1; then status=0; else status=$?; fi`,
    // The log is copied into the guest's shared folder before the status is tested, so a failed arm's own account of
    // itself reaches the host.
    `cp /root/${armLog} /mnt/shared/${armLog} || true`,
    `tail --lines=200 /root/${armLog}`,
    'test "$status" = 0',
  ].join('\n',);
  await startTheScript({
    chosen,
    script,
    log: armStepLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
  /**
   What the step's own script answered, and why it refused when it did: the arm log is pulled either way, because a failed
   run's account of itself is what a reader needs most.
   */
  const settled: TheStepSettled = await theStepSettled({ chosen, },);
  /**
   The arm's own log, pulled before the refusal so a failed run keeps its account of itself.
   */
  const pulled: string = await pullGuestLog({
    chosen,
    file: armLog,
    log: `host-pull-${armLog}`,
  },);
  if (settled.refusal !== '') {
    throw new Error(`${settled.refusal}; the arm log ends with:\n${theLogTail({ text: pulled, },)}`,);
  }
  requireMarkers({
    text: pulled,
    markers: expectedMarkers,
    subject: 'the pulled arm log',
    because: 'the guest ran the arms without performing them',
  },);
  if (holdsLineStartingWith({
    text: pulled,
    word: 'SKIP',
  },))
    throw new Error('the pulled arm log holds a SKIP line, so the arms did not run',);
  return pulled;
}
