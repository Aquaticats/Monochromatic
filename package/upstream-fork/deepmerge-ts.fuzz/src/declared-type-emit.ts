/**
 TypeScript emission of drawn declared-type cases.

 Each case passes every input as `widen<T>(value)` (so the declared type, not
 the literal, drives inference) and assigns the runtime result, written as a
 literal, to a variable of the call's static result type. `tsc` accepting a
 case proves the runtime result fits the declared-input result type.

 @module
 */

import {
  emitType,
  SYMBOL_NAMES,
} from './declared-type-ast.ts';
import {
  CUSTOM_OPTIONS,
  type DrawnCase,
} from './declared-type-generate.ts';
import { RUNTIME_SYMBOLS, } from './declared-type-sample.ts';

/**
 Constant name for a runtime symbol key.

 @param symbol - Key found on a value.

 @returns Name declared in the generated header.

 @throws When the symbol is not one of {@link RUNTIME_SYMBOLS}.

 @example
 ```ts
 symbolName(RUNTIME_SYMBOLS.SYM_A); // 'SYM_A'
 ```
 */
function symbolName(symbol: symbol,): string {
  /**
   Matching constant, when the symbol is one of ours.
   */
  const found = Object.entries(RUNTIME_SYMBOLS,)
    .find(function matches([, candidate,]: readonly [
      string,
      symbol
    ],) {
      return candidate === symbol;
    },);
  if (found === undefined)
    throw new Error(`unknown symbol key ${String(symbol,)}`,);
  return found[0];
}

/**
 Counter tuple type source of a given length, for the header's `ElemAt`.

 @param length - Tuple length.

 @returns Source such as `[unknown, unknown]`.

 @example
 ```ts
 counterTuple(2); // '[unknown, unknown]'
 ```
 */
function counterTuple(length: number,): string {
  return `[${Array.from(
    { length, },
    function slot() {
    return 'unknown';
  },
  )
    .join(', ',)}]`;
}

/**
 Key source and key type of an own property.

 @param key - Own key.

 @returns Computed symbol constant or quoted string, with its type source.

 @example
 ```ts
 keyParts('a'); // ['"a"', '"a"']
 ```
 */
function keyParts(key: string | symbol,): readonly [
  string,
  string
] {
  if ((typeof key) === 'string')
    return [
      JSON.stringify(key,),
      JSON.stringify(key,),
    ];
  /**
   Header constant naming this symbol.
   */
  const name = symbolName(key as symbol,);
  return [
    `[${name}]`,
    `typeof ${name}`,
  ];
}

/**
 Write a runtime result as a TypeScript expression checked against the static
 type at its position.

 Objects and arrays are contextually typed by the assignment. Sets and Maps get
 explicit type arguments computed from `typeExpression`, the static type at this
 position expressed through the header's path helpers (`PropOf`, `ElemAt`,
 `SetElem`, `MapKey`, `MapValue`), so every element is checked against the
 declared-input result type and never re-inferred. A position whose static
 type is not a Set or Map yields `never` arguments, which fail as they should.

 @param value - Runtime result or a part of it.
 
 @param typeExpression - Static type at this position.

 @returns Expression source.

 @throws When the value has no literal form.

 @example
 ```ts
 emitValue({ value: new Set([1,]), typeExpression: 'typeof merged', });
 // 'new Set<SetElem<typeof merged>>([1])'
 ```
 */
export function emitValue({
  value,
  typeExpression,
}: {
  readonly value: unknown;
  readonly typeExpression: string
},): string {
  if (value === undefined)
    return 'undefined';
  if ((value === null) || ((typeof value) === 'boolean')
    || ((typeof value) === 'number')
    || ((typeof value) === 'string'))
    return JSON.stringify(value,);
  if (value instanceof Date)
    return `new Date(${String(value.getTime(),)})`;
  if (value instanceof RegExp)
    return `new RegExp(${JSON.stringify(value.source,)}, ${JSON.stringify(value.flags,)})`;
  if (Array.isArray(value,)) {
    // Position as two counter tuples (index from the start, distance from the end), so ElemAt can
    // resolve both fixed prefixes and fixed tails around a rest element.
    return `[${value.map(function element(
      item: unknown,
      index: number,
    ) {
      return emitValue({
        typeExpression: `ElemAt<${typeExpression}, ${counterTuple(index,)}, ${counterTuple(value.length - index
          - 1,)}>`,
        value: item,
      },);
    },)
      .join(', ',)}]`;
  }
  if (value instanceof Set) {
    return `new Set<SetElem<${typeExpression}>>([${[...value,].map(function element(item: unknown,) {
      return emitValue({
        typeExpression: `SetElem<${typeExpression}>`,
        value: item,
      },);
    },)
      .join(', ',)}])`;
  }
  if (value instanceof Map) {
    return `new Map<MapKey<${typeExpression}>, MapValue<${typeExpression}>>([${[...value,].map(function entry([key, entryValue,]: readonly [
      unknown,
      unknown
    ],) {
      return `[${emitValue({
        typeExpression: `MapKey<${typeExpression}>`,
        value: key,
      },)}, ${emitValue({
        typeExpression: `MapValue<${typeExpression}>`,
        value: entryValue,
      },)}]`;
    },)
      .join(', ',)}])`;
  }
  if ((typeof value) === 'object') {
    return `{ ${Reflect.ownKeys(value,)
      .map(function property(key: string | symbol,) {
      /**
       Key source and key type.
       */
      const [keySource, keyType,] = keyParts(key,);
      return `${keySource}: ${emitValue({
        typeExpression: `PropOf<${typeExpression}, ${keyType}>`,
        value: Reflect.get(
          value,
          key,
        ),
      },)}`;
    },)
      .join(', ',)} }`;
  }
  throw new TypeError(`emitValue: no literal form for ${typeof value}`,);
}

