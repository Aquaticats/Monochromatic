import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { UpInterruptedError, } from './errors.ts';

/**
 Module logger for the interruption watch.
 */
const l = tagged({ tag: 'interrupt-signal', },);

/**
 Signals that end an `up` before its last step.
 A terminal's interrupt and a shutdown request are the same thing here:
 either one stops an up that has changed something,
 and the rollback of the up removes what it created.
 */
const INTERRUPT_SIGNALS: readonly NodeJS.Signals[] = [
  'SIGINT',
  'SIGTERM',
];

/**
 The signal a watch starts out naming,
 before any arrives.
 */
const FIRST_INTERRUPT_SIGNAL: NodeJS.Signals = 'SIGINT';

/**
 How an `up` learns that the person or the system interrupted it.
 */
export type UpInterruption = {
  /**
   Refuses the up when a signal arrived,
   naming the step that would have run next.

   @param step - What the up was about to do.
   */
  readonly beforeStep: (step: string,) => void;

  /**
   The refusal to report after a step failed.

   An interrupted step usually fails because the signal ended the command it ran,
   and that command's own error says nothing about the interrupt,
   so a signal turns any step failure into the interrupt refusal.

   @param step - The step that failed.

   @param failure - What the step threw.

   @returns The refusal to report.
   */
  readonly insteadOfStepFailure: (step: string, failure: unknown,) => unknown;
};

/**
 Watches for the signals that end an up,
 and ends the sequence at the next step boundary.

 A signal's default action ends the process without a JavaScript exception,
 so the catch that rolls a failed up back never runs;
 with this watch the up stops between two steps,
 runs the rollback,
 and names the signal,
 which is what keeps a half-configured tunnel out of the machine.

 A second signal stops at once,
 because a person who asks twice wants the process gone and not another step of a rollback.

 @returns The watch, which removes its handlers when disposed.

 @example
 ```ts
 await using interruption = watchForInterruption();
 await bringUp({ exemptMark, steps, rollback, interruption });
 ```
 */
export function watchForInterruption(): UpInterruption & Disposable {
  /**
   How many signals arrived,
   in an object because the handlers share it.
   */
  const arrived = { count: 0, };
  /**
   The last signal that arrived, for the refusal's words.
   */
  const last: { signal: NodeJS.Signals } = { signal: FIRST_INTERRUPT_SIGNAL, };
  /**
   Records a signal, and stops at once on the second one.

   @param signal - The signal that arrived.
   */
  function onSignal(signal: NodeJS.Signals,): void {
    arrived.count += 1;
    last.signal = signal;
    if (arrived.count > 1) {
      l.warn(`${signal} again: stopping at once, and the rollback of this up is abandoned`,);
      // The handlers go first, so the re-raised signal has its default action
      // and the process ends the way a shell reports an interrupted program.
      stopWatching();
      process.kill(process.pid, signal,);
      return;
    }
    l.warn(`${signal}: the up stops before its next step, and what this up created is removed`,);
  }
  /**
   Removes the handlers of this watch.
   */
  function stopWatching(): void {
    for (const signal of INTERRUPT_SIGNALS)
      process.off(
        signal,
        onSignal,
      );
  }
  for (const signal of INTERRUPT_SIGNALS)
    process.on(
      signal,
      onSignal,
    );
  /**
   The refusal that names the signal and the step.

   @param step - What the up was doing.

   @returns The refusal.
   */
  function refusal(step: string,): UpInterruptedError {
    return new UpInterruptedError({
      signal: last.signal,
      step,
    },);
  }
  return {
    beforeStep: function beforeStep(step: string,): void {
      if (arrived.count === 0)
        return;
      throw refusal(step,);
    },
    insteadOfStepFailure: function insteadOfStepFailure(step: string, failure: unknown,): unknown {
      if (arrived.count === 0)
        return failure;
      return refusal(step,);
    },
    [Symbol.dispose]: stopWatching,
  };
}
