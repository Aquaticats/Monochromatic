/**
 Build this crate's handler inside the guest from a snapshot of the worktree, so
 the arm compiles against the path crates the wrapper links rather than against a
 copy somebody remembered to add.

 Two builds happen here, because two arms need them: the release the shipped
 modes run, and the same build with the probe's raised-deadline feature, which is
 the only build the one control whose point is a step that finishes after the
 shipped bound may run.

 The build runs as a detached script whose end this host reads from the log the
 script appends its status to (`bin/mvm-guest-detached-script.ts`): a guest build
 runs for seven minutes, and the guest agent has lost the result of a long guest
 command (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`).
 The guest's own two `cargo build` lines, the order they run in, their options and
 the log each writes are the ones this step always sent; only how the command is
 started and how its end is read changed, which is the shape
 `e2e/adoption-guest-arms/detached-step.ts` has.

 This driver's launcher carries the guest-name check, so a command that reached
 another machine's guest refuses before the step file is started at all: a check
 inside the step file is one the step file runs on and then compiles anyway.

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */

import {
  join,
} from 'node:path';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import {
  GuestDriverError,
  refusalOf,
} from '../../bin/mvm-guest-error.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
} from '../../bin/mvm-guest-host-command.ts';
import { readLog, } from '../../bin/mvm-guest-step-command.ts';


import {
  buildFinishMarginMilliseconds,
  crateBuildTimeoutMilliseconds,
  crateDirectory,
  guestBuildEndOf,
} from './probe-fixture.ts';

/**
 The log the guest's build writes inside the run's log folder, which is also the
 name the guest's own shared-folder log and the two launcher files carry.
 */
const buildLog = 'guest-crate-build.log';

/**
 The refusal a guest build whose wait refused is reported with: its own status and
 the tail of its own log.

 @param text - the build's own log, as the wait's pull left it on this host

 @param caught - what the wait for the script threw

 @returns the error to throw: the shared refusal when the log names a zero status,
 and the build's own answer otherwise

 @example
 ```ts
 throw refusalFromTheLog({ text: pulled, caught });
 ```
 */
function refusalFromTheLog({
  text,
  caught,
}: {
  readonly text: string;
  readonly caught: unknown;
},): Error {
  /**
   How the build ended, as its log says.
   */
  const ended = guestBuildEndOf({ text, },);
  if (!ended.failed)
    return refusalOf({ caught, },);
  /**
   The status the guest reported, and the tail of the log beside it.
   */
  const {
    status,
    tail,
  } = ended;
  return new GuestDriverError(
    `${buildLog}: the guest build ended ${status}; the last ${String(tail.length,)} line(s) of its own log are:\n${tail.join('\n',)}`,
    { cause: caught, },
  );
}

/**
 The error to throw for a guest build whose wait refused, read from the log that wait
 pulled onto this host.

 A host copy this account cannot read at all leaves the wait's own refusal as the
 only answer there is, said once beside the read's failure.

 @param chosen - the run's guest and log folder

 @param caught - what the wait for the script threw

 @returns the error to throw

 @example
 ```ts
 throw await refusalAfterTheWait({ chosen, caught });
 ```
 */
async function refusalAfterTheWait({
  chosen,
  caught,
}: {
  readonly chosen: GuestFacts;
  readonly caught: unknown;
},): Promise<Error> {
  try {
    /**
     The build's own log, as the wait's pull left it on this host.
     */
    const text = await readLog({ path: join(
      chosen.logDir,
      buildLog,
    ), },);
    return refusalFromTheLog({
      text,
      caught,
    },);
  }
  catch (unreadable) {
    console.error(`${buildLog}: the host copy could not be read back after the wait refused: ${caughtValueText(unreadable,)}`,);
    return refusalOf({ caught, },);
  }
}

/**
 Push this checkout into the guest and build the handler there.

 @param chosen - the run's guest and log folder

 @throws Error naming the guest's status, with the tail of the build's own log

 @example
 ```ts
 await buildInGuest({ chosen });
 ```
 */
export async function buildInGuest({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  /**
   The guest's two builds: the release the modes run, and the raised-deadline one
   into a target folder of its own, so neither replaces the other's bytes.
   */
  const script = [
    'set -eu',
    'if test -f /mnt/shared/subject.tar.gz; then cp /mnt/shared/subject.tar.gz /root/subject.tar.gz; fi',
    `rm --recursive --force ${crateDirectory}`,
    `mkdir --parents ${crateDirectory}`,
    `tar --extract --gzip --file /root/subject.tar.gz --directory ${crateDirectory}`,
    `cd ${crateDirectory}/package/git-policy/cli`,
    'export PATH=/root/.cargo/bin:$PATH',
    'cargo build --release --locked --features test-only-switches --bins > /root/probe-build.log 2>&1',
    'cargo build --release --locked --bin monochromatic-git-policy-cli --features test-only-switches,start-probe-raised-deadline --target-dir target/raised-deadline > /root/probe-build-raised.log 2>&1',
    'tail --lines=3 /root/probe-build.log',
    'tail --lines=3 /root/probe-build-raised.log',
  ].join('\n',);
  await startDetachedGuestScript({
    chosen,
    script,
    log: buildLog,
    timeoutMilliseconds: crateBuildTimeoutMilliseconds,
    nameCheck: 'launcher',
  },);
  try {
    await awaitDetachedGuestScript({
      chosen,
      log: buildLog,
      finishTimeoutMilliseconds: crateBuildTimeoutMilliseconds + buildFinishMarginMilliseconds,
    },);
  }
  catch (caught) {
    throw await refusalAfterTheWait({
      chosen,
      caught,
    },);
  }
}
