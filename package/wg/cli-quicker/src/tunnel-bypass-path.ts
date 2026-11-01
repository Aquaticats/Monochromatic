import { createHash, } from 'node:crypto';
import {
  isAbsolute,
  join,
} from 'node:path';

import { BypassStateError, } from './errors.ts';

/**
 Default root-owned runtime state directory.
 */
const DEFAULT_RUNTIME_DIRECTORY = '/run/wg-quicker';

/**
 Environment override used by disposable integration fixtures.
 */
const RUNTIME_DIRECTORY_ENVIRONMENT = 'WG_QUICKER_RUNTIME_DIRECTORY';

/**
 Hex characters retained from interface hash.
 */
const STATE_KEY_LENGTH = 32;

/**
 Resolves runtime directory without workspace assumptions.
 
 @returns Root-owned default or explicit fixture override.
 
 @example
 ```ts
 bypassRuntimeDirectory();
 ```
 */
export function bypassRuntimeDirectory(): string {
  /**
   Explicit runtime path for isolated execution.
   */
  const configured = process.env[RUNTIME_DIRECTORY_ENVIRONMENT];
  if ((configured === undefined) || (configured === ''))
    return DEFAULT_RUNTIME_DIRECTORY;
  if (!isAbsolute(configured,)) {
    throw new BypassStateError(
      `${RUNTIME_DIRECTORY_ENVIRONMENT} must be an absolute path: ${configured}`,
    );
  }
  return configured;
}

/**
 Produces collision-resistant filesystem and unit key for interface.
 
 @param interfaceName - Interface identity.
 
 @returns Truncated SHA-256 hex key.
 
 @example
 ```ts
 bypassStateKey({ interfaceName: 'wg0' });
 ```
 */
export function bypassStateKey(
  { interfaceName, }: { readonly interfaceName: string; },
): string {
  return createHash('sha256',)
    .update(interfaceName,)
    .digest('hex',)
    .slice(
    0,
    STATE_KEY_LENGTH,
    );
}

/**
 Resolves interface state path.
 
 @param interfaceName - Interface identity.
 
 @returns JSON state path under runtime directory.
 
 @example
 ```ts
 bypassStatePath({ interfaceName: 'wg0' });
 ```
 */
export function bypassStatePath(
  { interfaceName, }: { readonly interfaceName: string; },
): string {
  return join(
    bypassRuntimeDirectory(),
    `interface-${bypassStateKey({ interfaceName, },)}.json`,
  );
}
