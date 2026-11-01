import {
  BYPASS_STATE_ABSENT,
  readBypassState,
} from './tunnel-bypass-state.ts';
import { claimBypassInterfaceOperation, } from './tunnel-bypass-operation-lock.ts';
import {
  startBypassWatcher,
  stopBypassWatcher,
} from './tunnel-bypass-watch-service.ts';

/**
 Stops the route watcher of an interface and nothing else,
 so a failure to stop it is found before any routing is removed.
 An interface with no persisted state has no watcher.

 @param interfaceName - Tunnel interface.

 @throws {@link BypassStateError} when the persisted state cannot be read.

 @throws {@link BypassRouteError} when the watcher cannot be shown to be the one that was started,
 or survives `SIGKILL`.

 @example
 ```ts
 await stopRouteWatcher({ interfaceName: 'wg0' });
 ```
 */
export async function stopRouteWatcher({ interfaceName, }: { readonly interfaceName: string; },): Promise<void> {
  /**
   Kernel lock preventing concurrent replacement or duplicate teardown.
   */
  await using interfaceOperation = await claimBypassInterfaceOperation({ interfaceName, },);
  /**
   Persisted state when bypass setup completed.
   */
  const state = await readBypassState({ interfaceName, },);
  if (state === BYPASS_STATE_ABSENT)
    return;
  await stopBypassWatcher({ state, },);
}

/**
 Starts the route watcher again after a `down` that was given up,
 so the tunnel stays as it was.

 @param interfaceName - Tunnel interface.

 @throws {@link BypassRouteError} when the watcher cannot start.

 @example
 ```ts
 await restartRouteWatcher({ interfaceName: 'wg0' });
 ```
 */
export async function restartRouteWatcher({ interfaceName, }: { readonly interfaceName: string; },): Promise<void> {
  /**
   Kernel lock preventing concurrent replacement or duplicate teardown.
   */
  await using interfaceOperation = await claimBypassInterfaceOperation({ interfaceName, },);
  /**
   Persisted state when bypass setup completed.
   */
  const state = await readBypassState({ interfaceName, },);
  if (state === BYPASS_STATE_ABSENT)
    return;
  await startBypassWatcher({ state, },);
}
