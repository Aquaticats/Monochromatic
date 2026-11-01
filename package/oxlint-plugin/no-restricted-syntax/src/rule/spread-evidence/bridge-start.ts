/**
 Start of the TypeScript semantic bridge when the plugin loads, and the account of why it did not start.

 The spread rules ask the bridge for types. This plugin bundles its own copy of the bridge, so the readonly
 plugin starting its copy does not help: the native child of this copy is spawned on first use, which is after
 Oxlint has reserved several GiB of address space for each worker. A process that has reserved more than the host
 holds in memory and swap cannot fork, so on a host with enough workers that first spawn fails with `ENOMEM`.
 Starting the child at load, before the workers exist, avoids it.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  SemanticBridgeError,
  type SemanticBridgeFailureReason,
} from '@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type/ts/prefer-readonly-parameter-types/semantic-bridge-error.ts';
import { initializeSemanticBridge, } from '@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type/ts/prefer-readonly-parameter-types/semantic-bridge-lifecycle.ts';

/**
 Module logger for the bridge start.
 */
const l = tagged({ tag: 'no-restricted-syntax/spread-evidence/bridge-start', },);

/**
 Sentinel for a bridge whose start has not failed.
 */
export const NO_START_FAILURE: unique symbol = Symbol('the TypeScript bridge has not failed to start',);

/**
 Why the load-time start failed, kept so later answers do not spawn again.
 */
const startState: { failure: SemanticBridgeError | typeof NO_START_FAILURE; } = { failure: NO_START_FAILURE, };

/**
 Starts the native TypeScript child, keeping the failure when it cannot start.

 A failure is kept instead of thrown, because only the spread rules need the bridge and every other rule of the
 plugin works without it. The first spread rule that needs a type reports it through {@link bridgeStartFailure}.

 @throws Rethrows failures other than {@link SemanticBridgeError}, which mean a bridge defect.

 @example
 ```ts
 startSpreadEvidenceBridge();
 ```
 */
export function startSpreadEvidenceBridge(): void {
  /**
   Function-tagged start logger.
   */
  const rl = tagged({
    tag: startSpreadEvidenceBridge.name,
    l,
  },);
  try {
    initializeSemanticBridge();
  }
  catch (error) {
    if (!(error instanceof SemanticBridgeError)) {
      rl.error(`semantic bridge failed unexpectedly when the plugin loaded: ${String(error,)}`,);
      throw error;
    }
    rl.error(`semantic bridge could not start when the plugin loaded (${error.reason}): ${error.message}`,);
    startState.failure = error;
  }
}

/**
 Returns why the bridge could not start when the plugin loaded.

 @returns the start failure, or {@link NO_START_FAILURE} when the bridge started.

 @example
 ```ts
 const failure = bridgeStartFailure();
 ```
 */
export function bridgeStartFailure(): SemanticBridgeError | typeof NO_START_FAILURE {
  return startState.failure;
}

/**
 Builds the error that stops the lint when the bridge cannot start.

 @param reason - Failure category of the bridge.

 @param message - What the bridge reported when it could not start.

 @param fileName - File whose type was needed.

 @returns error naming the file, the spawn failure and every way to get the bridge running.

 @example
 ```ts
 throw bridgeUnavailableError({ reason, message, fileName });
 ```
 */
export function bridgeUnavailableError(
  {
    reason,
    message,
    fileName,
  }: {
    readonly reason: SemanticBridgeFailureReason;
    readonly message: string;
    readonly fileName: string;
  },
): SemanticBridgeError {
  return new SemanticBridgeError({
    reason,
    message: [
      `${message}.`,
      `The spread rules need it to tell arrays from strings and iterators in ${fileName},`,
      'and without it they would drop findings or report them as ambiguous, so the lint stops here.',
      'A process cannot spawn the native child once it has reserved more memory than the host holds in memory and swap,',
      'and Oxlint reserves several GiB of address space for each worker it starts.',
      'Lower Oxlint\'s `--threads` (the root `lint` task of this repository sets `OXLINT_THREADS` to 1),',
      'or free memory or swap if the host is short of them, or fix the TypeScript installation if the error above',
      'names a TypeScript version.',
    ].join(' ',),
  },);
}
