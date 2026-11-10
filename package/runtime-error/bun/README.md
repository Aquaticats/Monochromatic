# runtime-error-bun

Standalone scripts that each trigger a specific runtime error category.
Used for testing error handling,
 crash reporting,
 and observing
how the Bun runtime surfaces different failure modes.

## Scripts

- `abort.ts`:
   `process.abort()` (SIGABRT,
   core dump)
- `infinite-loop.ts`:
   infinite loop (hangs the event loop)
- `not-a-function.ts`:
   calling a non-function value (TypeError)
- `null-deref.ts`:
   property access on null (TypeError)
- `oom.ts`:
   out-of-memory allocation (heap exhaustion)
- `stack-overflow.ts`:
   unbounded recursion (RangeError)
- `uncaught-throw.ts`:
   unhandled thrown error
- `unhandled-rejection.ts`:
   unhandled promise rejection

## Running

```sh
bun package/runtime-error/bun/src/abort.ts
```

## Testing

```sh
mise run //package/runtime-error/bun:test:unit
```

`src/documented-outcome.unit.test.ts` runs every fixture under Bun
and asserts the exit code,
terminating signal,
and error marker its TSDoc promises.
