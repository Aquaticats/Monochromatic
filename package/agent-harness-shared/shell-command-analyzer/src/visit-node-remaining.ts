/**
 Less-common `unbash` node visitors for shell command analysis.
 
 @module
 */

import type {
  SyntaxNode as UnbashNode,
  Redirection as UnbashRedirect,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { redirectOnlyCommand, } from './convert.ts';
import {
  EMPTY_REDIRECTS,
  EMPTY_VISIT_RESULT,
  type VisitResult,
} from './internal-types.ts';
import type { ShellCommandContext, } from './types.ts';
import {
  caseItemWorkItems,
  redirectWordItems,
  redirectWorkItems,
  statementWorkItems,
  wordWorkItems,
} from './work-items.ts';

/**
 Visit redirects attached to compound syntax.
 
 @param redirects - redirects inherited from wrapping statement nodes
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @param context - execution context inherited by redirect words
 
 @returns redirect-only command plus redirect word traversal
 
 @example
 ```ts
 visitRedirectsItem({ redirects, paramRefs: [], context });
 ```
 */
function visitRedirectsItem(
  {
    redirects,
    paramRefs,
    context,
  }: {
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly paramRefs: readonly string[];
    readonly context: ShellCommandContext;
  },
): VisitResult {
  if (redirects.length === 0)
    return EMPTY_VISIT_RESULT;
  return {
    ...EMPTY_VISIT_RESULT,
    commands: [redirectOnlyCommand({
      redirects,
      paramRefs,
      context,
    },),],
    workItems: redirectWordItems({
      redirects,
      context,
    },),
  };
}

/**
 Visit remaining node variants after case statements.
 
 @param node - AST node to visit
 
 @param redirects - redirects inherited from wrapping statement nodes
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @param context - execution context inherited by node
 
 @returns commands, child work, and flags emitted by node
 
 @example
 ```ts
 visitRemainingAfterCase({ node, redirects: [], paramRefs: [], context });
 ```
 */
function visitRemainingAfterCase(
  {
    node,
    redirects,
    paramRefs,
    context,
  }: {
    readonly node: ForeignBorrowed<UnbashNode>;
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly paramRefs: readonly string[];
    readonly context: ShellCommandContext;
  },
): VisitResult {
  if (node.type === 'Coproc') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...(node.name === undefined ? [] : wordWorkItems({
          word: node.name,
          context,
        })),
        {
          kind: 'node',
          node: node.body,
          redirects: EMPTY_REDIRECTS,
          context,
        },
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'ArithmeticFor') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...(node.initialize === undefined ? [] : [{
          kind: 'arithmetic' as const,
          expression: node.initialize,
          context,
        }]),
        ...(node.test === undefined ? [] : [{
          kind: 'arithmetic' as const,
          expression: node.test,
          context,
        }]),
        ...(node.update === undefined ? [] : [{
          kind: 'arithmetic' as const,
          expression: node.update,
          context,
        }]),
        ...statementWorkItems({
          statements: node.body
            .commands,
          context,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'TestCommand') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        {
          kind: 'test',
          expression: node.expression,
          context,
        },
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'ArithmeticCommand') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...(node.expression === undefined ? [] : [{
          kind: 'arithmetic' as const,
          expression: node.expression,
          context,
        }]),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  return visitRedirectsItem({
    redirects,
    paramRefs,
    context,
  },);
}

/**
 Visit less-common node variants after primary command forms.
 
 @param node - AST node to visit
 
 @param redirects - redirects inherited from wrapping statement nodes
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @param context - execution context inherited by node
 
 @returns commands, child work, and flags emitted by node
 
 @example
 ```ts
 visitRemainingNode({ node, redirects: [], paramRefs: [], context });
 ```
 */
function visitRemainingNode(
  {
    node,
    redirects,
    paramRefs,
    context,
  }: {
    readonly node: ForeignBorrowed<UnbashNode>;
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly paramRefs: readonly string[];
    readonly context: ShellCommandContext;
  },
): VisitResult {
  if (node.type === 'Function') {
    /**
     Context assigned to commands stored in function body.
     */
    const functionContext: ShellCommandContext = {
      kind: 'functionDefinition',
      functionName: node.name
        .value,
      loopBindings: [],
      conditionLoopDepth: 0,
    };
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...wordWorkItems({
          word: node.name,
          context,
        },),
        {
          kind: 'node',
          node: node.body,
          redirects: EMPTY_REDIRECTS,
          context: functionContext,
        },
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if ((node.type === 'Subshell') || (node.type === 'BraceGroup')) {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...statementWorkItems({
          statements: node.body
            .commands,
          context,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'CompoundList') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...statementWorkItems({
          statements: node.commands,
          context,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'Case') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...wordWorkItems({
          word: node.word,
          context,
        },),
        ...caseItemWorkItems({
          node,
          context,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  return visitRemainingAfterCase({
    node,
    redirects,
    paramRefs,
    context,
  },);
}

export {
  visitRedirectsItem,
  visitRemainingNode,
};
