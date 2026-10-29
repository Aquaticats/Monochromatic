/**
 Test-only throwaway corpus and settled artifacts for the settled rendering
 audit: a git repository holding one cat entry's two pages, and an archive
 directory holding artifacts written for it as the builder writes them, with
 the first slice replaced and every other kept. Moved out of
 `rendering-audit-settled-input.unit.test.ts` when the audit driver's tests and
 the built command's cases needed the same archive.

 Fixtures are cat-themed invention on a throwaway git repository. No corpus
 content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { resolveRealGit as resolveGit, } from '@monochromatic-dev/git-executable-core/ts';

import type { SliceDeliveryRecord, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { settledArtifactOverRows, } from './settled-artifact.test-fixture.ts';

//region Settled archive fixtures
// A throwaway corpus and the artifacts written for it.

/**
 Real git binary every fixture command runs through.

 RESOLVED rather than taken from PATH, which in this repository exposes a
 policy shim. The shim rejects fixture staging patterns and settles worktree
 copies against the REAL repository, so a throwaway corpus built through it
 races other cases on a lock that has nothing to do with this test.
 */
const REAL_GIT = await resolveGit();

/**
 Configuration sink keeping fixture repositories away from real git config.
 */
const DEV_NULL = '/dev/null';

/**
 Entry every fixture artifact is written for.
 */
export const ENTRY_ID = 'mittens';

/**
 Original page, whose front matter declares a name, an alias and a location so
 the identity block has something to carry.
 */
export const SOURCE_PAGE = `---
name: 毛毛
info:
    alias: 小猫
    location: 猫村
desc: 窗台上的猫。
---

## 第一节

猫猫在窗台上睡觉。

## 第二节

猫猫有自己的碗。
`;

/**
 Archive translation of it, structured the same way.
 */
export const TARGET_PAGE = `---
name: Mittens
info:
    alias: Kitty
    location: Cat Village
desc: The cat on the sill.
---

## Section one

The cat sleeps on the sill.

## Section two

The cat has a bowl.
`;

/**
 A second pair with no front matter at all, for the entry that declares
 nothing.
 */
export const BARE_SOURCE_PAGE = '猫猫在门口等着。\n\n猫猫喜欢晒太阳。\n';

/**
 Archive translation of the bare pair.
 */
export const BARE_TARGET_PAGE = 'The cat waits at the door.\n\nThe cat likes the sun.\n';

/**
 Runs one git command inside a fixture repository.

 @param cloneDir - repository the command runs in

 @param args - arguments after the directory selector

 @returns Standard output

 @example
 ```ts
 await fixtureGit({ cloneDir, args: ['rev-parse', 'HEAD',], },);
 ```
 */
async function fixtureGit(
  {
    cloneDir,
    args,
  }: {
    readonly cloneDir: string;
    readonly args: readonly string[];
  },
): Promise<string> {
  /**
   Subprocess result; only stdout is consumed.
   */
  const { stdout, } = await spawnKeyless({
    file: REAL_GIT,
    args: [
      '-C',
      cloneDir,
      ...args,
    ],
    extra: {
      GIT_CONFIG_GLOBAL: DEV_NULL,
      GIT_CONFIG_SYSTEM: DEV_NULL,
    },
  },);
  return stdout;
}

/**
 Commits one entry's two pages into a fixture repository.

 @param cloneDir - repository to write into

 @param entryId - entry directory name

 @param sourcePage - original page content

 @param targetPage - archive translation content

 @param infoYml - content of the entry's `info.yml`, none written when absent

 @returns Commit the write landed in

 @example
 ```ts
 const sha = await commitEntry({ cloneDir, entryId, sourcePage, targetPage, },);
 ```
 */
