/**
 The Git a test fixture builds and reads its own checkout with: the system Git, never this repository's wrapper.

 A fixture is a throwaway checkout, so the Git that makes it is the one pacman installed, not the wrapper this repository
 puts first on `PATH`. A wrapper commit is landed through the commit service of the checkout it runs in
 (`src/native/landing/service_route.rs`), which the wrapper asks the account's user service manager to start, so a fixture
 committed through `PATH` would put a unit of a throwaway repository into a manager people and other tests share, and would
 fail on a machine where no manager answers. Reading the wrapper's output instead of Git's also hides which program
 answered. What the command under test runs stays what it runs in real use: this helper is for the fixture only.
 */
import { runGateCommandOrThrow } from './native-gate-run.ts';

/**
 Milliseconds one git command of a fixture is given, which is far more than a throwaway checkout needs.
 */
const fixtureGitDeadlineMilliseconds = 120_000;

/**
 The system Git, by its own path rather than as a word `PATH` is searched for.

 A search order is what a wrapper install changes: this repository puts the wrapper first on `PATH`
 (`native-gate-paths.ts`, `gitProgram`), and the fixture asks for the program a person's package manager installed
 (`/usr/bin/git`, the path the package's benchmark and end-to-end fixtures name).

 It is exported because the controls must ask the same program what it reads from its environment
 (`fixture-git-that-is-never-the-wrapper-for-tests.unit.test.ts`), rather than naming a path of their own that a later
 change here could leave behind.
 */
export const fixtureGitProgram = '/usr/bin/git';

/**
 The directories a fixture's git commands find the programs Git itself starts in, which are the system ones and never a
 directory a wrapper is installed in.
 */
const systemGitPath = '/usr/bin:/bin';

/**
 The variables a fixture's git commands run with, built here rather than taken from the caller's environment.

 Git reads its repository, its index, its object store, its configuration, its identity, its dates and the programs it
 starts from the environment, so an environment passed whole to a fixture command lets the caller move the command into
 another checkout, redirect its objects or its index, replace its identity or fail its commit with a date Git cannot
 read. A test process started from a Git hook, a rebase or an editor has exactly those variables set, and the folders it
 names are not the fixture's. A list of the names to remove cannot be complete, because the next Git release may read a
 further one, so this builds the environment instead: what the system Git needs to run, and nothing of the caller's.

 What Git needs for a local fixture, measured with nothing else set on this host: `PATH` for the programs Git starts,
 which are the system ones and never a directory a wrapper is installed in; the two switches that read no configuration
 outside the fixture, so neither `~/.gitconfig` nor the system's is consulted and no `HOME` is needed; the two that stop
 a prompt, because nothing here may wait for a person; and the locale, so the sentences a failing control quotes are the
 English ones. The commit identity, the dates and the repository itself come from the fixture's own arguments and folder.

 A future caller that needs a variable of the caller's environment has to add it here, where a reader sees what a
 fixture command depends on.

 @returns the environment of one fixture git command
 */
function fixtureGitEnvironment(): NodeJS.ProcessEnv {
  return {
    PATH: systemGitPath,
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
    GIT_TERMINAL_PROMPT: '0',
    GIT_ASKPASS: '',
    LANG: 'C',
    LC_ALL: 'C',
  };
}

/**
 Runs one git command in a fixture checkout with the system Git and answers what it printed.

 @param directory - the checkout to run in

 @param args - the arguments after the `-C <checkout>` pair

 @returns what git printed on standard output

 @throws NativeGateError naming the step and the status or deadline when the system Git refused or ran past its bound

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
    program: fixtureGitProgram,
    args: [
      '-C',
      directory,
      ...args,
    ],
    deadlineMilliseconds: fixtureGitDeadlineMilliseconds,
    environment: fixtureGitEnvironment(),
  })).stdout;
}
