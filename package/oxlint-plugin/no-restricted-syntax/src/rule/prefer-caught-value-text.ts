import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  duplicatesCaughtValueFormatter,
  errorDetectorIdentifier,
  isDuplicateFallback,
  readsErrorDiagnostic,
} from './prefer-caught-value-text.syntax.ts';

/**
 Canonical implementation path exempt from duplicate-implementation reports.
 */
const CANONICAL_FORMATTER_PATH = 'package/caught-value/core/src/index.ts';

/**
 Tests whether current file owns canonical formatter implementation.
 
 @param fileName - Absolute or workspace-relative lint target path.
 
 @returns whether target is canonical implementation module.
 
 @example
 ```ts
 isCanonicalFormatterFile('/repo/packages/module/caught-value/src/index.ts');
 ```
 */
function isCanonicalFormatterFile(fileName: string,): boolean {
  return fileName
    .replaceAll(
    '\\',
    '/',
    )
    .endsWith(CANONICAL_FORMATTER_PATH,);
}

/**
 Prefers shared caught-value diagnostics over package-local implementations.
 
 @example
 ```ts
 caughtValueText(error,);
 caughtValueStack(error,);
 ```
 */
export const preferCaughtValueText: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Use shared caught-value formatting instead of duplicating Error and fallback branches.',
      recommended: true,
    },
    messages: {
      duplicate: 'Use caughtValueText or caughtValueStack from @monochromatic-dev/caught-value-core/ts.',
    },
  },
  /**
   Creates duplicate formatter visitor.
   
   @param context - Foreign rule context receiving diagnostics.
   
   @mutates context - Emits diagnostics through foreign rule context.
   
   @example
   ```ts
   preferCaughtValueText.createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Reports duplicate formatter syntax.
     
     @param node - Duplicated formatter node.
     */
    function reportDuplicate(node: ForeignBorrowed<ESTree.Node>,): void {
      context.report({
        node,
        messageId: 'duplicate',
      },);
    }

    return {
      ConditionalExpression(node: ForeignBorrowed<ESTree.ConditionalExpression>,): void {
        if (isCanonicalFormatterFile(context.filename,))
          return;
        /**
         Identifier tested by conditional Error branch.
         */
        const identifier = errorDetectorIdentifier({
          context,
          expression: node.test,
        },);
        if ((typeof identifier) === 'symbol')
          return;
        if (!readsErrorDiagnostic({
          expression: node.consequent,
          identifier,
        },))
          return;
        if (isDuplicateFallback({
          context,
          expression: node.alternate,
          identifier,
        },))
          reportDuplicate(node,);
      },
      ArrowFunctionExpression(node: ForeignBorrowed<ESTree.ArrowFunctionExpression>,): void {
        if (isCanonicalFormatterFile(context.filename,))
          return;
        if (duplicatesCaughtValueFormatter({
          context,
          node,
        },))
          reportDuplicate(node,);
      },
      FunctionDeclaration(node: ForeignBorrowed<ESTree.Function>,): void {
        if (isCanonicalFormatterFile(context.filename,))
          return;
        if (duplicatesCaughtValueFormatter({
          context,
          node,
        },))
          reportDuplicate(node,);
      },
      FunctionExpression(node: ForeignBorrowed<ESTree.Function>,): void {
        if (isCanonicalFormatterFile(context.filename,))
          return;
        if (duplicatesCaughtValueFormatter({
          context,
          node,
        },))
          reportDuplicate(node,);
      },
    };
  },
};