export async function commitEntry(
  {
    cloneDir,
    entryId,
    sourcePage,
    targetPage,
    infoYml,
  }: {
    readonly cloneDir: string;
    readonly entryId: string;
    readonly sourcePage: string;
    readonly targetPage: string;
    readonly infoYml?: string;
  },
): Promise<string> {
  /**
   Paths this commit adds, the entry's pages and its `info.yml` where one is given.
   */
  const paths = [
    `people/${entryId}/page.md`,
    `people/${entryId}/page.en.md`,
    ...((infoYml === undefined) ? [] : [`people/${entryId}/info.yml`,]),
  ];
  /**
   Entry directory inside the fixture repository.
   */
  const dir = join(
    cloneDir,
    'people',
    entryId,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      'page.md',
    ),
    sourcePage,
    'utf8',
  );
  await writeFile(
    join(
      dir,
      'page.en.md',
    ),
    targetPage,
    'utf8',
  );
  if (infoYml !== undefined) {
    await writeFile(
      join(
        dir,
        'info.yml',
      ),
      infoYml,
      'utf8',
    );
  }
  // EXPLICIT PATHSPECS rather than `--all`: the repository's own git policy
  // rejects bulk staging, and a fixture that bypassed it would be teaching the
  // habit the guard exists to stop.
  await fixtureGit({
    cloneDir,
    args: [
      'add',
      ...paths,
    ],
  },);
  await fixtureGit({
    cloneDir,
    args: [
      '-c',
      'user.name=cat',
      '-c',
      'user.email=cat@example.org',
      'commit',
      '--message',
      `add ${entryId}`,
      '--no-gpg-sign',
      '--',
      ...paths,
    ],
  },);
  return (await fixtureGit({
    cloneDir,
    args: [
      'rev-parse',
      'HEAD',
    ],
  },))
    .trim();
}

/**
 Initialises the throwaway repository and commits the cat pair into it.

 @param path - directory made for the repository

 @param infoYml - content of the entry's `info.yml`, none written when absent

 @returns Clone directory and commit the pair landed in

 @example
 ```ts
 const { cloneDir, commitSha, } = await seededCorpus({ path, },);
 ```
 */
async function seededCorpus(
  {
    path: cloneDir,
    infoYml,
  }: {
    readonly path: string;
    readonly infoYml?: string;
  },
): Promise<{
  readonly cloneDir: string;
  readonly commitSha: string;
}> {
  await spawnKeyless({
    file: REAL_GIT,
    args: [
      'init',
      cloneDir,
    ],
    extra: {
      GIT_CONFIG_GLOBAL: DEV_NULL,
      GIT_CONFIG_SYSTEM: DEV_NULL,
    },
  },);

  /**
   Commit the pair landed in, which every fixture artifact pins.
   */
  const commitSha = await commitEntry({
    cloneDir,
    entryId: ENTRY_ID,
    sourcePage: SOURCE_PAGE,
    targetPage: TARGET_PAGE,
    ...((infoYml === undefined) ? {} : { infoYml, }),
  },);

  return {
    cloneDir,
    commitSha,
  };
}

/**
 Throwaway corpus holding the cat pair, removed on dispose.

 @param infoYml - content of the entry's `info.yml`, none written when absent

 @returns Clone directory, the commit holding the pair, and a disposer

 @example
 ```ts
 await using corpus = await makeCorpus();
 ```
 */
export async function makeCorpus(
  { infoYml, }: { readonly infoYml?: string; } = {},
): Promise<
  AsyncDisposable & {
    readonly cloneDir: string;
    readonly commitSha: string;
  }
  > {
  // Fresh temp directory holding the throwaway repository.
  return await scratchDirWith({
    prefix: 'settled-audit-corpus-',
    setup: async function seeded({ path, },) {
      return await seededCorpus({
        path,
        ...((infoYml === undefined) ? {} : { infoYml, }),
      },);
    },
  },);
}

/**
 Builds a ledger where the lane replaced the archive at the FIRST slice and
 kept it everywhere else.

 Both delivery kinds in one artifact, so the retained-versus-replaced split
 has something to separate.

 @param prepared - preparation to build rows from

 @returns One row per prepared slice, in document order

 @example
 ```ts
 const rows = replacedFirstSlice({ prepared, },);
 ```
 */
