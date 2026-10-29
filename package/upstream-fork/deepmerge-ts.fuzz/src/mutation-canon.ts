/**
 Canonical serialization of a merge outcome for `./mutation-differential.ts`:
 two builds behave the same on one case exactly when their serializations
 are equal.

 The serialization records, for every object reachable from the result and
 the inputs after the call: whether it is fresh, an input node (by its
 position in a walk of the inputs taken before the call), or a back
 reference; its prototype and `Object.prototype.toString` tag; Date time,
 Map entries, and Set items; and every own key with its descriptor flags and
 data value. Getters are never invoked. A throw serializes as its error
 class.

 @module
 */

/**
 Node shape in the serialization.
 */
type CanonNode = {
  readonly id: number;
  readonly origin: string;
  readonly detail: unknown;
};

/**
 Whether a value can hold identity.

 @param value - Any value.

 @returns True for objects and functions.

 @example
 ```ts
 hasIdentity({}); // true
 ```
 */
function hasIdentity(value: unknown,): value is object {
  return (((typeof value) === 'object') && (value !== null)) || ((typeof value) === 'function');
}

/**
 Label for a prototype: the built-ins by name, anything else by its tag.

 @param value - Object whose prototype is labelled.

 @returns Stable label.

 @example
 ```ts
 prototypeLabel([]); // 'Array'
 ```
 */
function prototypeLabel(value: object,): string {
  /**
   Prototype of `value`.
   */
  const prototype: unknown = Object.getPrototypeOf(value,);
  /**
   Built-in prototypes by name.
   */
  const named = new Map<unknown, string>([
    [
      null,
      'null',
    ],
    [
      Object.prototype,
      'Object',
    ],
    [
      Array.prototype,
      'Array',
    ],
    [
      Map.prototype,
      'Map',
    ],
    [
      Set.prototype,
      'Set',
    ],
    [
      Date.prototype,
      'Date',
    ],
  ],);
  return named.get(prototype,) ?? `other:${Object.prototype
    .toString
    .call(prototype,)}`;
}

/**
 Values an object holds directly: data property values, Map keys and values,
 and Set items. Reading a revoked Proxy throws.

 @param value - Object to read.

 @returns Held values in a fixed order.

 @example
 ```ts
 heldValues(new Map([[1, 2,],],),); // [1, 2]
 ```
 */
function heldValues(value: object,): readonly unknown[] {
  /**
   Map entries flattened to key, value, key, value.
   */
  const entries: readonly unknown[] = (value instanceof Map) ? [...(value as ReadonlyMap<unknown, unknown>),].flat() : [];
  /**
   Set items.
   */
  const items: readonly unknown[] = (value instanceof Set) ? [...(value as ReadonlySet<unknown>),] : [];
  /**
   Own data property values.
   */
  const properties = Reflect.ownKeys(value,)
    .map(function descriptorOf(key,) {
      return Object.getOwnPropertyDescriptor(
        value,
        key,
      );
    },)
    .filter(function isData(descriptor,) {
      return (descriptor !== undefined) && ('value' in descriptor);
    },)
    .map(function valueOf(descriptor,): unknown {
      return descriptor?.value;
    },);
  return [
    ...entries,
    ...items,
    ...properties,
  ];
}

/**
 Number every object reachable from the inputs, in walk order.

 @param inputs - Merge arguments before the call.

 @returns Position of each input node.

 @example
 ```ts
 const labels = labelInputs([{ a: {}, },]);
 ```
 */
export function labelInputs(inputs: readonly unknown[],): ReadonlyMap<object, number> {
  /**
   Labels assigned so far.
   */
  const labels = new Map<object, number>();
  /**
   Values still to visit, in order.
   */
  const pending: unknown[] = [...inputs,];
  // A for...of over an array visits elements pushed during the loop, so this is a breadth-first walk.
  for (const value of pending) {
    if (hasIdentity(value,) && (!labels.has(value,))) {
      labels.set(
        value,
        labels.size,
      );
      try {
        pending.push(...heldValues(value,),);
      }
      catch (error) {
        // A revoked Proxy throws a TypeError on any read: it keeps its label and has no readable children.
        if (!(error instanceof TypeError))
          throw error;
      }
    }
  }
  return labels;
}

