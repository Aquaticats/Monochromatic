/**
 Value kinds and one-line renders for the failure difference walker in
 `value-difference.ts`.

 A "role" is how the differ compares two values of one kind:
 text by lines,
 ordered lists and typed buffers copy-for-copy with order,
 unordered collections copy-for-copy without order,
 records key by key,
 and leaves by identity or scalar equality.
 Kinds outside their role's reach render as leaves.

 @module
 */

import {
  readProperty,
} from './harness-frames.ts';

/**
 Characters before the kind name inside `Object.prototype.toString`
 output (`'[object '.length`),
 sized as a named constant so tag slicing names what it cuts.
 */
const TYPED_TAG_PREFIX_LENGTH = 8;

/**
 Kinds whose values carry bytes rather than entries:
 an `ArrayBuffer` holds bytes directly,
 a `DataView` holds a window onto them.
 */
const BYTE_KINDS: ReadonlySet<string> = new Set([
  'ArrayBuffer',
  'DataView',
],);

/**
 Kinds compared as one opaque value rather than walked key by key:
 time and pattern kinds by their payload,
 errors by name,
 message,
 and own enumerable keys,
 boxed primitives by their inner value alone
 (chai ignores extra boxed props),
 and reference kinds by identity.
 */
const LEAF_KINDS: ReadonlySet<string> = new Set([
  'null',
  'Date',
  'RegExp',
  'Error',
  'String',
  'Number',
  'Boolean',
  'Promise',
  'WeakMap',
  'WeakSet',
],);

//region Value kinds

/**
 The display kind of a value, matching how chai's deep equality groups it:
 boxed primitives keep their box kind,
 class instances read as `Object` since chai ignores constructor identity,
 and errors get their own kind since chai compares their message.

 @param value - value read

 @returns Kind label such as `Array`,
   `Uint8Array`,
   `Object`,
   or `number`

 @mutates value - Tag reads may invoke `Symbol.toStringTag` getters or proxy traps.

 @example
 ```ts
 valueKind([1,]);       // 'Array'
 valueKind(new Map(,),); // 'Map'
 ```
 */
export function valueKind(value: unknown,): string {
  if (value === null)
    return 'null';
  if ((typeof value) !== 'object')
    return typeof value;
  if (Array.isArray(value,))
    return 'Array';
  /**
   Object tag of the value,
   read after the real-array check since an object inheriting
   `Array.prototype` claims `Array` without being one.
   */
  const tag = Object.prototype
    .toString
    .call(value,)
    .slice(
    TYPED_TAG_PREFIX_LENGTH,
    -1,
    );
  return tag === 'Array'
    ? 'ArrayLike'
    : tag;
}

/**
 How the differ walks two values of the same kind.
 `text` splits into lines,
 `ordered` compares entries copy-for-copy with order,
 `unordered` compares entries copy-for-copy without order,
 `record` compares keys,
 and `leaf` renders both values on one line.

 @param value - value read

 @returns Role name for the value's kind

 @example
 ```ts
 valueRole(new Set([1,],),); // 'unordered'
 ```
 */
export function valueRole(value: unknown,): 'text' | 'ordered' | 'unordered' | 'record' | 'leaf' {
  /**
   Kind of the value read once,
   so each family below names its member kinds in one place.
   */
  const kind = valueKind(value,);
  if (kind === 'string')
    return 'text';
  if ((kind === 'Array') || (kind === 'Arguments'))
    return 'ordered';
  /**
   Whether the kind names a typed-array class,
   checked by suffix since each element type brings its own class name.
   */
  const isTypedArray = kind
    .endsWith('Array',);
  if (BYTE_KINDS.has(kind,) || isTypedArray)
    return 'ordered';
  if ((kind === 'Map') || (kind === 'Set'))
    return 'unordered';
  if ((typeof value) !== 'object')
    return 'leaf';
  if (LEAF_KINDS.has(kind,))
    return 'leaf';
  return 'record';
}

//endregion Value kinds

//region One-line value render

