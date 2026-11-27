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
  type ContainerMemoryWatch,
  NO_MEMORY_KILL,
  containerMemoryKilledLine,
  watchContainerMemory,
} from './native-gate-container-memory.ts';
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

 @param limits - the bound every container of this gate runs under, which the gate's own limit options and the person's
   `--memory`, `--cpus` and `--pids-limit` become (`bin/native-gate-container-limits.ts`); this function takes it as its
   own argument so a container cannot be started without one

 @param args - complete `podman run` arguments after the program name, the container's identity and its limits

 @param capture - whether to capture output instead of streaming it

 @param allowFailure - whether a nonzero exit is evidence the caller inspects

 @param deadlineMilliseconds - the bound the gate waits under, which this container's client is given

 @param environment - environment both clients run in, which defaults to this process's

 @param onStarted - receives the container's ID as soon as its client writes it, which is how the run's evidence
   records it even though `--rm` takes the ID file away with the container; this waits for it, so the evidence is on
   disk while the container still runs

 @param watchMemory - starts the watch of the container's own memory accounting, which
   `bin/native-gate-container-memory.ts` implements and a test hands its own

 @returns exit state and captured output of the client

 @example
 ```ts
 const suite = await runGateContainer({
   cidFile: join(evidence, 'suite.cid'),
   identity: gateContainerIdentityOf({ runId, role: 'suite' }),
   limits,
   args: [image],
   deadlineMilliseconds,
 });
 ```
 */
export async function runGateContainer({
  cidFile,
  identity,
  limits,
  args,
  capture = false,
  allowFailure = false,
  deadlineMilliseconds,
  environment = process.env,
  onStarted,
  watchMemory = watchContainerMemory,
}: {
  readonly cidFile: string;
  readonly identity: GateContainerIdentity;
  readonly limits: readonly string[];
  readonly args: readonly string[];
  readonly capture?: boolean;
  readonly allowFailure?: boolean;
  readonly deadlineMilliseconds: number;
  readonly environment?: NodeJS.ProcessEnv;
  readonly onStarted?: (id: string) => Promise<void>;
  readonly watchMemory?: (options: { readonly containerId: string; }) => ContainerMemoryWatch;
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
      ...limits,
      ...args,
    ],
    capture,
    allowFailure,
    deadlineMilliseconds,
    env: environment,
  },);
  /**
   The memory watch of this container, started once its client reported the container's ID.

   It is held in an object because the client's own callback is what starts it, and a container killed for memory has to
   be named after the client ends, which is the only moment the gate knows the check is over.
   */
  const watched: { current?: ContainerMemoryWatch; } = {};
  await announceContainerId({
    identity,
    cidFile,
    running,
    onStarted: async function started(id, ) {
      watched.current = watchMemory({ containerId: id, },);
      if (onStarted !== undefined)
        await onStarted(id,);
    },
  },);
  /**
   The client's answer, which the container's own memory watch is reported beside either way.
   */
  // A container killed for memory ends like any other, so the reading is kept while it runs and reported here, beside
  // whatever the check itself said: a client the deadline stopped still leaves its container's kill to be named.
  const answered = await settleContainer({
    running,
    watched,
    identity,
  },);
  if ('reason' in answered)
    throw answered.reason;
  return answered.outcome;
}

/**
 Run one gate container's client to its answer, and report what the container's own memory watch found either way.

 @param running - the client that runs the container

 @param watched - the watch of the container's memory, once the client reported the container's ID

 @param identity - the container's name, label and role, which the line a kill prints names

 @returns the client's answer, which is either its exit state or the failure it threw
 */
async function settleContainer({
  running,
  watched,
  identity,
}: {
  readonly running: Promise<CommandOutcome>;
  readonly watched: { readonly current?: ContainerMemoryWatch; };
  readonly identity: GateContainerIdentity;
},): Promise<{ readonly outcome: CommandOutcome; } | { readonly reason: unknown; }> {
  try {
    /**
     What the client answered.
     */
    const outcome = await running;
    await reportMemoryKill({
      watched,
      identity,
    },);
    return { outcome, };
  }
  catch (error) {
    await reportMemoryKill({
      watched,
      identity,
    },);
    return { reason: error, };
  }
}

/**
 Stop one container's memory watch and print what it says about a kill inside the container's bound, if any.

 @param watched - the watch of the container's memory, once the client reported the container's ID

 @param identity - the container's name, label and role, which the line a kill prints names
 */
async function reportMemoryKill({
  watched,
  identity,
}: {
  readonly watched: { readonly current?: ContainerMemoryWatch; };
  readonly identity: GateContainerIdentity;
},): Promise<void> {
  if (watched.current === undefined)
    return;
  /**
   The watch of this container, stopped so its last read is the moment before the container ended.
   */
  const memory = watched.current;
  await memory.stop();
  /**
   What the gate says about a kill inside the bound, or {@link NO_MEMORY_KILL} when the bound decided nothing.
   */
  const killed = containerMemoryKilledLine({
    role: identity.role,
    sample: memory.lastRead(),
  },);
  if (killed !== NO_MEMORY_KILL)
    console.log(killed,);
}
