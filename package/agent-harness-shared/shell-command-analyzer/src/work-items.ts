/**
 Work-item builders for `unbash` analyzer traversal.
 
 @module
 */

import type {
  AssignmentPrefix as UnbashAssignmentPrefix,
  Case as UnbashCase,
  Command as UnbashCommand,
  SyntaxNode as UnbashNode,
  ParameterExpansionPart as UnbashParameterExpansionPart,
  Redirection as UnbashRedirect,
  Statement as UnbashStatement,
  Word as UnbashWord,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { ShellCommandContext, } from './types.ts';
import {
  EMPTY_REDIRECTS,
  EMPTY_VISIT_RESULT,
  visitResultForUnreadNodeKind,
  type VisitResult,
  type WorkItem,
} from './internal-types.ts';

/**
 Build one node work item.
 
 @param node - node to enqueue
 
 @param context - execution context inherited by node
 
 @returns node work item with no inherited redirects
 
 @example
 ```ts
 nodeWorkItem({ node, context });
 ```
 */
function nodeWorkItem(
  {
    node,
    context,
  }: {
    readonly node: ForeignBorrowed<UnbashNode>;
    readonly context: ShellCommandContext;
  },
): WorkItem {
  return {
    kind: 'node',
    node,
    redirects: EMPTY_REDIRECTS,
    context,
  };
}

/**
 Build node work items in source order.
 
 @param nodes - nodes to enqueue
 
 @param context - execution context inherited by nodes
 
 @returns work items in source order
 
 @example
 ```ts
 nodeWorkItems({ nodes: pipeline.commands, context });
 ```
 */
function nodeWorkItems(
  {
    nodes,
    context,
  }: {
    readonly nodes: readonly ForeignBorrowed<UnbashNode>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return nodes.map(function nodeToWorkItem(node,): WorkItem {
    return nodeWorkItem({
      node,
      context,
    },);
  },);
}

/**
 Build statement work items in source order.
 
 @param statements - statements to enqueue
 
 @param context - execution context inherited by statements
 
 @returns work items in source order
 
 @example
 ```ts
 statementWorkItems({ statements: script.commands, context });
 ```
 */
function statementWorkItems(
  {
    statements,
    context,
  }: {
    readonly statements: readonly ForeignBorrowed<UnbashStatement>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return statements.map(function statementToWorkItem(statement,): WorkItem {
    return nodeWorkItem({
      node: statement,
      context,
    },);
  },);
}

/**
 Build one word work item.
 
 @param word - word to enqueue
 
 @param context - execution context inherited by word
 
 @returns singleton word work item list
 
 @example
 ```ts
 wordWorkItems({ word, context });
 ```
 */
function wordWorkItems(
  {
    word,
    context,
  }: {
    readonly word: ForeignBorrowed<UnbashWord>;
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return [{
    kind: 'word',
    word,
    context,
  },];
}

/**
 Build word work items in source order.
 
 @param words - words to enqueue
 
 @param context - execution context inherited by words
 
 @returns work items in source order
 
 @example
 ```ts
 wordsWorkItems({ words: command.suffix, context });
 ```
 */
function wordsWorkItems(
  {
    words,
    context,
  }: {
    readonly words: readonly ForeignBorrowed<UnbashWord>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return words.flatMap(function wordToWorkItems(word,): WorkItem[] {
    return wordWorkItems({
      word,
      context,
    },);
  },);
}

/**
 Build nested word work from parameter expansion.

 Every operation form that carries words is listed here,
 and the parser's own `Unknown` operation is reported as unread rather than
 treated as wordless: its operator keeps the source text the parser could not
 split into words.

 @param part - parameter expansion part

 @param context - execution context inherited by nested words

 @returns word work items in source order, or the unread operation kind

 @example
 ```ts
 parameterExpansionResult({ part, context });
 ```
 */
function parameterExpansionResult(
  {
    part,
    context,
  }: {
    readonly part: ForeignBorrowed<UnbashParameterExpansionPart>;
    readonly context: ShellCommandContext;
  },
): VisitResult {
  /**
   Index substitutions execute independently of an optional parameter operation.
   */
  const indexItems = part.index === undefined ? [] : wordWorkItems({
    word: part.index,
    context,
  },);
  /**
   Discriminated operation contains only the words belonging to that expansion form.
   */
  const { operation, } = part;
  if (operation === undefined) {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: indexItems,
    };
  }
  /**
   Parser kind of this operation, kept so an operation no visitor covers is named.
   */
  const operationKind: string = operation.type;
  if (operation.type === 'Unknown')
    return visitResultForUnreadNodeKind(operationKind,);
  if ((operation.type === 'Default')
    || (operation.type === 'Remove')
    || (operation.type === 'CaseModification')
    || (operation.type === 'Transform')) {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...indexItems,
        ...(operation.operand === undefined ? [] : wordWorkItems({
          word: operation.operand,
          context,
        },)),
      ],
    };
  }
  if (operation.type === 'Slice') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...indexItems,
        ...wordWorkItems({
          word: operation.offset,
          context,
        },),
        ...(operation.length === undefined ? [] : wordWorkItems({
          word: operation.length,
          context,
        },)),
      ],
    };
  }
  if (operation.type === 'Replace') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: [
        ...indexItems,
        ...wordWorkItems({
          word: operation.pattern,
          context,
        },),
        ...wordWorkItems({
          word: operation.replacement,
          context,
        },),
      ],
    };
  }
  if (operation.type === 'Names') {
    return {
      ...EMPTY_VISIT_RESULT,
      workItems: indexItems,
    };
  }
  return visitResultForUnreadNodeKind(operationKind,);
}