/**
 Emit one case as a function expression returning the call result and the
 checked literal.

 @param drawn - Case to emit.
 
 @param result - Its runtime result.
 
 @param id - Case number, echoed in a comment for triage.

 @returns Source lines of the case.

 @example
 ```ts
 emitCase({ drawn, result, id: 0, });
 ```
 */
export function emitCase({
  drawn,
  result,
  id,
}: {
  readonly drawn: DrawnCase;
  readonly result: unknown;
  readonly id: number
},): readonly string[] {
  /**
   Inputs as `widen<T>(value)` so declared types drive inference.
   */
  const inputs = drawn.types
    .map(function input(
      type,
      position: number,
    ) {
      return `widen<${emitType(type,)}>(${drawn.sources[position] ?? 'undefined'})`;
    },);
  if (drawn.kind === 'into') {
    return [
      `  // case ${String(id,)} into`,
      '  () => {',
      `    const intoTarget = ${inputs[0] ?? ''};`,
      `    deepmergeInto(intoTarget, ${inputs.slice(1,)
        .join(', ',)});`,
      '    const after = intoTarget;',
      `    const check: typeof after = ${emitValue({
        typeExpression: 'typeof after',
        value: result,
      },)};`,
      '    return { merged: after, value: check, };',
      '  },',
    ];
  }
  /**
   Call expression per entry point.
   */
  const call = {
    custom: `deepmergeCustom(${CUSTOM_OPTIONS[drawn.option] ?? '{}'})(${inputs.join(', ',)})`,
    fast: `deepmergeFastUnsafe(${inputs.join(', ',)})`,
    merge: `deepmerge(${inputs.join(', ',)})`,
  }[drawn.kind];
  return [
    `  // case ${String(id,)} ${drawn.kind} ${drawn.option}`,
    '  () => {',
    `    const merged = ${call};`,
    `    const check: typeof merged = ${emitValue({
      typeExpression: 'typeof merged',
      value: result,
    },)};`,
    '    return { merged, value: check, };',
    '  },',
  ];
}

/**
 Header shared by every generated case file.

 @param exportName - Name of the exported case array.

 @returns Source lines before the cases.

 @example
 ```ts
 emitHeader('DECLARED_TYPE_CASES');
 ```
 */
export function emitHeader(exportName: string,): readonly string[] {
  return [
    '// Generated by src/declared-type-emit.ts. Do not edit.',
    '// Each case type-checks only if the runtime result fits the declared-input result type.',
    'import { deepmerge, deepmergeCustom, deepmergeFastUnsafe, deepmergeInto, } from \'deepmerge-ts\';',
    '',
    'export type DeclaredTypeCase = () => { readonly merged: unknown; readonly value: unknown; };',
    '',
    'type FuzzBrand = number & { readonly fuzzBrand?: \'declared-type-fuzz\'; };',
    'type Box<T> = { v: T; tag: string; };',
    ...SYMBOL_NAMES.map(function declareSymbol(name: string,) {
      return `const ${name}: unique symbol = Symbol(${JSON.stringify(name,)},);`;
    },),
    '',
    '// Pass a value through its declared type, so control flow cannot narrow it to the literal.',
    'function widen<T,>(value: T,): T {',
    '  return value;',
    '}',
    '',
    '// Static type at a position of a result, for explicit Set and Map type arguments.',
    '// Each distributes over unions and drops members that lack the position.',
    '// A member that admits any such value (for example `{}` or `object`) yields `unknown`.',
    'type PropOf<T, K extends PropertyKey,> = T extends unknown ? (K extends keyof T ? T[K] : ({} extends T ? unknown : never)) : never;',
    'type AtFromEnd<T, K extends unknown[],> = T extends readonly [...infer Head, infer Last,] ? (K extends [] ? Last : AtFromEnd<Head, K extends [unknown, ...infer R,] ? R : []>) : (T extends readonly (infer E)[] ? E : never);',
    'type ElemAt<T, I extends unknown[], K extends unknown[],> = T extends readonly unknown[]',
    '  ? (number extends T["length"] ? (T extends readonly [infer First, ...infer Rest,] ? (I extends [] ? First : ElemAt<Rest, I extends [unknown, ...infer R,] ? R : [], K>) : AtFromEnd<T, K>) : T[I["length"]])',
    '  : (unknown[] extends T ? unknown : never);',
    'type SetElem<T,> = T extends ReadonlySet<infer V> ? V : (Set<unknown> extends T ? unknown : never);',
    'type MapKey<T,> = T extends ReadonlyMap<infer K, unknown> ? K : (Map<unknown, unknown> extends T ? unknown : never);',
    'type MapValue<T,> = T extends ReadonlyMap<unknown, infer V> ? V : (Map<unknown, unknown> extends T ? unknown : never);',
    '',
    '// Keep the brand alias referenced even when no case draws it.',
    'export type DeclaredTypeBrand = FuzzBrand;',
    '',
    `export const ${exportName}: readonly DeclaredTypeCase[] = [`,
  ];
}
