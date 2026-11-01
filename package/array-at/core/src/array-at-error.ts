/**
 Aggregated runtime error for `arrayAt` and safe-integer proof helpers.
 
 @module
 */

import type {
  NonEmptyRuntimeArrayAtDiagnostics,
  RuntimeArrayAtDiagnostic,
} from './diagnostic-types.ts';

/**
 Construction options for aggregated array-access error.
 
 @example
 ```ts
 declare const options: ArrayAtErrorOptions;
 const error = new ArrayAtError(options);
 ```
 */
export type ArrayAtErrorOptions = {
  readonly diagnostics: NonEmptyRuntimeArrayAtDiagnostics;
  readonly index: number;
  readonly length?: number;
};

/**
 Error carrying every independently actionable runtime diagnostic.
 
 Diagnostic order is intentionally not part of the interface. `message` joins
 diagnostic messages with newlines so ordinary logs retain all evidence.
 Category-specific metadata lives on each diagnostic; common call context
 remains available as `index` and `length`.
 
 @example
 ```ts
 try {
   arrayAt({ array: [], index: 1.5, });
 }
 catch (error) {
   if (error instanceof ArrayAtError)
     error.diagnostics.map(({ code, }) => code);
 }
 ```
 */
export class ArrayAtError extends Error {
  /**
   Runtime diagnostics detected for one operation.
   
   @example
   ```ts
   const codes = error.diagnostics.map(({ code, }) => code);
   ```
   */
  readonly diagnostics: readonly RuntimeArrayAtDiagnostic[];

  /**
   Requested index.
   
   @example
   ```ts
   const requested = error.index;
   ```
   */
  readonly index: number;

  /**
   Array length when operation included an array.
   
   Safe-integer proof helpers set this to `undefined` because they validate a
   number without array context.
   
   @example
   ```ts
   const available = error.length;
   ```
   */
  readonly length?: number;

  /**
   Creates aggregated array-access error.
   
   @param diagnostics - Non-empty runtime diagnostic collection
   
   @param index - Requested numeric index
   
   @param length - Array length when operation includes array context
   
   @example
   ```ts
   const error = new ArrayAtError({
     diagnostics: [diagnostic],
     index: 1.5,
   });
   ```
   */
  constructor({
    diagnostics,
    index,
    length,
  }: ArrayAtErrorOptions) {
    /**
     Immutable diagnostic snapshot detached from caller-owned collection.
     */
    const frozenDiagnostics = Object.freeze([...diagnostics]);
    /**
     Newline-joined messages retained by ordinary Error reporting.
     */
    const combinedMessage = frozenDiagnostics
      .map(function diagnosticMessage({ message, }) {
        return message;
      },)
      .join('\n');
    super(combinedMessage);
    this.name = 'ArrayAtError';
    this.diagnostics = frozenDiagnostics;
    this.index = index;
    if (length !== undefined)
      this.length = length;
  }
}