/**
 Renders one value on a single line,
 never throwing and never containing a line break,
 so one difference stays one log line.

 Strings quote and escape,
 zero keeps its sign,
 cycles mark `[Circular]`,
 and a getter that throws renders `[unrepresentable]`.

 @param value - value to render

 @returns Single-line rendering

 @mutates value - Rendering may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.

 @example
 ```ts
 renderValue({ naps: 2, },); // '{ naps: 2 }'
 renderValue('a\nb',);       // '"a\\nb"'
 ```
 */
export function renderValue(value: unknown,): string {
  return renderValueInner({
    value,
    open: [],
  },);
}

/**
 Recursive worker behind {@link renderValue},
 carrying the container stack so a self-referential value marks its cycle
 instead of recursing forever.

 @param value - value to render

 @param open - containers still being rendered above this value

 @returns Single-line rendering

 @mutates value - Rendering may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.
 */
function renderValueInner({
  value,
  open,
}: {
  readonly value: unknown;
  readonly open: readonly unknown[];
},): string {
  if (value === null)
    return 'null';
  if ((typeof value) === 'string')
    return JSON.stringify(value,);
  if ((typeof value) === 'number')
    return numberText(value,);
  if ((typeof value) === 'bigint')
    return `${value.toString()}n`;
  if ((typeof value) === 'symbol')
    return String(value,);
  if ((typeof value) === 'function')
    return functionText({
      name: value.name,
    },);
  if (value === undefined)
    return 'undefined';
  if ((typeof value) === 'boolean')
    return value ? 'true' : 'false';
  if ((typeof value) !== 'object')
    throw new Error(`unreachable: typeof ${typeof value} was not handled`,);
  if (open.includes(value,))
    return '[Circular]';
  /**
   Kind of the value,
   read once so every dispatch below compares against one label.
   */
  const kind = valueKind(value,);
  /**
   Container stack for nested renders,
   extended with this value so a child pointing back at it reads `[Circular]`.
   */
  const nested = [
    ...open,
    value,
  ];
  if (kind === 'Array')
    return listText({
      entries: arrayLikeEntries(value,),
      open: nested,
    },);
  if (kind === 'Arguments')
    return `Arguments${listText({
      entries: arrayLikeEntries(value,),
      open: nested,
    },)}`;
  if (BYTE_KINDS.has(kind,))
    return `${kind}${listText({
      entries: byteEntries(value,),
      open: nested,
    },)}`;
  if (kind.endsWith('Array',))
    return `${kind}${listText({
      entries: arrayLikeEntries(value,),
      open: nested,
    },)}`;
  if (kind === 'Map')
    return mapText({
      entries: mapEntries(value,),
      open: nested,
    },);
  if (kind === 'Set')
    return `Set${listText({
      entries: setEntries(value,),
      open: nested,
    },)}`;
  if (kind === 'Date')
    return dateText(dateValue(value,),);
  if (kind === 'RegExp')
    return regExpText(value,);
  if (Error.isError(value,))
    return errorText({
      value,
      open: nested,
    },);
  if ((kind === 'String') || (kind === 'Number')
    || (kind === 'Boolean'))
    return boxedText({
      kind,
      value,
    },);
  if ((kind === 'Promise') || (kind === 'WeakMap')
    || (kind === 'WeakSet'))
    return `[${kind}]`;
  return recordText({
    value,
    open: nested,
  },);
}

/**
 Renders a number,
 keeping the sign of zero and naming the non-finite spellings so `+0`
 against `-0` and `NaN` against `1` read apart.

 @param value - number to render

 @returns Number text

 @example
 ```ts
 numberText(-0,); // '-0'
 ```
 */
function numberText(value: number,): string {
  if (Object.is(
    value,
    -0,
  ))
    return '-0';
  if (value === 0)
    return '+0';
  return String(value,);
}

/**
 Renders a function by name so two named functions with one body read apart
 from one anonymous function.

 @param name - function name read; absent where the function is anonymous

 @returns `[Function: name]` or `[Function]`

 @example
 ```ts
 functionText({ name: 'go', },); // '[Function: go]'
 ```
 */
