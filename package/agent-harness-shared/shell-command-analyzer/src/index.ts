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
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
} from './process-name-wait.ts';
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
