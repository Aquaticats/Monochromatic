import decircular from 'decircular';

import { logger as defaultLogger, } from '@monochromatic-dev/logger-core/ts';

import { hasCycle, } from './has-cycle.ts';
import type { Serializer, } from './types.ts';

/**
 Max characters for value previews in log messages.
 */
const DEFAULT_LOG_LIMIT = 100;

/**
 Named-parameter options for value serialization with cycle handling.
 
 @example
 ```ts
 const opts: SerializeOptions = {
   value: { x: 1 },
   serializer: JSON.stringify,
   lossyForCircular: true,
 };
 ```
 */
export type SerializeOptions = {
  /**
   Input data to serialize.
   */
  readonly value: unknown;
  /**
   Serialization function that converts a value to a string.
   */
  readonly serializer: Serializer;
  /**
   When `true`, cyclic graphs are decycled and persisted lossy
   instead of throwing. When `false`, throws `TypeError` on cycles.
   */
  readonly lossyForCircular: boolean;
};

/**
 Serialize a value for storage, handling cyclic graphs based on configuration.
 
 Cycles are detected with {@link hasCycle}. When the value contains circular
 references and `lossyForCircular` is `true`, the value is decycled (lossy)
 and a warning is logged. When `lossyForCircular` is `false`, a `TypeError`
 is thrown instead.
 
 Internal helper shared by both store variants; not part of the package public API.
 
 @param options - serialization configuration
 
 @returns serialized string representation
 
 @throws TypeError when value has cycles and `lossyForCircular` is `false`
 
 @example
 Acyclic value:
 ```ts
 const serialized = serializeValue({
   value: { x: 1 },
   serializer: JSON.stringify,
   lossyForCircular: true,
 });
 // '{"x":1}'
 ```
 
 @example
 Cyclic value with lossy allowed:
 ```ts
 const obj: Record<string, unknown> = { a: 1 };
 obj.self = obj;
 const serialized = serializeValue({
   value: obj,
   serializer: JSON.stringify,
   lossyForCircular: true,
 });
 // Logs warning, returns decycled serialization
 ```
 
 @example
 Cyclic value with lossy disallowed:
 ```ts
 const obj: Record<string, unknown> = {};
 obj.self = obj;
 serializeValue({
   value: obj,
   serializer: JSON.stringify,
   lossyForCircular: false,
 });
 // throws TypeError
 ```
 */
export function serializeValue(options: SerializeOptions,): string {
  /**
   Inputs destructured from options for direct use inside the function body.
   */
  const {
    value,
    serializer,
    lossyForCircular,
  } = options;

  if (hasCycle(value,)) {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- hasCycle verified value is an object with cycles */
    /**
     Cycle-stripped copy of value used so the serializer cannot blow the stack.
     */
    const decycled = decircular(value as object,);
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    /**
     Serialised representation of the decycled value.
     */
    const serialized = serializer(decycled,);
    if (!lossyForCircular) {
      throw new TypeError(
        `Cannot store value perfectly because it has cycles: ${
          serialized.slice(
            0,
            DEFAULT_LOG_LIMIT,
          )
        }`,
      );
    }
    defaultLogger.warn(
      `Value has cycles, storing decycled version: ${
        serialized.slice(
          0,
          DEFAULT_LOG_LIMIT,
        )
      }`,
    );
    return serialized;
  }
  return serializer(value,);
}
