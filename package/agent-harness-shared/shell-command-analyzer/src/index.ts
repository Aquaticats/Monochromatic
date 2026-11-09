/**
 Shared shell-command analyzer exports.
 
 @module
 */

export { analyzeShellCommand, } from './analyzer.ts';
export {
  parseShellReviewStructure,
  ShellReviewSyntaxError,
} from './review-structure.ts';
export {
  BUN_TEST_BAN_REASON,
  invokesBunTest,
} from './bun-test.ts';
export {
  invokesMutatingGitThroughRealGitPath,
  REAL_GIT_MUTATION_BAN_REASON,
} from './real-git-mutation.ts';
export type { RealGitMutationCheck, } from './real-git-mutation.ts';
export {
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
} from './process-name-wait.ts';
export {
  hasNotUnderstoodShellSyntax,
  NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
} from './not-understood-shell-syntax.ts';
export {
  extractParamRefs,
  looksLikePath,
} from './refs.ts';
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
} from './types.ts';
