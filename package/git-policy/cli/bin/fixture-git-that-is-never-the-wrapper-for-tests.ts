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
 The directories a fixture's git commands find their Git in, which are the system ones and never this repository's wrapper.
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
