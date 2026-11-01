//region JSON record guard
// Shared narrowing for probing parsed JSON of unknown shape; both provider
// protocol parsing (quotas, completions) and model-content validation build on it.

/**
 Narrows unknown JSON to a plain record for field probing. An array is
 refused: every reader that probes named fields reads an object, and an array
 let through read as an event naming no type, a resumable cache record, or
 answers keyed "0" and "1" (ledger B92). The package's other checks for an
 object call this one, which a source scan holds (`record-checks.unit.test.ts`).

 @param value - candidate from parsed JSON

 @returns Whether value can be probed for properties

 @example
 ```ts
 if (isJsonRecord(parsed,)) probe(parsed['choices'],);
 ```
 */
export function isJsonRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Narrows unknown JSON to an element-unknown array,
 avoiding the `any[]` that bare `Array.isArray` narrowing introduces.

 @param value - candidate from parsed JSON

 @returns Whether value is an array of unknowns

 @example
 ```ts
 if (isJsonArray(parsed,)) probe(parsed[0],);
 ```
 */
export function isJsonArray(value: unknown,): value is readonly unknown[] {
  return Array.isArray(value,);
}

//endregion JSON record guard

//region JSON parse refusal
// `JSON.parse` over a string refuses only with a `SyntaxError`; a catch around
// it narrows to that refusal here, where a case reaches the rethrow (ledger
// T8, ninth batch), rather than rethrowing inline where none can.

/**
 The syntax refusal a catch around `JSON.parse` holds, for a catch that acts
 on the refusal alone.

 @param error - what the catch caught

 @returns The refusal

 @throws The caught value unchanged when it is anything but a syntax
 refusal, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireJsonSyntaxRefusal({ error, },);
 ```
 */
export function requireJsonSyntaxRefusal({ error, }: { readonly error: unknown; },): SyntaxError {
  if (error instanceof SyntaxError)
    return error;
  throw error;
}

//endregion JSON parse refusal
