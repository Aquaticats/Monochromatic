/**
 Library surface of the agent branch landing commands. The bin is `land-agent-branches.ts`.

 @module
 */

export {
  A_MERGE_IS_OPEN,
  A_MERGE_IS_OPEN_AFTER_A_FAILED_BUILD,
  A_RELEASE_GATE_IS_INVALIDATED,
  CHEAPEST_CHECK_TASKS,
  COMMIT_PAUSE_SECONDS,
  COMMIT_PROBLEM_CHARACTERS,
  COMMIT_TIMEOUT_SECONDS,
  COMMIT_TRIES,
  COMPILE_ONLY_ARGUMENT,
  type CommandLimits,
  FAILED_STATUS,
  GIT_ENVIRONMENT,
  GIT_OUTPUT_BYTES,
  GIT_TIMEOUT_SECONDS,
  INCREMENTAL_BUILD_SUFFIX,
  KNOWN_BUILD_OUTPUT_NAMES,
  LOCKFILE_PATH,
  MERGE_OUTPUT_CHARACTERS,
  MERGE_SUBJECT_CHARACTERS,
  MILLISECONDS_PER_DAY,
  MILLISECONDS_PER_SECOND,
  MISUSED_STATUS,
  NO_MERGE_WAS_LEFT_OPEN,
  OFFLINE_INSTALL_LIMITS,
  OFFLINE_INSTALL_WORDS,
  PACKAGES_PREFIX,
  PACKAGE_PATH_SEGMENTS,
  PACKAGE_TASK_LIMITS,
  PATHS_A_RELEASE_DEPENDS_ON,
  REFLOW_COMMIT_MESSAGE,
  REMOVAL_TIMEOUT_SECONDS,
  SUCCESS_STATUS,
  SYSTEMCTL_PROGRAM,
  SYSTEMD_RUN_PROGRAM,
  TASK_ERROR_LINES,
  TASK_OUTPUT_BYTES,
  TASK_TAIL_CHARACTERS,
  TASK_TIMEOUT_SECONDS,
  WORKTREES_FOLDER,
  WORKTREE_COMPILE_LIMITS,
  WRAPPER_COMPILE_LIMITS,
  WRAPPER_COMPILE_TASKS,
  WRAPPER_PATH_PREFIX,
  isKnownBuildOutput,
  worktreeFolderOf,
} from './constants.ts';
export { LandingConfigurationError, } from './errors.ts';
export type {
  Looked,
} from './lookups.ts';
export {
  found,
  missing,
} from './lookups.ts';
export type {
  ProgramOutput,
  RunProgram,
  RunProgramRequest,
} from './run-a-program.ts';
export { runProgramWithChildProcess, } from './run-a-program.ts';
export type { Report, } from './report.ts';
export { reportToTheTerminal, } from './report.ts';
export type {
  AskGitRequest,
  Repository,
  WorktreeRecord,
} from './git-commands.ts';
export {
  answered,
  askGit,
  changedPathsSinceMain,
  commitsOfBranchNotOnMain,
  currentBranchOf,
  fileAtRevision,
  headOf,
  ignoredEntriesOf,
  lastCommitOnMain,
  mainCheckoutOf,
  mergeIsOpen,
  openMergeMarker,
  pathsIn,
  refsHolding,
  stagedPaths,
  statusOf,
  unresolvedPaths,
  worktreeRecords,
} from './git-commands.ts';
export {
  commitStaged,
  mergeMainInto,
  mergeWithoutCommitting,
  removeWorktree,
  stagePaths,
} from './git-changes.ts';
export type { KindOfUncommittedEdit, } from './uncommitted-edits.ts';
export {
  blockedFilesIn,
  kindOfUncommittedEdit,
} from './uncommitted-edits.ts';
export {
  changesTheLockfile,
  isReleasePath,
  packagesAmong,
  releasePathsAmong,
} from './paths-of-a-branch.ts';
export {
  cheapestCheckTaskAmong,
  checkTasksFor,
  declaredTasksOf,
  tasksDeclaredIn,
} from './package-tasks.ts';
export {
  cgroupWordsOf,
  compileArgumentsOf,
  errorLinesOf,
  runTaskInACgroup,
  tailOf,
  tallyLineOf,
  tallyOf,
} from './task-in-a-cgroup.ts';
export type {
  CheckKind,
  CheckOutcome,
  TaskOutcome,
} from './check-a-branch.ts';
export {
  checkABranch,
  checkPassed,
} from './check-a-branch.ts';
export type { LandingOutcome, } from './land-a-branch.ts';
export type { LandingReport, } from './landing-report.ts';
export { landingReportOf, } from './landing-report.ts';
export {
  landABranch,
  landedCompletely,
} from './land-a-branch.ts';
export type { CommitOutcome, } from './open-merge.ts';
export { commitTheOpenMerge, } from './open-merge.ts';
export type { LandingEntry, } from './landing-list.ts';
export {
  entriesIn,
  entriesOfFile,
} from './landing-list.ts';
export type {
  EntryOutcome,
  ListOutcome,
} from './land-a-list.ts';
export {
  landAList,
  listLandedCompletely,
  worktreeRepositoryOf,
} from './land-a-list.ts';
export type {
  ResolvedText,
  SideToKeep,
} from './conflict-regions.ts';
export {
  keepOneSideOfEachRegion,
  SIDES_TO_KEEP,
} from './conflict-regions.ts';
export type {
  ConflictToSettle,
  ResolutionOutcome,
} from './resolve-conflicts.ts';
export {
  reportOnAConflict,
  settleConflicts,
} from './resolve-conflicts.ts';
export type { MachineState, } from './machine-state.ts';
export {
  activeAgentUnits,
  workingFoldersOfProcesses,
} from './machine-state.ts';
export type { WorktreeReport, } from './worktrees.ts';
export { reportOnWorktrees, } from './worktrees.ts';
export type { RemovalOutcome, } from './remove-worktrees.ts';
export { removeRemovableWorktrees, } from './remove-worktrees.ts';
export type {
  Bounds,
  Invocation,
} from './command-line.ts';
export {
  parseCommandLine,
  parseSideToKeep,
} from './command-line.ts';
export {
  boundsOf,
  requiredTextOption,
  textOption,
  textsOption,
} from './options.ts';
export {
  PROGRAM_NAME,
  USAGE,
} from './usage.ts';
export { runTheCommandLine, } from './run-the-command-line.ts';
