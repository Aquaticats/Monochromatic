/**
 Shared guardrail helper for shell syntax the analyzer cannot read.
 
 An analyzer that answers "no command here" for a subtree it never walked turns
 a guard into an allow,
 so a node kind the traversal has no visitor for is surfaced as its own answer
 and refused at the guard boundary instead of being read as empty.
 
 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';

//region Constants

/**
 Refusal reason used when the analyzer cannot read part of a command's syntax.
 
 @example
 ```ts
 return NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON;
 ```
 */
const NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON: string = [
  'Blocked: the shell-command analyzer does not understand part of this command,',
  'so it cannot say which commands the command would run.',
  'The analyzer names the parser node kinds it has no visitor for',
  '(`analyzeShellCommand(...).notUnderstoodNodeKinds`),',
  'and a guard refuses rather than treating the empty command list as proof.',
  'Rewrite the command so its syntax is one the analyzer reads,',
  'for example by replacing a parameter operation it reports as `Unknown`',
  'with a `-`, `=`, `+`, `?`, `#`, `%`, `/`, `:`, `^`, `,` or `@` operation,',
  'or by running the parts as separate commands.',
]
  .join(' ',);

//endregion Constants

//region Predicates

/**
 Checks whether the analyzer met shell syntax it has no visitor for.
 
 Returns `true` whenever any part of the parsed tree was named instead of
 walked,
 which is the only answer a guard may refuse on:
 `analyzeShellCommand` reports such a tree as `parsed`,
 so reading `commands` alone would allow the unread subtree through.
 
 @param command - shell command from agent harness Bash tool input
 
 @returns whether some node kind in the command is unread
 
 @example
 ```ts
 hasNotUnderstoodShellSyntax('printf %s ${value;text}'); // true
 hasNotUnderstoodShellSyntax('printf %s ${value:-text}'); // false
 ```
 */
function hasNotUnderstoodShellSyntax(command: string,): boolean {
  /**
   Parsed shell command analysis.
   */
  const analysis = analyzeShellCommand(command,);
  /**
   Parser node kinds the walk could not read.
   */
  const { notUnderstoodNodeKinds, } = analysis;
  return notUnderstoodNodeKinds.length > 0;
}

//endregion Predicates

export {
  hasNotUnderstoodShellSyntax,
  NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
};
