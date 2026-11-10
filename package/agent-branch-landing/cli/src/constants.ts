/**
 Fixed values of the landing commands, each with where it comes from. Every limit a caller may need to change for one run is an
 option instead; what stays here is what a landing has needed every time.

 @module
 */

/**
 Milliseconds in one second.
 */
export const MILLISECONDS_PER_SECOND = 1_000;

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

//region Exit codes

/**
 What a command exits with when it did what it was asked. `doc/decision/cli-output-and-exit-codes.md` owns these three, and no
 landing adds a fourth: what a landing found is printed, so a caller reads a sentence and not a code table.
 */
export const SUCCESS_STATUS = 0;

/**
 What a command exits with when it tried and did not finish: a merge that failed, a tree that does not compile, a commit the
 wrapper refused every time, or commits of the branch that are still not on main.
 */
export const FAILED_STATUS = 1;

/**
 What a command exits with when the caller or the checkout is wrong: an unknown option, a message missing from a list entry, a
 folder that is not a repository, a worktree on a detached HEAD, or a merge already open before the command started.
 */
export const MISUSED_STATUS = 2;

//endregion Exit codes

//region Where a landing runs

/**
 Folder under the main checkout that holds every agent's worktree, as `AGENTS.md` and the agent skill name them.
 */
export const WORKTREES_FOLDER = '.claude/worktrees';

/**
 Names the folder the worktrees of a checkout live in.

 @param mainCheckout - the main checkout

 @returns the folder those worktrees sit in

 @example
 ```ts
 const folder = worktreeFolderOf('/repository',);
 ```
 */
export function worktreeFolderOf(mainCheckout: string,): string {
  return `${mainCheckout}/${WORKTREES_FOLDER}`;
}

/**
 The program that says which units are running, so a worktree whose agent still works is never removed.
 */
export const SYSTEMCTL_PROGRAM = 'systemctl';

/**
 Milliseconds in a day, which is how recently a worktree must have been left alone before anything considers removing it.
 */
export const MILLISECONDS_PER_DAY = 86_400_000;

/**
 The last segment of an ignored entry that may be removed with its worktree: a build output or an install, which a package
 manager or a compiler makes again. `AGENTS.md` rules `GCL` and `GC2` ask for the ignored entries to be listed before anything
 is cleaned, and this is what each listed entry's own name is judged against.
 */
export const KNOWN_BUILD_OUTPUT_NAMES: ReadonlySet<string> = new Set([
  'node_modules',
  'dist',
  'target',
  '.cache',
  '.turbo',
  'build',
  'coverage',
  'bundle',
  'out',
  '.mise',
  '.rustc_info.json',
  '.gradle',
],);

/**
 The suffix of an incremental compiler's record, which is a build output whatever its prefix.
 */
export const INCREMENTAL_BUILD_SUFFIX = '.tsbuildinfo';

/**
 Tells whether an ignored entry is a build output, which is the only kind of ignored file a worktree removal may take with it.

 @param entry - the entry's path, as `git clean --dry-run -d -X` printed it, with or without a trailing slash

 @returns true when the entry is a build output or an install

 @example
 ```ts
 const removable = isKnownBuildOutput('package/a/b/node_modules/',);
 ```
 */
export function isKnownBuildOutput(entry: string,): boolean {
  /**
   The entry without the spaces around it.
   */
  const trimmed = entry.trim();
  /**
   The entry without the slash a directory ends with.
   */
  const withoutTrailingSlash = trimmed.endsWith('/',) ? trimmed.slice(
    0,
    -1,
  ) : trimmed;
  /**
   The entry's own name, which is all this judges.
   */
  const name = withoutTrailingSlash.split('/',)
    .at(-1,)
    ?? '';
  return KNOWN_BUILD_OUTPUT_NAMES.has(name,) || name.endsWith(INCREMENTAL_BUILD_SUFFIX,);
}

/**
 Seconds a worktree removal may take before it is killed. A removal that stops halfway leaves a half-deleted tree, which is why
 it gets far longer than a question.
 */
export const REMOVAL_TIMEOUT_SECONDS = 1_800;

/**
 What every Git question and every Git mutation of a landing is asked with: no credential prompt, so a run that cannot answer
 never waits on a terminal nobody is looking at.
 */
