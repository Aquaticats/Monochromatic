import {
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  refuseFailedRun,
  startProgram,
} from './lint-program.ts';
import { contextRoot, } from './log-context.ts';
import { loneSurrogateIndex, } from './lone-surrogate.ts';
import {
  CONFIG_FILE_NAME,
  LINT_BINARY_NAME,
  MOST_VERSION_CHARACTERS,
  PASSAGE_FILE_NAME,
  PINNED_LINT_BINARY_VERSION,
} from './semantic-wrap-binary-constants.ts';
import {
  LintBinaryVersionError,
  UNREADABLE_ANSWER,
  versionClause,
} from './semantic-wrap-binary-error.ts';
import {
  countedCharacters,
  LintBinaryRunError,
  LintBinaryTextError,
} from './semantic-wrap-run-error.ts';

//region Semantic wrap binary
// THE PROGRAM THAT PUTS THE LINE BREAKS BACK, started once per passage.
//
// The rule that wraps shipped text lives in the repository's Rust linter, the
// `monochromatic-lint` executable that `toolchain.lock.json` pins as a cargo
// tool. This module is the only place that decides what it is asked. A
// passage goes in on standard input and the wrapped passage comes back on
// standard output: never in an argument, never through a file, never through
// a shell, so quoting, backslashes and a leading dash are not a concern of
// this code.
//
// FOUND BY NAME. The program is started as `monochromatic-lint`, and the
// operating system's lookup of the PATH finds it, which is why a run started
// by `mise run` finds the pinned one: the root `mise.toml` puts the tree's
// `.cache/toolchain-bin`, which `mise run toolchain:install` fills from
// `toolchain.lock.json`, on the PATH ahead of any ambient or global tool. No
// path and no user name is written here.
//
// ITS VERSION IS CHECKED BEFORE ITS FIRST PASSAGE. The rule's behavior is the
// binary's, so a different version is a different wrap, and the cache replay
// relies on a second application changing nothing. A mismatch stops the run
// at its first wrap with a message naming both versions, rather than
// shipping text wrapped by a rule nobody pinned.
//
// ITS CONFIGURATION IS A FILE THE PACKAGE SHIPS, `semantic-wrap.config.jsonc`,
// because the program reads its configuration from a path. The file sits in
// `src`, which the package publishes, and the build copies it beside the
// built chunks in `dist/final/node`, where the bundle reads it.
//
// THE CHILD GETS NO KEY AND NO SETTING: its environment is built by
// `childEnvironment` in `lint-program.ts`, as for every child of production
// code.

/**
 Root logger for this module; lines it writes inside an entry name the entry
 and slice. It never writes a passage, only counts and statuses.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Programs whose version has been read and matched, so the question is asked
 once per program in a process and not once per passage.
 */
const confirmedPrograms = new Set<string>();

/**
 Locates the configuration file the package ships.

 READ BESIDE THIS MODULE, wherever it runs. From source the file stands in
 `src` next to the module; from the build a copy stands in `dist/final/node`
 next to every chunk, put there by the build configs. A copy beside the chunks
 is what lets every bundle that inlines this module find the file wherever
 that bundle is placed, where a path climbing out of the build directory to
 `src` would point into whichever package holds the bundle. It also puts the
 file's bytes in the directory the pipeline digest hashes, so changing the
 configuration, which changes the line breaks the program produces, changes
 the digest. A program that cannot read the path exits with a status, which
 the run error states.

 @returns Path of the file

 @example
 ```ts
 const config = semanticWrapConfigPath();
 ```
 */
function semanticWrapConfigPath(): string {
  return join(
    import.meta.dirname,
    CONFIG_FILE_NAME,
  );
}

/**
 Reads the version token off the program's answer to `--version`.

 @param answer - standard output of the question, `monochromatic-lint 0.2.0` and a newline

 @returns The version token, or the stand-in for an answer that is not the program's name and one version token

 @example
 ```ts
 versionTokenOf({ answer: 'monochromatic-lint 0.2.0\n', },); // '0.2.0'
 ```
 */
function versionTokenOf({ answer, }: { readonly answer: string; },): string | typeof UNREADABLE_ANSWER {
  /**
   The answer's words.
   */
  const words = answer.trim()
    .split(' ',);
  if ((words.length !== 2) || (words[0] !== LINT_BINARY_NAME))
    return UNREADABLE_ANSWER;

  /**
   The second word, which must be nothing but version characters to be repeated.
   */
  const token = nonNullishOrThrow(words[1],);
  if ((token.length === 0) || (token.length > MOST_VERSION_CHARACTERS))
    return UNREADABLE_ANSWER;
  for (const character of token) {
    if (!(isAsciiAlphanumeric({ character, },) || ('.+-'.includes(character,))))
      return UNREADABLE_ANSWER;
  }
  return token;
}

