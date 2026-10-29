import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import type { WatchRestartHandle, } from './start.ts';

/**
 Installs a one-shot SIGINT/SIGTERM handler that stops the
 orchestrator and exits the process.
 
 One-shot: subsequent signals during shutdown skip the handler so a
 frustrated double-Ctrl+C does not race two `stop()` calls; the
 second signal lands as a hard exit via Node's default disposition.
 
 @param signal - signal name to handle
 
 @param handle - orchestrator handle whose `stop()` runs first
 
 @example
 ```ts
 installShutdownHandler({ signal: 'SIGINT', handle, },);
 ```
 */
export function installShutdownHandler(
  {
    signal,
    handle,
  }: {
    readonly signal: NodeJS.Signals;
    readonly handle: WatchRestartHandle;
  },
): void {
  process.once(
    signal,
    function onSignal(): void {
      void (async function doShutdown(): Promise<void> {
        try {
          await handle.stop();
          process.exit(0,);
        }
        catch (error) {
          /**
           Human-readable error string used in the shutdown-failure stderr line.
           */
          const message = caughtValueText(error,);
          console.error(`shutdown failed: ${message}`,);
          process.exit(1,);
        }
      })();
    },
  );
}
