# test-fixture-data-sequence

Reusable numeric test data sequences for benchmarking,
 async pattern testing,
 and generator behavior validation.

## Exports

- **`array0to999`**:
   static 1000-element array of consecutive integers (typed readonly)
- **`gen0to999`**:
   sync generator yielding 0 to 999
- **`gen0to999error`**:
   sync generator yielding 0 to 998,
   then throws RangeError
- **`gen0to999Async`**:
   async generator yielding 0 to 999
- **`gen0to999errorAsync`**:
   async generator yielding 0 to 998,
   then throws RangeError
- **`gen0to999AsyncSlow`**:
   async generator with progressive delays (i ms per value)
- **`gen0to999errorAsyncSlow`**:
   async generator with progressive delays,
   then throws
- **`promises0to999`**:
   1000 promises resolving to their index after index-ms delay
- **`generateConsecutiveArray(count)`**:
   parameterized array factory
- **`generateProgressivePromises(count)`**:
   parameterized promise array factory

## Design notes

The 0-999 range includes a falsy value (0) and 999 truthy values,
making it useful for truthiness predicate testing.
Progressive-delay variants simulate real-world async operations with varying response times.
