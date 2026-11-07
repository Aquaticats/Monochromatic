#!/usr/bin/env node

/**
 The install step of `.github/workflows/catalog-specifiers.yml`: installs the git policy CLI release that
 `toolchain.lock.json` pins, from a prebuilt `cargo-binstall` archive that the same lock pins, and tells the later steps
 which program, command word and settings file that release uses.

 `cargo-binstall` comes from the archive row of `cargo-binstall` in the lock, checked against the SHA-256 recorded there;
 compiling it with `cargo install` costs minutes and bypasses the lock. It is run with `--disable-strategies compile`, so
 a missing release archive fails the step instead of compiling the crate. The release may predate the rename of the
 program (2026-11-04): it then holds `cli-git-native`, answers `git cli-git <command>` and reads `cli-git.config.jsonc`;
 a release after it holds `monochromatic-git-policy-cli` for all three. The later steps use whichever the release has,
 through `CLI_PROGRAM`, `CLI_WORD` and `CLI_SETTINGS` in `GITHUB_ENV`.

 Reads `RUNNER_TEMP`, `GITHUB_PATH` and `GITHUB_ENV` from the environment (and passes `GITHUB_TOKEN` on to
 `cargo-binstall` untouched) and `toolchain.lock.json` from the current directory. Imports only `node:` modules and
 sibling files that do, because the workflow runs it right after checkout, before any dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/catalog-specifiers-install-policy-cli.ts
 ```
 */

import {
  constants,
  copyFile,
  mkdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  appendToRunnerFile,
  isExecutable,
  jsonAt,
  pathFacts,
  rawJsonText,
  readJsonFile,
  requiredEnvironment,
  runProgramOrEnd,
  runStep,
  say,
  StepExitError,
} from './github-actions-step-support.ts';

//region What the lock must hold

/**
 Where the pinned `cargo-binstall` release archives are published; the lock's archive address must start with this.
 */
const BINSTALL_ADDRESS_PREFIX = 'https://github.com/cargo-bins/cargo-binstall/releases/download/v';

/**
 The crate the release is installed from.
 */
const POLICY_CLI_CRATE = 'monochromatic-git-policy-cli';

/**
 Program name of the release after the rename.
 */
const RENAMED_PROGRAM = 'monochromatic-git-policy-cli';

/**
 Program name of a release before the rename.
 */
const EARLIER_PROGRAM = 'cli-git-native';

/**
 Settings file name of a release before the rename.
 */
const EARLIER_SETTINGS = 'cli-git.config.jsonc';

/**
 Whether a one-character text is an ASCII digit.

 @param character - Text to test

 @returns True for `0` to `9`

 @example
 ```ts
 const ok = isDigit('7',);
 ```
 */
function isDigit(character: string,): boolean {
  return (character.length === 1) && (character >= '0')
    && (character <= '9');
}

/**
 Whether text has the shape of a version, as the shell step's pattern `[0-9]*.[0-9]*.[0-9]*` read it: a digit first, then
 two dots after it that each have a digit right behind them.

 @param text - Version text from the lock

 @returns True when the text has that shape

 @example
 ```ts
 const ok = looksLikeVersion('0.6.1-alpha.1',);
 ```
 */
function looksLikeVersion(text: string,): boolean {
  if (!isDigit(text.charAt(0,),))
    return false;
  /**
   Positions after the first character where a dot has a digit right behind it.
   */
  const dotsBeforeDigits = Array.from(text,)
    .filter(function isDotBeforeDigit(
      character: string,
      index: number,
    ) {
      return (index >= 1) && (character === '.')
        && isDigit(text.charAt(index + 1,),);
    },);
  return dotsBeforeDigits.length >= 2;
}

//endregion What the lock must hold

//region Telling the later steps which program was installed

/**
 What the later steps need to run the installed release.
 */
type PolicyCliNames = {
  /**
   Program to run.
   */
  readonly program: string;
  /**
   Command word after the program (`git cli-git` before the rename).
   */
  readonly word: string;
  /**
   Settings file the program reads.
   */
  readonly settings: string;
};

/**
 Names after the rename: one name for all three.
 */
const RENAMED_NAMES: PolicyCliNames = {
  program: RENAMED_PROGRAM,
  word: RENAMED_PROGRAM,
  settings: `${RENAMED_PROGRAM}.config.jsonc`,
};

/**
 Names before the rename.
 */
const EARLIER_NAMES: PolicyCliNames = {
  program: EARLIER_PROGRAM,
  word: 'cli-git',
  settings: EARLIER_SETTINGS,
};

/**
 Tells the later steps which program was installed, by appending to `GITHUB_ENV`, and gives an earlier release the settings
 file name it reads.

 @param names - Names the installed release uses

 @example
 ```ts
 await announce(RENAMED_NAMES,);
 ```
 */
async function announce(names: PolicyCliNames,): Promise<void> {
  await appendToRunnerFile({
    variable: 'GITHUB_ENV',
    text: `CLI_PROGRAM=${names.program}\nCLI_WORD=${names.word}\nCLI_SETTINGS=${names.settings}\n`,
  },);
}

//endregion Telling the later steps which program was installed

//region The whole step

