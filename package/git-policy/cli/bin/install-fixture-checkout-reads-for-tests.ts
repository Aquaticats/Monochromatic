/**
 Reads of one install fixture's checkout: what a step wrote, what a commit holds, and what is left staged.

 These are the questions a test asks after an install ran, kept beside the fixture that makes the checkout
 (`stand-in-install-programs-for-tests.ts`) and apart from it, so the stand-in programs and the fixture stay one file of
 their own.
 */
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';

import { runGateCommandOrThrow } from './native-gate-run.ts';

/**
 Milliseconds one git command of a fixture is given, which is far more than a throwaway checkout needs.
 */
const fixtureGitDeadlineMilliseconds = 120_000;

/**
 The directories a fixture's git commands find their Git in, which are the system ones and never this repository's wrapper.

 A fixture is a throwaway checkout, so its commits want the gate's Git, pacman's current one, and not the wrapper this
 repository puts first on `PATH`: the wrapper routes a commit through the service of the checkout it runs in, and a
 fixture must not need a service to test a command. Reading the wrapper's output instead of Git's also hid which program
 answered.
 */
const systemGitPath = '/usr/bin:/bin';

/**
 The variables a fixture's git commands run with, so no user configuration decides what a fixture's commit does.

 @returns the environment of one fixture git command
 */
function fixtureGitEnvironment(): NodeJS.ProcessEnv {
  return {
    ...process.env,
    PATH: systemGitPath,
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
  };
}

/**
 Runs one git command in a fixture checkout and answers what it printed.

 @param directory - the checkout to run in

 @param args - the arguments after the `-C <checkout>` pair

 @returns what git printed on standard output

 @example
 ```ts
 await fixtureGit({ directory: fixture.repository, args: ['log', '--format=%s'] });
 ```
 */
export async function fixtureGit({
  directory,
  args,
}: {
  readonly directory: string;
  readonly args: readonly string[];
}): Promise<string> {
  return (await runGateCommandOrThrow({
    program: 'git',
    args: [
      '-C',
      directory,
      ...args,
    ],
    deadlineMilliseconds: fixtureGitDeadlineMilliseconds,
    environment: fixtureGitEnvironment(),
  })).stdout;
}

/**
 The commits one fixture checkout holds, newest first.

 @param repository - the checkout to read

 @returns every commit's subject

 @example
 ```ts
 await commitSubjects({ repository: fixture.repository });
 ```
 */
export async function commitSubjects({ repository }: { readonly repository: string; }): Promise<readonly string[]> {
  return (await fixtureGit({
    directory: repository,
    args: [
      'log',
      '--format=%s',
    ],
  })).split('\n')
    .filter(function hasText(line) {
      return line !== '';
    });
}

/**
 The text of one file as the last commit holds it, which is what a release commit of the lockfile has to name.

 @param repository - the checkout to read

 @param name - the file's name relative to the checkout

 @returns its text in `HEAD`

 @example
 ```ts
 await committedFileText({ repository: fixture.repository, name: 'toolchain.lock.json' });
 ```
 */
export async function committedFileText({
  repository,
  name,
}: {
  readonly repository: string;
  readonly name: string;
}): Promise<string> {
  return await fixtureGit({
    directory: repository,
    args: [
      'show',
      `HEAD:${name}`,
    ],
  });
}

/**
 The paths a commit made now would record from the index, which is how a test reads that nothing is left staged.

 @param repository - the checkout to read

 @returns the path names `git diff --cached` names, empty when nothing is staged

 @example
 ```ts
 await stagedPathNames({ repository: fixture.repository });
 ```
 */
export async function stagedPathNames({ repository }: { readonly repository: string; }): Promise<readonly string[]> {
  return (await fixtureGit({
    directory: repository,
    args: [
      'diff',
      '--cached',
      '--name-only',
    ],
  })).split('\n')
    .filter(function hasText(line) {
      return line !== '';
    });
}

/**
 The text of one file of a fixture checkout, which is how a test reads what a step wrote.

 @param repository - the checkout to read

 @param name - the file's name relative to the checkout

 @returns its text

 @example
 ```ts
 await fileTextOf({ repository: fixture.repository, name: 'toolchain.lock.json' });
 ```
 */
export async function fileTextOf({
  repository,
  name,
}: {
  readonly repository: string;
  readonly name: string;
}): Promise<string> {
  return await readFile(
    join(
      repository,
      name
    ),
    'utf8'
  );
}