function functionText({ name, }: { readonly name?: string; },): string {
  return (name !== undefined) && (name !== '')
    ? `[Function: ${name}]`
    : '[Function]';
}

/**
 The indexed entries of an array-like value (array, arguments object,
 or typed array),
 read through one guarded view so the dispatch never casts inline.

 @param value - array-like value read

 @returns Entries by index

 @mutates value - `Array.from` may invoke an iterator or proxy trap.

 @example
 ```ts
 arrayLikeEntries([1, 2,],); // [1, 2]
 ```
 */
export function arrayLikeEntries(value: object,): readonly unknown[] {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- callers dispatch on `valueKind`, which returned 'Array', 'Arguments', or a typed-array class, all array-like at runtime
  return Array.from(value as ArrayLike<unknown>,);
}

/**
 The bytes an `ArrayBuffer` or `DataView` holds,
 read through one guarded view so byte renders and byte diffs agree.

 @param value - byte-bearing value read

 @returns Byte values

 @mutates value - Buffer reads may invoke proxy traps.

 @example
 ```ts
 byteEntries(new Uint8Array([1,],),); // [1]
 ```
 */
export function byteEntries(value: object,): readonly number[] {
  if (value instanceof DataView)
    return [...new Uint8Array(
      value.buffer,
      value.byteOffset,
      value.byteLength,
    )];
  if (value instanceof ArrayBuffer)
    return [...new Uint8Array(value,)];
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- callers dispatch on `valueKind`, which returned 'ArrayBuffer' or 'DataView'; the instanceof guards above catch same-realm values and this covers cross-realm ones
  return [...new Uint8Array(value as ArrayBuffer,)];
}

/**
 The entries of a kind-confirmed `Map`,
 read through one guarded view so the dispatch never casts inline.

 @param value - map value read

 @returns Map entries

 @mutates value - Map reads may invoke proxy traps.

 @example
 ```ts
 mapEntries(new Map([[1, 'a',],],),); // Map { 1 => 'a' }
 ```
 */
export function mapEntries(value: object,): ReadonlyMap<unknown, unknown> {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- callers dispatch on `valueKind`, which returned 'Map'
  return value as ReadonlyMap<unknown, unknown>;
}

/**
 The items of a kind-confirmed `Set`,
 read through one guarded view so the dispatch never casts inline.

 @param value - set value read

 @returns Set items

 @mutates value - Set iteration may invoke proxy traps.

 @example
 ```ts
 setEntries(new Set([1,],),); // [1]
 ```
 */
export function setEntries(value: object,): readonly unknown[] {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- callers dispatch on `valueKind`, which returned 'Set'
  return [...(value as ReadonlySet<unknown>)];
}

/**
 A kind-confirmed `Date`,
 read through one guarded view so date rendering never casts inline.

 @param value - date value read

 @returns Date view

 @mutates value - Later `toJSON` may invoke a user hook.
 */
function dateValue(value: object,): Date {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- callers dispatch on `valueKind`, which returned 'Date'
  return value as Date;
}

/**
 Renders a kind-confirmed `RegExp` as its literal,
 through one guarded view so the dispatch never casts inline.

 @param value - regular expression read

 @returns `/source/flags` rendering

 @mutates value - Source and flags reads may invoke proxy traps.
 */
function regExpText(value: object,): string {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- callers dispatch on `valueKind`, which returned 'RegExp' */
  /**
   Kind-confirmed expression,
   read once so the source and flags below cannot disagree.
   */
  const pattern = value as RegExp;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return `/${pattern.source}/${pattern.flags}`;
}

/**
 Renders a list body as `[ a, b ]`,
 quoting each entry through {@link renderValueInner}.

 @param entries - entries to render

 @param open - containers still being rendered above the entries

 @returns Bracketed entry list

 @mutates entries - Entry rendering may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.
 */
function listText({
  entries,
  open,
}: {
  readonly entries: readonly unknown[];
  readonly open: readonly unknown[];
},): string {
  /**
   One rendering per entry,
   collected before joining so the bracket body stays one line.
   */
  const parts: string[] = [];
  for (const entry of entries) {
    parts.push(renderValueInner({
      value: entry,
      open,
    },),);
  }
  return `[ ${parts.join(', ',)} ]`;
}

