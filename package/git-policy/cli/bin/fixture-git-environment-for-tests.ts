/**
 The environment a fixture's `git` runs with: the standing Git, no session place of this account, and a runtime directory
 of the fixture's own where the fixture has one to name.

 On this machine the `git` on a run's own `PATH` is the wrapper, which hands a commit to the commit service of the
 checkout it runs in and has this account's user manager start one for that checkout, and which points the shared program
 link at itself as it does. Measured 2026-11-27 (issue 6620): a run of this package's TypeScript unit tests committed in
 four fixture checkouts through the wrapper, the manager started four `monochromatic-git-policy-cli@<instance>` services
 for them, and the link named another release afterwards.

 A released wrapper cannot be kept on that `PATH` and still make a fixture's commits: with no bus address and a runtime
 directory of a fixture it refuses the commit outright ("This computer cannot start the commit service on demand, so this
 commit was not made", measured 2026-11-27 with release `0.6.1-alpha.44`), because the service is the only writer of a
 commit. So a fixture that commits starts the standing Git, which reads neither session place, and it is given neither of
 them either: a wrapper that found its way back onto a fixture's `PATH` would then have no route to this account's
 manager, and would refuse the commit in the open instead of handing it to a service.

 The Rust controls reach the same two places through `foundation/session_runtime_directory_and_bus_address.rs`, which
 reads them only from what a control hands in (`doc/how-to-write-this-packages-code-and-tests.md`, "Nothing a test changes
 may be process-wide").
 */
import { dirname } from 'node:path';

/**
 The directories a fixture's `git` is found in when the fixture names none: the standing Git, and never a folder that
 holds this repository's wrapper.
 */
export const standingGitSearchPath = '/usr/bin:/bin';

/**
 The two variables that name this session's runtime directory and its session-bus address, which no fixture child
 inherits.

 They are the two facts a commit's route to its service is chosen from
 (`src/native/foundation/session_runtime_directory_and_bus_address.rs`), and an inherited pair names this account's
 manager whatever `HOME` or `TMPDIR` a fixture hands its child.
 */
export const sessionPlaceNames: readonly string[] = [
  'DBUS_SESSION_BUS_ADDRESS',
  'XDG_RUNTIME_DIR',
];

/**
 The part of this process's environment a fixture child may inherit: everything but the two places that name this session.

 A fixture hands a child this and never `process.env`, because a child handed the whole environment is handed this
 account's bus address and its runtime directory along with it.

 @returns this process's variables without `DBUS_SESSION_BUS_ADDRESS` or `XDG_RUNTIME_DIR`

 @example
 ```ts
 environmentAFixtureChildMayInherit();
 ```
 */
export function environmentAFixtureChildMayInherit(): NodeJS.ProcessEnv {
  /**
   The variables a fixture child may inherit, gathered as they are read.
   */
  const kept: NodeJS.ProcessEnv = {};
  for (const name of Object.keys(process.env))
    if (!sessionPlaceNames.includes(name))
      kept[name] = process.env[name];
  return kept;
}

/**
 The environment one git command of a fixture runs with.

 A caller that names no runtime directory gives its child none at all, which is the stricter of the two: nothing under a
 runtime directory can be reached, so no service socket and no program link of this account is either.

 @param runtimeDirectory - the fixture's own runtime directory, which the fixture makes; absent when the fixture names none

 @param leadingSearchDirectories - directories to look in before the standing Git's, such as a stand-in program's own

 @returns the variables of the child, with no session place of this account among them

 @example
 ```ts
 fixtureGitEnvironment({ runtimeDirectory: join(fixture, 'runtime') });
 ```
 */
export function fixtureGitEnvironment({
  runtimeDirectory,
  leadingSearchDirectories = [],
}: {
  readonly runtimeDirectory?: string;
  readonly leadingSearchDirectories?: readonly string[];
} = {}): NodeJS.ProcessEnv {
  return {
    ...environmentAFixtureChildMayInherit(),
    // The running Node's own folder is on the path because a stand-in program is a Node program, and the run's own path
    // may name a Node that is not the system's.
    PATH: [
      ...leadingSearchDirectories,
      dirname(process.execPath),
      standingGitSearchPath,
    ].join(':'),
    // No user configuration decides what a fixture's commit does, so a fixture never reads this machine's settings.
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
    ...(runtimeDirectory === undefined ? {} : { XDG_RUNTIME_DIR: runtimeDirectory, }),
  };
}
