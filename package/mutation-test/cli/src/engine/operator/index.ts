/**
 Operator registry: every mutation family the engine applies.
 
 @example
 ```ts
 import { allOperators } from './index.ts';
 ```
 */

import { binaryOperatorReplacements, } from './binary.ts';
import { chainMethodReplacements, } from './chain-method.ts';
import { collectionReplacements, } from './collections.ts';
import { conditionalReplacements, } from './conditional.ts';
import { functionReplacements, } from './functions.ts';
import { literalReplacements, } from './literals.ts';
import { regexReplacements, } from './regex.ts';
import { unaryUpdateReplacements, } from './unary-update.ts';
import type {
  EstreeNode,
  Replacement,
} from '../types.ts';

/**
 Operator callback signature shared by every family.
 */
export type OperatorFn = (options: {
  readonly node: EstreeNode;
  readonly parent?: EstreeNode;
  readonly source: string;
},) => readonly Replacement[];

/**
 Every operator family, applied to every visited node.
 */
export const allOperators: readonly OperatorFn[] = [
  binaryOperatorReplacements,
  chainMethodReplacements,
  collectionReplacements,
  conditionalReplacements,
  functionReplacements,
  literalReplacements,
  regexReplacements,
  unaryUpdateReplacements,
];