/**
 Prototype, tag, and contents of one object; a throwing read (revoked Proxy)
 records the error class instead.

 @param value - Object to describe.

 @param walk - Serializer for the values it holds.

 @returns Prototype label, tag, time, entries, items, and own keys with descriptor flags.

 @example
 ```ts
 describeObject({ value: new Set([1,],), walk: String, });
 ```
 */
function describeObject(
  {
    value,
    walk,
  }: {
    readonly value: object;
    readonly walk: (held: unknown,) => unknown;
  },
): unknown {
  try {
    return {
      entries: (value instanceof Map)
        ? [...value,].map(function entry([key, entryValue,],) {
          return [
            walk(key,),
            walk(entryValue,),
          ];
        },)
        : null,
      items: (value instanceof Set) ? [...value,].map(function item(held,) {
        return walk(held,);
      },) : null,
      keys: Reflect.ownKeys(value,)
        .map(function describe(key,) {
          /**
           Own descriptor of `key`.
           */
          const descriptor = Object.getOwnPropertyDescriptor(
            value,
            key,
          );
          return [
            walk(key,),
            [
              descriptor?.enumerable === true ? 'e' : '',
              descriptor?.writable === true ? 'w' : '',
              descriptor?.configurable === true ? 'c' : '',
              descriptor?.get === undefined ? '' : 'g',
              descriptor?.set === undefined ? '' : 's',
            ].join('',),
            (descriptor !== undefined) && ('value' in descriptor) ? walk(descriptor.value,) : null,
          ];
        },),
      proto: prototypeLabel(value,),
      tag: Object.prototype
        .toString
        .call(value,),
      time: (value instanceof Date) ? value.getTime() : null,
    };
  }
  catch (error) {
    return { error: (Error.isError(error,)) ? error.name : typeof error, };
  }
}

/**
 Origin label of one object.

 @param label - Its input position, when it was an input node.

 @returns `input:<position>` for input nodes, `fresh` otherwise.

 @example
 ```ts
 originOf(labels.get(value,),);
 ```
 */
function originOf(label?: number,): string {
  return (label === undefined) ? 'fresh' : `input:${String(label,)}`;
}

/**
 Serialize roots after a call.

 @param roots - Result and inputs after the call.

 @param labels - Input positions from {@link labelInputs} before the call.

 @returns Canonical JSON text.

 @example
 ```ts
 canonicalize({ labels: labelInputs(inputs,), roots: [result, ...inputs,], });
 ```
 */
export function canonicalize(
  {
    roots,
    labels,
  }: {
    readonly roots: readonly unknown[];
    readonly labels: ReadonlyMap<object, number>;
  },
): string {
  /**
   Serialization id of each object already visited.
   */
  const seen = new Map<object, number>();
  /**
   Serialize one value; objects recurse through {@link describeObject}.

   @param value - Value to serialize.

   @returns Primitive label, back reference, or node.

   @example
   ```ts
   walk({});
   ```
   */
  function walk(value: unknown,): unknown {
    if ((typeof value) === 'symbol')
      return `symbol:${String(value.description,)}`;
    if (Object.is(
      value,
      -0,
    ))
      return 'number:-0';
    if (!hasIdentity(value,))
      return `${typeof value}:${String(value,)}`;
    /**
     Id of an earlier visit.
     */
    const known = seen.get(value,);
    if (known !== undefined)
      return `ref:${String(known,)}`;
    /**
     Id of this visit, fixed before its children are visited.
     */
    const id = seen.size;
    seen.set(
      value,
      id,
    );
    return {
      detail: describeObject({
        value,
        walk,
      },),
      id,
      origin: originOf(labels.get(value,),),
    } satisfies CanonNode;
  }
  return JSON.stringify(roots.map(walk,),);
}

/**
 Run one call and serialize what it left behind.

 @param inputs - Merge arguments, labelled before the call.

 @param run - Call returning the roots to serialize (result and inputs).

 @returns Canonical text, or `throw:<error class>`.

 @example
 ```ts
 outcomeOf({ inputs, run: () => [deepmerge(...inputs), ...inputs,], });
 ```
 */
export function outcomeOf(
  {
    inputs,
    run,
  }: {
    readonly inputs: readonly unknown[];
    readonly run: () => readonly unknown[];
  },
): string {
  /**
   Input positions before the call mutates anything.
   */
  const labels = labelInputs(inputs,);
  try {
    return canonicalize({
      labels,
      roots: run(),
    },);
  }
  catch (error) {
    return `throw:${(Error.isError(error,)) ? error.name : typeof error}`;
  }
}
