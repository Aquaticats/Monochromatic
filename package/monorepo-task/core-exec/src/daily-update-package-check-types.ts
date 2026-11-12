/**
 The shapes of the daily update's package checks: what a check is, what it found, what an issue says, and the outside
 world the checks act on.

 Rationale: `doc/planning/toolchain-always-latest.md` ("Checking every package after the update").

 @module
 */

/**
 The kind of package a check covers, which decides the task, the time bound and how many run side by side.
 */
export type CheckKind = 'rust' | 'typescript';

/**
 One check of one package, run through the package's own `mise` task.
 */
export type PackageCheck = {
  /**
   Repository-relative package directory, for example `package/music-player/desktop-app`.
   */
  readonly package: string;
  /**
   Kind of package.
   */
  readonly kind: CheckKind;
  /**
   Name of the package's `mise` task, for example `lint:clippy`.
   */
  readonly task: string;
  /**
   Longest the task may run before it is stopped, in milliseconds.
   */
  readonly timeoutMs: number;
};

/**
 How a check ended: `passed`, `failed` (a nonzero exit before the time bound), `timed-out` (stopped at the time bound,
 which says nothing about the package), or `not-run` (the pass ran out of time before it started, or it could not start).
 */
export type CheckStatus = 'failed' | 'not-run' | 'passed' | 'timed-out';

/**
 The first error a failed check printed.
 */
export type FirstError = {
  /**
   Compiler error code such as `E0433` or `TS2307`, empty when the error carries none.
   */
  readonly code: string;
  /**
   The error's message without its code.
   */
  readonly message: string;
  /**
   The error's line and the lines that continue it, up to the next blank line.
   */
  readonly excerpt: string;
};

/**
 What one check did in one pass.
 */
export type CheckResult = {
  /**
   The check.
   */
  readonly check: PackageCheck;
  /**
   How it ended.
   */
  readonly status: CheckStatus;
  /**
   Seconds it ran.
   */
  readonly seconds: number;
  /**
   The first error of a failed check, absent for any other status and for a failure that printed none.
   */
  readonly firstError?: FirstError;
};

/**
 What a finished check command reports.
 */
export type CheckOutcome = {
  /**
   Exit status; nonzero for a failure or a stop.
   */
  readonly exitCode: number;
  /**
   Whether the command was stopped for running past its time bound.
   */
  readonly timedOut: boolean;
  /**
   Seconds it ran.
   */
  readonly seconds: number;
  /**
   The first error the check printed, absent when no line of its output starts one.
   */
  readonly firstError?: FirstError;
};

/**
 What comparing the pass before the update with the pass after it says about a check.
 */
export type CheckVerdict = 'already-red' | 'newly-red' | 'recovered' | 'still-green' | 'unknown';

/**
 One check compared across the update.
 */
export type CheckComparison = {
  /**
   The check.
   */
  readonly check: PackageCheck;
  /**
   What the comparison says.
   */
  readonly verdict: CheckVerdict;
  /**
   Why the verdict is `unknown`, otherwise a short statement of what was seen.
   */
  readonly reason: string;
  /**
   Result before the update, absent when the check did not exist then.
   */
  readonly before?: CheckResult;
  /**
   Result after the update.
   */
  readonly after: CheckResult;
};

/**
 One locked version that moved.
 */
export type LockMove = {
  /**
   Crate, package or tool name.
   */
  readonly name: string;
  /**
   Version before, empty when the name is new.
   */
  readonly from: string;
  /**
   Version after, empty when the name is gone.
   */
  readonly to: string;
};

/**
 The moves found in one lock.
 */
export type MoveSource = {
  /**
   Which lock, for example `package/a/b/Cargo.lock`.
   */
  readonly source: string;
  /**
   Moves in that lock.
   */
  readonly moves: readonly LockMove[];
};

/**
 An issue ready to be opened, or kept for a later run.
 */
export type RenderedIssue = {
  /**
   Hidden line that names the package, the check and the cause, which later runs search for.
   */
  readonly marker: string;
  /**
   Issue title.
   */
  readonly title: string;
  /**
   Issue body, ending with the marker.
   */
  readonly body: string;
  /**
   Text of the comment that stands in for the issue when an open one has the same marker.
   */
  readonly comment: string;
  /**
   Labels, from `doc/agent/triage-labels.md`.
   */
  readonly labels: readonly string[];
};

