/**
 The step commands one guest driver has started and still has running, and what happens to them when the driver itself is
 interrupted.

 A driver's step runs in a process group of its own, so its own bound can stop everything the step started: `mvm` hands a
 guest's command to the far side of a connection, and a bound that reached only the client would leave the guest working
 with nobody reading it. The same separation is what a person's interrupt key runs into: the kernel sends that signal to
 the driver's own process group and to nothing else, so a step that is bounded at minutes, or at the two hours a guest
 build may take, went on working after the driver that started it had ended. Every driver met that after the three helper
 sets became one, and the drivers before it did not, because their steps stayed in the driver's own group.

 So a signal that ends a driver stops every step the driver still has running, through the same group stop a step's own
 deadline makes (`process-group-stop.ts`), says in one line what it stopped, and then ends the driver the way the signal
 would have ended it: the handlers go first, so the signal the process sends itself has its default action and a shell
 reports the interrupt as it always did.

 A step is recorded here while this process waits for its command's own end, and forgotten when that wait answers, so an
 entry names a command that is still running and never one that has already ended.
 */
import type { ChildProcess, } from 'node:child_process';

import { stopProcessGroup, } from './process-group-stop.ts';

/**
 The signals whose arrival ends a driver before its steps have ended.

 A terminal's interrupt reaches the driver's group as `SIGINT`, a request to stop it as `SIGTERM`, and a terminal that
 went away as `SIGHUP`; each of them used to end the driver alone and leave its step working.
 */
const INTERRUPT_SIGNALS: readonly NodeJS.Signals[] = [
  'SIGHUP',
  'SIGINT',
  'SIGTERM',
];

/**
 Every step command this process has started and still has running, by the command it is, so a signal can name what it
 stopped.
 */
const liveSteps = new Map<ChildProcess, string>();

/**
 Stop every step this process still has running, say what was stopped, and end the process as the signal ends it.

 The entries are kept while the stop runs, so a second signal that arrives during it finds the same commands.

 @param signal - the signal that arrived
 */
function stopEveryLiveStepAndEndAsTheSignalEnds(signal: NodeJS.Signals,): void {
  /**
   What this process stopped, each named by its command and the process group that was signalled.
   */
  const stopped: string[] = [];
  for (const [child, step,] of liveSteps) {
    stopProcessGroup({ child, },);
    /**
     The command's own process number, which is the number of the group it leads.
     */
    const { pid, } = child;
    stopped.push(pid === undefined ? step : `${step} (process group ${String(pid,)})`,);
  }
  if (stopped.length === 0) {
    console.error(`git-policy guest step: ${signal}, and this run had no step command still running`,);
  }
  else {
    console.error(`git-policy guest step: ${signal}, so this run stopped every step command it had started and still had running: ${stopped.join('; ',)}`,);
  }
  // The handlers go first, so the signal the process sends itself has its default action and a shell reports the
  // interrupt as it reported one before a driver's steps left its own process group.
  stopWatching();
  process.kill(
    process.pid,
    signal,
  );
}

/**
 Watch for the signals that end a driver, until nothing is left to stop.

 The watch is installed when the first step is recorded and removed when the last one is forgotten, so a driver that runs
 no step keeps the default action of every signal.
 */
function watchForSignals(): void {
  for (const signal of INTERRUPT_SIGNALS)
    process.on(
      signal,
      stopEveryLiveStepAndEndAsTheSignalEnds,
    );
}

/**
 Remove the watch, so the next signal has its default action.
 */
function stopWatching(): void {
  for (const signal of INTERRUPT_SIGNALS) {
    process.off(
      signal,
      stopEveryLiveStepAndEndAsTheSignalEnds,
    );
  }
}

/**
 One step command this process is waiting for, which is forgotten when its wait answers.

 The hold is what a caller keeps with `using`, so the entry lasts exactly as long as the wait does, whichever way the wait
 ends.
 */
export type LiveStepHeld = {
  /**
   Forget the step, so a later signal neither stops it again nor names a command that has already ended.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Record one step command this process has started, so a signal that ends the driver stops it too.

 @param child - the started command, leading a process group of its own

 @param step - the command as a refusal names it

 @returns the hold that forgets the command when the wait for it answers

 @example
 ```ts
 using held = holdTheLiveStep({ child, step: 'mvm exec guest-a -- sh /mnt/shared/build.log.launch.sh' });
 ```
 */
export function holdTheLiveStep({
  child,
  step,
}: {
  readonly child: ChildProcess;
  readonly step: string;
},): LiveStepHeld {
  liveSteps.set(
    child,
    step,
  );
  if (liveSteps.size === 1)
    watchForSignals();
  return {
    [Symbol.dispose](): void {
      if (!liveSteps.delete(child,))
        return;
      if (liveSteps.size === 0)
        stopWatching();
    },
  };
}
