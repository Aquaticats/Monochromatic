//region JSON object editing: parse, merge, and omit on flat JSON objects

/* oxlint-disable no-restricted-syntax/no-nullish-union -- JSON null is a legitimate JSON data value in this union, not a nullish-absence escape. */
/**
 JSON value, mirroring the data model `JSON.parse` yields.
 The object member is an inline index signature rather than {@link JsonObject}
 so the alias recurses only through object types, avoiding a self-reference cycle.
 */
export type JsonValue =
  | string
    | number
    | boolean
    | null
    | readonly JsonValue[]
    | { readonly [key: string]: JsonValue; };
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 Mutable JSON object keyed by string, used as the working shape for edits.
 */
export type JsonObject = Record<string, JsonValue>;

/**
 Readonly JSON object view accepted by nonmutating helpers.
 */
export type ReadonlyJsonObject = Readonly<Record<string, JsonValue>>;

/**
 Checks whether a value is a non-array JSON object.
 
 @param value - Value to test.
 
 @returns Whether value is a non-null, non-array object.
 
 @example
 ```ts
 isJsonObject({});
 ```
 */
export function isJsonObject(value: unknown,): value is JsonObject {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Assigns an own enumerable data property, safe for special keys.
 
 Plain bracket assignment routes the key `__proto__` through the
 `Object.prototype` setter instead of creating an own property, silently
 dropping such a key. `Object.defineProperty` always creates an own data
 property, so JSON content carrying a `__proto__` key (legal JSON the
 parser preserves) survives every edit intact.
 
 @param target - Object receiving the property.
 
 @param key - Property key, including special keys like `__proto__`.
 
 @param value - JSON value to store.
 
 @mutates target through Object.defineProperty property creation, proxy hooks, and value retention
 
 @example
 ```ts
 setOwnJsonValue({ target: {}, key: '__proto__', value: 1 });
 ```
 */
function setOwnJsonValue(
  {
    target,
    key,
    value,
  }: {
    readonly key: string;
    readonly target: JsonObject;
    readonly value: JsonValue;
  },
): void {
  Object.defineProperty(
    target,
    key,
    {
      value,
      writable: true,
      enumerable: true,
      configurable: true,
    },
  );
}

/**
 Parses JSON text expected to hold an object.
 
 @param content - JSON text.
 
 @param label - Identifier surfaced in the error when content is not an object.
 
 @returns Parsed JSON object.
 
 @throws Error when content does not parse to a non-array object, per {@link isJsonObject}.
 
 @example
 ```ts
 parseJsonObject({ content: '{"a":1}', label: 'settings' });
 ```
 */
export function parseJsonObject(
  {
    content,
    label,
  }: {
    readonly content: string;
    readonly label: string
  },
): JsonObject {
  /**
   Raw parse result before object validation.
   */
  const parsed: unknown = JSON.parse(content,);
  if (!isJsonObject(parsed,)) throw new Error(`'${label}' must be a JSON object`,);
  return parsed;
}

/**
 Formats a JSON object with two-space indentation and no trailing newline.
 Matches how JetBrains LSP4IJ template settings store embedded JSON.
 
 @param value - JSON object to format.
 
 @returns Indented JSON text.
 
 @mutates value - `JSON.stringify` may invoke record accessors or proxy traps.
 
 @example
 ```ts
 formatJsonObject({ value: { a: 1 } });
 ```
 */
export function formatJsonObject({ value, }: { value: JsonObject; },): string {
  return JSON.stringify(
    value,
    null,
    2,
  );
}

/**
 Copies an object without one key, preserving the order of remaining keys
 by assigning each via {@link setOwnJsonValue}.
 
 @param object - Source object.
 
 @param key - Key to exclude.
 
 @returns Copy without key.
 
 @example
 ```ts
 omitJsonKey({ object: { a: 1, b: 2 }, key: 'a' });
 ```
 */
export function omitJsonKey(
  {
    object,
    key,
  }: {
    readonly key: string;
    readonly object: ReadonlyJsonObject;
  },
): JsonObject {
  /**
   Working copy without the omitted key.
   */
  const updated: JsonObject = {};
  for (const [entryKey, entryValue,] of Object.entries(object,)) {
    if (entryKey !== key) {
      setOwnJsonValue({
        target: updated,
        key: entryKey,
        value: entryValue,
      },);
    }
  }
  return updated;
}

/**
 Merges scalar overrides and array unions into a flat JSON object.
 Existing keys keep their position; new keys append in argument order, so
 stable inputs produce byte-stable {@link formatJsonObject} output. `set` is applied
 before `arrayUnion`; array unions keep existing string members then append new
 ones, deduplicated.
 
 @param base - Source object copied before edits.
 
 @param set - Scalar or structured values to assign by key, each via {@link setOwnJsonValue}.
 
 @param arrayUnion - String arrays unioned into the existing array at each key.
 
 @returns Merged copy of base.
 
 @example
 ```ts
 mergeFlatJson({ base: { a: 1 }, set: { b: false }, arrayUnion: { c: ['x'] } });
 ```
 */
export function mergeFlatJson(
  {
    base,
    set,
    arrayUnion,
  }: {
    readonly arrayUnion?: Readonly<Record<string, readonly string[]>>;
    readonly base: ReadonlyJsonObject;
    readonly set?: ReadonlyJsonObject;
  },
): JsonObject {
  /**
   Working copy of base with edits applied.
   */
  const updated: JsonObject = { ...base, };
  if (set !== undefined) {
    for (const [key, value,] of Object.entries(set,)) {
      setOwnJsonValue({
        target: updated,
        key,
        value,
      },);
    }
  }
  if (arrayUnion !== undefined) {
    for (const [key, additions,] of Object.entries(arrayUnion,)) {
      /**
       Existing own value at the key, or null when the key is not an own
       property, so inherited members of special keys never leak in.
       */
      const existing = Object.hasOwn(
        updated,
        key,
      )
        ? updated[key]
        : null;
      /**
       Existing string members, ignoring non-string entries.
       */
      const existingMembers = Array.isArray(existing,)
        ? existing.filter(function keepString(member,): member is string {
          return (typeof member) === 'string';
        },)
        : [];
      setOwnJsonValue({
        target: updated,
        key,
        value: [...new Set([
          ...existingMembers,
          ...additions,
        ],),],
      },);
    }
  }
  return updated;
}

/**
 Adds default keys to an object only where absent, preserving existing values.
 
 @param base - Source object copied before edits.
 
 @param defaults - Values applied only for keys missing from base, each via {@link setOwnJsonValue}.
 
 @returns Merged copy of base.
 
 @example
 ```ts
 mergeObjectDefaults({ base: { a: 1 }, defaults: { a: 9, b: 2 } });
 // { a: 1, b: 2 }
 ```
 */
export function mergeObjectDefaults(
  {
    base,
    defaults,
  }: {
    readonly base: ReadonlyJsonObject;
    readonly defaults: ReadonlyJsonObject;
  },
): JsonObject {
  /**
   Working copy of base with defaults applied.
   */
  const updated: JsonObject = { ...base, };
  for (const [key, value,] of Object.entries(defaults,)) {
    // Fill only genuinely absent keys; an own value (including an explicit
    // null or false) is an existing value and is preserved, matching the
    // documented contract. The own-property check also keeps special keys
    // like __proto__ from reading inherited members.
    if (!Object.hasOwn(
      updated,
      key,
    )) {
      setOwnJsonValue({
        target: updated,
        key,
        value,
      },);
    }
  }
  return updated;
}

//endregion JSON object editing
