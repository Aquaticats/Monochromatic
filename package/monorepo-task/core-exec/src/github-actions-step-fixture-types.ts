/**
 The shapes a disposable step runner takes and gives back: the answers of the stand-in `curl`, what a run leaves behind,
 and the runner itself.

 @module
 */

//region Requests and results

/**
 One answer of the stand-in `curl`: the first rule whose `urlIncludes` text the address contains wins.
 */
export type CurlRule = {
  /**
   Text the address must contain.
   */
  readonly urlIncludes: string;
  /**
   HTTP status to answer with.
   */
  readonly status: number;
  /**
   Body to write to `--output`.
   */
  readonly body?: string;
  /**
   File whose bytes are the body, for an archive.
   */
  readonly bodyFile?: string;
  /**
   Exit status for a transport failure; the answer's `status` is then not used.
   */
  readonly exit?: number;
};

/**
 What one step run left behind.
 */
export type StepRun = {
  /**
   Exit status of the command.
   */
  readonly status: number;
  /**
   Standard output.
   */
  readonly stdout: string;
  /**
   Standard error.
   */
  readonly stderr: string;
  /**
   Text appended to `GITHUB_OUTPUT`.
   */
  readonly githubOutput: string;
  /**
   Text appended to `GITHUB_ENV`.
   */
  readonly githubEnv: string;
  /**
   Text appended to `GITHUB_PATH`.
   */
  readonly githubPath: string;
  /**
   Text appended to `GITHUB_STEP_SUMMARY`.
   */
  readonly githubSummary: string;
};

/**
 A disposable runner.
 */
export type StepFixture = {
  /**
   The checkout the step runs in.
   */
  readonly checkout: string;
  /**
   The runner temporary folder (`RUNNER_TEMP`).
   */
  readonly runnerTemp: string;
  /**
   Folder that holds the stand-in programs and what they recorded.
   */
  readonly records: string;
  /**
   Folder of the fake release files that the stand-in `curl` serves.
   */
  readonly served: string;
  /**
   Everything under this fixture that can hold a record, for a search for a secret.
   */
  readonly root: string;
  /**
   Sets what the stand-in `curl` answers.
   */
  readonly answerCurl: (rules: readonly CurlRule[],) => Promise<void>;
  /**
   Writes `toolchain.lock.json` into the checkout.
   */
  readonly writeLock: (lock: unknown,) => Promise<void>;
  /**
   Packs a gzip archive of fake programs under one top folder into the served folder.
   */
  readonly makeArchive: (request: ArchiveRequest,) => Promise<ArchiveResult>;
  /**
   Writes a stand-in program into the folder that comes first on `PATH`.
   */
  readonly writeProgram: (request: ProgramFile,) => Promise<void>;
  /**
   Runs a command in the checkout with the runner's files and the stand-ins on `PATH`.
   */
  readonly run: (request: RunRequest,) => Promise<StepRun>;
  /**
   Removes the fixture when a `await using` scope ends.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 What to pack into an archive.
 */
export type ArchiveRequest = {
  /**
   File name of the archive, such as `tool-1.0-target.tar.gz`.
   */
  readonly name: string;
  /**
   The one folder every member sits in, which `--strip-components=1` removes; empty puts the members at the archive's root.
   */
  readonly top: string;
  /**
   Member names under the top folder, each with the Node script source it holds.
   */
  readonly members: Readonly<Record<string, string>>;
};

/**
 A packed archive.
 */
export type ArchiveResult = {
  /**
   Where it was written.
   */
  readonly path: string;
  /**
   Its SHA-256 as 64 lowercase hex digits.
   */
  readonly sha256: string;
};

/**
 A stand-in program to write.
 */
export type ProgramFile = {
  /**
   File name on `PATH`.
   */
  readonly name: string;
  /**
   Node script source, without the interpreter line.
   */
  readonly source: string;
};

/**
 What to run.
 */
export type RunRequest = {
  /**
   The program and its arguments.
   */
  readonly command: readonly string[];
  /**
   Environment variables of the step, added to the runner's own.
   */
  readonly env: Readonly<Record<string, string>>;
  /**
   Folder to run in; the checkout when left out.
   */
  readonly cwd?: string;
};

//endregion Requests and results
