/**
 Return-type and generator detection utilities for TSDoc rules.
 
 Extracted from `tsdoc-params.ts` to keep files under 100 countable lines.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Span, } from '@oxlint/plugins';

import {
  isRecord,
  isRecordArray,
  type ReadonlyRecord,
  unwrapMethodDefinition,
} from './ast-access.ts';

/**
 Checks whether a function-like node (unwrapped via
 {@link unwrapMethodDefinition}) has a non-void return type or return
 statements.
 
 @param node - AST node to inspect
 
 @returns true when function appears to return a value
 
 @example
 ```ts
 if (functionReturnsValue(node)) { /* check for \@returns tag *\/ }
 ```
 */
export function functionReturnsValue(node: ForeignBorrowed<Span & ReadonlyRecord>,): boolean {
  // Check kind on the outer MethodDefinition BEFORE unwrapping to .value,
  // because `kind` ("constructor", "get", "set", "method") is a property
  // of MethodDefinition, not of the inner FunctionExpression.
  if ((node.type
    === 'MethodDefinition')
    || (node.type
      === 'TSAbstractMethodDefinition'))
    {
    /**
     Method kind (constructor/get/set/method); read on the outer MethodDefinition before unwrap.
     */
    const { kind, } = node;
    if ((kind === 'constructor') || (kind === 'set'))
      return false;
  }

  /**
   Inner function value (for methods) or the node itself; supplies the return-type info below.
   */
  const target = unwrapMethodDefinition(node,);

  // Check for explicit void/never return type annotation
  /**
   TS return-type annotation node; absent when the function has no explicit return type.
   */
  const { returnType, } = target;
  if (!isRecord(returnType,))
    return true;

  /**
   Inner annotation wrapped by `returnType`; the actual TS type lives one level deeper.
   */
  const { typeAnnotation, } = returnType;
  if (!isRecord(typeAnnotation,))
    return true;

  /**
   AST node-type tag (`TSVoidKeyword`, `TSTypeReference`, etc.) that drives the branch below.
   */
  const tsType = typeAnnotation.type;
  if ((tsType === 'TSVoidKeyword') || (tsType === 'TSNeverKeyword'))
    return false;
  if (tsType !== 'TSTypeReference')
    return true;

  // Handle `Promise<void>` and `Promise<never>` return types. The AST represents
  // these as `TSTypeReference` with `typeName.name === 'Promise'` and a single type
  // parameter of `TSVoidKeyword` or `TSNeverKeyword`.
  /**
   Type reference name node; used to detect the `Promise<...>` shape.
   */
  const { typeName, } = typeAnnotation;
  if (!isRecord(typeName,))
    return true;
  if (typeName.name !== 'Promise')
    return true;

  // oxc AST uses `typeArguments` (not `typeParameters`) for generic type arguments.
  /**
   Generic argument container for the `Promise<...>` reference.
   */
  const { typeArguments, } = typeAnnotation;
  if (!isRecord(typeArguments,))
    return true;
  /**
   Concrete type parameters of `Promise<...>`; exactly one is the valid shape.
   */
  const { params, } = typeArguments;
  if ((!isRecordArray(params,)) || (params.length
    !== 1))
    return true;

  /**
   Single Promise type argument; its node-type decides the void/never special case.
   */
  const [innerArg,] = params;
  if (!isRecord(innerArg,))
    return true;
  /**
   AST node-type of the single Promise type argument, e.g. `TSVoidKeyword`.
   */
  const innerType = innerArg.type;
  if ((innerType === 'TSVoidKeyword') || (innerType === 'TSNeverKeyword'))
    return false;

  return true;
}

/**
 Checks whether a function-like node (unwrapped via
 {@link unwrapMethodDefinition}) is a generator (has `generator: true`).
 
 @param node - AST node to inspect
 
 @returns true when the function is a generator
 
 @example
 ```ts
 if (isGeneratorFunction(node)) { /* check for \@yields tag *\/ }
 ```
 */
export function isGeneratorFunction(node: ForeignBorrowed<Span & ReadonlyRecord>,): boolean {
  /**
   Inner function value (for methods) or the node itself; carries the `generator` flag.
   */
  const target = unwrapMethodDefinition(node,);

  return target.generator
    === true;
}
