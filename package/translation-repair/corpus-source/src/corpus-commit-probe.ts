import spawn, { SubprocessError, } from 'nano-spawn';

import {
  CORPUS_GIT_FLAGS,
  corpusGitEnvironment,
  gitSearchCeiling,
} from './corpus-git-context.ts';

//region Corpus commit probe
// What a clone says of the pinned commit after a read of it failed, which
// decides the kind of a failure whose git text is ambiguous. Split out of
// `corpus-source.ts`, which keeps to its line budget and builds the refusal
// from the answer.

/**
 What a probe of the pinned commit found, which decides the kind of a failure
 whose git text is ambiguous.

 @example
 ```ts
 const state: CorpusCommitState = 'held';
 ```
 */
export type CorpusCommitState =
  /**
   The clone holds the commit (`rev-parse` exit status 0).
   */
  | 'held'
  /**
   The clone answered and the revision names no commit in it (exit status 1).
   */
    | 'lacking'
  /**
   Git could not open the clone as a repository: the probe of the commit and
   the probe of the clone's git directory both ended at exit status 128.
   */
    | 'unopened'
  /**
   The probe got no answer: git was stopped by a signal, never started, ended
   with a status this probe does not read, or died (exit status 128) in a
   clone whose git directory it could name, as it does on a commit object it
   cannot inflate.
   */
    | 'unasked';

/**
 What the probe of the pinned commit came to: its answer, or that it got none
 and the failures that left it without one, which the refusal keeps beside the
 read's own.

 @example
 ```ts
 const probe: CommitProbe = { state: 'held', };
 ```
 */
type CommitProbe = {
  /**
   The answer git's exit status gave.
   */
  readonly state: Exclude<CorpusCommitState, 'unasked'>;
} | {
  /**
   No status the probe reads.
   */
  readonly state: 'unasked';

  /**
   What the probe's children failed with, in the order they ran: a signal, a
   refused spawn, a status other than 0, 1 and 128, or a 128 in a clone whose
   git directory the second probe named.
   */
  readonly failures: readonly SubprocessError[];
};

/**
 The clone and commit a probe asks about, as the failed read named them.

 @example
 ```ts
 const pin: ProbedPin = { cloneDir: '/scratch/clone', commitSha: 'a41fc607ea5a70d8a7625cc67d5ed8c444f53379', };
 ```
 */
type ProbedPin = {
  /**
   Clone directory the failed read ran git in.
   */
  readonly cloneDir: string;

  /**
   Commit the failed read resolved against.
   */
  readonly commitSha: string;
};

/**
 Exit status of `git rev-parse --verify --quiet` when the clone answered and
 the revision names no commit in it.
 */
const REV_PARSE_LACKING = 1;

/**
 Exit status git ends with when it dies: at a directory it cannot open as a
 repository, and in a clone it opened, on an object it cannot inflate
 (measured with git 2.55), so a second probe asks which.
 */
const GIT_FATAL = 128;

/**
 What asking a clone for its git directory found.

 @example
 ```ts
 const answer: GitDirAnswer = { kind: 'named', };
 ```
 */
type GitDirAnswer =
  | {
    /**
     Git opened the clone and named its git directory.
     */
    readonly kind: 'named';
  }
    | {
    /**
     Git could not open the clone (exit status 128).
     */
      readonly kind: 'unopened';
    }
    | {
    /**
     Git ended another way, which answers nothing.
     */
      readonly kind: 'failed';

    /**
     What the child failed with, kept beside the commit probe's own.
     */
      readonly failure: SubprocessError;
    };

/**
 Asks a clone for its git directory, after the probe of its commit ended at
 exit status 128, to tell a clone git cannot open from one it opened and died
 in.

 MEASURED with git 2.55: `rev-parse --git-dir` exits 0 in a clone whose
 commit object is corrupt, where `show`, `ls-tree` and the probe of the
 commit all exit 128 with `inflate: data stream error`, and exits 128 at a
 missing directory, at a path through a file, at a plain directory, and at a
 plain directory inside another repository that the ceiling stops git short
 of.

 @param pin - clone the failed read ran in, which this probe asks about

 @param gitPath - git binary the failed read ran

 @param searchCeiling - where the failed read's search for a repository
 stopped, so this probe searches no higher than the read did

 @returns Whether git named the clone's git directory, could not open the
 clone, or failed another way, with that failure

 @throws Error naming the broken invariant where nano-spawn's preparation of
 the call throws, which preparing the probe of the commit already did without
 a throw

 @example
 ```ts
 const gitDir = await probeGitDir({ pin, gitPath, searchCeiling, },);
 ```
 */
async function probeGitDir(
  {
    pin,
    gitPath,
    searchCeiling,
  }: {
    readonly pin: ProbedPin;
    readonly gitPath: string;
    readonly searchCeiling: string;
  },
): Promise<GitDirAnswer> {
  try {
    await spawn(
      gitPath,
      [
        ...CORPUS_GIT_FLAGS,
        '-C',
        pin.cloneDir,
        'rev-parse',
        '--git-dir',
      ],
      { env: corpusGitEnvironment({ searchCeiling, },), },
    );
    return { kind: 'named', };
  }
  catch (error) {
    // nano-spawn throws raw only while preparing a call: reading the working
    // directory, which the probe of the commit read just before and node keeps
    // until a `process.chdir` this package never makes, and naming the command
    // parts, which that probe named too (`getOptions`, `getContext`). The
    // environment handed it is built by code that throws nothing.
    if (!(error instanceof SubprocessError))
      throw new Error(
        'unreachable: preparing the probe of the clone\'s git directory threw before git ran, though preparing the probe of its commit, with the same working directory and command parts, did not',
        { cause: error, },
      );
    if (error.exitCode === GIT_FATAL)
      return { kind: 'unopened', };
    return {
      kind: 'failed',
      failure: error,
    };
  }
}

