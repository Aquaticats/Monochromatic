import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l, } from '../logger.ts';

/**
 Maximum preview length for inspect output
 */
const INSPECT_PREVIEW_LENGTH = 200;

/**
 Logs the current value and returns it unchanged (a debug tap).
 Preserves the input type so it can be inserted anywhere in a pipeline
 without breaking type inference.
 
 @param value - Value to log and pass through
 
 @returns Same value, unmodified
 
 @mutates value - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps for non-string values.
 
 @example
 ```ts
 const result = inspect(await readCached('./src/index.ts'));
 ```
 */
export function inspect<const TValue,>(value: TValue,): TValue {
  /**
   Function-scoped logger tagged with the call site for traceable pipeline inspection.
   */
  const rl = tagged({
    tag: inspect.name,
    l,
  },);
  /**
   Stringified representation for logging
   */
  const preview = ((typeof value) === 'string')
    ? value
    : JSON.stringify(
      value,
      null,
      2,
    );
  rl.info(
    preview.length
      > INSPECT_PREVIEW_LENGTH
      ? `${
        preview.slice(
          0,
          INSPECT_PREVIEW_LENGTH,
        )
      }...`
      : preview,
  );
  return value;
}
