//region Probe run clock
// The instant a probe run stamps on its record, read where only a real
// process can read it and handed to the library as a function, so a case
// supplies its own.

/**
 Instant the process's clock reads now.

 @returns ISO 8601 instant

 @example
 ```ts
 const startedAt = nowAsIso();
 ```
 */
export function nowAsIso(): string {
  return new Date().toISOString();
}

//endregion Probe run clock
