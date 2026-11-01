/**
 Observation model the invariant checker consumes.

 The observation gatherer reads real repositories and reduces them to these plain records
 (content digests rather than bytes),
 so the checker itself stays pure and unit-testable.

 @module
 */

//region Content

/**
 Path content at one point:
 absent,
 or present with a SHA-256 digest of its bytes.
 */
export type ContentState =
  | Readonly<{ state: 'absent'; }>
    | Readonly<{
      state: 'present';
      digest: string
    }>;

/**
 Landed content of one selected path and every content the invariant accepts for it.
 */
export type PathLanding = Readonly<{
  /**
   Repository path.
   */
  path: string;
  /**
   Content in the landed commit.
   */
  landed: ContentState;
  /**
   Captured bytes,
   followed by clean three-way merges of them onto the landed parent from each candidate preparation base.
   */
  acceptable: readonly ContentState[];
}>;

//endregion Content

//region Attempts

/**
 One history commit carrying an attempt's token.
 */
export type LandingObservation = Readonly<{
  /**
   Commit ID.
   */
  oid: string;
  /**
   Whether the branch recorded at invocation reaches this commit.
   */
  onExpectedBranch: boolean;
  /**
   Paths the commit changes against its first parent.
   */
  changedPaths: readonly string[];
  /**
   Per selected path landing.
   */
  paths: readonly PathLanding[];
  /**
   Whether the remote's copy of the expected branch reaches this commit.
   */
  remoteContains: boolean;
  /**
   Whether this commit sits on top of an amend that replaced an already-published commit,
   so auto-push cannot fast-forward the remote to it
   (`doc/decision/cli-git-concurrent-commits.md` "Amending published history").
   */
  onAmendedPublishedHistory: boolean;
  /**
   Hook-derived facts,
   present only in scenarios that install the corresponding hook.
   */
  hooks?: Readonly<{
    /**
     `commit-msg` trailer present in the landed message.
     */
    commitMsgTrailer?: boolean;
    /**
     `post-commit` runs recorded for this attempt's token.
     */
    postCommitRuns?: number;
    /**
     `git verify-commit` succeeded.
     */
    signatureValid?: boolean;
  }>;
}>;

/**
 One commit attempt and what history shows for it.
 */
export type AttemptObservation = Readonly<{
  /**
   Human-readable attempt label.
   */
  label: string;
  /**
   Process exit code;
   `-1` when the harness killed it.
   */
  exitCode: number;
  /**
   Whether the harness killed the process group on purpose.
   */
  killed: boolean;
  /**
   Paths the attempt selected;
   a landed commit may change only these.
   */
  selectedPaths: readonly string[];
  /**
   Every history commit whose message carries the attempt's token.
   */
  landings: readonly LandingObservation[];
  /**
   JSONL decoding and exit-contract problems.
   */
  eventIssues: readonly string[];
  /**
   `commit-landed` OID from JSONL, when emitted.
   */
  landedEventOid?: string;
  /**
   Whether auto-push applies,
   so the remote must contain the landed commit.
   */
  requiresRemote: boolean;
  /**
   Whether the wrapper surfaced a non-fast-forward auto-push rejection and the local-commit note.
   */
  pushRejectionSurfaced: boolean;
}>;

/**
 One non-commit wrapper command such as `git add` or `git status`.
 */
export type AuxiliaryObservation = Readonly<{
  /**
   Human-readable command label.
   */
  label: string;
  /**
   Process exit code.
   */
  exitCode: number;
  /**
   Whether the accepted design requires this command to succeed.
   */
  mustSucceed: boolean;
  /**
   JSONL decoding and exit-contract problems.
   */
  eventIssues: readonly string[];
}>;

//endregion Attempts

//region Repository

/**
 Worktree path whose final bytes the harness knows.
 */
export type WorktreeExpectation = Readonly<{
  /**
   Repository path.
   */
  path: string;
  /**
   Content the harness wrote last.
   */
  expected: ContentState;
  /**
   Content found after the run.
   */
  actual: ContentState;
}>;

/**
 Path whose real-index entry differs from `HEAD` after the run.
 */
export type StagedDifference = Readonly<{
  /**
   Repository path.
   */
  path: string;
  /**
   Whether the harness itself staged this difference and expects it.
   */
  expectedByHarness: boolean;
  /**
   Whether the staged blob equals content a landed commit replaced,
   meaning the next commit would silently revert landed work.
   */
  revertsLandedContent: boolean;
}>;

/**
 Complete observation of one scenario run.
 */
export type RunObservation = Readonly<{
  /**
   Commit attempts.
   */
  attempts: readonly AttemptObservation[];
  /**
   Other wrapper commands.
   */
  auxiliaries: readonly AuxiliaryObservation[];
  /**
   Worktree paths the harness tracks.
   */
  worktree: readonly WorktreeExpectation[];
  /**
   Real-index differences from `HEAD`.
   */
  staged: readonly StagedDifference[];
  /**
   Leftover `refs/cli-git/` refs,
   transaction directories,
   and lock files.
   */
  leftovers: readonly string[];
  /**
   `git fsck --strict` result.
   */
  fsck: Readonly<{
    exitCode: number;
    output: string
  }>;
  /**
   Scenario-specific expectations of the accepted design.
   */
  expectations: readonly Readonly<{
    name: string;
    holds: boolean;
    detail: string
  }>[];
}>;

/**
 One violated invariant.
 */
export type Violation = Readonly<{
  /**
   Invariant name.
   */
  invariant: string;
  /**
   Attempt,
   command,
   or path concerned.
   */
  subject: string;
  /**
   What was observed.
   */
  detail: string;
}>;

//endregion Repository