/**
 Build assignment value word items.
 
 @param assignments - assignment prefixes to scan
 
 @param context - execution context inherited by assignment words
 
 @returns word work items in source order
 
 @example
 ```ts
 assignmentWordItems({ assignments: command.prefix, context });
 ```
 */
function assignmentWordItems(
  {
    assignments,
    context,
  }: {
    readonly assignments: readonly ForeignBorrowed<UnbashAssignmentPrefix>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return assignments.flatMap(function assignmentWords(assignment,): WorkItem[] {
    return [
      ...(assignment.index === undefined ? [] : wordWorkItems({
        word: assignment.index,
        context,
      },)),
      ...wordsWorkItems({
        words: assignment.value
          .type
          === 'ArrayValue' ? assignment.value
            .elements : [assignment.value,],
        context,
      },),
    ];
  },);
}

/**
 Build work items for words attached to redirects.
 
 @param redirects - redirects whose words should be scanned
 
 @param context - execution context inherited by redirect words
 
 @returns word work items in source order
 
 @example
 ```ts
 redirectWordItems({ redirects, context });
 ```
 */
function redirectWordItems(
  {
    redirects,
    context,
  }: {
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return redirects.flatMap(function redirectWords(redirect,): WorkItem[] {
    if (redirect.type === 'HereDoc') {
      if (redirect.body
        .parts
        === undefined)
        return [];
      return [{
        kind: 'parts',
        parts: redirect.body
          .parts,
        context,
      },];
    }
    return redirect.target === undefined ? [] : wordWorkItems({
      word: redirect.target,
      context,
    },);
  },);
}

/**
 Build redirect-only work item.
 
 @param redirects - redirects to surface after child commands
 
 @param context - execution context inherited by redirect item
 
 @returns singleton redirect item when redirects exist
 
 @example
 ```ts
 redirectWorkItems({ redirects, context });
 ```
 */
function redirectWorkItems(
  {
    redirects,
    context,
  }: {
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  if (redirects.length === 0)
    return [];
  return [{
    kind: 'redirects',
    redirects,
    context,
  },];
}

/**
 Build work items for words attached to command.
 
 @param command - command whose words should be scanned
 
 @param redirects - inherited redirects to scan with command redirects
 
 @param context - execution context inherited by command words
 
 @returns word work items in source order
 
 @example
 ```ts
 commandWordItems({ command, redirects: [], context });
 ```
 */
function commandWordItems(
  {
    command,
    redirects,
    context,
  }: {
    readonly command: ForeignBorrowed<UnbashCommand>;
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return [
    ...assignmentWordItems({
      assignments: command.prefix
        .filter(function isAssignment(prefix,) {
          return prefix.type === 'Assignment';
        },),
      context,
    },),
    ...(command.name === undefined
      ? []
      : wordWorkItems({
        word: command.name,
        context,
      })),
    ...command.args
      .flatMap(function argumentItems(argument,): WorkItem[] {
        if (argument.type === 'Assignment')
          return assignmentWordItems({
            assignments: [argument,],
            context,
          },);
        return wordWorkItems({
          word: argument,
          context,
        },);
      },),
    ...redirectWordItems({
      redirects: [
        ...command.redirects,
        ...redirects,
      ],
      context,
    },),
  ];
}

/**
 Build words and body commands from case statement.
 
 @param node - case node to scan
 
 @param context - execution context inherited by case items
 
 @returns work items in source order
 
 @example
 ```ts
 caseItemWorkItems({ node, context });
 ```
 */
function caseItemWorkItems(
  {
    node,
    context,
  }: {
    readonly node: ForeignBorrowed<UnbashCase>;
    readonly context: ShellCommandContext;
  },
): WorkItem[] {
  return node.items
    .flatMap(function caseItemWords(
      item: ForeignBorrowed<UnbashCase['items'][number]>,
    ): WorkItem[] {
      return [
        ...wordsWorkItems({
          words: item.pattern,
          context,
        },),
        nodeWorkItem({
          node: item.body,
          context,
        },),
      ];
    },);
}

export {
  caseItemWorkItems,
  commandWordItems,
  nodeWorkItem,
  nodeWorkItems,
  parameterExpansionResult,
  redirectWordItems,
  redirectWorkItems,
  statementWorkItems,
  wordWorkItems,
  wordsWorkItems,
};
