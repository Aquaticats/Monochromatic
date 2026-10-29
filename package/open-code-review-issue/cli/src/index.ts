/**
 OpenCodeReview finding to GitHub Issue adapter.
 
 @module
 */

export { readStructuredInputFile, } from './file-input.ts';
export {
  runGitHubApi,
  type GitHubApiMethod,
  type GitHubApiRequest,
} from './github-api.ts';
export {
  checkGitHubCliVersion,
  createGitHubApiClient,
} from './github-client.ts';
export type {
  GitHubApiClient,
  GitHubRepository,
  PublicationPreflight,
} from './github-model.ts';
export {
  GitHubProcessError,
  GitHubProcessTimeoutError,
  runBoundedProcess,
  type BoundedProcessRequest,
  type BoundedProcessResult,
  type BoundedProcessRunner,
} from './github-process.ts';
export {
  IncludedResponseError,
  parseIncludedResponse,
  type IncludedResponse,
} from './github-response.ts';
export {
  GitHubCliVersionError,
  parseGitHubCliVersion,
  type GitHubCliVersion,
} from './github-version.ts';
export type {
  AppliedFailure,
  AppliedResult,
} from './cli-result.ts';
export {
  parseCliArguments,
  type CliArguments,
  type HelpCliArguments,
  type RunCliArguments,
} from './cli-args.ts';
export type { CliInputArgument, } from './cli-input.ts';
export { CliInvocationError, } from './cli-invocation-error.ts';
export {
  SecurityAuthorityError,
  selectApplyPlan,
} from './authority.ts';
export { InputValidationError, } from './input-validation-error.ts';
export type {
  InteractiveSelection,
  PromptStreams,
  SquareCheckboxTheme,
} from './interactive-model.ts';
export {
  createSquareCheckboxTheme,
  isPromptCancellation,
  promptForExplicitDecision,
  promptForIssues,
} from './interactive-prompts.ts';
export { selectInteractiveIssues, } from './interactive-selection.ts';
export {
  createPublicationInterruptControl,
  type PublicationInterruptControl,
  type PublicationSignalHost,
} from './interrupt.ts';
export type {
  ClassificationMarker,
  RenderedIssue,
  SourceLink,
} from './issue-model.ts';
export {
  renderIssue,
  renderIssueBody,
} from './issue-render.ts';
export {
  capIssueTitle,
  renderIssueTitle,
} from './issue-title.ts';
export { parseStructuredInput, } from './ingest.ts';
export type {
  ApplyAuthority,
  ApplySelection,
  LabelStrategy,
  NonInteractivePreview,
  PreviewIssue,
  PublicationPlan,
  SecurityPreview,
  SourceReferenceStrategy,
} from './plan-model.ts';
export {
  buildNonInteractivePreview,
  buildPublicationPlan,
} from './plan.ts';
export { runCli, } from './cli-entry.ts';
export {
  CliRuntimeError,
  executeRun,
  type CliStreams,
} from './run-cli.ts';
export {
  parseRepositoryUrl,
  RepositorySelectionError,
  selectRepository,
} from './repository.ts';
export {
  preflightPublication,
  PublicationPreflightError,
} from './preflight.ts';
export type {
  CreatedIssue,
  PublicationResult,
  PublicationStopCheck,
  PublicationWait,
} from './publisher-model.ts';
export {
  AmbiguousReconciliationError,
  IssuePublicationError,
  PublicationInterruptedError,
  PublicationStoppedError,
} from './publication-error.ts';
export { publishIssues, } from './publisher.ts';
export type {
  FindingCategory,
  FindingSeverity,
  InputPosition,
  NormalizedFinding,
  NormalizedInput,
} from './model.ts';
