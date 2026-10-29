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
