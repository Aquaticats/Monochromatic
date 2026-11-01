/**
 Shared helpers for reading the untyped host AST exposed by `@oxlint/plugins`.
 
 The oxlint plugin API surfaces AST nodes as opaque `Span` objects whose
 child properties are untyped, so reads land on `unknown`. These guards
 narrow `unknown` to the record / array shapes the TSDoc rules walk,
 without a banned nullish union and without an unchecked `as` assertion
 (a type-predicate body carries the narrowing instead).
 
 @module
 */

import {
  isRecord as sharedIsRecord,
  type ReadonlyRecord,
} from '@monochromatic-dev/oxlint-plugin-shared/ts';

export {
  isRecord,
  type ReadonlyRecord,
} from '@monochromatic-dev/oxlint-plugin-shared/ts';

/**
 Narrows an unknown value to a readonly array of AST records.
 
 Element membership is not re-checked per item; AST child arrays hold
 node objects, so callers treat each element as a record and guard the
 rare hole (`null` array slot) with {@link isRecord} at the use site.
 
 @param value - candidate read from an untyped AST property
 
 @returns whether value is an array
 
 @example
 ```ts
 if (isRecordArray(node.params)) { for (const p of node.params) {} }
 ```
 */
export function isRecordArray(value: unknown,): value is readonly ReadonlyRecord[] {
  return Array.isArray(value,);
}

/**
 Unwraps a MethodDefinition / TSAbstractMethodDefinition to its inner
 function value; returns the node itself for other function-like types.
 
 Real parsed methods always carry a `.value` FunctionExpression, so the
 inner lookup never falls through; the node-itself branch covers the
 non-method function-like types that have no wrapper.
 
 @param node - function-like AST node
 
 @returns inner function value for methods, otherwise node unchanged
 
 @example
 ```ts
 const fn = unwrapMethodDefinition(methodNode);
 ```
 */
export function unwrapMethodDefinition(node: ReadonlyRecord,): ReadonlyRecord {
  if ((node.type
    === 'MethodDefinition')
    || (node.type
      === 'TSAbstractMethodDefinition'))
    {
    /**
     Inner FunctionExpression of the method; carries the return-type and params info.
     */
    const { value, } = node;
    if (sharedIsRecord(value,))
      return value;
  }
  return node;
}

/**
 Extracts the raw `params` array from a function-like AST node, unwrapped
 via {@link unwrapMethodDefinition}.
 
 @param node - function-like AST node
 
 @returns parameter AST records; empty when the node declares none
 
 @example
 ```ts
 for (const param of extractRawParams(node)) { extractBindingName(param); }
 ```
 */
export function extractRawParams(node: ReadonlyRecord,): readonly ReadonlyRecord[] {
  /**
   Inner function value (for methods) or node itself; the `.params` array lives here.
   */
  const target = unwrapMethodDefinition(node,);
  /**
   Raw `.params`; unknown until narrowed to an array of records.
   */
  const { params, } = target;
  if (isRecordArray(params,))
    return params;
  return [];
}

/**
 Tests whether a record is a supported binding-pattern wrapper.
 
 @param node - candidate wrapper record.
 
 @returns whether wrapper carries one inner binding pattern.
 
 @example
 ```ts
 isBindingPatternWrapper({ type: 'RestElement' });
 ```
 */
function isBindingPatternWrapper(node: ReadonlyRecord,): boolean {
  return (node.type === 'AssignmentPattern')
    || (node.type === 'RestElement')
    || (node.type === 'TSParameterProperty');
}

/**
 Removes binding-pattern wrapper nodes whose child carries the real binding.
 
 Handles default values, rest wrappers, and TypeScript parameter properties.
 Unknown or malformed wrappers stop the walk and return the current node.
 
 @param pattern - binding pattern candidate
 
 @returns innermost binding-pattern record reached by supported wrappers
 
 @example
 ```ts
 const binding = unwrapBindingPattern(parameterNode);
 ```
 */
export function unwrapBindingPattern(pattern: ReadonlyRecord,): ReadonlyRecord {
  /**
   Mutable cursor object avoids function-root `let` while walking wrapper nodes iteratively.
   */
  const cursor = { current: pattern, };
  while (isBindingPatternWrapper(cursor.current,)) {
    /**
     Current wrapper candidate for this loop iteration.
     */
    const { current, } = cursor;
    /**
     Inner binding selected by current wrapper kind.
     */
    const inner = current.type === 'AssignmentPattern'
      ? current.left
      : current.type === 'RestElement'
        ? current.argument
        : current.parameter;
    if (!sharedIsRecord(inner,))
      return current;
    cursor.current = inner;
  }
  return cursor.current;
}
