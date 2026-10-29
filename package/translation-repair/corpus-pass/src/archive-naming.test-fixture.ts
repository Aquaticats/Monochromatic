import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';
import { resolveRealGit as resolveGit, } from '@monochromatic-dev/git-executable-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { foldCarriageReturns, } from '@monochromatic-dev/translation-repair-slice-reading';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source';
import { passArchiveText, } from '../dist/final/node/index.mjs';
import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Disposable history fixtures
// Native Git writes only invented content inside individually owned repositories.

/**
 Fixture logger used by the same archive normalizer as production.
 */
const l = tagged({ tag: 'archive-naming-fixture', },);
/**
 Native executable, never the current repository's command-policy wrapper.
 */
const REAL_GIT = await resolveGit();
/**
 Invented entry path shared by ordinary fixtures.
 */
const ARCHIVE_PATH = 'people/starlit-cat/page.en.md';
/**
 Invented prior reference, without any licensed corpus text.
 */
const BEFORE_ARCHIVE = 'She joined 星猫亭.\n';
/**
 Invented current reference whose entire markup is replaced in history.
 */
const AFTER_ARCHIVE = 'She joined *Starlit Paws*.\n';

/**
 Runs native Git hermetically inside a disposable fixture.

 @param cloneDir - owned fixture directory

 @param args - literal Git arguments

 @returns Captured metadata without Git's final output newline

 @example
 ```ts
 const sha = await namingFixtureGit({ cloneDir, args: ['rev-parse', 'HEAD'] });
 ```
 */
export async function namingFixtureGit({
  cloneDir,
  args,
}: {
  readonly cloneDir: string;
  readonly args: readonly string[];
},): Promise<string> {
  /**
   Metadata-only command result.
   */
  const result = await spawnKeyless({
    file: REAL_GIT,
    args: [
      '--literal-pathspecs',
      '-C',
      cloneDir,
      '-c',
      `core.hooksPath=${devNull}`,
      '-c',
      'user.name=archive-fixture',
      '-c',
      'user.email=archive-fixture@example.invalid',
      '-c',
      'commit.gpgSign=false',
      ...args,
    ],
    extra: {
      GIT_CONFIG_GLOBAL: devNull,
      GIT_CONFIG_SYSTEM: devNull,
    },
  },);
  return result.stdout;
}

/**
 Commits one invented archive version at its literal path.

 @param cloneDir - owned fixture repository

 @param relPath - archive path inside the fixture

 @param text - next invented archive snapshot

 @returns Commit containing that snapshot

 @example
 ```ts
 const sha = await commitNamingArchive({ cloneDir, relPath: ARCHIVE_PATH, text: AFTER_ARCHIVE });
 ```
 */
async function commitNamingArchive({
  cloneDir,
  relPath,
  text,
}: {
  readonly cloneDir: string;
  readonly relPath: string;
  readonly text: string;
},): Promise<string> {
  /**
   Location of the fixture's next archive version.
   */
  const path = join(
    cloneDir,
    relPath,
  );
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    text,
    'utf8',
  );
  await namingFixtureGit({
    cloneDir,
    args: [
      'add',
      '--',
      relPath
    ],
  },);
  await namingFixtureGit({
    cloneDir,
    args: [
      'commit',
      '--message',
      'record invented archive'
    ],
  },);
  return await namingFixtureGit({
    cloneDir,
    args: [
      'rev-parse',
      'HEAD'
    ],
  },);
}

/**
 Creates a pinned two-version archive and exposes its owned repository for controls.

 @param before - invented predecessor

 @param after - invented current snapshot

 @param relPath - literal path, including unusual-path controls

 @returns Fixture with complete cleanup and immutable current pin

 @example
 ```ts
 await using fixture = await makeNamingArchive({});
 ```
 */
export async function makeNamingArchive({
  before = BEFORE_ARCHIVE,
  after = AFTER_ARCHIVE,
  relPath = ARCHIVE_PATH,
}: {
  readonly before?: string;
  readonly after?: string;
  readonly relPath?: string;
},): Promise<AsyncDisposable & {
  readonly pin: CorpusPin;
  readonly parentCommit: string;
  readonly relPath: string;
  readonly archiveText: string;
}> {
  // Individually disposable repository; no user/shared Git state is mutated.
  return await scratchDirWith({
    prefix: 'translation-naming-',
    setup: async function seeded({ path: cloneDir, },): Promise<{
      readonly pin: CorpusPin;
      readonly parentCommit: string;
      readonly relPath: string;
      readonly archiveText: string;
    }> {
      await namingFixtureGit({
        cloneDir,
        args: ['init'],
      },);
      /**
       Sole predecessor of the naming revision.
       */
      const parentCommit = await commitNamingArchive({
        cloneDir,
        relPath,
        text: before,
      },);
      /**
       Pinned current naming revision.
       */
      const commitSha = await commitNamingArchive({
        cloneDir,
        relPath,
        text: after,
      },);
      return {
        pin: {
          cloneDir,
          commitSha,
          gitPath: REAL_GIT,
        },
        parentCommit,
        relPath,
        archiveText: passArchiveText({
          text: foldCarriageReturns({ text: after, },)
            .text,
          l,
        },),
      };
    },
  },);
}

//endregion Disposable history fixtures

//region Initial use fixtures
// Fixture observations declare syntactic use only, never correct naming or identity.
