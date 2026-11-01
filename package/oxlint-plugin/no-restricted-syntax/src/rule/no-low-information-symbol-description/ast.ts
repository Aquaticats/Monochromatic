import type { ESTree, } from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { getStaticMemberName, } from '../ast-shared.ts';

/**
 Genuine sentinel for {@link staticDescription} when no static string
 description is present (absent, dynamic, or non-string). A unique `Symbol`,
 never `null`, so the return type carries no nullish union.
 */
export const NO_STATIC_DESCRIPTION: unique symbol = Symbol('static Symbol description absent or dynamic',);

/**
 Checks whether a call is `Symbol(...)` via the global identifier.
 
 @param node - call expression to inspect
 
 @returns whether callee is the bare `Symbol` identifier
 
 @example
 ```ts
 isSymbolCall({ node }); // true for Symbol('id')
 ```
 */
export function isSymbolCall({ node, }: ForeignBorrowed<{ readonly node: ESTree.CallExpression; }>,): boolean {
  /**
   Callee of the call expression.
   */
  const { callee, } = node;
  return (callee.type === 'Identifier') && (callee.name === 'Symbol');
}

/**
 Checks whether a call is `Symbol.for(...)` via a static member access,
 read with {@link getStaticMemberName}.
 
 @param node - call expression to inspect
 
 @returns whether callee is the static `Symbol.for` member
 
 @example
 ```ts
 isSymbolForCall({ node }); // true for Symbol.for('id')
 ```
 */
export function isSymbolForCall({ node, }: ForeignBorrowed<{ readonly node: ESTree.CallExpression; }>,): boolean {
  /**
   Callee of the call expression.
   */
  const { callee, } = node;
  if (callee.type !== 'MemberExpression')
    return false;
  if (callee.computed)
    return false;
  /**
   Object of the member access, expected to be the `Symbol` identifier.
   */
  const { object, } = callee;
  if ((object.type !== 'Identifier') || (object.name !== 'Symbol'))
    return false;
  return getStaticMemberName({ member: callee, },) === 'for';
}

/**
 Extracts a static string description from a Symbol call's first argument:
 a string literal, or a zero-expression template literal. Absent, dynamic, and
 non-string descriptions yield {@link NO_STATIC_DESCRIPTION}.
 
 @param node - Symbol or Symbol.for call expression
 
 @returns static description text, or sentinel when none is statically known
 
 @example
 ```ts
 staticDescription({ node }); // 'id' for Symbol('id') and Symbol(`id`)
 ```
 */
export function staticDescription(
  { node, }: ForeignBorrowed<{ readonly node: ESTree.CallExpression; }>,
): string | typeof NO_STATIC_DESCRIPTION {
  /**
   First argument node, the description position.
   */
  const [firstArgument,] = node.arguments;
  if (firstArgument === undefined)
    return NO_STATIC_DESCRIPTION;
  if ((firstArgument.type === 'Literal') && ((typeof firstArgument.value) === 'string'))
    return firstArgument.value;
  if (
    (firstArgument.type === 'TemplateLiteral')
      && (firstArgument.expressions
        .length
        === 0)
      && (firstArgument.quasis
        .length
        === 1)
  ) {
    /**
     Sole template quasi when no interpolation is present.
     */
    const [onlyQuasi,] = firstArgument.quasis;
    if (onlyQuasi === undefined)
      return NO_STATIC_DESCRIPTION;
    /**
     Cooked value of the quasi, null on an invalid escape.
     */
    const { cooked, } = onlyQuasi.value;
    if (cooked === null)
      return NO_STATIC_DESCRIPTION;
    return cooked;
  }
  return NO_STATIC_DESCRIPTION;
}
