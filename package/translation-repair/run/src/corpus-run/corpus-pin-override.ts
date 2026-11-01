import { isAbsolute, } from 'node:path';

import { isLowerHexDigit, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Corpus pin override
// Which clone and commit a run reads its corpus from.
//
// The pinned defaults serve every benchmark and calibration run; fixture runs
// against an unmerged corpus pull request need a different clone and commit
// WITHOUT editing source, which is what the 2026-08-29 Carena runs did and
// what this dial replaces. Refusal over fallback for the same reason as the
// overlap dial: a mistyped override that silently became the pin would run
// the wrong corpus and record its conclusion as the fixture's.

/**
 Environment variable naming the corpus clone directory a run reads.
 */
export const CORPUS_CLONE_DIR_VAR = 'TRANSLATION_REPAIR_CORPUS_CLONE_DIR';

/**
 Environment variable naming the corpus commit every read resolves against.
 */
export const CORPUS_COMMIT_VAR = 'TRANSLATION_REPAIR_CORPUS_COMMIT';

/**
 Where each half of a pin came from, for launch logs.

 @example
 ```ts
 const source: CorpusPinSource = 'fallback';
 ```
 */
export type CorpusPinSource =
  | 'fallback'
    | typeof CORPUS_CLONE_DIR_VAR
    | typeof CORPUS_COMMIT_VAR;

/**
 One resolved pin beside where each half came from.

 @example
 ```ts
 const setting: CorpusPinSetting = {
   pin: { cloneDir: '/cats/corpus', commitSha: 'a'.repeat(40,), },
   cloneDirSource: 'fallback',
   commitSource: 'fallback',
 };
 ```
 */
export type CorpusPinSetting = {
  /**
   Pin every corpus read resolves through.
   */
  readonly pin: CorpusPin;

  /**
   Fallback or environment variable that supplied the clone directory.
   */
  readonly cloneDirSource: CorpusPinSource;

  /**
   Fallback or environment variable that supplied the commit.
   */
  readonly commitSource: CorpusPinSource;
};

/**
 Length of one full git object name.
 */
const COMMIT_SHA_LENGTH = 40;

/**
 Whether written text is one full lowercase hexadecimal commit name.
 Abbreviated names are refused because the pin must stay unambiguous
 across clones that resolve abbreviations differently.

 @param written - text as the invoking environment wrote it

 @returns Whether every position is lowercase hexadecimal at full length

 @example
 ```ts
 isFullCommitSha({ written: 'a'.repeat(40,), },);
 ```
 */
function isFullCommitSha(
  { written, }: { readonly written: string; },
): boolean {
  if (written.length !== COMMIT_SHA_LENGTH)
    return false;
  // Indexed scan instead of iteration: sha positions are single code units,
  // and grapheme-aware decomposition would hide a multi-unit intruder.
  for (let position = 0; position < written.length; position += 1) {
    if (!isLowerHexDigit({ character: written.charAt(position,), },))
      return false;
  }
  return true;
}

/**
 Reads the corpus pin, overriding either half from the environment it is
 handed.

 Refuses rather than falls back on invalid input, because a run against
 the wrong corpus records fixture conclusions as pinned-corpus ones.

 THE ENVIRONMENT IS HANDED IN, never read here: an entry reads its process's
 environment inside its refusal boundary, so an unreadable value is the
 command's stated refusal, not a fault while a module loads (ledger B302,
 B381).

 @param env - environment the two variables are read from: the process's
 own at an entry, a case's own in a test

 @param fallback - pin used for any half the environment leaves unset

 @returns Valid pin beside a per-half source

 @throws StatedRefusalError when the clone dir is not absolute or the commit is not one full lowercase sha

 @example
 ```ts
 const setting = readCorpusPinSetting({ env: process.env, fallback: { cloneDir: '/cats/corpus', commitSha: CORPUS_COMMIT_SHA, }, },);
 ```
 */
export function readCorpusPinSetting(
  {
    env,
    fallback,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly fallback: CorpusPin;
  },
): CorpusPinSetting {
  /**
   Clone directory as the invoking environment wrote it, empty when unset.
   */
  const writtenDir = env[CORPUS_CLONE_DIR_VAR] ?? '';
  if ((writtenDir !== '') && (!isAbsolute(writtenDir,))) {
    throw new StatedRefusalError({
      says: `${CORPUS_CLONE_DIR_VAR} must be an absolute path, and ${writtenDir} is not one`,
    },);
  }

  /**
   Commit as the invoking environment wrote it, empty when unset.
   */
  const writtenCommit = env[CORPUS_COMMIT_VAR] ?? '';
  if ((writtenCommit !== '') && (!isFullCommitSha({ written: writtenCommit, },))) {
    throw new StatedRefusalError({
      says: `${CORPUS_COMMIT_VAR} must be one full lowercase hexadecimal commit name,`
        + ` and ${writtenCommit} is not one`,
    },);
  }

  return {
    pin: {
      ...fallback,
      ...((writtenDir === '') ? {} : { cloneDir: writtenDir, }),
      ...((writtenCommit === '') ? {} : { commitSha: writtenCommit, }),
    },
    cloneDirSource: (writtenDir === '') ? 'fallback' : CORPUS_CLONE_DIR_VAR,
    commitSource: (writtenCommit === '') ? 'fallback' : CORPUS_COMMIT_VAR,
  };
}

/**
 Names where the corpus pin came from when a launch overrode either half, so
 a run's log says which corpus it read (ledger D13: the setting was kept "for
 launch logs" and no launch printed it).

 EMPTY FOR THE BUILT-IN PIN, like the other override notes a pass prints
 after its START line: a line on every launch saying nothing changed would
 bury the one that says something did.

 @param setting - the resolved pin beside where each half came from

 @returns The launch line, or empty when neither half was overridden

 @example
 ```ts
 const note = corpusPinOverrideNote({ setting: readRunCorpusPinSetting({ env: process.env, },), },);
 ```
 */
export function corpusPinOverrideNote(
  { setting, }: { readonly setting: CorpusPinSetting; },
): string {
  if ((setting.cloneDirSource === 'fallback') && (setting.commitSource === 'fallback'))
    return '';
  return `CORPUS PIN OVERRIDDEN: clone ${setting.pin
    .cloneDir} from ${setting.cloneDirSource}, `
    + `commit ${setting.pin
      .commitSha} from ${setting.commitSource}`;
}

//endregion Corpus pin override
