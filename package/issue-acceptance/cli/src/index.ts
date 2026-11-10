/**
 Settles GitHub issues against what a branch actually holds.

 A plan names the issues,
 each issue's acceptance checks,
 and the words it receives once every check holds.
 The command decides every check on the branch first,
 and writes on GitHub only when asked with `--apply`.

 @module
 */

export {
  EXIT_FAILED,
  EXIT_MISUSE,
  EXIT_OK,
  runCli,
  type CliStreams,
  type RunCliRequest,
  type RunReadyBugsRequest,
  type RunSettleRequest,
  type RunShowRequest,
} from './cli-entry.ts';
export {
  parseArguments,
  USAGE,
  type Command,
  type HelpCommand,
  type ReadyBugsCommand,
  type SettleCommand,
  type ShowCommand,
} from './cli-args.ts';
export {
  CHILD_DEADLINE_MS,
  runChild,
  type ChildOutcome,
  type ChildRequest,
} from './child-process.ts';
export {
  ChildProcessError,
  CliInvocationError,
  IssueAcceptanceError,
  PlanError,
} from './errors.ts';
export {
  createGitReader,
  type FileText,
  type GitReader,
  type GitReaderOptions,
} from './git-reader.ts';
export {
  createIssueTracker,
  type IssueListing,
  type IssueSnapshot,
  type IssueTracker,
  type IssueTrackerOptions,
} from './issue-tracker.ts';
export { readCheck, } from './plan-check.ts';
export {
  readCommitCorrection,
  readIssueEntry,
  refuseRepeatedIssues,
} from './plan-entry.ts';
export type {
  Check,
  CommitCorrection,
  IssueAction,
  IssueEntry,
  Plan,
} from './plan-model.ts';
export {
  parsePlan,
  readPlan,
} from './plan-read.ts';
export {
  asArray,
  asCommit,
  asInteger,
  asRecord,
  asString,
  asStringArray,
  describeValue,
  isRecord,
  refuse,
  refuseUnknownKeys,
} from './plan-value.ts';
export {
  groupReadyBugs,
  READY_BUG_LABELS,
  renderReadyBugReport,
  type ReadyBugGroup,
  type ReadyBugReport,
} from './ready-bugs.ts';
export {
  settlePlan,
  type CorrectionSettlement,
  type IssueSettlement,
  type SettleCorrectionRequest,
  type SettleIssueRequest,
  type SettleReport,
  type SettleRequest,
  type SettlementKind,
} from './settle.ts';
export {
  renderIssue,
  showIssues,
  type RenderIssueRequest,
  type ShowIssuesRequest,
  type ShowMode,
} from './show.ts';
export {
  evaluateChecks,
  type CheckOutcome,
  type Verdict,
  type VerdictRequest,
} from './verdict.ts';
