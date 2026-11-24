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
/**
 Splits shell source the parser rejected into runs of command words,
 exposed for built-artifact verification.

 @internal
 */
export { rawCommandWordRuns, } from './raw-command-word-runs.ts';
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
export {
  findLiteralRmPaths,
  findLiteralRmPathsInScriptFile,
  readSources,
} from './literal-rm-paths.ts';
export { findLiteralRmPathsInCommand, } from './literal-rm-paths-in-command.ts';
export {
  findingIdentity,
  findingsNotPresentBefore,
  literalRmPathFindingsAddedToScriptFile,
} from './literal-rm-paths-added-to-script-file.ts';
export {
  LITERAL_RM_PATH_BAN_REASON,
  literalRmPathRefusal,
} from './literal-rm-path-refusal.ts';
export type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';
export { shellWordCarriesExpansion, } from './shell-word-carries-expansion.ts';
export {
  fileExtensionOf,
  isShellScriptFile,
  NO_SHEBANG_PROGRAM,
  shellShebangProgramName,
} from './shell-script-file.ts';
export {
  NO_SCRIPT_FILE_OPERAND,
  scriptFilesACommandRuns,
  shellFileOperand,
} from './script-files-a-command-runs.ts';
export type { ScriptFileCandidate, } from './script-files-a-command-runs.ts';
export {
  textAfterEdits,
  textAfterOneEdit,
} from './text-after-edits.ts';
export type { TextEdit, } from './text-after-edits.ts';
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