/**
 Reads an entry of the lock as `jq --raw-output` prints it, `null` when it is missing.

 @param lock - Parsed `toolchain.lock.json`

 @param keys - Keys to follow

 @returns The text `jq --raw-output` prints

 @example
 ```ts
 const url = lockText({ lock, keys: ['tools', 'cargo-binstall', 'artifacts', 'linux-x64', 'url',], },);
 ```
 */
function lockText(
  {
    lock,
    keys,
  }: {
    readonly lock: unknown;
    readonly keys: readonly string[];
  },
): string {
  return rawJsonText(jsonAt({
    value: lock,
    keys,
  },),);
}

/**
 Installs the pinned policy CLI and announces its names.

 @throws {@link StepExitError} With the status the shell step would have ended with

 @example
 ```ts
 await installPolicyCli();
 ```
 */
async function installPolicyCli(): Promise<void> {
  /**
   The lock, read once; the shell step read it three times.
   */
  const lock = await readJsonFile('toolchain.lock.json',);
  /**
   Address of the `cargo-binstall` archive for this runner.
   */
  const binstallUrl = lockText({
    lock,
    keys: [
      'tools',
      'cargo-binstall',
      'artifacts',
      'linux-x64',
      'url',
    ],
  },);
  /**
   SHA-256 the lock records for that archive; not looked at until the archive is checked, as in the shell step.
   */
  const binstallSha256 = lockText({
    lock,
    keys: [
      'tools',
      'cargo-binstall',
      'artifacts',
      'linux-x64',
      'sha256',
    ],
  },);
  /**
   Version of the policy CLI the lock pins.
   */
  const policyCliVersion = lockText({
    lock,
    keys: [
      'tools',
      `cargo:${POLICY_CLI_CRATE}`,
      'version',
    ],
  },);
  if (!binstallUrl.startsWith(BINSTALL_ADDRESS_PREFIX,)) {
    say(`::error::Unexpected cargo-binstall archive in toolchain.lock.json: '${binstallUrl}'`,);
    throw new StepExitError(1,);
  }
  if (!looksLikeVersion(policyCliVersion,)) {
    say(`::error::Unexpected git-policy-cli version in toolchain.lock.json: '${policyCliVersion}'`,);
    throw new StepExitError(1,);
  }
  /**
   The runner's temporary folder.
   */
  const runnerTemp = requiredEnvironment('RUNNER_TEMP',);
  /**
   Folder `cargo-binstall` is unpacked into.
   */
  const binstallFolder = join(
    runnerTemp,
    'binstall',
  );
  /**
   The downloaded archive.
   */
  const archive = join(
    runnerTemp,
    'binstall.tgz',
  );
  /**
   Folder `cargo-binstall` installs the release under.
   */
  const installRoot = join(
    runnerTemp,
    'cli-git',
  );
  await mkdir(
    binstallFolder,
    { recursive: true, },
  );
  await runProgramOrEnd({
    file: 'curl',
    args: [
      '--silent',
      '--show-error',
      '--fail',
      '--location',
      '--output',
      archive,
      binstallUrl,
    ],
    cwd: process.cwd(),
  },);
  await runProgramOrEnd({
    file: 'sha256sum',
    args: [
      '--check',
      '--strict',
    ],
    input: `${binstallSha256}  ${archive}\n`,
    cwd: process.cwd(),
  },);
  await runProgramOrEnd({
    file: 'tar',
    args: [
      '--extract',
      '--gzip',
      '--file',
      archive,
      '--directory',
      binstallFolder,
    ],
    cwd: process.cwd(),
  },);
  await runProgramOrEnd({
    file: join(
      binstallFolder,
      'cargo-binstall',
    ),
    args: [
      '--no-confirm',
      '--disable-strategies',
      'compile',
      '--root',
      installRoot,
      '--version',
      policyCliVersion,
      POLICY_CLI_CRATE,
    ],
    cwd: process.cwd(),
  },);
  await appendToRunnerFile({
    variable: 'GITHUB_PATH',
    text: `${join(
      installRoot,
      'bin',
    )}\n`,
  },);
  /**
   Names of the release that was installed.
   */
  const names = await isExecutable(join(
    installRoot,
    'bin',
    RENAMED_PROGRAM,
  ),) ? RENAMED_NAMES : EARLIER_NAMES;
  await announce(names,);
  if (names === EARLIER_NAMES)
    await copySettingsForEarlierRelease();
  await runProgramOrEnd({
    file: join(
      installRoot,
      'bin',
      names.program,
    ),
    args: ['--version',],
    cwd: process.cwd(),
  },);
}

/**
 Gives an earlier release, which reads only the earlier settings file name, a copy of the checkout's settings file under
 that name, once the checkout's file carries the new name (the lock step renames it) and no copy exists yet.

 @example
 ```ts
 await copySettingsForEarlierRelease();
 ```
 */
async function copySettingsForEarlierRelease(): Promise<void> {
  /**
   The settings file under the new name.
   */
  const renamed = await pathFacts(RENAMED_NAMES.settings,);
  /**
   The settings file under the earlier name.
   */
  const earlier = await pathFacts(EARLIER_SETTINGS,);
  if (renamed.isFile && (!earlier.exists))
    // Exclusive, like `cp`, which refuses to write through a link that points nowhere instead of creating its target.
    await copyFile(
      RENAMED_NAMES.settings,
      EARLIER_SETTINGS,
      constants.COPYFILE_EXCL,
    );
}

//endregion The whole step

await runStep(installPolicyCli,);
