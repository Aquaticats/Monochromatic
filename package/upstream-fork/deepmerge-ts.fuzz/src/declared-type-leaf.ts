/**
 Leaf generators shared by the declared-type tree generator
 (`./declared-type-arbitrary.ts`) and the value sampler
 (`./declared-type-sample.ts`).

 @module
 */

import {
  constantFrom,
  integer,
  oneof,
  type Arbitrary,
} from 'fast-check';

import type { TypeNode, } from './declared-type-ast.ts';

/**
 Leaf and literal type generator. Brands are emitted by the emitter but
 never drawn: a literal can only be shown to be a branded value with a cast,
 which would defeat the check.
 */
export const LEAF_TYPE_ARBITRARY: Arbitrary<TypeNode> = oneof(
  constantFrom<TypeNode>(
    { kind: 'number', },
    { kind: 'string', },
    { kind: 'boolean', },
    { kind: 'null', },
    { kind: 'undefined', },
    { kind: 'date', },
    { kind: 'regexp', },
  ),
  constantFrom<boolean | number | string>(
    0,
    1,
    'a',
    'b',
    true,
    false,
  )
    .map(function toLiteral(value: boolean | number | string,): TypeNode {
      return {
        kind: 'literal',
        value,
      };
    },),
);

/**
 Small integers for leaf values; exported so the sampler shares the range.
 */
export const LEAF_INTEGER: Arbitrary<number> = integer({
  max: 9,
  min: -2,
},);
