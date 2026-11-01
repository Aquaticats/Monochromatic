/**
 Build a synthetic {@link ValueNode} from a JS input passed to {@link tomlSet}.
 
 Scalar leaves precompute their render text via `jsValueToTomlText` (so wrapped
 inputs keep their intended spelling) and store the plain JS value for reads.
 When an `existing` parse-time node is supplied and the new value equals it,
 `jsValueToTomlText` preserves the original style/raw spelling, matching the
 pre-rewrite re-set behavior. Arrays and objects recurse into structured nodes
 so later edits touch only the changed leaf.
 
 @module
 */

import type {
  KeyValueNode,
  ScalarKind,
  ValueNode,
} from './document.ts';
import type { CanonicalOptions, } from './types.ts';
import {
  isPlainObject,
  isWrappedInput,
} from './value-encoders.ts';
import {
  type ExistingNode,
  jsValueToTomlText,
} from './values.ts';

/**
 Build a synthetic value node for `input`.
 
 @returns Computed {@link ValueNode}.
 
 @throws `TomlTypeError` from value encoding for `null`, `undefined`, or an unencodable value.
 
 @mutates input - Recursive build can invoke caller-owned proxy, getter, and prototype hooks.
 
 @example
 ```ts
 buildValueFromInput({ input: { x: 1, }, options, },);
 ```
 */
export function buildValueFromInput(
  {
    input,
    options,
    existing,
  }: {
    readonly input: unknown;
    readonly options: CanonicalOptions;
    readonly existing?: ExistingNode;
  },
): ValueNode {
  if (Array.isArray(input,)) {
    /**
     Parse-time element nodes so per-element raw spelling survives an equal re-set.
     */
    const elementExistings = (existing !== undefined) && (existing.node
      .type
      === 'TOMLArray')
      ? existing.node
        .elements
      : null;
    return {
      kind: 'array',
      elements: input.map(function each(
        el,
        i,
      ) {
        return buildValueFromInput({
          input: el,
          options,
          ...((elementExistings === null) || (elementExistings[i] === undefined)
            ? {}
            : { existing: { node: elementExistings[i], }, }),
        },);
      },),
      origin: { kind: 'synthetic', },
    };
  }
  if ((!isWrappedInput(input,)) && isPlainObject(input,)) {
    return {
      kind: 'inline-table',
      entries: Object.entries(input,)
        .map(function each([key, value,],): KeyValueNode {
          return {
            kind: 'keyvalue',
            keySegments: [key,],
            value: buildValueFromInput({
              input: value,
              options,
            },),
            origin: { kind: 'synthetic', },
            commentsBefore: [],
          };
        },),
      origin: { kind: 'synthetic', },
    };
  }
  return {
    kind: 'scalar',
    tomlKind: scalarKindOf({ input, },),
    jsValue: plainOf({ input, },),
    renderText: jsValueToTomlText({
      input,
      options,
      ...(existing === undefined ? {} : { existing, }),
    },),
    origin: { kind: 'synthetic', },
  };
}

/**
 Describe the TOML scalar kind for a leaf input.
 
 @returns Computed {@link ScalarKind}.
 */
function scalarKindOf({ input, }: { readonly input: unknown; },): ScalarKind {
  if (isWrappedInput(input,))
    return input.tomlKind;
  if ((typeof input) === 'string')
    return 'string';
  if ((typeof input) === 'boolean')
    return 'boolean';
  if ((typeof input) === 'bigint')
    return 'integer';
  if (input instanceof Date)
    return 'offset-date-time';
  if (((typeof input) === 'number') && Number.isInteger(input as number,))
    return 'integer';
  return 'float';
}

/**
 Plain JS value a read should return for a synthetic leaf.
 
 @returns Computed value.
 */
function plainOf({ input, }: { readonly input: unknown; },): unknown {
  if (isWrappedInput(input,)) {
    if ((input.tomlKind
      === 'integer') || (input.tomlKind
        === 'float'))
      return Number(input.value,);
    return input.value;
  }
  return input;
}
