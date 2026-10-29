/**
 Semantic-equality oracle for TOML documents.
 
 The round-trip and metamorphic properties need to ask "do these two TOML
 texts mean the same thing", not "are they the same bytes". The model here
 is the parser's own `getStaticTOMLValue` projection: tables and dotted keys
 collapse to nested objects, arrays-of-tables to arrays of objects, and
 scalars to native JS values. Both sides of supported comparisons pass through
 the same projection,
 so datetime and large-integer lossiness remains symmetric.
 Sources containing `__proto__` keys are classified before projection because
 the upstream ordinary-object assignment changes prototypes instead of preserving that TOML key.
 
 Comparison is structural and iterative (a work stack of left/right pairs),
 never recursive: the value tree can only be as deep as the parser already
 built, but a degenerate array spine must not be walked with the call stack.
 
 @module
 */

import {
  getStaticTOMLValue,
  parseTOML,
} from '@monochromatic-dev/toml-edit-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { AST, } from 'toml-eslint-parser';

import type { TomlEditOptions, } from '@monochromatic-dev/toml-edit-core/ts/types.ts';

/**
 Native projection of a TOML value as produced by `getStaticTOMLValue`.
 */
export type SemanticValue =
  | string
    | number
    | boolean
    | Date
    | readonly SemanticValue[]
    | { readonly [key: string]: SemanticValue; };

/**
 Key whose ordinary property assignment changes a plain object's prototype.
 */
const PROTOTYPE_SETTER_KEY = '__proto__';

/**
 Whether `toml-eslint-parser` can faithfully project a source through
 `getStaticTOMLValue`.
 
 Version 1.0.3 assigns table keys into ordinary objects.
 A `__proto__` segment therefore invokes the inherited prototype setter instead
 of creating an own TOML property,
 so semantic comparisons must discard that upstream-oracle case.
 
 @returns Whether every authored key avoids the upstream prototype setter.
 
 @example
 ```ts
 staticSemanticOracleSupports({ source: 'value = 1\n', }); // true
 staticSemanticOracleSupports({ source: 'value = { "__proto__" = 1 }\n', }); // false
 ```
 */
export function staticSemanticOracleSupports(
  {
    source,
    tomlVersion,
  }: {
    readonly source: string;
    readonly tomlVersion?: TomlEditOptions['tomlVersion'];
  },
): boolean {
  /**
   Parsed document inspected before invoking the lossy projection.
   */
  const program: ForeignBorrowed<AST.TOMLProgram> = parseTOML(
    source,
    tomlVersion === undefined ? undefined : { tomlVersion, },
  );
  /**
   Structural work stack avoiding recursion over nested array and table spines.
   */
  const pending: AST.TOMLNode[] = [program,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (node.type === 'TOMLKey') {
      for (const segment of node.keys) {
        /**
         Authored key-segment text.
         */
        const key = segment.type === 'TOMLBare'
          ? segment.name
          : segment.value;
        if (key === PROTOTYPE_SETTER_KEY)
          return false;
      }
    }
    if ((node.type === 'Program')
      || (node.type === 'TOMLTopLevelTable')
      || (node.type === 'TOMLInlineTable')) {
      for (const child of node.body)
        pending.push(child,);
      continue;
    }
    if (node.type === 'TOMLTable') {
      pending.push(node.key,);
      for (const child of node.body)
        pending.push(child,);
      continue;
    }
    if (node.type === 'TOMLKeyValue') {
      pending.push(
        node.key,
        node.value,
      );
      continue;
    }
    if (node.type === 'TOMLArray') {
      for (const child of node.elements)
        pending.push(child,);
    }
  }
  return true;
}

/**
 Parse `source` and project it to the native semantic model.
 
 @param source - TOML text to project.
 
 @param tomlVersion - Forwarded to the parser so version-sensitive grammar
                      (for example TOML 1.1 newlines in inline tables) projects
                      under the intended dialect.
 
 @returns Native value tree of `source`.
 
 @throws Whatever the parser throws on rejected input; callers that fuzz
         invalid text wrap this in their own totality assertion.
 
 @example
 ```ts
 semanticModel({ source: 'a = 1\n', },); // { a: 1 }
 ```
 */