export const GIT_ENVIRONMENT: Readonly<Record<string, string>> = {
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 Longest text of one merging message kept on a report line, so one long commit subject cannot fill a log.
 */
export const MERGE_SUBJECT_CHARACTERS = 110;

/**
 Longest text of a failing commit's complaint kept on a report line, which is what says why the wrapper refused it.
 */
export const COMMIT_PROBLEM_CHARACTERS = 160;

/**
 Longest text of a failed merge kept in a report. A conflict is long; what a reader needs first is the files it names.
 */
export const MERGE_OUTPUT_CHARACTERS = 1_500;

//endregion Where a landing runs

//region What a landing prints

/**
 What a landing says about a merge it could not commit. Callers keep only the last lines, so this sentence comes last and names
 the one thing to do next.
 */
export const A_MERGE_IS_OPEN: string
  = 'A MERGE IS OPEN IN THE MAIN CHECKOUT: resolve it before anything else commits there';

/**
 What a landing says about a merge that failed and left nothing open, so a caller knows the checkout is usable.
 */
export const NO_MERGE_WAS_LEFT_OPEN = 'no merge was left open';

/**
 What a landing says when the merged tree does not compile, which leaves the merge open on purpose: the commit is the only step
 that would land it, and it must not happen.
 */
export const A_MERGE_IS_OPEN_AFTER_A_FAILED_BUILD: string
  = 'A MERGE IS OPEN IN THE MAIN CHECKOUT: fix it, or run git merge --abort, before anything else commits there';

//endregion What a landing prints

//region The Git wrapper

/**
 The folder whose changes make the merged tree a Rust build: `package/git-policy/cli` is the wrapper every commit of this
 repository goes through.
 */
export const WRAPPER_PATH_PREFIX = 'package/git-policy/cli/';

/**
 The two targets of the wrapper's build, compiled without running them because the agent already ran them on its own tree.
 Owner, 2026-11-10: "I don't think you'd need to rerun the same lints or tests another agent did, do you?" Both targets,
 because `b9000c1929` broke only the binary target with the unit target and every other check green, and no landing noticed until
 a release gate did.
 */
export const WRAPPER_COMPILE_TASKS: readonly string[] = [
  '//package/git-policy/cli:native:test:host',
  '//package/git-policy/cli:native:test:binary:host',
];

/**
 What makes a `cargo nextest run` task compile its tests and stop, which is the whole check a landing runs on them.
 */
export const COMPILE_ONLY_ARGUMENT = '--no-run';

/**
 What a commit of the reflow a format run left in the main checkout is called. It is a commit of its own so the merge that
 follows is the branch's alone, and so `git log` shows why the file changed.
 */
export const REFLOW_COMMIT_MESSAGE = 'style(*): commit the reflow a format run left in the main checkout';

/**
 How many times a merge's commit is tried before the landing gives up. The installed wrapper's scratch commit ceiling stopped
 that step under load, and another commit landing while a merge is open gets it refused, so one try is not enough; a release
 that lands a merge's commit on a moved branch removes the reason for this workaround.
 */
export const COMMIT_TRIES = 15;

/**
 Seconds between two tries of a merge's commit, long enough for the wrapper's scratch commit ceiling to clear.
 */
export const COMMIT_PAUSE_SECONDS = 20;

//endregion The Git wrapper

//region Bounds on every command a landing starts

/**
 Mebibytes of one Git command's output kept in memory. A conflicting merge of a large branch names every file it touches.
 */
const GIT_OUTPUT_MEBIBYTES = 64;

/**
 Mebibytes of one mise task's output kept in memory, which a compiling test suite fills with warnings.
 */
const TASK_OUTPUT_MEBIBYTES = 256;

/**
 Maximum bytes of one Git command's output kept in memory, as the bound `execFile` reads.
 */
export const GIT_OUTPUT_BYTES: number = GIT_OUTPUT_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Maximum bytes of one mise task's output kept in memory, as the bound `execFile` reads.
 */
export const TASK_OUTPUT_BYTES: number = TASK_OUTPUT_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Seconds one Git command may take before it is killed. The check's merge of main into a worktree was bounded at 15 minutes, and
 a merge of this repository's main takes seconds, so a Git still running then is one that will not answer.
 */
export const GIT_TIMEOUT_SECONDS = 900;

/**
 Seconds a merge's commit may take before it is killed, which is longer than the wrapper's own answer ceiling for one commit.
 */
export const COMMIT_TIMEOUT_SECONDS = 120;

/**
 Seconds one mise task may take before it is killed. A package's type check answers in minutes, and a build of the wrapper's two
 targets in a few more.
 */
export const TASK_TIMEOUT_SECONDS = 900;

//endregion Bounds on every command a landing starts

//region Limits of one command's own cgroup

/**
 What one command a landing starts is limited to. `AGENTS.md` asks every non-trivial command to run in its own cgroup (rules
 `RXI` and `RXS`); these are sized per kind of work, never one set for all of them.
 */
export type CommandLimits = {
  /**
   Share of one CPU this work may use, as `systemd` reads it: `800%` is eight CPUs.
   */
  readonly cpuQuota: string;

  /**
   Memory this work may hold, as `systemd` reads it.
   */
  readonly memoryMax: string;

  /**
   Swap this work may hold, when withholding it is part of the bound; `0` keeps a build in memory it was given.
   */
  readonly memorySwapMax?: string;

  /**
   Processes and threads this work may start. Rust compiles and Node test runs both fan out.
   */
  readonly tasksMax: string;

  /**
   I/O weight of this work, so a compile never starves the commits of the agents still working.
   */
  readonly ioWeight: string;
};

/**
 The wrapper's own Rust build, run from `mise run` in the main checkout: the widest of these, because it compiles the wrapper
 and its two test targets.
 */
export const WRAPPER_COMPILE_LIMITS: CommandLimits = {
  cpuQuota: '800%',
  memoryMax: '12G',
  memorySwapMax: '0',
  tasksMax: '3000',
  ioWeight: '10',
};

/**
 The same wrapper build inside an agent's worktree, which is one command of a check and not the check itself.
 */
export const WORKTREE_COMPILE_LIMITS: CommandLimits = {
  cpuQuota: '600%',
  memoryMax: '10G',
  tasksMax: '2500',
  ioWeight: '10',
};

/**
 One package's own task, such as its type check, run in the agent's worktree.
 */
export const PACKAGE_TASK_LIMITS: CommandLimits = {
  cpuQuota: '400%',
  memoryMax: '8G',
  memorySwapMax: '0',
  tasksMax: '1500',
  ioWeight: '10',
};

/**
 The offline `pnpm install` a landed lockfile change needs in the main checkout, which resolves the whole workspace.
 */
export const OFFLINE_INSTALL_LIMITS: CommandLimits = {
  cpuQuota: '800%',
  memoryMax: '8G',
  memorySwapMax: '0',
  tasksMax: '3000',
  ioWeight: '10',
};

/**
 The program that runs one command in a cgroup of its own, and the words that make it a scope around that command.
 */
export const SYSTEMD_RUN_PROGRAM = 'systemd-run';

//endregion Limits of one command's own cgroup

//region What checks a branch before it lands

/**
 The tasks a package may declare that prove the least while still proving the merge with main fits, in the order tried. Owner,
 2026-11-10: "I don't think you'd need to rerun the same lints or tests another agent did, do you?" The agent ran the full suite
 on its own tree; what it could not run is its work combined with what main gained since.
 */
export const CHEAPEST_CHECK_TASKS: readonly string[] = [
  'lint:types',
  'build',
  'test:unit',
  'test',
];

/**
 How many leading path segments name a package: `package/<category>/<name>`, so `package/` and two more.
 */
export const PACKAGE_PATH_SEGMENTS = 3;

/**
 Prefix every repository-relative package path starts with.
 */
export const PACKAGES_PREFIX = 'package/';

/**
 Lines of a failed task's output kept in a report, and how far a compilation error line is read for. A Rust error names its
 code on one line and its location on the next, so both are kept and the rest is left out.
 */
export const TASK_ERROR_LINES = 30;

/**
 Lines of a failed task's own output kept when the report wants the tail instead, which is what a test failure explains itself
 in.
 */
export const TASK_TAIL_CHARACTERS = 900;

//endregion What checks a branch before it lands

//region What a release gate depends on

/**
 The paths a release depends on, reported whenever a landed branch changes one. A dispatch refuses a send when a path it
 watched changed after its gate ran (`package/crate-release/cli/README.md`), and the paths it watches come from `cargo metadata`
 (`package/crate-release/cli/src/watch-paths.ts`): the wrapper crate's folders, and each workspace root's manifest and lockfile.
 */
export const PATHS_A_RELEASE_DEPENDS_ON: {
  readonly directories: readonly string[];
  readonly fileNames: readonly string[];
} = {
  directories: ['package/git-policy'],
  fileNames: [
    'Cargo.toml',
    'Cargo.lock'
  ],
};

/**
 What a report says when a branch changed a path a release depends on, so the gate that covers it is run again before a
 dispatch.
 */
export const A_RELEASE_GATE_IS_INVALIDATED: string
  = 'a release refuses a dispatch when those paths changed after its gate, so run that gate again before dispatching';

//endregion What a release gate depends on

//region A lockfile change

/**
 The lockfile whose change leaves the main checkout needing an install, which the landing runs there once the branch is on
 main.
 */
export const LOCKFILE_PATH = 'pnpm-lock.yaml';

/**
 What installs a landed lockfile change offline in the main checkout, through `mise` so the workspace's own pnpm answers.
 */
export const OFFLINE_INSTALL_WORDS: readonly string[] = [
  'exec',
  '--',
  'pnpm',
  'install',
  '--offline',
  '--frozen-lockfile',
];

//endregion A lockfile change
