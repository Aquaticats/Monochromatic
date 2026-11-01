/**
 Public shell-command analyzer types.
 
 @module
 */

import type { Redirection as UnbashRedirection, } from 'unbash';

//region Command context

/**
 Literal values assigned by one enclosing shell `for` loop.
 */
type ShellLoopBinding = {
  /**
   Loop variable name referenced by body commands.
   */
  readonly name: string;
  /**
   Parsed source words assigned across loop iterations.
   */
  readonly values: readonly string[];
  /**
   Original shell spellings paired with parsed values.
   */
  readonly sourceTexts: readonly string[];
};

/**
 Context describing when a parsed command can execute.
 */
type ShellCommandContext = {
  /**
   Lexically enclosing `for` bindings from outermost to innermost loop.
   */
  readonly loopBindings: readonly ShellLoopBinding[];
  /**
   Count of lexically enclosing `while` and `until` loops, including their condition lists.
   Zero outside any such loop.
   */
  readonly conditionLoopDepth: number;
} & (
  | {
    /**
     Command is evaluated as part of script's immediate control flow.
     */
    readonly kind: 'executed';
  }
    | {
    /**
     Command is stored in a function body and runs only if that function is called.
     */
      readonly kind: 'functionDefinition';
    /**
     Function name whose body contains the command.
     */
      readonly functionName: string;
    }
);

//endregion Command context

//region Command records

/**
 Parsed shell word paired with original source spelling.
 */
type ShellWordSource = {
  /**
   Value after quote removal but before shell runtime expansion.
   */
  readonly value: string;
  /**
   Original shell source spelling including quotes and escapes.
   */
  readonly sourceText: string;
};

/**
 Environment assignment prefix parsed before shell command name.
 */
type ShellEnvAssignment = {
  /**
   Variable name receiving assignment.
   */
  readonly name: string;
  /**
   Assignment value after shell word parsing.
   */
  readonly value: string;
};

/**
 Redirect target classification derived from operator and target shape.
 */
type ShellRedirectKind =
  | 'file'
    | 'fileDescriptor'
    | 'heredoc'
    | 'hereString';

/**
 Parsed redirect attached to a command or compound command.
 */
type ShellRedirect = {
  /**
   Bash redirect operator reported by `unbash`.
   */
  readonly operator: UnbashRedirection['operator'];
  /**
   Parsed target word value, when present.
   */
  readonly target?: string;
  /**
   Original source spelling for target word, when present.
   */
  readonly targetSourceText?: string;
  /**
   Explicit file descriptor before redirect operator, when present.
   */
  readonly fileDescriptor?: number;
  /**
   Variable receiving a dynamically allocated descriptor, distinct from a numeric descriptor.
   */
  readonly fileDescriptorVariable?: string;
  /**
   Destination category useful for callers deciding whether redirects touch files.
   */
  readonly kind: ShellRedirectKind;
  /**
   Whether redirect writes output to a file-like target.
   */
  readonly writesFile: boolean;
  /**
   Whether redirect reads input from a file-like target.
   */
  readonly readsFile: boolean;
};

/**
 Parsed simple shell command.
 */
type ShellCommandInfo = {
  /**
   Command name, for example `rm` or `sudo`.
   */
  readonly name: string;
  /**
   Environment assignments that prefix command invocation.
   */
  readonly envAssignments: readonly ShellEnvAssignment[];
  /**
   Positional arguments and flags after command name.
   */
  readonly args: readonly string[];
  /**
   Argument values paired with original shell source spellings.
   */
  readonly argSources: readonly ShellWordSource[];
  /**
   Redirects attached to command.
   */
  readonly redirects: readonly ShellRedirect[];
  /**
   File-like redirect targets in source order.
   */
  readonly redirectTargets: readonly string[];
  /**
   File-like redirect values paired with original shell source spellings.
   */
  readonly redirectTargetSources: readonly ShellWordSource[];
  /**
   Pre-scanned shell parameter references from raw source.
   */
  readonly paramRefs: readonly string[];
  /**
   Execution context for this command.
   */
  readonly context: ShellCommandContext;
};

//endregion Command records

//region Analysis results

/**
 Parser diagnostic surfaced by `unbash` or parser wrapper.
 */
type ShellParseError = {
  /**
   Human-readable parse diagnostic.
   */
  readonly message: string;
  /**
   Source offset reported by parser.
   */
  readonly pos: number;
};

/**
 Result of analyzing a shell command string.
 */
type ShellCommandAnalysis = {
  /**
   Whether complete command parsed without diagnostics.
   */
  readonly parsed: boolean;
  /**
   Parser diagnostics when `parsed` is false.
   */
  readonly parseErrors: readonly ShellParseError[];
  /**
   Commands in traversal order, including commands stored in function bodies.
   */
  readonly commands: readonly ShellCommandInfo[];
  /**
   Commands evaluated outside function definitions.
   */
  readonly executedCommands: readonly ShellCommandInfo[];
  /**
   Commands stored inside function bodies.
   */
  readonly functionDefinitionCommands: readonly ShellCommandInfo[];
  /**
   Whether parsed syntax contains pipeline operator.
   */
  readonly isPipeline: boolean;
  /**
   Whether parsed syntax backgrounds any statement.
   */
  readonly hasBackground: boolean;
  /**
   Whether parsed syntax contains actual command substitution.
   */
  readonly hasCommandSubstitution: boolean;
  /**
   Whether parsed syntax contains actual process substitution.
   */
  readonly hasProcessSubstitution: boolean;
  /**
   Whether parsed syntax contains heredoc or here-string redirect.
   */
  readonly hasHeredoc: boolean;
  /**
   Parameter references across complete command source.
   */
  readonly allParamRefs: readonly string[];
  /**
   File-like command arguments and redirect targets across commands.
   */
  readonly allFiles: readonly string[];
};

//endregion Analysis results

export type {
  ShellCommandAnalysis,
  ShellCommandContext,
  ShellCommandInfo,
  ShellEnvAssignment,
  ShellLoopBinding,
  ShellParseError,
  ShellRedirect,
  ShellRedirectKind,
  ShellWordSource,
};
