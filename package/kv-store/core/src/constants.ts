/**
 Sentinel marking the absence of a value across the store's read pipeline.
 
 Distinguishes "no backend holds this key" (and "no key was evicted") from any
 real serialized or deserialized value, including a stored `null`. A unique
 module-local `Symbol` is used so it can never collide with a domain value:
 strings, numbers, `null`, and `undefined` are all values the store may
 legitimately round-trip, so none of them can serve as the missing marker.
 
 Narrow against it with an identity check (`value === ABSENT`).
 
 @example
 ```ts
 const value = await store.get<number>('count');
 if (value === ABSENT) {
   // key is missing; seed a default
 }
 ```
 */
export const ABSENT: unique symbol = Symbol('kv-store/value absent from every backend',);
