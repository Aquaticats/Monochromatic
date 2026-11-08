/**
 Throwaway Git repository used as the real Pi session's workspace.
 
 The repository is created with plain `git` from PATH,
 so the fixture never spells the real Git binary itself.
 
 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Constants

/**
 Name of the fixture file whose staged content the guarded session tries to commit.
 */
const FIXTURE_FILE_NAME = 'a.txt';

/**
 Fixture file content, so a status read has something to show.
 */
const FIXTURE_FILE_CONTENT = 'guardrail fixture\n';

/**
 Commit identity kept in the throwaway repository's own config.
 */
const FIXTURE_IDENTITY = {
  name: 'pi guardrail verifier',
  email: 'pi-guardrail-verifier@example.invalid',
} as const;

/**
 Exit status of a Git invocation that succeeded.
 */
const GIT_SUCCESS_STATUS = 0;

//endregion Constants

//region Types

/**
 Result of one Git invocation, including a refusal instead of a throw.
 */
type GitOutcome = {
  /**
   Standard output, trimmed.
   */
  readonly standardOutput: string;
  /**
   Standard error, trimmed.
   */
  readonly standardError: string;
  /**
   Whether Git exited with a non-zero status.
   */
  readonly failed: boolean;
};

/**
 Throwaway repository the guarded session works in.
 */
type ThrowawayRepository = {
  /**
   Repository root, which is also the session's working directory.
   */
  readonly root: string;
  /**
   Staged file name, so assertions name what they checked.
   */
  readonly stagedFileName: string;
};

/**
 One Git invocation's words and where they run.
 */
type GitInvocation = {
  /**
   Directory Git runs in.
   */
  readonly root: string;
  /**
   Git words after the executable.
   */
  readonly args: readonly string[];
};

//endregion Types

//region Git invocation

/**
 Reads one child stream to its end.
 
 @param stream - child stream to drain
 
 @returns everything the stream carried
 
 @example
 ```typescript
 const text = await readStream(child.stdout);
 ```
 */
async function readStream(stream: NodeJS.ReadableStream,): Promise<string> {
  stream.setEncoding('utf8',);
  /**
   Pieces the stream carried, joined once at the end.
   */
  const pieces: string[] = [];
  for await (const piece of stream)
    pieces.push(String(piece,));
  return pieces.join('',);
}

/**
 Runs Git from PATH and reports a refusal instead of throwing.
 
 @param root - directory Git runs in
 
 @param args - Git words after the executable
 
 @returns Git's output and whether it failed
 
 @throws {@link GitLaunchError} when Git cannot be started at all
 
 @example
 ```typescript
 await runGit({ root: '/repo', args: ['status', '--short'] });
 ```
 */