/**
 Renders a map as `Map{ k => v }`,
 entry by entry through {@link renderValueInner}.

 @param entries - map to render

 @param open - containers still being rendered above the entries

 @returns Map rendering

 @mutates entries - Entry rendering may invoke getters, proxy traps, `Symbol.toPrimitive`, `toString`, or `valueOf`.
 */
function mapText({
  entries,
  open,
}: {
  readonly entries: ReadonlyMap<unknown, unknown>;
  readonly open: readonly unknown[];
},): string {
  /**
   One `key => value` rendering per map entry,
   collected before joining so the map body stays one line.
   */
  const parts: string[] = [];
  for (const [key, entryValue] of entries) {
    parts.push(`${renderValueInner({
      value: key,
      open,
    },)} => ${renderValueInner({
      value: entryValue,
      open,
    },)}`);
  }
  return `Map{ ${parts.join(', ',)} }`;
}

/**
 Renders a date by its ISO text,
 naming `Invalid Date` where no time exists.

 @param value - date to render

 @returns Date rendering

 @mutates value - `toJSON` may invoke a user hook.

 @example
 ```ts
 dateText(new Date(0,),); // '1970-01-01T00:00:00.000Z'
 ```
 */
function dateText(value: Date,): string {
  /**
   ISO text of the date,
   absent exactly when the date holds no valid time.
   */
  const json = value.toJSON();
  return json ?? 'Invalid Date';
}

/**
 Renders an error as `Name: message` plus own enumerable keys so two
 errors differing in a carried field read apart on one line.

 @param value - error-like value to render

 @param open - containers still being rendered above the error

 @returns Error rendering

 @mutates value - `readProperty` reads and key reads may invoke getters or proxy traps.
 */
function errorText({
  value,
  open,
}: {
  readonly value: object;
  readonly open: readonly unknown[];
},): string {
  /**
   Error name read defensively,
   falling back to the label every error-like value can claim.
   */
  const label = readProperty({
    source: value,
    key: 'name',
  },);
  /**
   Error message read defensively,
   falling back to a marker when the field is absent or unprintable.
   */
  const message = readProperty({
    source: value,
    key: 'message',
  },);
  /**
   Head rendering naming the error class and its message.
   */
  const head = `${(typeof label) === 'string'
    ? label
    : 'Error'}: ${messageText(message,)}`;
  /**
   Own enumerable keys, which chai's deep equality compares on errors
   alongside the name and message.
   */
  const carried = enumerableKeys(value,);
  return carried.length === 0
    ? head
    : `${head} ${recordText({
      value,
      open,
    },)}`;
}

/**
 Prints an error message field,
 keeping an absent or non-string message visible as a marker.

 @param message - message field read

 @returns Message text or marker

 @mutates message - `String` conversion may invoke `Symbol.toPrimitive`, `toString`, or `valueOf`.
 */
function messageText(message: unknown,): string {
  if ((typeof message) === 'string')
    return message;
  if (((typeof message) === 'number') || ((typeof message) === 'boolean')
    || ((typeof message) === 'bigint'))
    return String(message,);
  return message === undefined
    ? '<no message>'
    : '<unprintable message>';
}

/**
 Renders a boxed primitive as `[String: "a"]` so a box reads apart from the
 primitive it holds.

 @param kind - box kind name

 @param value - boxed value

 @returns Boxed rendering

 @mutates value - `valueOf` may invoke a user hook.
 */
function boxedText({
  kind,
  value,
}: {
  readonly kind: string;
  readonly value: object;
},): string {
  return `[${kind}: ${renderValueInner({
    value: (value as { readonly valueOf: () => unknown; }).valueOf(),
    open: [],
  },)}]`;
}

/**
 Renders a record as `{ k: v }` over its own enumerable string and symbol
 keys,
 matching which keys chai's deep equality reads.

 @param value - record to render

 @param open - containers still being rendered above the record

 @returns Record rendering

 @mutates value - Key reads may invoke getters or proxy traps.
 */
