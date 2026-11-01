/**
 Internal traversal types for shell command analyzer.
 
 @module
 */

import type {
  ArithmeticExpression as UnbashArithmeticExpression,
  DoubleQuotedChild as UnbashDoubleQuotedChild,
  SyntaxNode as UnbashNode,
  Redirection as UnbashRedirect,
  ParsedScript as UnbashParsedScript,
  TestExpression as UnbashTestExpression,
  Word as UnbashWord,
  WordPart as UnbashWordPart,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  ShellCommandContext,
  ShellCommandInfo,
  ShellParseError,
} from './types.ts';

/**
 `unbash` script plus tolerant parser diagnostics.
 */
type ParsedUnbashScript = ForeignBorrowed<UnbashParsedScript>;

/**
 Word part union reached from full words or quoted child lists.
 */
type TraversablePart = ForeignBorrowed<UnbashWordPart> | ForeignBorrowed<UnbashDoubleQuotedChild>;

/**
 AST node work item.
 */
type NodeWorkItem = {
  /**
   Work item discriminant.
   */
  readonly kind: 'node';
  /**
   AST node to visit.
   */
  readonly node: ForeignBorrowed<UnbashNode>;
  /**
   Redirects inherited from wrapping statement nodes.
   */
  readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
  /**
   Execution context inherited by child commands.
   */
  readonly context: ShellCommandContext;
};

/**
 Shell word work item.
 */
type WordWorkItem = {
  /**
   Work item discriminant.
   */
  readonly kind: 'word';
  /**
   Word whose parts may contain nested scripts.
   */
  readonly word: ForeignBorrowed<UnbashWord>;
  /**
   Execution context inherited by expansions inside word.
   */
  readonly context: ShellCommandContext;
};

/**
 Word-parts work item.
 */
type PartsWorkItem = {
  /**
   Work item discriminant.
   */
  readonly kind: 'parts';
  /**
   Parts to scan for nested scripts and words.
   */
  readonly parts: readonly TraversablePart[];
  /**
   Execution context inherited by expansions inside parts.
   */
  readonly context: ShellCommandContext;
};

/**
 Arithmetic expression work item.
 */
type ArithmeticWorkItem = {
  /**
   Work item discriminant.
   */
  readonly kind: 'arithmetic';
  /**
   Expression that may contain command expansion.
   */
  readonly expression: ForeignBorrowed<UnbashArithmeticExpression>;
  /**
   Execution context inherited by expansions inside expression.
   */
  readonly context: ShellCommandContext;
};

/**
 Bash test expression work item.
 */
type TestWorkItem = {
  /**
   Work item discriminant.
   */
  readonly kind: 'test';
  /**
   Expression whose operands may contain command expansion.
   */
  readonly expression: ForeignBorrowed<UnbashTestExpression>;
  /**
   Execution context inherited by expansions inside expression.
   */
  readonly context: ShellCommandContext;
};

/**
 Redirect-only work item for compound command redirects.
 */
type RedirectsWorkItem = {
  /**
   Work item discriminant.
   */
  readonly kind: 'redirects';
  /**
   Redirects to surface as path signals.
   */
  readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
  /**
   Execution context inherited by redirect words.
   */
  readonly context: ShellCommandContext;
};

/**
 Union of all iterative traversal work items.
 */
type WorkItem =
  | NodeWorkItem
    | WordWorkItem
    | PartsWorkItem
    | ArithmeticWorkItem
    | TestWorkItem
    | RedirectsWorkItem;

/**
 Boolean feature flags gathered while walking AST.
 */
type TraversalFlags = {
  /**
   Whether a pipeline operator appears.
   */
  readonly isPipeline: boolean;
  /**
   Whether any statement is backgrounded.
   */
  readonly hasBackground: boolean;
  /**
   Whether command substitution appears.
   */
  readonly hasCommandSubstitution: boolean;
  /**
   Whether process substitution appears.
   */
  readonly hasProcessSubstitution: boolean;
};

/**
 Result of visiting one traversal work item.
 */
type VisitResult = {
  /**
   Commands emitted by this work item.
   */
  readonly commands: readonly ShellCommandInfo[];
  /**
   Child work items to process in source order.
   */
  readonly workItems: readonly WorkItem[];
  /**
   Feature flags emitted by this work item.
   */
  readonly flags: TraversalFlags;
  /**
   Parse errors emitted by nested parsing.
   */
  readonly parseErrors: readonly ShellParseError[];
};

/**
 Command collection returned to public parser wrapper.
 */
type CommandCollection = {
  /**
   Parsed command records.
   */
  readonly commands: readonly ShellCommandInfo[];
  /**
   Feature flags gathered while walking AST.
   */
  readonly flags: TraversalFlags;
  /**
   Parse errors emitted by nested parsing.
   */
  readonly parseErrors: readonly ShellParseError[];
};

/**
 Sentinel returned when no nested script exists.
 */
const NO_SCRIPT: unique symbol = Symbol('nested script payload absent from unbash command',);

/**
 Context for commands executed by script evaluation.
 */
const EXECUTED_CONTEXT: ShellCommandContext = {
  kind: 'executed',
  loopBindings: [],
  conditionLoopDepth: 0,
};

/**
 Empty redirects singleton used for child work items.
 */
const EMPTY_REDIRECTS: readonly ForeignBorrowed<UnbashRedirect>[] = [];

/**
 Empty traversal flags singleton.
 */
const EMPTY_FLAGS: TraversalFlags = {
  isPipeline: false,
  hasBackground: false,
  hasCommandSubstitution: false,
  hasProcessSubstitution: false,
};

/**
 Empty visit result for leaf work items.
 */
const EMPTY_VISIT_RESULT: VisitResult = {
  commands: [],
  workItems: [],
  flags: EMPTY_FLAGS,
  parseErrors: [],
};

export {
  EMPTY_FLAGS,
  EMPTY_REDIRECTS,
  EMPTY_VISIT_RESULT,
  EXECUTED_CONTEXT,
  NO_SCRIPT,
};
export type {
  CommandCollection,
  ParsedUnbashScript,
  TraversalFlags,
  TraversablePart,
  VisitResult,
  WorkItem,
};
