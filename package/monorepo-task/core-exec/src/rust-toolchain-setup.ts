#!/usr/bin/env node

/**
 The step of `.github/actions/rust-toolchain-setup`: installs the Rust toolchain `toolchain.lock.json` records and makes
 it the toolchain the rest of the job compiles with.

 Every workflow of this repository that compiles Rust calls that action, so no job installs a compiler of its own, and
 the daily update that moves `.tools.rust.version` moves every build with it. The owner decided it on 2026-11-15:
 "Everything nightly. [...] And the lock's nightly is always the latest nightly anyway."

 Reads `toolchain.lock.json` from the current directory and appends `RUSTUP_TOOLCHAIN` to the file in `GITHUB_ENV`, which
 is how the later steps of the job come to use this toolchain without naming it. Imports only `node:` modules and sibling
 files that do, because the action runs on a runner that has no dependencies installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/rust-toolchain-setup.ts
 ```
 */

import {
  appendToRunnerFile,
  readJsonFile,
  runProgramOrEnd,
  runStep,
  say,
} from './github-actions-step-support.ts';
import {
  lockedRustToolchainVersion,
  rustupInstallArguments,
  rustupToolchainEnvironmentLine,
} from './locked-rust-toolchain.ts';

//region The whole step

/**
 Installs the toolchain the lock names, hands it to the job through `GITHUB_ENV`, and prints the compiler it got.

 @throws {@link StepExitError} With `rustup`'s own status when an install or a version question fails, and with 1 when
 the runner's environment file is not set

 @example
 ```ts
 await installLockedRustToolchain();
 ```
 */
async function installLockedRustToolchain(): Promise<void> {
  /**
   The parsed lock, read from the checkout the step runs in.
   */
  const lock = await readJsonFile('toolchain.lock.json',);
  /**
   Rust toolchain the lock records.
   */
  const version = lockedRustToolchainVersion({ lock, },);
  say(`rust-toolchain-setup: toolchain.lock.json names ${version}`,);
  await runProgramOrEnd({
    file: 'rustup',
    args: rustupInstallArguments({ version, },),
    cwd: process.cwd(),
  },);
  await appendToRunnerFile({
    variable: 'GITHUB_ENV',
    text: rustupToolchainEnvironmentLine({ version, },),
  },);
  // The compiler every later step of the job gets, saying what it is: its release, its commit, and its host. It is read
  // through `rustup run` because this step's own environment predates the line appended above.
  await runProgramOrEnd({
    file: 'rustup',
    args: [
      'run',
      version,
      'rustc',
      '--version',
      '--verbose',
    ],
    cwd: process.cwd(),
  },);
}

//endregion The whole step

await runStep(installLockedRustToolchain,);