export function semanticModel(
  {
    source,
    tomlVersion,
  }: {
    readonly source: string;
    readonly tomlVersion?: TomlEditOptions['tomlVersion'];
  },
): SemanticValue {
  // getStaticTOMLValue is re-exported by the package under test, keeping the
  // oracle anchored to the same parser version the package ships.
  return getStaticTOMLValue(
    parseTOML(
      source,
      tomlVersion === undefined ? undefined : { tomlVersion, },
    ),
  ) as SemanticValue;
}

/**
 True when `left` and `right` are the same primitive under TOML semantics.
 
 `NaN` equals `NaN` (TOML treats every `nan` spelling as one value); signed
 infinities stay distinct; `-0` equals `0`; `Date`s compare by instant, which
 is stable within one process even for the parser's zone-shifted local kinds.
 
 @returns Whether the two leaves are semantically equal.
 */
function leafEquals(
  {
    left,
    right,
  }: {
    readonly left: unknown;
    readonly right: unknown;
  },
): boolean {
  if (((typeof left) === 'number') && ((typeof right) === 'number')) {
    if (Number.isNaN(left,) && Number.isNaN(right,)) return true;
    return left === right;
  }
  if ((left instanceof Date) && (right instanceof Date)) {
    return left.getTime() === right.getTime();
  }
  return left === right;
}

/**
 Type guard for the record arm of a semantic value.
 
 @param value - Candidate to test for plain-record shape.
 
 @returns Whether `value` is a TOML table object (not an array, not a `Date`).
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object')
    && (value !== null)
    && (!Array.isArray(value,))
    && (!(value instanceof Date));
}

/**
 Compare two semantic models for TOML-meaning equality.
 
 Branching uses `Array.isArray` and {@link isRecord} guards rather than type
 assertions, so each arm narrows both sides structurally. A shape mismatch at
 any node (one array versus one record, one structure versus one leaf) is an
 immediate inequality.
 
 @param left - First projected document.
 
 @param right - Second projected document.
 
 @returns Whether both documents mean the same thing.
 
 @example
 ```ts
 semanticEquals({
   left: semanticModel({ source: 'a = 0x10\n', },),
   right: semanticModel({ source: 'a = 16\n', },),
 },); // true
 ```
 */
export function semanticEquals(
  {
    left,
    right,
  }: {
    readonly left: SemanticValue;
    readonly right: SemanticValue;
  },
): boolean {
  /**
   Pending left/right pairs still to compare; structural children are pushed
   as they are reached so no two trees are ever walked with the call stack.
   */
  const stack: {
    readonly a: unknown;
    readonly b: unknown;
  }[] = [{
    a: left,
    b: right,
  }];

  for (let pair = stack.pop(); pair !== undefined; pair = stack.pop()) {
    if (Array.isArray(pair.a,) || Array.isArray(pair.b,)) {
      if ((!Array.isArray(pair.a,)) || (!Array.isArray(pair.b,))) return false;
      if (pair.a
        .length
        !== pair.b
          .length) return false;
      for (const [index, element,] of pair.a
        .entries()) {
        stack.push({
          a: element,
          b: pair.b[index],
        },);
      }
      continue;
    }

    if (isRecord(pair.a,) || isRecord(pair.b,)) {
      if ((!isRecord(pair.a,)) || (!isRecord(pair.b,))) return false;
      /**
       Both key sets sorted so key-order differences never matter.
       */
      const keysA = Object.keys(pair.a,)
        .toSorted();
      /**
       Right key set, sorted to match {@link keysA} positionally.
       */
      const keysB = Object.keys(pair.b,)
        .toSorted();
      if (keysA.length !== keysB.length) return false;
      if (!keysA.every(function sameKey(
        key,
        index,
      ) { return key === keysB[index]; },)) return false;
      for (const key of keysA) {
        stack.push({
          a: pair.a[key],
          b: pair.b[key],
        },);
      }
      continue;
    }

    if (!leafEquals({
      left: pair.a,
      right: pair.b,
    },)) return false;
  }

  return true;
}
