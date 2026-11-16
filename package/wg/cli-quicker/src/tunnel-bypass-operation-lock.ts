import { mkdir, } from 'node:fs/promises';

import { BypassStateError, } from './errors.ts';
import { claimOperationLock, } from './operation-lock.ts';
import {
  bypassRuntimeDirectory,
  bypassStateKey,
} from './tunnel-bypass-path.ts';

/**
 Ensures private runtime directory for advisory lock paths.
 
 @example
 ```ts
 await ensureRuntimeDirectory();
 ```
 */
async function ensureRuntimeDirectory(): Promise<void> {
  await mkdir(
    bypassRuntimeDirectory(),
    {
      mode: 0o700,
      recursive: true,
    },
  );
}

/**
 Creates bypass-state error for shared lock implementation.
 
 @param message - Lock failure diagnostic.
 
 @returns Bypass-specific failure.
 
 @example
 ```ts
 makeBypassStateError('busy');
 ```
 */
function makeBypassStateError(message: string,): BypassStateError {
  return new BypassStateError(message,);
}

/**
 Acquires lock serializing one interface lifecycle.
 
 @param interfaceName - Interface lifecycle identity.
 
 @returns Crash-safe kernel-lock guard.
 
 @example
 ```ts
 await using lock = await claimBypassInterfaceOperation({ interfaceName: 'wg0' });
 ```
 */
export async function claimBypassInterfaceOperation(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<AsyncDisposable> {
  await ensureRuntimeDirectory();
  return await claimOperationLock({
    lockPath: `${bypassRuntimeDirectory()}/interface-${bypassStateKey({ interfaceName, },)}.operation.lock`,
    conflictMessage: `Another wg-quicker lifecycle is operating on ${interfaceName}.`,
    errorFactory: makeBypassStateError,
  },);
}

/**
 Acquires global lock spanning bypass allocation and installation.
 
 Routes and rules become kernel-visible before guard releases,
 so later allocators observe occupancy without persistent lock files.
 
 @returns Crash-safe kernel-lock guard.
 
 @example
 ```ts
 await using lock = await claimBypassAllocationOperation();
 ```
 */
export async function claimBypassAllocationOperation(): Promise<AsyncDisposable> {
  await ensureRuntimeDirectory();
  return await claimOperationLock({
    lockPath: `${bypassRuntimeDirectory()}/allocation.operation.lock`,
    conflictMessage: 'Another wg-quicker lifecycle is allocating bypass routing resources.',
    errorFactory: makeBypassStateError,
  },);
}

/**
 Acquires the lock that covers one interface's whole lifecycle.

 A start and a stop of the same interface are two sequences that both change the
 same link, rules and tables, and neither used to take a lock of the whole kind:
 an `up` could be held just before its kill switch, a `down` could run to its end,
 and the `up` then went on to report success with its link already gone and an
 orphan kill switch table behind it.
 The lock is taken for the whole of `up` and the whole of `down`, and it is
 non-blocking like the others, so the second lifecycle refuses with the conflict
 instead of racing:
 a person who asks twice is told that the other lifecycle is running.

 It is a different file from the interface lock the bypass and watcher calls take,
 so those calls nest inside it without conflicting with it.

 @param interfaceName - Interface whose lifecycle is claimed.

 @returns Crash-safe kernel-lock guard.

 @example
 ```ts
 await using lifecycle = await claimLifecycleOperation({ interfaceName: 'wg0' });
 ```
 */
export async function claimLifecycleOperation(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<AsyncDisposable> {
  await ensureRuntimeDirectory();
  return await claimOperationLock({
    lockPath: `${bypassRuntimeDirectory()}/interface-${bypassStateKey({ interfaceName, },)}.lifecycle.lock`,
    conflictMessage: `Another wg-quicker lifecycle is running for ${interfaceName}: wait for it to end and run the command again.`,
    errorFactory: makeBypassStateError,
  },);
}
