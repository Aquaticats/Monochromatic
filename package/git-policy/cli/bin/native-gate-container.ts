/**
 One gate container, run under its own name, label and ID file, so the session that started it can prove which container
 is its own and remove it whatever ended the client that ran it.

 `podman run` keeps its container running when its client is stopped, and the gate stops a client at the deadline it gives
 every check (`bin/native-gate-checks.ts`). The ID file is where the client wrote the container's identity while it ran,
 so the container is removed even then; a run whose `--rm` already removed the container leaves the removal nothing to
 find, and takes the ID file with it.

 The name and label come from this run's own id (`bin/native-gate-container-identity.ts`), and the lines printed here
 name them: several sessions run gates at once, and a session stops only a container carrying its own label. The ID
 itself is read from the client's ID file (`bin/native-gate-container-id-file.ts`).
 */
import { announceContainerId, } from './native-gate-container-id-file.ts';
import {
  type GateContainerIdentity,
  gateContainerArgumentsOf,
  gateContainerLineOf,
} from './native-gate-container-identity.ts';
import {
  type CommandOutcome,
  runCommand,
} from './native-verification-process.ts';

/**
 Milliseconds removing one gate container may take before that removal is stopped.

 A removal answers in well under a second on every recorded gate; the bound is there so a removal that stopped answering
 cannot hold the gate once the container it removes already stopped answering.
 */
export const containerRemovalDeadlineMilliseconds = 120_000;

/**
 Remove one gate container when the scope that started it ends.

 The removal is idempotent: `--ignore` keeps a container that `--rm` already removed from failing it, and `--time 0`
 sends no signal that waits for a container that never stops.

 @param cidFile - file the client writes the container's ID to, under this run's evidence directory

 @returns the removal, run when the caller's scope ends

 @example
 ```ts
 await using removal = containerRemoval({ cidFile: join(evidence, 'suite.cid') });
 ```
 */
function containerRemoval({
  cidFile,
  environment,
}: {
  readonly cidFile: string;
  readonly environment: NodeJS.ProcessEnv;
},): { readonly [Symbol.asyncDispose]: () => Promise<void>; } {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await runCommand({
        command: 'podman',
        args: [
          'rm',
          '--force',
          '--ignore',
          '--time',
          '0',
          '--cidfile',
          cidFile,
        ],
        capture: true,
        allowFailure: true,
        deadlineMilliseconds: containerRemovalDeadlineMilliseconds,
        env: environment,
      },);
    },
  };
}

/**
 Run one gate container under its own name and label, print both and then the container's ID before its own output, and
 remove it, whether it ended by itself or its client was stopped at the deadline.

 @param cidFile - file the client writes the container's ID to, under this run's evidence directory

 @param identity - the name and label every person and every listing proves this container against

 @param args - complete `podman run` arguments after the program name and the container's identity

 @param capture - whether to capture output instead of streaming it

 @param allowFailure - whether a nonzero exit is evidence the caller inspects

 @param deadlineMilliseconds - the bound the gate waits under, which this container's client is given

 @param environment - environment both clients run in, which defaults to this process's

 @param onStarted - receives the container's ID as soon as its client writes it, which is how the run's evidence
   records it even though `--rm` takes the ID file away with the container

 @returns exit state and captured output of the client

 @example
 ```ts
 const suite = await runGateContainer({
   cidFile: join(evidence, 'suite.cid'),
   identity: gateContainerIdentityOf({ runId, role: 'suite' }),
   args: [...limits, image],
   deadlineMilliseconds,
 });
 ```
 */
export async function runGateContainer({
  cidFile,
  identity,
  args,
  capture = false,
  allowFailure = false,
  deadlineMilliseconds,
  environment = process.env,
  onStarted,
}: {
  readonly cidFile: string;
  readonly identity: GateContainerIdentity;
  readonly args: readonly string[];
  readonly capture?: boolean;
  readonly allowFailure?: boolean;
  readonly deadlineMilliseconds: number;
  readonly environment?: NodeJS.ProcessEnv;
  readonly onStarted?: (id: string) => void;
},): Promise<CommandOutcome> {
  /**
   The container's removal, run when this scope ends whatever the client did.
   */
  await using removal = containerRemoval({
    cidFile,
    environment,
  },);
  // The name and label are printed before the client starts, so they reach the log before any of the container's own
  // output and a person reading along can stop the container from the line alone.
  console.log(gateContainerLineOf({
    identity,
    cidFile,
  },),);
  /**
   The client whose container this scope removes whatever ends it.
   */
  const running = runCommand({
    command: 'podman',
    args: [
      'run',
      '--cidfile',
      cidFile,
      ...gateContainerArgumentsOf({ identity, },),
      ...args,
    ],
    capture,
    allowFailure,
    deadlineMilliseconds,
    env: environment,
  },);
  await announceContainerId({
    identity,
    cidFile,
    running,
    ...(onStarted === undefined ? {} : { onStarted, }),
  },);
  return await running;
}