/**
 Asks the clone whether it holds the pinned commit, after a read of it failed.

 WHY A SECOND GIT CALL: git words a path absent at a held commit and a full
 hash the clone lacks the same (`does not exist in`), so the failed read's own
 text cannot say which it was. `rev-parse --verify --quiet` answers with its
 exit status, which no message language changes: 0 for a commit the clone
 holds, 1 when the clone answered and the revision names no commit in it, and
 128 when git died, all MEASURED against git 2.55. Git dies both where it
 cannot open the clone (a missing directory, a directory that is no
 repository's top, which git meets as no repository since it searches no
 higher than the clone) and in a clone it opened, on a commit object it
 cannot inflate, so a 128 is read by asking the clone for its git directory
 (`probeGitDir`). A child with no exit status (a signal, a spawn the system
 refused) or another status answered nothing, and what it failed with is
 kept, so the refusal says why it is `other`.
 It runs only after a read failed, so a walk pays one more git call per page
 it does not find.

 A FAILURE THAT IS NO SUBPROCESS FAILURE IS NEVER READ AS AN ANSWER: nano-spawn
 throws raw only while preparing the call (a working directory removed under
 this process, a command part that is no string), which is a fault of this
 process and no fact about the clone, so it propagates, and
 `probeKeepingRead` stops the read with it and the read's own failure.

 @param pin - clone and commit the failed read resolved against, the same the
 probe asks about so its answer is about the read that failed

 @param gitPath - git binary the failed read ran, so the probe is answered by
 the same git

 @returns Answer the exit status gave, or that none came and the failures
 that left the probe without one; a probe that ends any way but exit status
 0, 1 or 128 is never read as the commit being held or absent

 @throws Whatever nano-spawn's preparation of the probe of the commit throws,
 unchanged, and an Error naming the broken invariant where only the
 preparation of the probe of the git directory throws

 @example
 ```ts
 const probe = await probeCommit({ pin, gitPath, },);
 ```
 */
async function probeCommit(
  {
    pin,
    gitPath,
  }: {
    readonly pin: ProbedPin;
    readonly gitPath: string;
  },
): Promise<CommitProbe> {
  /**
   Where git's search for a repository stops, the same the failed read used.
   */
  const searchCeiling = await gitSearchCeiling({ cloneDir: pin.cloneDir, },);
  try {
    await spawn(
      gitPath,
      [
        ...CORPUS_GIT_FLAGS,
        '-C',
        pin.cloneDir,
        'rev-parse',
        '--verify',
        '--quiet',
        `${pin.commitSha}^{commit}`,
      ],
      { env: corpusGitEnvironment({ searchCeiling, },), },
    );
    return { state: 'held', };
  }
  catch (error) {
    if (!(error instanceof SubprocessError))
      throw error;
    // The exit status is the whole answer; git's words are not read here.
    if (error.exitCode === REV_PARSE_LACKING)
      return { state: 'lacking', };
    if (error.exitCode !== GIT_FATAL) {
      return {
        state: 'unasked',
        failures: [error,],
      };
    }

    /**
     What the clone said of its own git directory, which tells a clone git
     cannot open from one it opened and died in.
     */
    const gitDir = await probeGitDir({
      pin,
      gitPath,
      searchCeiling,
    },);
    if (gitDir.kind === 'unopened')
      return { state: 'unopened', };
    // Git opened the clone and died on the commit: nothing says whether it
    // holds that commit, so the probe has no answer.
    return {
      state: 'unasked',
      failures: (gitDir.kind === 'failed')
        ? [
          error,
          gitDir.failure,
        ]
        : [error,],
    };
  }
}

/**
 Probes the pinned commit after a read of it failed, keeping the read's
 failure beside the probe's own where the probe fails in this process.

 @param pin - clone and commit the failed read resolved against

 @param gitPath - git binary the failed read ran, which the probe runs too

 @param cause - failure the read raised, which must reach the caller however
 the probe ends

 @returns What the probe of the pinned commit came to

 @throws AggregateError holding the read's failure and then what the probe's
 preparation threw, when nano-spawn threw before any git child ran

 @example
 ```ts
 const probe = await probeKeepingRead({ pin, gitPath, cause: error, },);
 ```
 */
export async function probeKeepingRead(
  {
    pin,
    gitPath,
    cause,
  }: {
    readonly pin: ProbedPin;
    readonly gitPath: string;
    readonly cause: unknown;
  },
): Promise<CommitProbe> {
  try {
    return await probeCommit({
      pin,
      gitPath,
    },);
  }
  catch (probeFailure) {
    throw new AggregateError(
      [
        cause,
        probeFailure,
      ],
      'the corpus read failed, and the probe of its commit failed in this process before git ran',
      { cause: probeFailure, },
    );
  }
}

//endregion Corpus commit probe
