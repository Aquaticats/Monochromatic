/**
 Order-insensitive structural equality for TOML-shaped values.
 
 The enforcement engine compares each manifest's current effective value
 against the canonical value and writes only on a real difference, because
 re-setting an already-correct value reformats it (`["derive"]` becomes
 `[ "derive", ]`). Object key order in the parsed value follows source order,
 which need not match a canonical literal's order, so plain `JSON.stringify`
 comparison would report spurious differences; this walk ignores key order.
 
 @module
 */

/**
 Whether an unknown value is a plain object (not null, not an array).
 
 @param value - Candidate whose object-ness gates recursive key comparison
 
 @returns Whether value is a non-null non-array object
 
 @example
 ```ts
 isPlainObject({ version: '1', }); // true
 ```
 */
function isPlainObject(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Structural, order-insensitive equality for JSON-shaped values.
 
 Primitives compare by identity; arrays compare element-wise in order (TOML
 arrays are ordered); objects compare by key set and per-key recursion, so a
 canonical literal need not preserve the manifest's key order to match.
 
 @param left - Effective value read from a manifest
 
 @param right - Canonical value to compare against
 
 @returns Whether both describe the same TOML value
 
 @example
 ```ts
 deepEqual({
   left: { features: ['derive'], version: '1' },
   right: { version: '1', features: ['derive'] },
 }); // true
 ```
 */
export function deepEqual(
  {
    left,
    right,
  }: {
    readonly left: unknown;
    readonly right: unknown;
  },
): boolean {
  if (left === right)
    return true;

  if (Array.isArray(left,) || Array.isArray(right,)) {
    if ((!Array.isArray(left,)) || (!Array.isArray(right,)))
      return false;
    if (left.length !== right.length)
      return false;
    return left.every(function elementEqual(
      item,
      index,
    ): boolean {
      return deepEqual({
        left: item,
        right: right[index],
      },);
    },);
  }

  if (isPlainObject(left,) && isPlainObject(right,)) {
    /**
     Own enumerable keys of the manifest-side value; the canonical side must
     carry the same set for equality.
     */
    const leftKeys = Object.keys(left,);
    if (leftKeys.length
      !== Object.keys(right,)
        .length)
      return false;
    return leftKeys.every(function keyEqual(key,): boolean {
      return Object.hasOwn(
        right,
        key,
      )
        && deepEqual({
          left: left[key],
          right: right[key],
        },);
    },);
  }

  return false;
}