function recordText({
  value,
  open,
}: {
  readonly value: object;
  readonly open: readonly unknown[];
},): string {
  /**
   One `key: value` rendering per enumerable key,
   collected before joining so the record body stays one line.
   */
  const parts: string[] = [];
  for (const key of enumerableKeys(value,)) {
    parts.push(`${renderValueInner({
      value: key,
      open,
    },)}: ${renderValueInner({
      value: Reflect.get(
        value,
        key,
      ),
      open,
    },)}`);
  }
  return `{ ${parts.join(', ',)} }`;
}

/**
 The own enumerable string and symbol keys of a record,
 in the order chai's deep equality compares them.
 Non-enumerable keys stay out since deep equality ignores them.

 @param value - record to read

 @returns Own enumerable keys

 @mutates value - `Reflect.ownKeys` and the enumerability read may invoke proxy traps.

 @example
 ```ts
 enumerableKeys({ a: 1, },); // ['a']
 ```
 */
export function enumerableKeys(value: object,): (string | symbol)[] {
  /**
   Enumerable own keys in `Reflect.ownKeys` order,
   filtered through `propertyIsEnumerable` so hidden keys drop out.
   */
  const keys: (string | symbol)[] = [];
  for (const key of Reflect.ownKeys(value,)) {
    if (Object.prototype
      .propertyIsEnumerable
      .call(
      value,
      key,
      ))
      keys.push(key,);
  }
  return keys;
}

//endregion One-line value render

//region Side summary for kind mismatches

/**
 Describes one side of a kind mismatch.
 Scalars render whole since they are small by nature;
 containers name their kind and size so a large container never floods a line
 that already answers how the kinds differ.

 @param value - mismatched side

 @returns Side description

 @mutates value - Kind and size reads may invoke getters or proxy traps.

 @example
 ```ts
 describeSide([1, 2,],); // 'Array of 2 items'
 describeSide(3,);       // '3'
 ```
 */
export function describeSide(value: unknown,): string {
  /**
   Kind of the mismatched side,
   read once so the size lookup below names one unit per family.
   */
  const kind = valueKind(value,);
  if ((typeof value) === 'string')
    return `String of ${value.length} characters`;
  if (((typeof value) !== 'object') || (value === null))
    return renderValue(value,);
  if ((kind === 'Array') || (kind === 'Arguments'))
    return sizedText({
      value,
      kind,
      sizeKey: 'length',
      unit: 'items',
    },);
  if (BYTE_KINDS.has(kind,))
    return sizedText({
      value,
      kind,
      sizeKey: 'byteLength',
      unit: 'bytes',
    },);
  if (kind.endsWith('Array',))
    return sizedText({
      value,
      kind,
      sizeKey: 'length',
      unit: 'items',
    },);
  if ((kind === 'Map') || (kind === 'Set'))
    return sizedText({
      value,
      kind,
      sizeKey: 'size',
      unit: 'entries',
    },);
  if (kind === 'Object')
    return `Object with ${enumerableKeys(value,)
      .length} keys`;
  return renderValue(value,);
}

/**
 Names a container's kind and size,
 keeping the size read defensive so a malformed value degrades to its bare
 kind instead of throwing mid-diagnosis.

 @param value - container to size

 @param kind - kind label already read

 @param sizeKey - property holding the count

 @param unit - what the count counts

 @returns `Kind of N unit` or the bare kind

 @mutates value - The size read may invoke a getter or proxy trap.
 */
function sizedText({
  value,
  kind,
  sizeKey,
  unit,
}: {
  readonly value: object;
  readonly kind: string;
  readonly sizeKey: string;
  readonly unit: string;
},): string {
  /**
   Container size read defensively,
   absent when the property is missing or not a number.
   */
  const size = readProperty({
    source: value,
    key: sizeKey,
  },);
  return (typeof size) === 'number'
    ? `${kind} of ${size} ${unit}`
    : kind;
}

//endregion Side summary for kind mismatches