/**
 Asks the program for its version, once per program in this process, and
 refuses one that is not the pinned version.

 @param binary - program to ask

 @param cwd - directory to run in

 @throws {@link LintBinaryMissingError} when nothing stands at the program's path

 @throws {@link LintBinaryRunError} when the question could not be answered

 @throws {@link LintBinaryVersionError} when the answer is not the pinned version

 @example
 ```ts
 confirmPinnedVersion({ binary: 'monochromatic-lint', cwd, },);
 ```
 */
function confirmPinnedVersion({
  binary,
  cwd,
}: {
  readonly binary: string;
  readonly cwd: string
},): void {
  /**
   Logger for this question.
   */
  const fl = tagged({
    tag: confirmPinnedVersion.name,
    l,
  },);
  if (confirmedPrograms.has(binary,)) {
    fl.trace('already confirmed in this process',);
    return;
  }

  /**
   The program's answer.
   */
  const run = startProgram({
    binary,
    args: ['--version',],
    cwd,
  },);
  refuseFailedRun({
    binary,
    characters: 0,
    run,
  },);

  /**
   The version it reports, or the stand-in where its answer was not a version line.
   */
  const found = versionTokenOf({ answer: run.stdout, },);
  if (found !== PINNED_LINT_BINARY_VERSION) {
    fl.error(`the program is ${versionClause({ found, },)}, not the pinned ${PINNED_LINT_BINARY_VERSION}`,);
    throw new LintBinaryVersionError({
      binary,
      found,
    },);
  }
  confirmedPrograms.add(binary,);
  fl.info(`confirmed version ${found}`,);
}

/**
 Wraps one passage at its semantic boundaries with the program.

 @param binary - program to start, by name for the PATH lookup or by path

 @param text - passage as the lane produced it

 @returns Same passage with semantic line breaks inserted; an empty passage is returned without starting the program

 @throws {@link LintBinaryTextError} when the passage holds an unpaired surrogate

 @throws {@link LintBinaryMissingError} when nothing stands at the program's path

 @throws {@link LintBinaryVersionError} when the program is not the pinned version

 @throws {@link LintBinaryRunError} when the program fails or answers with nothing

 @example
 ```ts
 const wrapped = wrapWithLintBinary({ binary: LINT_BINARY_NAME, text: 'It naps. It wakes.', },);
 ```
 */
export function wrapWithLintBinary({
  binary,
  text,
}: {
  readonly binary: string;
  readonly text: string
},): string {
  /**
   Logger for this wrap.
   */
  const fl = tagged({
    tag: wrapWithLintBinary.name,
    l,
  },);
  if (text === '') {
    fl.trace('empty passage; the program is not started',);
    return text;
  }

  /**
   The passage's length, named for the log messages of this function that write it.
   */
  const counted = countedCharacters({ count: text.length, },);

  /**
   Where an unpaired surrogate stands, or -1.
   */
  const unpaired = loneSurrogateIndex({ text, },);
  if (unpaired !== (-1)) {
    fl.error(`a passage of ${counted} holds an unpaired surrogate at index ${String(unpaired,)}`,);
    throw new LintBinaryTextError({ unpairedIndex: unpaired, },);
  }

  /**
   Configuration file, whose directory the program runs in.
   */
  const config = semanticWrapConfigPath();
  /**
   Directory holding the configuration.
   */
  const cwd = dirname(config,);
  confirmPinnedVersion({
    binary,
    cwd,
  },);

  fl.trace(`wrapping a passage of ${counted}`,);
  /**
   The program's answer.
   */
  const run = startProgram({
    binary,
    args: [
      '--config',
      config,
      '--stdin',
      '--stdin-filename',
      PASSAGE_FILE_NAME,
      '--fix',
    ],
    input: text,
    cwd,
  },);
  try {
    refuseFailedRun({
      binary,
      characters: text.length,
      run,
    },);
  }
  catch (error) {
    fl.error(`the program failed for a passage of ${counted}`,);
    throw error;
  }
  if (run.stdout === '') {
    fl.error(`the program wrote nothing for a passage of ${counted}`,);
    throw new LintBinaryRunError({
      binary,
      characters: text.length,
      failure: { kind: 'empty-reply', },
    },);
  }
  return run.stdout;
}

//endregion Semantic wrap binary
