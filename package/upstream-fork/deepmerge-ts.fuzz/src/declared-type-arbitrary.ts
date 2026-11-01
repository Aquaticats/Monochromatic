/**
 fast-check generators of declared type trees (`./declared-type-ast.ts`).

 Trees mix leaves, literals, arrays, tuples, Sets, Maps, `Record`s, the
 generic `Box<T>`, intersections of two disjoint object halves, object types
 with optional, readonly, and symbol keys, index signatures, and utility
 wrappers, and (in the union scope) unions.

 @module
 */

import {
  array,
  boolean,
  constant,
  constantFrom,
  letrec,
  oneof,
  record,
  uniqueArray,
  type Arbitrary,
} from 'fast-check';

import type {
  IndexNode,
  ObjectNode,
  PropNode,
  TypeNode,
} from './declared-type-ast.ts';
import { LEAF_TYPE_ARBITRARY, } from './declared-type-leaf.ts';

/**
 Deepest nesting of generated type trees.
 */
const MAX_TYPE_DEPTH = 3;

/**
 Most properties, tuple elements, or union members per node.
 */
const MAX_CHILDREN = 3;

/**
 Weight of objects without an index signature against those with one.
 */
const NO_INDEX_WEIGHT = 3;

/**
 Generators of one recursion scope.
 */
export type DeclaredTypeScope = {
  readonly type: Arbitrary<TypeNode>;
  readonly object: Arbitrary<ObjectNode>;
};

/**
 Drawn property before its key is classified as string or symbol.
 */
type DrawnProp = {
  readonly key: string;
  readonly optional: boolean;
  readonly readonly: boolean;
  readonly type: TypeNode;
};

/**
 Uniqueness selector of drawn properties.

 @param prop - Drawn property.

 @returns Its key.

 @example
 ```ts
 propKeyOf({ key: 'a', optional: false, readonly: false, type, }); // 'a'
 ```
 */
function propKeyOf(prop: DrawnProp,): string {
  return prop.key;
}

/**
 Rename an object's string keys into a fixed pool, so intersection halves
 never share a key.

 @param node - Object to rename.
 
 @param keys - Pool for this half.

 @returns Object whose string keys come from `keys`, symbol keys dropped.

 @example
 ```ts
 withKeys({ node, keys: ['a', 'b',], });
 ```
 */
function withKeys({
  node,
  keys,
}: {
  readonly node: ObjectNode;
  readonly keys: readonly string[]
},): ObjectNode {
  /**
   String-keyed properties that fit the pool, renamed in order.
   */
  const props = node.props
    .filter(function stringKeyed(prop: PropNode,) {
      return !prop.symbolKey;
    },)
    .slice(
    0,
    keys.length,
    )
    .map(function rename(
      prop: PropNode,
      position: number,
    ): PropNode {
      return {
        ...prop,
        key: keys[position] ?? prop.key,
      };
    },);
  return {
    ...node,
    index: { keyKind: 'none', },
    props,
  };
}

/**
 Collection type generators over the recursive element type.

 @param element - Recursive type generator.

 @returns Array, tuple, Set, and Map generators.

 @example
 ```ts
 const collections = collectionArbitraries(tie('type',));
 ```
 */
function collectionArbitraries(element: Arbitrary<TypeNode>,): readonly Arbitrary<TypeNode>[] {
  return [
    record({
      element,
      readonly: boolean(),
    },)
      .map(function toArray(fields: {
        readonly element: TypeNode;
        readonly readonly: boolean
      },): TypeNode {
        return {
          kind: 'array',
          ...fields,
        };
      },),
    record({
      elements: array(
        element,
        { maxLength: MAX_CHILDREN, },
      ),
      readonly: boolean(),
    },)
      .map(function toTuple(fields: {
        readonly elements: readonly TypeNode[];
        readonly readonly: boolean
      },): TypeNode {
        return {
          kind: 'tuple',
          ...fields,
        };
      },),
    record({
      element,
      readonly: boolean(),
    },)
      .map(function toSet(fields: {
        readonly element: TypeNode;
        readonly readonly: boolean
      },): TypeNode {
        return {
          kind: 'set',
          ...fields,
        };
      },),
    record({
      keyKind: constantFrom<'number' | 'string'>(
        'number',
        'string',
      ),
      value: element,
      readonly: boolean(),
    },)
      .map(function toMap(fields: {
        readonly keyKind: 'number' | 'string';
        readonly readonly: boolean;
        readonly value: TypeNode
      },): TypeNode {
        return {
          kind: 'map',
          ...fields,
        };
      },),
  ];
}

/**
 Record and generic-alias type generators over the recursive value type.

 @param element - Recursive type generator.

 @returns Record and Box generators.

 @example
 ```ts
 const wrappers = wrapperArbitraries(tie('type',));
 ```
 */
