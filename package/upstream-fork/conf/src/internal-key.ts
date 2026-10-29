/**
 Reserved bookkeeping key handling for the config store.
 
 The store keeps migration bookkeeping in the config file under
 `__internal__` so it survives rewrites,
 but that key must never surface through the user-facing store.
 
 @module
 */

//region Keys

/**
 Key under which all internal bookkeeping is stored.
 
 @example
 ```ts
 INTERNAL_KEY; // '__internal__'
 ```
 */
export const INTERNAL_KEY = '__internal__';

/**
 Dotted path of the recorded migration version inside {@link INTERNAL_KEY}.
 
 @example
 ```ts
 MIGRATION_KEY; // '__internal__.migrations.version'
 ```
 */
export const MIGRATION_KEY: string = `${INTERNAL_KEY}.migrations.version`;

//endregion Keys

//region Predicates

/**
 Reports whether one key name addresses the reserved bookkeeping subtree.
 
 @param candidate - Key name exactly as the caller wrote it.
 
 @returns `true` for `__internal__` and any `__internal__.`-prefixed path.
 
 @example
 ```ts
 isReservedKeyPath('__internal__.migrations.version'); // => true
 isReservedKeyPath('theme'); // => false
 ```
 */
export function isReservedKeyPath(candidate: string,): boolean {
  return (candidate === INTERNAL_KEY) || candidate.startsWith(`${INTERNAL_KEY}.`,);
}

/**
 Reports whether a value is a plain object worth recursing into.
 
 @param value - Candidate value from a nested walk.
 
 @returns `true` when the value is a non-null object.
 
 @example
 ```ts
 isRecord({}); // => true
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Reports whether a whole `set` payload contains a reserved key anywhere in
 its nested shape.
 
 Strings are checked as key paths;
 objects are walked breadth-first with a visited set so caller-supplied
 cyclic structures terminate instead of looping,
 a boundedness improvement over upstream `conf`'s unbounded recursion.
 
 @param value - Key string or nested value object from a `set` call.
 
 @returns `true` when any key at any depth is reserved.
 
 @example
 ```ts
 containsReservedKey({ nested: { __internal__: {}, }, }); // => true
 containsReservedKey({ theme: 'dark', }); // => false
 ```
 */
export function containsReservedKey(value: unknown,): boolean {
  if ((typeof value) === 'string')
    return isReservedKeyPath(value,);
  if (!isRecord(value,))
    return false;
  /**
   Objects already expanded,
   so cyclic caller payloads terminate instead of looping.
   */
  const visited = new WeakSet<object>();
  /**
   Breadth-first work stack of nested objects still to scan.
   */
  const pending: Record<string, unknown>[] = [value,];
  while (pending.length > 0) {
    /**
     Object whose own key names are checked in this step.
     */
    const current = pending.pop();
    if ((current === undefined) || visited.has(current,))
      continue;
    visited.add(current,);
    for (const [candidateKey, candidateValue,] of Object.entries(current,)) {
      if (isReservedKeyPath(candidateKey,))
        return true;
      if (isRecord(candidateValue,))
        pending.push(candidateValue,);
    }
  }
  return false;
}

//endregion Predicates
