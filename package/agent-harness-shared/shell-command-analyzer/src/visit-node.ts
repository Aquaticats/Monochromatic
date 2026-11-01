/**
 Primary `unbash` node visitor for shell command analysis.
 
 @module
 */

import type {
  SyntaxNode as UnbashNode,
  Redirection as UnbashRedirect,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { commandToInfo, } from './convert.ts';
import {
  EMPTY_REDIRECTS,
  EMPTY_VISIT_RESULT,
  type VisitResult,
} from './internal-types.ts';
import type { ShellCommandContext, } from './types.ts';
import { visitRemainingNode, } from './visit-node-remaining.ts';
import {
  commandWordItems,
  nodeWorkItems,
  redirectWorkItems,
  statementWorkItems,
  wordWorkItems,
  wordsWorkItems,
} from './work-items.ts';

/**
 Visit one AST node.
 
 @param node - AST node to visit
 
 @param redirects - redirects inherited from wrapping statement nodes
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @param context - execution context inherited by node
 
 @returns commands, child work, and flags emitted by node
 
 @example
 ```ts
 visitNode({ node, redirects: [], paramRefs: [], context });
 ```
 */
function visitNode(
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
  if (node.type === 'Statement') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'node',
        node: node.command,
        redirects,
        context,
      },],
      flags: {
        ...EMPTY_VISIT_RESULT.flags,
        hasBackground: node.background === true,
      },
    };
  }
  if (node.type === 'Redirected') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [{
        kind: 'node',
        node: node.command,
        redirects: [
          ...node.redirects,
          ...redirects,
        ],
        context,
      },],
    };
  }
  if (node.type === 'Command') {
    return {
      ...EMPTY_VISIT_RESULT,
      commands: [commandToInfo({
        command: node,
        inheritedRedirects: redirects,
        paramRefs,
        context,
      },),],
      workItems: commandWordItems({
        command: node,
        redirects,
        context,
      },),
    };
  }
  // `Time` and `Negation` are pipeline prefixes in the unbash 6 tree and carry no
  // analyzable children, so only `commands` and inherited redirects are walked here;
  // a bare prefix node falls through to the redirect-only fallback.
  if (node.type === 'Pipeline') {
    return {
      ...EMPTY_VISIT_RESULT,
      flags: {
        ...EMPTY_VISIT_RESULT.flags,
        isPipeline: node.operators
          .length
          > 0,
      },
      workItems: [
        ...nodeWorkItems({
          nodes: node.commands,
          context,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'AndOr') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...nodeWorkItems({
          nodes: node.commands,
          context,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'If') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...statementWorkItems({
          statements: node.clause
            .commands,
          context,
        },),
        ...statementWorkItems({
          statements: node.then
            .commands,
          context,
        },),
        ...(node.else === undefined ? [] : [{
          kind: 'node' as const,
          node: node.else,
          redirects: EMPTY_REDIRECTS,
          context,
        }]),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'For') {
    /**
     Loop syntax parts used by pre-body work and body context.
     */
    const {
      body,
      name,
      wordlist,
    } = node;
    /**
     Literal source values assigned to loop variable.
     */
    const loopValues: string[] = [];
    /**
     Original shell spellings paired with loop values.
     */
    const loopSourceTexts: string[] = [];
    for (const word of wordlist ?? []) {
      loopValues[loopValues.length] = word.value;
      loopSourceTexts[loopSourceTexts.length] = word.text;
    }
    /**
     Body context carrying literal source values assigned to loop variable.
     */
    const loopContext: ShellCommandContext = {
      ...context,
      loopBindings: [
        ...context.loopBindings,
        {
          name: name.value,
          values: loopValues,
          sourceTexts: loopSourceTexts,
        },
      ],
    };
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...wordWorkItems({
          word: name,
          context,
        },),
        ...wordsWorkItems({
          words: wordlist ?? [],
          context,
        },),
        ...statementWorkItems({
          statements: body.commands,
          context: loopContext,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  if (node.type === 'Select') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...wordWorkItems({
          word: node.name,
          context,
        },),
        ...wordsWorkItems({
          words: node.wordlist ?? [],
          context,
        },),
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
  if (node.type === 'While') {
    /**
     Context for the condition list and body, which both repeat on every iteration.
     */
    const conditionLoopContext: ShellCommandContext = {
      ...context,
      conditionLoopDepth: context.conditionLoopDepth + 1,
    };
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...statementWorkItems({
          statements: node.clause
            .commands,
          context: conditionLoopContext,
        },),
        ...statementWorkItems({
          statements: node.body
            .commands,
          context: conditionLoopContext,
        },),
        ...redirectWorkItems({
          redirects,
          context,
        },),
      ],
    };
  }
  return visitRemainingNode({
    node,
    redirects,
    paramRefs,
    context,
  },);
}

export { visitNode, };