function wrapperArbitraries(element: Arbitrary<TypeNode>,): readonly Arbitrary<TypeNode>[] {
  return [
    record({
      keys: uniqueArray(
        constantFrom(
          'a',
          'b',
          'c',
        ),
        {
          maxLength: MAX_CHILDREN,
          minLength: 1,
        },
      ),
      value: element,
    },)
      .map(function toRecord(fields: {
        readonly keys: readonly string[];
        readonly value: TypeNode
      },): TypeNode {
        return {
          kind: 'record',
          ...fields,
        };
      },),
    element.map(function toBox(inner: TypeNode,): TypeNode {
      return {
        inner,
        kind: 'box',
      };
    },),
  ];
}

/**
 Object type generator over the recursive property type.

 @param type - Recursive type generator.

 @returns Object types with optional, readonly, and symbol keys, index signatures, and wrappers.

 @example
 ```ts
 const objects = objectArbitrary(tie('type',));
 ```
 */
function objectArbitrary(type: Arbitrary<TypeNode>,): Arbitrary<ObjectNode> {
  // Field order is draw order; keep it so recorded seeds reproduce the same trees.
  return record({
    props: uniqueArray(
      record({
        key: constantFrom(
          'a',
          'b',
          'c',
          'SYM_A',
          'SYM_B',
        ),
        optional: boolean(),
        readonly: boolean(),
        type,
      },),
      {
        maxLength: MAX_CHILDREN,
        selector: propKeyOf,
      },
    ),
    index: oneof(
      {
        arbitrary: constant<IndexNode>({ keyKind: 'none', },),
        weight: NO_INDEX_WEIGHT,
      },
      {
        arbitrary: record({
          keyKind: constantFrom<'number' | 'string' | 'symbol' | 'template'>(
            'number',
            'string',
            'symbol',
            'template',
          ),
          value: type,
        },),
        weight: 1,
      },
    ),
    wrapper: constantFrom<ObjectNode['wrapper']>(
      'none',
      'none',
      'partial',
      'readonly',
      'required',
    ),
  },)
    .map(function toObject(fields: {
      readonly index: IndexNode;
      readonly props: readonly DrawnProp[];
      readonly wrapper: ObjectNode['wrapper']
    },): ObjectNode {
      return {
        ...fields,
        kind: 'object',
        props: fields.props
          .map(function toProp(prop: DrawnProp,): PropNode {
            return {
              ...prop,
              symbolKey: prop.key
                .startsWith('SYM_',),
            };
          },),
      };
    },);
}

/**
 Recursive type-tree generators.

 @param unions - Whether union nodes are drawn; off to search for classes
   other than the pinned union ones.

 @returns Type and object generators over one recursion scope.

 @example
 ```ts
 const scope = typeScopeFor({ unions: false, });
 ```
 */
function typeScopeFor({ unions, }: { readonly unions: boolean; },): DeclaredTypeScope {
  return letrec<{
    type: TypeNode;
    object: ObjectNode
  }>(function build(tie,) {
    /**
     Recursive type reference.
     */
    const type = tie('type',);
    /**
     Recursive object reference.
     */
    const object = tie('object',);
    return {
      object: objectArbitrary(type,),
      type: oneof(
        {
          depthIdentifier: 'declared-type',
          maxDepth: MAX_TYPE_DEPTH,
        },
        LEAF_TYPE_ARBITRARY,
        object,
        ...collectionArbitraries(type,),
        unions
          ? array(
            type,
            {
              maxLength: MAX_CHILDREN,
              minLength: 2,
            },
          )
            .map(function toUnion(members: readonly TypeNode[],): TypeNode {
              return {
                kind: 'union',
                members,
              };
            },)
          : LEAF_TYPE_ARBITRARY,
        ...wrapperArbitraries(type,),
        record({
          left: object,
          right: object,
        },)
          .map(function toIntersection(halves: {
            readonly left: ObjectNode;
            readonly right: ObjectNode
          },): TypeNode {
            return {
              kind: 'intersection',
              left: withKeys({
                keys: [
                  'a',
                  'b',
                ],
                node: halves.left,
              },),
              right: withKeys({
                keys: [
                  'c',
                  'd',
                ],
                node: halves.right,
              },),
            };
          },),
      ),
    };
  },);
}

/**
 Generators of declared type trees and object types, with and without unions.
 */
export const DECLARED_TYPE_SCOPES: Readonly<Record<'noUnions' | 'unions', DeclaredTypeScope>> = {
  noUnions: typeScopeFor({ unions: false, },),
  unions: typeScopeFor({ unions: true, },),
};
