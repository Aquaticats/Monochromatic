/**
 The questions `native:install` asks one checkout with Git: what a commit of the lockfile would record, what the last commit
 already holds, and what the installed Git says it is.

 These answer and print nothing, so the command keeps its own order and its own output; the steps it runs and the texts it
 reads back are in `native-install-steps.ts`.
 */
import {
  gitDeadlineMilliseconds,
  gitProgram,
  promptlessGitSettings,
} from './native-gate-paths.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  runGateCommand,
  runGateCommandOrThrow,
} from './native-gate-run.ts';
import { lockFileName } from './native-install-steps.ts';

/**
 The environment every git query of an install runs with, so no query stops at a password prompt.
 */
const gitEnvironment: NodeJS.ProcessEnv = {
  ...process.env,
  ...promptlessGitSettings,
};

/**
 What one checkout records about the lockfile, and whether it has a commit to record it in.
 */
export type LockRecord = {
  /**
   Whether the checkout points at a commit, which is what the difference is taken against.
   */
  readonly hasCommit: boolean;
  /**
   What a commit of the lockfile would record now, empty when the checkout already records that text.
   */
  readonly diff: string;
};

/**
 Runs one git query of an install against one checkout, bounded and promptless.

 @param repository - the checkout to ask

 @param args - the arguments after the `-C <checkout>` pair

 @returns what git printed on standard output

 @throws NativeGateError when git cannot answer

 @example
 ```ts
 await askGit({ repository: '/repo', args: ['diff', '--', 'toolchain.lock.json'] });
 ```
 */
async function askGit({
  repository,
  args,
}: {
  readonly repository: string;
  readonly args: readonly string[];
}): Promise<string> {
  return (await runGateCommandOrThrow({
    program: gitProgram,
    args: [
      '-C',
      repository,
      ...args,
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
  })).stdout;
}

/**
 Whether the checkout points at a commit at all.

 An install is asked for any checkout by `--repository`, and a checkout with no commit yet is one Git refuses `HEAD` as a
 revision in: `fatal: bad revision 'HEAD'`, status 128. Whether a commit of the lockfile would record anything depends on
 this, so it is asked before the difference, and a query that cannot answer is named rather than left as a raw Git error.

 @param repository - the checkout to ask

 @returns true when `HEAD` names a commit

 @throws NativeGateError when git could not answer

 @example
 ```ts
 await checkoutHasCommit({ repository: '/repo' });
 ```
 */
async function checkoutHasCommit({ repository }: { readonly repository: string; }): Promise<boolean> {
  /**
   The query this makes, which its refusal names whole.
   */
  const query = [
    '-C',
    repository,
    'rev-parse',
    '--verify',
    '--quiet',
    'HEAD',
  ];
  /**
   How the query ended, whose status is 1 with no output when the checkout has no commit.
   */
  const answered = await runGateCommand({
    program: gitProgram,
    args: query,
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
  });
  if (answered.timedOut)
    throw new NativeGateError(`${gitProgram} ${query.join(' ')} was still running after ${String(gitDeadlineMilliseconds)} ms, the bound it is given, so it was stopped and told nothing.`);
  if (answered.problem !== undefined)
    throw new NativeGateError(`${gitProgram} ${query.join(' ')} could not be run: ${answered.problem}`);
  return answered.status === 0;
}

/**
 The lockfile's difference from what the checkout already records, which is what a commit of that path would record.

 Compared with the last commit when the checkout points at one, and with the index when it has no commit yet, because Git
 refuses `HEAD` as a revision there; on a checkout with no commit the index is all a first commit reads, so a lockfile a
 step staged there is recorded and one nothing staged is left alone.

 Asked again right before the commit, because the worktree can be written between the install of the tools and the
 commit: another install of the same release then finds nothing to record, and a lockfile another process changed is
 caught before it is committed.

 @param repository - the checkout to ask

 @returns the difference and whether the checkout had a commit to take it against

 @throws NativeGateError when git cannot answer

 @example
 ```ts
 await lockRecordOf({ repository: '/repo' });
 ```
 */
export async function lockRecordOf({ repository }: { readonly repository: string; }): Promise<LockRecord> {
  /**
   Whether the checkout has a commit, which decides what the difference is taken against.
   */
  const hasCommit = await checkoutHasCommit({ repository, });
  return {
    hasCommit,
    diff: await askGit({
      repository,
      args: [
        'diff',
        ...(hasCommit ? ['HEAD',] : []),
        '--',
        lockFileName,
      ],
    }),
  };
}

/**
 Whether the last commit holds exactly this text for one path.

 Asked after a commit step failed, to tell a commit that lost the record to another install from one that failed on its
 own: only the first finds the text this install resolved in `HEAD` already.

 @param repository - the checkout to ask

 @param name - the file's name relative to the checkout

 @param text - the text the answer is compared with

 @returns true when `HEAD` holds exactly that text for the path

 @example
 ```ts
 await lastCommitHoldsFile({ repository: '/repo', name: 'toolchain.lock.json', text: installedLock });
 ```
 */
export async function lastCommitHoldsFile({
  repository,
  name,
  text,
}: {
  readonly repository: string;
  readonly name: string;
  readonly text: string;
}): Promise<boolean> {
  /**
   How the query ended, which answers status 128 when the commit or the path is not there.
   */
  const answered = await runGateCommand({
    program: gitProgram,
    args: [
      '-C',
      repository,
      'show',
      `HEAD:${name}`,
    ],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
  });
  return (answered.status === 0) && (answered.stdout === text);
}

/**
 What the Git the checkout finds says it is, which is how an install reads the release it installed.

 @param repository - the checkout to ask

 @returns what `git --version` printed

 @throws NativeGateError when git cannot answer

 @example
 ```ts
 await gitVersionLineOf({ repository: '/repo' });
 ```
 */
export async function gitVersionLineOf({ repository }: { readonly repository: string; }): Promise<string> {
  return (await runGateCommandOrThrow({
    program: gitProgram,
    args: ['--version',],
    deadlineMilliseconds: gitDeadlineMilliseconds,
    environment: gitEnvironment,
    workingDirectory: repository,
  })).stdout;
}
