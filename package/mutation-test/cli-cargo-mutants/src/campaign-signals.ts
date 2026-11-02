import {
  type PodmanResult,
  runPodman,
} from './podman.ts';

/**
 Signals that end a campaign: a unit stop, an interrupt and a closed terminal.
 */
const terminationSignals = [
  'SIGTERM',
  'SIGINT',
  'SIGHUP',
] as const;

/**
 Seconds a stopped campaign container gets to finish its current mutant before it is killed.
 */
const stopGraceSeconds = 10;

/**
 Stop the campaign container when the campaign process is told to end.

 A rootless container runs in a scope of its own, outside the unit that started it, so the end of the unit does not end
 the container; without this it would go on writing the report that a resumed attempt reads. With the handlers in
 place the signal no longer ends the process: the attached start returns once the container stops, and the campaign
 composes the partial report, records the exit and fails, which leaves every scored mutant for the next run to skip.

 @param containerId - ID of the running campaign container

 @returns a handle whose disposal removes the handlers

 @example
 ```ts
 using handlers = stopContainerOnTermination(containerId);
 ```
 */
function stopContainerOnTermination(containerId: string): Disposable {
  /**
   Stop the container.

   @param signal - name of the signal received
   */
  async function stopContainer(signal: (typeof terminationSignals)[number]): Promise<void> {
    console.error(`Received ${signal}: stopping container ${containerId}; the next run of this campaign resumes it.`);
    try {
      await runPodman({
        args: [
          'stop',
          '--time',
          String(stopGraceSeconds),
          containerId
        ],
        capture: true,
        allowFailure: true,
      });
    }
    catch (error) {
      console.error(`Stopping the container after ${signal} failed: ${String(error)}`);
    }
  }
  /**
   Handlers by signal, kept to remove them.
   */
  const handlers = terminationSignals.map(function handlerFor(signal) {
    /**
     Handler of one signal.
     */
    function handle(): void {
      void stopContainer(signal);
    }
    process.once(
      signal,
      handle,
    );
    return [
      signal,
      handle
    ] as const;
  });
  return {
    [Symbol.dispose](): void {
      for (const [signal, handle] of handlers) {
        process.off(
          signal,
          handle,
        );
      }
    },
  };
}

/**
 Start a created campaign container attached and wait for it, stopping it if the campaign process is told to end.

 @param containerId - ID of the created campaign container

 @returns how the container ended; a nonzero status can mean missed mutants rather than a failure

 @example
 ```ts
 await startAttached(containerId);
 ```
 */
export async function startAttached(containerId: string): Promise<PodmanResult> {
  /**
   Handlers that stop the container on a termination signal, removed when the container has ended.
   */
  using stopOnSignal = stopContainerOnTermination(containerId);
  return await runPodman({
    args: [
      'start',
      '--attach',
      containerId
    ],
    capture: false,
    allowFailure: true,
  });
}