async function runGit({
  root,
  args,
}: GitInvocation,): Promise<GitOutcome> {
  /**
   Git process under test.
   */
  const child = spawn(
    'git',
    [...args,],
    {
      cwd: root,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  try {
    /**
     Output of both streams, awaited with the close event so neither stream is left behind.
     */
    const [standardOutput, standardError,] = await Promise.all([
      readStream(nonNullishOrThrow(child.stdout,),),
      readStream(nonNullishOrThrow(child.stderr,),),
      once(
        child,
        'close',
      ),
    ],);
    return {
      standardOutput: standardOutput
        .trim(),
      standardError: standardError
        .trim(),
      failed: child.exitCode !== GIT_SUCCESS_STATUS,
    };
  }
  catch (error: unknown) {
    throw new GitLaunchError({
      args,
      cause: error,
    },);
  }
}

/**
 Runs Git from PATH and throws when it refuses.
 
 @param root - directory Git runs in
 
 @param args - Git words after the executable
 
 @returns Git's standard output
 
 @throws {@link GitRefusedError} when Git exits non-zero
 
 @example
 ```typescript
 await requireGit({ root: '/repo', args: ['add', 'a.txt'] });
 ```
 */
async function requireGit({
  root,
  args,
}: GitInvocation,): Promise<string> {
  /**
   Outcome of the required invocation.
   */
  const outcome = await runGit({
    root,
    args,
  },);
  if (outcome.failed) {
    throw new GitRefusedError({
      args,
      standardError: outcome.standardError,
    },);
  }
  return outcome.standardOutput;
}

/**
 Git invocation the fixture needed to succeed.
 */
class GitRefusedError extends Error {
  /**
   Name printed with the message.
   */
  override readonly name = 'GitRefusedError';

  /**
   Records the refused invocation.
   
   @param args - Git words that were refused
   
   @param standardError - what Git said
   */
  constructor({
    args,
    standardError,
  }: {
    readonly args: readonly string[];
    readonly standardError: string;
  },) {
    super(`git ${args.join(' ',)} was refused: ${standardError}`,);
  }
}

/**
 Git could not be started at all, which is not a refusal.
 */
class GitLaunchError extends Error {
  /**
   Name printed with the message.
   */
  override readonly name = 'GitLaunchError';

  /**
   Records the failed launch.
   
   @param args - Git words that were launched
   
   @param cause - what the launch threw
   */
  constructor({
    args,
    cause,
  }: {
    readonly args: readonly string[];
    readonly cause: unknown;
  },) {
    super(
      `git ${args.join(' ',)} could not be started: ${String(cause,)}`,
      { cause, },
    );
  }
}

//endregion Git invocation

//region Fixture construction

/**
 Creates a repository with one staged file and no commit yet.
 
 A commit is therefore possible but absent,
 so a guarded commit attempt leaves an observable difference and an allowed one lands.
 
 @param root - empty directory to initialize
 
 @returns the throwaway repository
 
 @throws {@link GitRefusedError} when fixture setup is refused
 
 @example
 ```typescript
 const repository = await createThrowawayRepository({ root: join(scratch, 'repo') });
 ```
 */
async function createThrowawayRepository({
  root,
}: {
  readonly root: string;
},): Promise<ThrowawayRepository> {
  await mkdir(
    root,
    { recursive: true, },
  );
  await requireGit({
    root,
    args: [
      'init',
      '--quiet',
    ],
  },);
  await requireGit({
    root,
    args: [
      'config',
      'user.name',
      FIXTURE_IDENTITY.name,
    ],
  },);
  await requireGit({
    root,
    args: [
      'config',
      'user.email',
      FIXTURE_IDENTITY.email,
    ],
  },);
  /**
   Fixture file path inside the throwaway repository.
   */
  const fixtureFilePath = join(
    root,
    FIXTURE_FILE_NAME,
  );
  await writeFile(
    fixtureFilePath,
    FIXTURE_FILE_CONTENT,
  );
  await requireGit({
    root,
    args: [
      'add',
      FIXTURE_FILE_NAME,
    ],
  },);
  return {
    root,
    stagedFileName: FIXTURE_FILE_NAME,
  };
}

//endregion Fixture construction

//region Fixture reads

/**
 Reads the commit subjects on the current branch.
 
 @param repository - throwaway repository
 
 @returns commit subjects, oldest first, empty while no commit exists
 
 @example
 ```typescript
 const subjects = await commitSubjects({ repository });
 ```
 */
async function commitSubjects({
  repository,
}: {
  readonly repository: ThrowawayRepository;
},): Promise<readonly string[]> {
  /**
   Log read of the current branch.
   */
  const outcome = await runGit({
    root: repository.root,
    args: [
      'log',
      '--format=%s',
    ],
  },);
  if (outcome.failed)
    return [];
  if (outcome.standardOutput === '')
    return [];
  return outcome.standardOutput
    .split('\n',);
}

/**
 Reports whether the repository still has no commit.
 
 @param repository - throwaway repository
 
 @returns whether `HEAD` names no commit
 
 @example
 ```typescript
 if (await hasNoCommit({ repository })) throw new Error('the guarded commit landed');
 ```
 */
async function hasNoCommit({
  repository,
}: {
  readonly repository: ThrowawayRepository;
},): Promise<boolean> {
  /**
   Head resolution that fails exactly while the branch is unborn.
   */
  const outcome = await runGit({
    root: repository.root,
    args: [
      'rev-parse',
      '--verify',
      'HEAD',
    ],
  },);
  return outcome.failed;
}

//endregion Fixture reads

export {
  commitSubjects,
  createThrowawayRepository,
  hasNoCommit,
};
export type {
  GitOutcome,
  ThrowawayRepository,
};
