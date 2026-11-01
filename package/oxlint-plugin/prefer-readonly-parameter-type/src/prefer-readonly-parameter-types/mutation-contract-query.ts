/**
 TypeScript declaration lookup for shared `@mutates` contracts.
 
 @module
 */

import {
  parseMutationContractBlocks,
  type ParsedMutationContractBlock,
} from '@monochromatic-dev/oxlint-plugin-shared/ts';
import type {
  BindingName,
  SourceFile,
} from 'typescript/ast';
import {
  isBindingElement,
  isIdentifier,
} from 'typescript/ast/is';

import type { EffectCallableDeclaration, } from './effect-summary-model.ts';

/**
 Sentinel when callable has no attached TSDoc comment.
 */
export const MUTATION_CONTRACT_UNAVAILABLE: unique symbol = Symbol(
  'callable declaration lacks attached mutation contract comment',
);

/**
 Mutation contracts attached to one callable declaration.
 
 @example
 ```ts
 const contracts: CallableMutationContracts = {
   commentStartOffset: 10,
   commentEndOffset: 80,
   commentBodyStartOffset: 12,
   blocks: [],
 };
 ```
 */
export type CallableMutationContracts = {
  readonly commentStartOffset: number;
  readonly commentEndOffset: number;
  readonly commentBodyStartOffset: number;
  readonly blocks: readonly ParsedMutationContractBlock[];
};

/**
 Finds first doc-comment delimiter inside TypeScript JSDoc span.
 */
const DOC_COMMENT_START = '/**';

/**
 Block-comment opening delimiter width.
 */
const COMMENT_OPEN_WIDTH = 2;

/**
 Block-comment closing delimiter width.
 */
const COMMENT_CLOSE_WIDTH = 2;

/**
 Reads shared mutation blocks from callable's final attached JSDoc comment.
 
 @param declaration - Callable declaration carrying optional JSDoc.
 
 @param sourceFile - Source text owning declaration.
 
 @returns contracts with exact source offsets or absent-comment sentinel.
 
 @example
 ```ts
 const contracts = mutationContractsForDeclaration({ declaration, sourceFile });
 ```
 */
export function mutationContractsForDeclaration({
  declaration,
  sourceFile,
}: {
  readonly declaration: EffectCallableDeclaration;
  readonly sourceFile: SourceFile;
},): CallableMutationContracts | typeof MUTATION_CONTRACT_UNAVAILABLE {
  /**
   Final JSDoc node attached to declaration.
   */
  const jsDoc = declaration.jsDoc
    ?.at(-1,);
  if (jsDoc === undefined)
    return MUTATION_CONTRACT_UNAVAILABLE;
  /**
   Full JSDoc node text, including leading trivia retained by TypeScript.
   */
  const jsDocText = sourceFile.text
    .slice(
    jsDoc.pos,
    jsDoc.end,
    );
  /**
   Relative doc-comment delimiter offset after leading trivia.
   */
  const relativeCommentStart = jsDocText.indexOf(DOC_COMMENT_START,);
  if (relativeCommentStart === (-1))
    return MUTATION_CONTRACT_UNAVAILABLE;
  /**
   Absolute opening delimiter offset.
   */
  const commentStartOffset = jsDoc.pos + relativeCommentStart;
  /**
   Absolute exclusive closing delimiter offset.
   */
  const commentEndOffset = jsDoc.end;
  /**
   Absolute comment-body start after `/*`.
   */
  const commentBodyStartOffset = commentStartOffset + COMMENT_OPEN_WIDTH;
  /**
   Comment body matching Oxlint Comment.value semantics.
   */
  const commentValue = sourceFile.text
    .slice(
    commentBodyStartOffset,
    commentEndOffset - COMMENT_CLOSE_WIDTH,
    );
  return {
    commentStartOffset,
    commentEndOffset,
    commentBodyStartOffset,
    blocks: parseMutationContractBlocks({ commentValue, },),
  };
}

/**
 Maps every identifier in parameter binding name to parameter index.
 
 @param name - Parameter identifier or nested destructuring pattern.
 
 @param parameterIndex - Owning source parameter index.
 
 @param sourceFile - Source file providing identifier text.
 
 @param targetIndexes - Map receiving target names.
 
 @mutates targetIndexes - Adds binding names mapped to source parameter.
 */
function collectParameterTargets({
  name,
  parameterIndex,
  sourceFile,
  targetIndexes,
}: {
  readonly name: BindingName;
  readonly parameterIndex: number;
  readonly sourceFile: SourceFile;
  readonly targetIndexes: Map<string, number>;
},): void {
  if (isIdentifier(name,)) {
    targetIndexes.set(
      name.getText(sourceFile,),
      parameterIndex,
    );
    return;
  }
  name.elements
    .forEach(function collect(element,): void {
      if (isBindingElement(element,) && (element.name !== undefined)) {
        collectParameterTargets({
          name: element.name,
          parameterIndex,
          sourceFile,
          targetIndexes,
        },);
      }
    },);
}

/**
 Maps callable parameter and destructured binding names to parameter indexes.
 
 @param declaration - Callable declaration whose targets are required.
 
 @param sourceFile - Source file providing authored names.
 
 @returns target-to-parameter map.
 
 @example
 ```ts
 const targets = mutationTargetIndexes({ declaration, sourceFile });
 ```
 */
export function mutationTargetIndexes({
  declaration,
  sourceFile,
}: {
  readonly declaration: EffectCallableDeclaration;
  readonly sourceFile: SourceFile;
},): ReadonlyMap<string, number> {
  /**
   Target map populated from every parameter binding.
   */
  const targetIndexes = new Map<string, number>();
  declaration.parameters
    .forEach(function collectParameter(
      parameter,
      parameterIndex,
    ): void {
      collectParameterTargets({
        name: parameter.name,
        parameterIndex,
        sourceFile,
        targetIndexes,
      },);
    },);
  return targetIndexes;
}
