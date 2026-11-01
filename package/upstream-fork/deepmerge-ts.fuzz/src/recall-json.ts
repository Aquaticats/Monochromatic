/**
 Field access on parsed JSON and caught values for the historical-recall
 scripts, without casts.

 @module
 */

/**
 Read one field of a value that may not be an object.

 @param value - Parsed JSON, a caught value, or anything else.

 @param key - Field name.

 @returns The field, or `undefined` when `value` is not an object.

 @example
 ```ts
 fieldOf({ key: 'a', value: JSON.parse('{"a":1}',), }); // 1
 ```
 */
export function fieldOf(
  {
    value,
    key,
  }: {
    readonly value: unknown;
    readonly key: string;
  },
): unknown {
  return (((typeof value) === 'object') && (value !== null)) ? Reflect.get(
    value,
    key,
  ) : undefined;
}

/**
 Read a path of fields.

 @param value - Root value.

 @param path - Field names from the root.

 @returns The value at the path, or `undefined` when a step is missing.

 @example
 ```ts
 pathOf({ path: ['a', 'b',], value: { a: { b: 2, }, }, }); // 2
 ```
 */
export function pathOf(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: readonly string[];
  },
): unknown {
  return path.reduce<unknown>(
    function step(
      current,
      key,
    ) {
      return fieldOf({
        key,
        value: current,
      },);
    },
    value,
  );
}
