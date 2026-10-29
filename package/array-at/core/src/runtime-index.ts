/**
 Runtime array-index validation and resolution.
 
 @module
 */

import { ArrayAtError, } from './array-at-error.ts';
import {
  createBeforeStartDiagnostic,
  createEmptyArrayDiagnostic,
  createNonSafeIntegerDiagnostic,
  createPastEndDiagnostic,
  createUnassignedSlotDiagnostic,
} from './diagnostic-factories.ts';
import type { NonEmptyRuntimeArrayAtDiagnostics, } from './diagnostic-types.ts';
import { isSafeInteger, } from './safe-integer.ts';

/**
 Runtime failure construction parameters.
 
 @example
 ```ts
 declare const options: ThrowRuntimeDiagnosticsOptions;
 ```
 */
type ThrowRuntimeDiagnosticsOptions = {
  readonly diagnostics: NonEmptyRuntimeArrayAtDiagnostics;
  readonly index: number;
  readonly length: number;
};

/**
 Throws one error containing supplied runtime diagnostics.
 
 @param diagnostics - Non-empty runtime diagnostic collection
 
 @param index - Requested numeric index
 
 @param length - Array length
 
 @throws {@link ArrayAtError} for supplied diagnostics
 
 @example
 ```ts
 throwRuntimeDiagnostics({
   diagnostics: [createEmptyArrayDiagnostic()],
   index: 0,
   length: 0,
 });
 ```
 */
function throwRuntimeDiagnostics({
  diagnostics,
  index,
  length,
}: ThrowRuntimeDiagnosticsOptions): never {
  throw new ArrayAtError({
    diagnostics,
    index,
    length,
  },);
}

/**
 Validates runtime index and returns resolved non-negative slot.
 
 Safe-integer and empty-array checks are independent, so both diagnostics are
 returned together when both fail. Range checks require safe integer and
 non-empty array; slot assignment requires an in-range resolved index.
 
 @param array - Array requiring indexed access
 
 @param index - Requested signed index
 
 @returns Resolved assigned array slot
 
 @throws {@link ArrayAtError} with every independently actionable diagnostic
 
 @example
 ```ts
 const resolved = runtimeIndexOrThrow({ array: [10, 20], index: -1, });
 ```
 */
export function runtimeIndexOrThrow({
  array,
  index,
}: {
  readonly array: readonly unknown[];
  readonly index: number;
}): number {
  /**
   Current array length used by every dependent check.
   */
  const { length, } = array;
  /**
   Whether index arithmetic remains exact in JavaScript.
   */
  const indexIsSafe = isSafeInteger(index,);

  if ((!indexIsSafe) && (length === 0))
    throwRuntimeDiagnostics({
      diagnostics: [
        createNonSafeIntegerDiagnostic({ index, }),
        createEmptyArrayDiagnostic(),
      ],
      index,
      length,
    },);

  if (!indexIsSafe)
    throwRuntimeDiagnostics({
      diagnostics: [createNonSafeIntegerDiagnostic({ index, })],
      index,
      length,
    },);

  if (length === 0)
    throwRuntimeDiagnostics({
      diagnostics: [createEmptyArrayDiagnostic()],
      index,
      length,
    },);

  /**
   Maximum valid non-negative index.
   */
  const lastIndex = length - 1;
  /**
   Non-negative candidate after translating count-back index.
   */
  const resolvedIndex = index < 0
    ? length + index
    : index;

  if (resolvedIndex < 0)
    throwRuntimeDiagnostics({
      diagnostics: [createBeforeStartDiagnostic({
        distance: Math.abs(index) - length,
        index,
        lastIndex,
        length,
        minimumNegativeIndex: -length,
      },)],
      index,
      length,
    },);

  if (resolvedIndex >= length)
    throwRuntimeDiagnostics({
      diagnostics: [createPastEndDiagnostic({
        distance: index - lastIndex,
        index,
        lastIndex,
        length,
        minimumNegativeIndex: -length,
      },)],
      index,
      length,
    },);

  if (!Object.hasOwn(
    array,
    resolvedIndex,
  ))
    throwRuntimeDiagnostics({
      diagnostics: [createUnassignedSlotDiagnostic({
        index,
        length,
        resolvedIndex,
      },)],
      index,
      length,
    },);

  return resolvedIndex;
}