function replacedFirstSlice(
  { prepared, }: { readonly prepared: PreparedDocumentPair; },
): readonly SliceDeliveryRecord[] {
  return prepared.slices
    .map(function toRow(
      slice,
      position,
    ): SliceDeliveryRecord {
      /**
       Both sides of this slice.
       */
      const {
        source,
        target,
      } = slice;

      /**
       Archive wording at this slice.
       */
      const incumbentText = target.text;

      /**
       Whether the archive holds wording here at all.

       A PAIRED PREPARATION LEAVES INSERTIONS: a section the pairing did not
       claim is placed as an insertion slice whose archive wording is absent,
       and the builder refuses a row calling that wording present. Such a
       slice ships fresh wording, as the lanes do.
       */
      const absent = target.kind === 'insertion';

      /**
       Whether this row ships a replacement: every insertion, and the first
       slice.
       */
      const replaced = absent || (position === 0);

      /**
       Wording the lane decided on, which differs at the first slice and at
       every insertion.
       */
      const acceptedText = absent
        ? 'The cat has been given a line.'
        : ((position === 0) ? `${incumbentText} It purrs.` : incumbentText);

      return {
        sliceIndex: target.sliceIndex,
        sourceText: source.text,
        incumbentKind: absent ? 'absent' : 'present',
        incumbentText,
        outcome: {
          kind: 'decided',
          acceptedText,
        },
        shippedText: acceptedText,
        delivery: replaced
          ? { kind: 'replacement-shipped', }
          : { kind: 'incumbent-retained', },
      };
    },);
}

/**
 Narrows a parsed JSON value to an object whose members are read by name.

 @param value - parsed JSON

 @returns Whether it is a plain object

 @example
 ```ts
 const named = isRecordValue({ a: 1, },); // true
 ```
 */
function isRecordValue(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (((typeof value) === 'object') && (value !== null)) && (!Array.isArray(value,));
}

/**
 Writes one artifact into an archive run set.

 @param archiveDir - throwaway archive

 @param runSet - subdirectory to write into

 @param prepared - preparation both lanes ran over

 @param corpusSha - commit the artifact claims its pair was read at

 @param entryId - entry the artifact is written for

 @param strip - preparation members removed from the file, as a file written before they existed lacks them

 @throws Error when the serialized artifact is not an object holding a preparation, which the builder rules out

 @example
 ```ts
 await writeArtifact({ archiveDir, runSet, prepared, corpusSha, entryId, },);
 ```
 */
export async function writeArtifact(
  {
    archiveDir,
    runSet,
    prepared,
    corpusSha,
    entryId,
    strip = [],
  }: {
    readonly archiveDir: string;
    readonly runSet: string;
    readonly prepared: PreparedDocumentPair;
    readonly corpusSha: string;
    readonly entryId: string;
    readonly strip?: readonly string[];
  },
): Promise<void> {
  /**
   Run-set directory this artifact lands in.
   */
  const dir = join(
    archiveDir,
    runSet,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  /**
   Artifact as the builder writes it, in its serialized form: what a reader
   holds is the bytes a file carries, and a clone would keep things JSON drops.
   */
  const serialized = JSON.stringify(settledArtifactOverRows({
    prepared,
    entryId,
    corpusSha,
    rows: replacedFirstSlice({ prepared, },),
  },),);

  /**
   Those bytes read back.
   */
  const written: unknown = JSON.parse(serialized,);
  if ((!isRecordValue(written,)) || (!isRecordValue(written.preparation,)))
    throw new Error('the serialized artifact must be an object holding a preparation',);

  /**
   Preparation record with the named keys removed, which is how a file
   written before those fields existed looks to a reader.
   */
  const preparation = Object.fromEntries(
    Object.entries(written.preparation,)
      .filter(function kept([key,],): boolean {
        return !strip.includes(key,);
      },),
  );
  await writeFile(
    join(
      dir,
      `${entryId}.json`,
    ),
    JSON.stringify(
      {
        ...written,
        preparation,
      },
      undefined,
      2,
    ),
    'utf8',
  );
}

//endregion Settled archive fixtures
