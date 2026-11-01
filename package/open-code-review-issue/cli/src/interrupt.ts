/**
 Two-stage publication interrupt handling.
 
 @module
 */

import process from 'node:process';
import { setTimeout as delay, } from 'node:timers/promises';

import type {
  PublicationStopCheck,
  PublicationWait,
} from './publisher-model.ts';

/**
 Interrupt count that forces immediate native termination.
 */
const FORCE_INTERRUPT_COUNT = 2;

/**
 Host signal operations injectable for lifecycle tests.
 */
export type PublicationSignalHost = {
  readonly onInterrupt: (listener: () => void) => void;
  readonly offInterrupt: (listener: () => void) => void;
  readonly forceInterrupt: () => void;
};

/**
 Active publication interrupt controls and cleanup.
 */
export type PublicationInterruptControl = {
  readonly shouldStop: PublicationStopCheck;
  readonly wait: PublicationWait;
  readonly [Symbol.dispose]: () => void;
};

/**
 Production process signal host.
 */
const PROCESS_SIGNAL_HOST: PublicationSignalHost = {
  onInterrupt(listener,) {
    process.on(
      'SIGINT',
      listener,
    );
  },
  offInterrupt(listener,) {
    process.off(
      'SIGINT',
      listener,
    );
  },
  forceInterrupt() {
    process.kill(
      process.pid,
      'SIGINT',
    );
  },
};

/**
 Installs first-stop and second-force interrupt behavior for publication phase.
 
 @param host - Injectable process signal host.
 
 @returns Stop check, abortable wait, and listener cleanup.
 
 @example
 ```ts
 using interrupts = createPublicationInterruptControl({});
 ```
 */
export function createPublicationInterruptControl({
  host = PROCESS_SIGNAL_HOST,
}: {
  readonly host?: PublicationSignalHost;
},): PublicationInterruptControl {
  /**
   Mutable signal state hidden behind one constant binding.
   */
  const state = {
    count: 0,
    stop: false,
  };
  /**
   Abort source used only for pacing and retry waits.
   */
  const waitAbort = new AbortController();
  /**
   Handles first graceful stop and second native force termination.
   */
  function interrupt(): void {
    state.count += 1;
    if (state.count < FORCE_INTERRUPT_COUNT) {
      state.stop = true;
      waitAbort.abort();
      return;
    }
    host.offInterrupt(interrupt,);
    host.forceInterrupt();
  }
  host.onInterrupt(interrupt,);
  return {
    shouldStop() {
      return state.stop;
    },
    async wait(milliseconds,) {
      await delay(
        milliseconds,
        undefined,
        { signal: waitAbort.signal, },
      );
    },
    [Symbol.dispose]() {
      host.offInterrupt(interrupt,);
    },
  };
}
