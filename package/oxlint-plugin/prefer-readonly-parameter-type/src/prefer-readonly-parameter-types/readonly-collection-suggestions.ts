/**
 Verified readonly standard-collection suggestions.
 
 @module
 */

import type {
  Context,
  Fixer,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { ParameterDeclaration, } from 'typescript/ast';
import {
  isIdentifier,
  isTypeReferenceNode,
} from 'typescript/ast/is';

import { classifyReadonlyType, } from './readonly-classifier.ts';
import type { ReadonlySuggestion, } from './readonly-suggestion.ts';

/**
 Supported mutable standard collection projections.
 */
const COLLECTION_PROJECTIONS: Readonly<Record<string, {
  readonly readonlyName: string;
  readonly typeArgumentCount: number;
}>> = {
  Array: {
    readonlyName: 'ReadonlyArray',
    typeArgumentCount: 1,
  },
};

/**
 Builds verified mutable standard-collection projection suggestion.
 
 @param context - Rule context providing fix range mapping.
 
 @param parameter - TypeScript parameter with collection annotation.
 
 @param project - TypeScript project proving owner and reachable types.
 
 @returns suggestion list, empty unless exact projection is proven.
 
 @example
 ```ts
 readonlyCollectionSuggestions({ context, parameter, project });
 ```
 */
export function readonlyCollectionSuggestions({
  context,
  parameter,
  project,
}: ForeignBorrowed<{
  readonly context: Context;
  readonly parameter: ParameterDeclaration;
  readonly project: Parameters<typeof classifyReadonlyType>[0]['project'];
}>): ReadonlySuggestion[] {
  if ((parameter.type === undefined)
    || (!isTypeReferenceNode(parameter.type,))
    || (!isIdentifier(parameter.type
      .typeName,)))
    return [];
  /**
   Projection specification selected by authored collection name.
   */
  const projection = COLLECTION_PROJECTIONS[parameter.type
    .typeName
    .text];
  if ((projection === undefined)
    || (parameter.type
      .typeArguments
      ?.length
      !== projection.typeArgumentCount))
    return [];
  /**
   Semantic collection type proving standard-library owner identity.
   */
  const collectionType = project.checker
    .getTypeFromTypeNode(parameter.type,);
  /**
   Collection owner symbol and declarations.
   */
  const owner = collectionType?.getSymbol();
  if ((collectionType === undefined)
    || (!collectionType.isTypeReference())
    || (owner === undefined)
    || (owner.name
      !== parameter.type
        .typeName
        .text)
    || (owner.declarations
      .length
      === 0)
    || (!owner.declarations
      .every(function defaultLibraryDeclaration(handle,): boolean {
      /**
       Resolved owner declaration used for exact provenance check.
       */
        const declaration = handle.resolve(project,);
        return (declaration !== undefined)
          && project.program
            .isSourceFileDefaultLibrary(declaration.getSourceFile(),);
      },)))
    return [];
  /**
   Semantic type arguments whose complete reachability must be readonly.
   */
  const typeArguments = project.checker
    .getTypeArguments(collectionType,);
  if ((typeArguments.length !== projection.typeArgumentCount)
    || (!typeArguments.every(function deepReadonlyTypeArgument(type,): boolean {
      return classifyReadonlyType({
        checker: project.checker,
        project,
        type,
      },)
        .kind
        === 'deep-readonly';
    },)))
    return [];
  /**
   Source file owning authored collection type.
   */
  const sourceFile = parameter.getSourceFile();
  /**
   Authored type retained except exact mutable owner name.
   */
  const authoredType = parameter.type
    .getText(sourceFile,);
  /**
   Readonly owner replacement preserving authored type arguments.
   */
  const replacement = `${projection.readonlyName}${authoredType.slice(parameter.type
    .typeName
    .text
    .length,)}`;
  /**
   BOM-aware Oxlint replacement range.
   */
  const range: [
    number,
    number,
  ] = [
    Math.max(
      0,
      parameter.type
        .getStart(sourceFile,)
        - (context.sourceCode
          .hasBOM ? 1 : 0),
    ),
    Math.max(
      0,
      parameter.type
        .end
        - (context.sourceCode
          .hasBOM ? 1 : 0),
    ),
  ];
  /**
   Authored mutable collection owner named without source trivia.
   */
  const mutableName = parameter.type
    .typeName
    .text;
  /**
   Exact one-line transformation shared by diagnostic and suggestion UI.
   */
  const diagnosticGuidance = `Replace the mutable collection owner \`${mutableName}\` with \`${projection.readonlyName}\`.`;
  return [
    {
      diagnosticGuidance,
      desc: diagnosticGuidance,
      fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['replaceTextRange']> {
        return fixer.replaceTextRange(
          range,
          replacement,
        );
      },
    },
  ];
}
