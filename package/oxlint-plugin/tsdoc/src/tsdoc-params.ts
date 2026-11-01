/**
 Parameter extraction utilities for TSDoc rules.
 
 Functions for extracting parameter names from function-like AST nodes,
 including support for binding patterns.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Span, } from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import {
  extractRawParams,
  type ReadonlyRecord,
  unwrapBindingPattern,
} from './ast-access.ts';
import type {
  ParsedDocComment,
  ParsedParamBlock,
} from './tsdoc-doc-model.ts';

/**
 Extracts parameter names from a function-like AST node, reading parameters
 via {@link extractRawParams} and resolving each binding name with
 {@link extractBindingName}.
 
 Handles FunctionDeclaration, FunctionExpression, ArrowFunctionExpression,
 MethodDefinition, and TSMethodSignature nodes.
 
 @param node - AST node representing a function-like declaration
 
 @returns array of parameter name strings, excluding rest-element `...` prefix
 
 @example
 ```ts
 const names = extractParamNames(functionNode);
 // ['first', 'second', 'options']
 ```
 */
export function extractParamNames(
  node: ForeignBorrowed<Span & ReadonlyRecord>,
): readonly string[] {
  return extractRawParams(node,)
    .flatMap(function extractName(param,): readonly string[] {
      return extractBindingName(param,);
    },);
}

/**
 Recursively extracts binding names from a parameter pattern, unwrapped via
 {@link unwrapBindingPattern}.
 
 @param pattern - AST binding pattern node
 
 @returns array of extracted name strings
 */
function extractBindingName(pattern: ReadonlyRecord,): readonly string[] {
  /**
   Pattern after shared unwrapping of defaults, rest elements, and TS parameter properties.
   */
  const unwrapped = unwrapBindingPattern(pattern,);
  if (unwrapped.type
    === 'Identifier') {
    /**
     Identifier text of the parameter binding; `this` is skipped because it is not a real param.
     */
    const { name, } = unwrapped;
    // Skip `this` parameter in TypeScript
    if ((typeof name)
      !== 'string')
      return [];
    return name === 'this' ? [] : [name,];
  }
  // ObjectPattern, ArrayPattern, and other types don't map to individual @param names
  return [];
}

/**
 Extracts documented param names from a parsed TSDoc comment.
 
 @param docComment - parsed TSDoc DocComment
 
 @returns array of parameter names found in param tags
 
 @example
 ```ts
 const docParamNames = extractDocParamNames(result.docComment);
 ```
 */
export function extractDocParamNames(docComment: ReadonlyDeep<ParsedDocComment>,): readonly string[] {
  return docComment.params
    .blocks
    .map(
    function getParamName(block: ReadonlyDeep<ParsedParamBlock>,): string {
      return block.parameterName;
    },
    );
}

export { extractDestructuredParamNames, } from './tsdoc-destructured.ts';

export {
  functionReturnsValue,
  isGeneratorFunction,
} from './tsdoc-params-returns.ts';
