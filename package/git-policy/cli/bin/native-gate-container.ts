/**
 One gate container, run under its own ID file so it can be removed whatever ended the client that ran it.

 `podman run` keeps its container running when its client is stopped, and the gate stops a client at the deadline it gives
 every check (`bin/native-gate-checks.ts`). The ID file is where the client wrote the container's identity while it ran,
 so the container is removed even then; a run whose `--rm` already removed it leaves the removal nothing to find.
 */
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
 Run one gate container and remove it, whether it ended by itself or its client was stopped at the deadline.

 @param cidFile - file the client writes the container's ID to, under this run's evidence directory

 @param args - complete `podman run` arguments after the program name

 @param capture - whether to capture output instead of streaming it

 @param allowFailure - whether a nonzero exit is evidence the caller inspects

 @param deadlineMilliseconds - the bound the gate waits under, which this container's client is given

 @param environment - environment both clients run in, which defaults to this process's

 @returns exit state and captured output of the client

 @example
 ```ts
 const suite = await runGateContainer({
   cidFile: join(evidence, 'suite.cid'),
   args: [...limits, image],
   deadlineMilliseconds,
 });
 ```
 */
export async function runGateContainer({
  cidFile,
  args,
  capture = false,
  allowFailure = false,
  deadlineMilliseconds,
  environment = process.env,
}: {
  readonly cidFile: string;
  readonly args: readonly string[];
  readonly capture?: boolean;
  readonly allowFailure?: boolean;
  readonly deadlineMilliseconds: number;
  readonly environment?: NodeJS.ProcessEnv;
},): Promise<CommandOutcome> {
  /**
   The container's removal, run when this scope ends whatever the client did.
   */
  await using removal = containerRemoval({
    cidFile,
    environment,
  },);
  return await runCommand({
    command: 'podman',
    args: [
      'run',
      '--cidfile',
      cidFile,
      ...args,
    ],
    capture,
    allowFailure,
    deadlineMilliseconds,
    env: environment,
  },);
}
