/**
 Types of the disposable bypass integration fixture:
 the namespace its commands run in,
 the results those commands return,
 and the staged copy of the built CLI a test runs.

 @module
 */

/**
 Result from allowed-failure integration command.
 */
export type FixtureCommandResult = {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Persisted state fields consumed by integration checks.
 */
export type FixtureBypassState = {
  readonly preference: number;
  readonly table: number;

  /**
   The routes the state records,
   which `down` deletes.
   */
  readonly routes: readonly {
    readonly proto: string;
    readonly tokens: readonly string[];
  }[];
};

/**
 Disposable network namespace fixture.
 */
export type BypassFixture = {
  readonly namespace: string;
  readonly hostPrimary: string;
  readonly hostSecondary: string;
  readonly peerPrimary: string;
  readonly peerSecondary: string;
  readonly stateDirectory: string;
  readonly statePath: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Staged copy of the built CLI paired with a harmless watcher.
 */
export type StagedCli = {
  /**
   Bundle entry point to run.
   */
  readonly cliPath: string;

  /**
   Removes the staged copy.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};
