/**
 The ID file one gate container's client writes its ID to, and the lines that put that ID in front of the container's own
 output.

 `podman run --cidfile` writes the file once the container exists and before the container's command runs, so a gate that
 waits for the file can print what its container is called, and what it is, before the suite says anything. A run whose
 `--rm` removed the container takes the file with it, so the ID is read while the container exists and the run's evidence
 records it then (`bin/test-native-container.ts`).
 */
import { readFile, } from 'node:fs/promises';

import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';
import { isMissing, } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import {
  type GateContainerIdentity,
  gateContainerIdLineOf,
  gateContainerNoIdLineOf,
  gateContainerNotCreatedLineOf,
} from './native-gate-container-identity.ts';
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 Milliseconds the gate waits for a container's ID to reach the ID file its client was given.

 Podman writes the ID once the container exists and before its command runs, so a wait this long means a podman that has
 stopped answering rather than a slow container: the gate then says a person has the container's name and label alone.
 The bound is the one a removal gets (`bin/native-gate-container.ts`), for the same reason.
 */
export const containerIdDeadlineMilliseconds = 120_000;

/**
 Milliseconds between two looks at a container's ID file.
 */
const containerIdPollMilliseconds = 25;

/**
 What one look for a container's ID found.
 */
export type ContainerIdWait =
  | {
    /**
     Marks an ID the client wrote, which is the container's identity in every listing.
     */
    readonly kind: 'written';
    /**
     The ID without the trailing newline podman writes.
     */
    readonly id: string;
  }
    | {
    /**
     Marks a client that ended before writing an ID, which means it created no container.
     */
      readonly kind: 'client-ended';
    }
    | {
    /**
     Marks a wait that used its whole bound with no ID written.
     */
      readonly kind: 'deadline';
    };

/**
 The ID one container's client wrote to its ID file.

 @param cidFile - file the client writes the container's ID to

 @returns the ID as the only element, or no element while the client has not written one yet, and also after `--rm`
 removed the container and its ID file with it

 @throws NativeVerificationError when the file is there and cannot be read, because a gate that cannot read its own
 evidence must not report an unnamed container instead

 @example
 ```ts
 await readContainerId('target/verification/native-HZPg8O/suite.cid');
 ```
 */
async function readContainerId(cidFile: string,): Promise<readonly string[]> {
  try {
    /**
     The file's text without the trailing newline podman writes.
     */
    const id = (await readFile(
      cidFile,
      'utf8',
    )).trim();
    return id === '' ? [] : [id,];
  }
  catch (error) {
    // A client that has not created its container yet has no ID file, which is not a failure; anything else is.
    if (isMissing(error,))
      return [];
    throw new NativeVerificationError(
      `The ID file ${cidFile} of this gate's container is there and could not be read, so the container cannot be named by its ID: ${String(error)}`,
      { cause: error, },
    );
  }
}

/**
 Wait, bounded, for a container's client to write its ID, and stop waiting the moment that client ends.

 The wait is what puts the ID line in front of the container's own output: podman writes the file before the container's
 command runs, so the gate prints the ID while the container is still starting.

 @param cidFile - file the client writes the container's ID to

 @param ended - the client's settlement, whichever way it ended, so a client that failed or a container that ended at
 once never holds the gate for the whole bound

 @param deadlineMilliseconds - how long to wait before answering that no ID came

 @param wait - waits one interval, which a test replaces with its own

 @returns the ID the client wrote, or that the client ended first, or that the bound passed

 @example
 ```ts
 await containerIdWhenWritten({ cidFile: join(evidence, 'suite.cid'), ended: running });
 ```
 */
export async function containerIdWhenWritten({
  cidFile,
  ended,
  deadlineMilliseconds = containerIdDeadlineMilliseconds,
  wait = waitUntil,
}: {
  readonly cidFile: string;
  readonly ended: Promise<unknown>;
  readonly deadlineMilliseconds?: number;
  readonly wait?: (milliseconds: number) => Promise<unknown>;
},): Promise<ContainerIdWait> {
  /**
   The client's settlement, which never rejects and which the caller still reads as a failure where it awaits the client
   itself.
   */
  const clientSettled = Promise.allSettled([ended,],);
  /**
   When this wait must be over.
   */
  const deadline = Date.now() + deadlineMilliseconds;
  for (;;) {
    /**
     The ID as the file stands now, so a client that wrote one before this look is never missed.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling must read the file before it waits again
    const [written] = await readContainerId(cidFile,);
    if (written !== undefined)
      return {
        kind: 'written',
        id: written,
      };
    /**
     How long is left before this wait must be over.
     */
    const left = deadline - Date.now();
    if (left <= 0)
      return { kind: 'deadline', };
    /**
     The next interval, as a value that names itself so the race below says which of the two ended this wait.
     */
    const interval = (async function waitedAnInterval(): Promise<'waited'> {
      await wait(Math.min(
        containerIdPollMilliseconds,
        left,
      ),);
      return 'waited';
    })();
    /**
     Which ended first: the client, or the interval.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- the client is the other thing that ends this wait, so every interval races it
    const first = await Promise.race([
      clientSettled,
      interval,
    ],);
    // A client that ended wrote its ID just before it did, if it wrote one at all, so this is the last look.
    if (first !== 'waited') {
      /**
       The ID as the file stands after the client ended.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- one last look is what keeps a fast client's ID from being lost
      const [lastWritten] = await readContainerId(cidFile,);
      return lastWritten === undefined
        ? { kind: 'client-ended', }
        : {
          kind: 'written',
          id: lastWritten,
        };
    }
  }
}

/**
 Print one container's ID as soon as its client writes it, and say what is left to identify it by when the client wrote
 none.

 @param identity - the container's name and label

 @param cidFile - file the client writes the container's ID to

 @param running - the client, whose settlement ends the wait for an ID that never arrives

 @param onStarted - tells the caller the ID, so the run's evidence records it, and is awaited so the evidence is on disk
   while the container still runs

 @param deadlineMilliseconds - how long this wait lasts before it says no ID came, which a test shortens

 @example
 ```ts
 await announceContainerId({ identity, cidFile, running, onStarted });
 ```
 */
export async function announceContainerId({
  identity,
  cidFile,
  running,
  onStarted,
  deadlineMilliseconds = containerIdDeadlineMilliseconds,
}: {
  readonly identity: GateContainerIdentity;
  readonly cidFile: string;
  readonly running: Promise<unknown>;
  readonly onStarted?: (id: string) => Promise<void>;
  readonly deadlineMilliseconds?: number;
},): Promise<void> {
  /**
   What the wait for the container's ID found.
   */
  const waited = await containerIdWhenWritten({
    cidFile,
    ended: running,
    deadlineMilliseconds,
  },);
  if (waited.kind === 'client-ended') {
    console.log(gateContainerNotCreatedLineOf({
      identity,
      cidFile,
    },),);
    return;
  }
  if (waited.kind === 'deadline') {
    console.log(gateContainerNoIdLineOf({
      identity,
      cidFile,
      waitedMilliseconds: deadlineMilliseconds,
    },),);
    return;
  }
  console.log(gateContainerIdLineOf({
    identity,
    id: waited.id,
  },),);
  await onStarted?.(waited.id,);
}
