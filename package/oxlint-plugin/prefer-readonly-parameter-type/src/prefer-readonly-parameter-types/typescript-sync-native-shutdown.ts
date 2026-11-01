/**
 Quiet native-process shutdown for TypeScript 7 synchronous API.
 
 @module
 */

import { version as typescriptVersion, } from 'typescript';
import type { API, } from 'typescript/sync';

import { SemanticBridgeError, } from './semantic-bridge-error.ts';

/**
 Signal that terminates native API child without invoking its cancellation logger.
 */
const NATIVE_API_TERMINATION_SIGNAL = 'SIGKILL' as const;

/**
 Native child-process surface retained inside TypeScript's unstable sync channel.
 */
export type NativeApiChild = {
  kill: (signal?: NodeJS.Signals | number,) => boolean;
};

/**
 Tests whether unstable TypeScript channel value exposes native child control.
 
 @param value - Candidate nested channel child.
 
 @returns whether value exposes expected kill operation.
 
 @example
 ```ts
 if (isNativeApiChild(value)) value.kill('SIGKILL');
 ```
 */
function isNativeApiChild(value: unknown,): value is NativeApiChild {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('kill' in value)
    && ((typeof value.kill) === 'function');
}

/**
 Creates failure for changed unstable TypeScript channel shape.
 
 @returns error that prevents noisy or leaked native shutdown.
 
 @example
 ```ts
 throw nativeChildUnavailable();
 ```
 */
function nativeChildUnavailable(): SemanticBridgeError {
  return new SemanticBridgeError({
    reason: 'api-unavailable',
    message: `TypeScript ${typescriptVersion} synchronous API omitted expected native child control.`,
  },);
}

/**
 Verifies installed compiler belongs to selected unstable major.
 
 @throws {@link SemanticBridgeError} when runtime compiler major is not 7.
 
 @example
 ```ts
 assertTypeScriptSeven();
 ```
 */
export function assertTypeScriptSeven(): void {
  /**
   Major component before first version separator.
   */
  const [major,] = typescriptVersion.split('.',);
  if (major !== '7') {
    throw new SemanticBridgeError({
      reason: 'api-unavailable',
      message: `Expected TypeScript 7 semantic bridge, received ${typescriptVersion}.`,
    },);
  }
}

/**
 Retrieves pinned TypeScript sync channel child process.
 
 @param api - Newly created unstable synchronous API.
 
 @returns native child control owned by API channel.
 
 @throws {@link SemanticBridgeError} when pinned internal channel shape changed.
 
 @example
 ```ts
 nativeApiChild(api).kill('SIGKILL');
 ```
 */
export function nativeApiChild(api: API,): NativeApiChild {
  /**
   Public object view used for guarded internal-property traversal.
   */
  const apiObject: object = api;
  if (!('client' in apiObject))
    throw nativeChildUnavailable();
  /**
   Sync client hidden behind unstable API implementation.
   */
  const { client, } = apiObject;
  if ((typeof client) !== 'object')
    throw nativeChildUnavailable();
  if (client === null)
    throw nativeChildUnavailable();
  if (!('channel' in client))
    throw nativeChildUnavailable();
  /**
   RPC channel hidden behind sync client.
   */
  const { channel, } = client;
  if ((typeof channel) !== 'object')
    throw nativeChildUnavailable();
  if (channel === null)
    throw nativeChildUnavailable();
  if (!('child' in channel))
    throw nativeChildUnavailable();
  /**
   Native process hidden behind RPC channel.
   */
  const { child, } = channel;
  if (!isNativeApiChild(child,))
    throw nativeChildUnavailable();
  return child;
}

/**
 Configures TypeScript-owned channel cleanup to terminate without native cancellation output.
 
 TypeScript 7.0.2 sends `SIGTERM` by default,
 which makes `tsgo --api` print `context canceled`.
 The pinned channel still owns pipe cleanup and process termination;
 only its termination signal changes.
 
 @param child - Native process owned by TypeScript sync channel.
 
 @mutates child - Replaces child kill operation with `SIGKILL`; `Reflect.apply` invokes original capability.
 
 @example
 ```ts
 configureNativeApiChildShutdown(nativeApiChild(api));
 ```
 */
export function configureNativeApiChildShutdown(child: NativeApiChild,): void {
  /**
   Original Node child-process operation retained with explicit receiver.
   */
  const originalKill = child.kill;
  child.kill = function terminateNativeApiWithoutCancellationOutput(): boolean {
    /**
     Native kill result normalized after reflective receiver invocation.
     */
    const result: unknown = Reflect.apply(
      originalKill,
      child,
      [NATIVE_API_TERMINATION_SIGNAL,],
    );
    return result === true;
  };
}