/**
 A call to `gh`.
 */
export type GhRequest = {
  /**
   Arguments after `gh`; never a token or a body.
   */
  readonly args: readonly string[];
  /**
   Text sent on standard input, empty when none.
   */
  readonly stdin: string;
};

/**
 What `gh` answered.
 */
export type GhResult = {
  /**
   Exit status.
   */
  readonly exitCode: number;
  /**
   Standard output.
   */
  readonly stdout: string;
  /**
   Standard error.
   */
  readonly stderr: string;
};

/**
 The outside world the package checks act on, beside the effects of the update itself.
 */
export type CheckEffects = {
  /**
   Names of every task of every package, as `mise tasks ls --all` prints them (`//package/x/y:task`).
   */
  readonly listCheckTaskNames: () => Promise<readonly string[]>;
  /**
   Runs one check within its time bound and reports how it ended.
   */
  readonly runCheck: (check: PackageCheck,) => Promise<CheckOutcome>;
  /**
   Text of a repository file at a commit; the empty string when the file does not exist there.
   */
  readonly fileAt: (request: {
    readonly commit: string;
    readonly path: string
  },) => Promise<string>;
  /**
   Runs `gh` within its time bound; rejects when it cannot start.
   */
  readonly gh: (request: GhRequest,) => Promise<GhResult>;
  /**
   Issues an earlier run could not publish.
   */
  readonly readPendingIssues: () => Promise<readonly RenderedIssue[]>;
  /**
   Replaces the list of issues waiting to be published.
   */
  readonly writePendingIssues: (issues: readonly RenderedIssue[],) => Promise<void>;
  /**
   Milliseconds on a clock that never goes backwards.
   */
  readonly now: () => number;
};

/**
 What publishing one issue came to.
 */
export type PublishOutcome = {
  /**
   The issue.
   */
  readonly issue: RenderedIssue;
  /**
   `created`, `commented` (an open issue had the same marker), or `failed`.
   */
  readonly action: 'commented' | 'created' | 'failed';
  /**
   Number of the issue created or commented on, absent for a failure or when `gh` printed no address.
   */
  readonly issueNumber?: number;
  /**
   Web address of the issue `gh` created, absent for a comment, a failure, or when `gh` printed none.
   */
  readonly address?: string;
  /**
   Why it failed, empty otherwise.
   */
  readonly reason: string;
};

/**
 What the package checks found, after the update.
 */
export type CheckReport = {
  /**
   Whether the second pass ran: it runs only when the update moved something.
   */
  readonly ran: boolean;
  /**
   Every check compared across the update.
   */
  readonly comparisons: readonly CheckComparison[];
  /**
   Checks that passed before the update and fail after it.
   */
  readonly newlyRed: readonly CheckComparison[];
  /**
   Checks that failed before the update and still fail.
   */
  readonly alreadyRed: readonly CheckComparison[];
  /**
   Checks whose result is not a clear pass or fail on one side.
   */
  readonly unknown: readonly CheckComparison[];
  /**
   Checks that failed before the update and pass after it.
   */
  readonly recovered: readonly CheckComparison[];
  /**
   What came of publishing the issues for the newly red checks.
   */
  readonly outcomes: readonly PublishOutcome[];
};

/**
 What is taken before the update and read after it.
 */
export type BeforeUpdate = {
  /**
   Commit the repository was at.
   */
  readonly headBefore: string;
  /**
   Texts of the files outside the repository that the update may change, by absolute path.
   */
  readonly externalBefore: ReadonlyMap<string, string>;
  /**
   The checks.
   */
  readonly checks: readonly PackageCheck[];
  /**
   Results of the first pass.
   */
  readonly results: readonly CheckResult[];
};

/**
 What a step that may fail came to: it failed, or it returned a value.
 */
export type Attempt<Value,> =
  | {
    readonly failed: false;
    readonly value: Value
  }
    | { readonly failed: true; };
