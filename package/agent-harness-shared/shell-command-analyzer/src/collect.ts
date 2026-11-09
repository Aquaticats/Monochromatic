/**
 Stack-based `unbash` AST collection for shell command analysis.
 
 @module
 */

import type { Script as UnbashScript, } from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  ShellCommandInfo,
  ShellParseError,
} from './types.ts';
import {
  EXECUTED_CONTEXT,
  type CommandCollection,
  type VisitResult,
  type WorkItem,
} from './internal-types.ts';
import {
  visitArithmetic,
  visitNode,
  visitParts,
  visitRedirectsItem,
  visitTest,
  visitWord,
} from './visitors.ts';
import { statementWorkItems, } from './work-items.ts';

/**
 Visit queued redirect work item.
 
 @param item - redirect work item to visit
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @returns visit result for item
 
 @example
 ```ts
 visitRedirectsWorkItem({ item, paramRefs: [] });
 ```
 */
function visitRedirectsWorkItem(
  {
    item,
    paramRefs,
  }: {
    readonly item: Extract<WorkItem, { readonly kind: 'redirects'; }>;
    readonly paramRefs: readonly string[];
  },
): VisitResult {
  return visitRedirectsItem({
    redirects: item.redirects,
    paramRefs,
    context: item.context,
  },);
}

/**
 Visit one queued work item.
 
 @param item - work item to visit
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @returns visit result for item
 
 @example
 ```ts
 visitWorkItem({ item, paramRefs: [] });
 ```
 */
function visitWorkItem(
  {
    item,
    paramRefs,
  }: {
    readonly item: WorkItem;
    readonly paramRefs: readonly string[];
  },
): VisitResult {
  if (item.kind === 'node') {
    return visitNode({
      node: item.node,
      redirects: item.redirects,
      paramRefs,
      context: item.context,
    },);
  }
  if (item.kind === 'word') {
    return visitWord({
      word: item.word,
      context: item.context,
    },);
  }
  if (item.kind === 'parts') {
    return visitParts({
      parts: item.parts,
      context: item.context,
    },);
  }
  if (item.kind === 'arithmetic') {
    return visitArithmetic({
      expression: item.expression,
      context: item.context,
    },);
  }
  if (item.kind === 'test') {
    return visitTest({
      expression: item.expression,
      context: item.context,
    },);
  }
  return visitRedirectsWorkItem({
    item,
    paramRefs,
  },);
}

/**
 Convert parsed `unbash` script to command collection.
 
 @param script - parsed script from `unbash.parse`
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @returns command collection for guardrail signal checks
 
 @example
 ```ts
 collectCommandInfoFromScript({ script, paramRefs: [] });
 ```
 */
function collectCommandInfoFromScript(
  {
    script,
    paramRefs,
  }: {
    readonly script: ForeignBorrowed<UnbashScript>;
    readonly paramRefs: readonly string[];
  },
): CommandCollection {
  /**
   Parsed command records accumulated in traversal order.
   */
  const commands: ShellCommandInfo[] = [];
  /**
   Nested parse errors accumulated in traversal order.
   */
  const parseErrors: ShellParseError[] = [];
  /**
   Parser node kinds the walk had no visitor for, in traversal order.
   */
  const notUnderstoodNodeKinds: string[] = [];
  /**
   Mutable traversal flags updated while stack drains.
   */
  const flags = {
    isPipeline: false,
    hasBackground: false,
    hasCommandSubstitution: false,
    hasProcessSubstitution: false,
  };
  /**
   LIFO work stack seeded with top-level statements.
   */
  const stack: WorkItem[] = statementWorkItems({
    statements: script.commands,
    context: EXECUTED_CONTEXT,
  },)
    .toReversed();

  for (let item = stack.pop(); item !== undefined; item = stack.pop()) {
    /**
     Result emitted by current traversal item.
     */
    const result = visitWorkItem({
      item,
      paramRefs,
    },);
    commands.push(...result.commands,);
    parseErrors.push(...result.parseErrors,);
    notUnderstoodNodeKinds.push(...result.notUnderstoodNodeKinds,);
    flags.isPipeline ||= result.flags
      .isPipeline;
    flags.hasBackground ||= result.flags
      .hasBackground;
    flags.hasCommandSubstitution ||= result.flags
      .hasCommandSubstitution;
    flags.hasProcessSubstitution ||= result.flags
      .hasProcessSubstitution;
    for (const workItem of result.workItems
      .toReversed())
      stack.push(workItem,);
  }

  return {
    commands,
    flags,
    parseErrors,
    notUnderstoodNodeKinds,
  };
}

export { collectCommandInfoFromScript, };
