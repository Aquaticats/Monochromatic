/**
 Tests for pinned-commit corpus reads.
 Exercised against a throwaway git repository built in a temp directory;
 fixture content is cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  mkdir,
  realpath,
  symlink,
  writeFile,
} from 'node:fs/promises';
import {
  devNull,
  tmpdir,
} from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  CorpusReadError,
  isMissingCorpusObject,
  listCorpusPeople,
  readCorpusBytes,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { spawnKeyless, } from './child-environment.test-fixture.ts';
import { fixtureGit, REAL_GIT, } from './hermetic-git-run.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  LACKED_COMMIT_SHA,
  makeCloneHoldingOneCommit,
  PAIRED_ENTRY,
} from './corpus-lacked-commit.test-fixture.ts';

/**
 Invented zh page content committed into the throwaway clone.
 */
const WHISKERS_PAGE = '---\nname: 小猫-whiskers\n---\n\n## 简介\n\n猫猫喜欢晒太阳。[^1]\n\n[^1]:[猫猫习性说明。](https://example.org/cat)\n';

/**
 Invented zh page written with CRLF endings, as the one such page in the
 pinned corpus is.
 */
const TABBY_CRLF_PAGE = '---\r\nname: 小猫-tabby\r\n---\r\n\r\n## 简介\r\n\r\n猫猫在窗台上睡觉。\r\n';

/**
 Builds a throwaway corpus-shaped git repository with one committed entry,
 removed on dispose.

 @returns Clone directory, pinned commit, and async disposer

 @example
 ```ts
 await using fixture = await makeThrowawayClone();
 ```
 */
async function makeThrowawayClone(): Promise<
  AsyncDisposable & {
    readonly cloneDir: string;
    readonly commitSha: string;
  }
  > {
  // Fresh temp directory holding the throwaway repository.
  return await scratchDirWith({
    prefix: 'translation-repair-corpus-',
    setup: async function seeded({ path: cloneDir, },): Promise<{
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
          GIT_CONFIG_GLOBAL: devNull,
          GIT_CONFIG_SYSTEM: devNull,
        },
      },);
      await mkdir(
        join(
          cloneDir,
          'people',
          'whiskers',
        ),
        { recursive: true, },
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          'whiskers',
          'page.md',
        ),
        WHISKERS_PAGE,
        'utf8',
      );
      await mkdir(
        join(
          cloneDir,
          'people',
          'tabby',
        ),
        { recursive: true, },
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          'tabby',
          'page.md',
        ),
        TABBY_CRLF_PAGE,
        'utf8',
      );
      await fixtureGit({
        cloneDir,
        args: [
          'add',
          'people/whiskers/page.md',
          'people/tabby/page.md',
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
          'add whiskers',
          '--no-gpg-sign',
        ],
      },);

      /**
       Commit every test read pins to.
       */
      const commitSha = (await fixtureGit({
        cloneDir,
        args: [
          'rev-parse',
          'HEAD',
        ],
      },))
        .trim();

      return {
        cloneDir,
        commitSha,
      };
    },
  },);
}

/**
 What a corpus read that must refuse threw, as the kind it names and the
 whole sentence it prints.

 @param act - read expected to reject with a corpus read refusal

 @returns Kind and printed sentence

 @example
 ```ts
 const failure = await failureOf(async function reads() { return await readCorpusFile({ pin, relPath, },); },);
 ```
 */
async function failureOf(
  act: () => Promise<unknown>,
): Promise<{ readonly kind: string; readonly says: string; }> {
  /**
   What the read threw.
   */
  const refusal = await rejectionOf(act,);
  if (!(refusal instanceof CorpusReadError))
    throw new Error(`the read must refuse with a CorpusReadError, and it refused with ${String(refusal,)}`,);
  return {
    kind: refusal.kind,
    says: String(refusal,),
  };
}

await describe({
  name: readCorpusFile.name,
  children: [
    it({
      name: 'reads committed content byte-for-byte at the pinned commit',
      fn: async () => {
        await using fixture = await makeThrowawayClone();
        // Explicit gitPath covers the pin-supplied-binary branch;
        // the other tests exercise default resolution.
        expect(
          await readCorpusFile({
            pin: {
              cloneDir: fixture.cloneDir,
              commitSha: fixture.commitSha,
              gitPath: REAL_GIT,
            },
            relPath: 'people/whiskers/page.md',
          },),
        ).toBe(WHISKERS_PAGE,);
      },
    },),

    it({
      name: 'FOLDS CRLF TO LF on the way in, since every splitter downstream looks for a line feed and the '
        + 'one CRLF page in the pinned corpus defeated the line-structure predicate, the invisible-line '
        + 'mask and the quote normalizer at once; the bytes are otherwise untouched',
      fn: async () => {
        await using fixture = await makeThrowawayClone();

        /**
         The CRLF page as the package reads it.
         */
        const read = await readCorpusFile({
          pin: {
            cloneDir: fixture.cloneDir,
            commitSha: fixture.commitSha,
          },
          relPath: 'people/tabby/page.md',
        },);
        expect(read.includes('\r',),).toBe(false,);
        expect(read,).toBe(TABBY_CRLF_PAGE.replaceAll('\r\n', '\n',),);
      },
    },),
    it({
      name: 'READS one file\'s bytes at the pinned commit untouched, the CRLF page still carrying its '
        + 'carriage returns where the text reader folds them',
      fn: async () => {
        await using fixture = await makeThrowawayClone();
        expect(new TextDecoder()
          .decode(await readCorpusBytes({
            pin: {
              cloneDir: fixture.cloneDir,
              commitSha: fixture.commitSha,
            },
            relPath: 'people/tabby/page.md',
          },),),).toBe(TABBY_CRLF_PAGE,);
      },
    },),
    it({
      name: 'lists person entry ids at the pinned commit',
      fn: async () => {
        await using fixture = await makeThrowawayClone();
        expect(
          await listCorpusPeople({
            pin: {
              cloneDir: fixture.cloneDir,
              commitSha: fixture.commitSha,
            },
          },),
        ).toEqual([
          'tabby',
          'whiskers',
        ],);
      },
    },),

    it({
      name: 'throws CorpusReadError for paths absent at the pinned commit, and NAMES THE FAILURE a missing '
        + 'object, which is the one failure a walk over the corpus may step past',
      fn: async () => {
        await using fixture = await makeThrowawayClone();
        /** Value caught from read of a path that never existed. */
        let caught: unknown;
        try {
          await readCorpusFile({
            pin: {
              cloneDir: fixture.cloneDir,
              commitSha: fixture.commitSha,
            },
            relPath: 'people/mittens/page.md',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught instanceof CorpusReadError,).toBe(true,);
        expect((caught as CorpusReadError).kind,).toBe('missing-object',);
        expect((caught as Error).message,).toContain('(missing-object)',);
        expect(isMissingCorpusObject(caught,),).toBe(true,);
      },
    },),

    it({
      name: 'REFUSES a byte path absent at the pinned commit as a missing object, naming the object '
        + 'asked for the way the text reader names it',
      fn: async () => {
        await using fixture = await makeThrowawayClone();
        /**
         What the byte read of a path that never existed threw.
         */
        const refusal = await rejectionOf(async function readsAbsentBytes(): Promise<unknown> {
          return await readCorpusBytes({
            pin: {
              cloneDir: fixture.cloneDir,
              commitSha: fixture.commitSha,
            },
            relPath: 'people/mittens/photos/intro.webp',
          },);
        },);
        expect(isMissingCorpusObject(refusal,),).toBe(true,);
        expect(String(refusal,),).toBe(
          `CorpusReadError: corpus read failed for ${fixture.commitSha}:people/mittens/photos/intro.webp `
            + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.',
        );
      },
    },),

    it({
      name: 'throws CorpusReadError when the clone directory does not exist, and NAMES THE FAILURE '
        + 'unreadable-clone: an unreadable clone is a fault in the run, not a fact about the corpus, and no '
        + 'walk may step past it',
      fn: async () => {
        /** Value caught from read against a nonexistent clone. */
        let caught: unknown;
        try {
          await readCorpusFile({
            pin: {
              cloneDir: join(
                tmpdir(),
                'translation-repair-no-such-clone',
              ),
              commitSha: 'deadbeef',
            },
            relPath: 'people/whiskers/page.md',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught instanceof CorpusReadError,).toBe(true,);
        expect((caught as CorpusReadError).kind,).toBe('unreadable-clone',);
        expect(isMissingCorpusObject(caught,),).toBe(false,);
      },
    },),

    it({
      name: 'NAMES A LISTING FAILURE unreadable-clone too, since the listing goes through the other subprocess '
        + 'layer and a clone that cannot be listed is the same fault in the run',
      fn: async () => {
        /** Value caught from a listing against a nonexistent clone. */
        let caught: unknown;
        try {
          await listCorpusPeople({
            pin: {
              cloneDir: join(
                tmpdir(),
                'translation-repair-no-such-clone',
              ),
              commitSha: 'deadbeef',
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught instanceof CorpusReadError,).toBe(true,);
        expect((caught as CorpusReadError).kind,).toBe('unreadable-clone',);
      },
    },),

    it({
      name: 'LETS a listing failure that is no subprocess failure propagate as itself: a working directory '
        + 'removed under the process names its own fault rather than reading as a corpus refusal',
      fn: async () => {
        await using fixture = await makeThrowawayClone();
        /**
         Child program that removes the directory it stands in and lists the
         clone, printing what the listing threw. Every value it carries is
         written as a JSON literal, which is a JavaScript literal too.
         */
        const program = [
          "import { mkdtempSync, rmdirSync, } from 'node:fs';",
          "import { tmpdir, } from 'node:os';",
          "import { join, } from 'node:path';",
          `const { CorpusReadError, listCorpusPeople, } = await import(${
            JSON.stringify(import.meta.resolve('@monochromatic-dev/translation-repair-corpus-source',),)
          });`,
          "const gone = mkdtempSync(join(tmpdir(), 'translation-repair-gone-cwd-'));",
          'process.chdir(gone);',
          'rmdirSync(gone);',
          'try {',
          `  await listCorpusPeople({ pin: { cloneDir: ${JSON.stringify(fixture.cloneDir,)}, commitSha: ${
            JSON.stringify(fixture.commitSha,)
          }, gitPath: ${JSON.stringify(REAL_GIT,)} } });`,
          "  console.log(JSON.stringify({ listed: true }));",
          '} catch (error) {',
          '  console.log(JSON.stringify({ isCorpusReadError: error instanceof CorpusReadError, name: error.name, code: error.code }));',
          '}',
        ].join('\n',);
        /**
         What the child printed.
         */
        const { stdout, } = await spawnKeyless({
          file: process.execPath,
          args: [
            '--input-type=module',
            '--eval',
            program,
          ],
        },);
        expect(JSON.parse(stdout,),).toEqual({
          isCorpusReadError: false,
          name: 'Error',
          code: 'ENOENT',
        },);
      },
    },),

    it({
      name: 'CLASSIFIES a cause that is no object, or null, or one with no stderr as other at a commit the '
        + 'probe found held, since none of them holds git stderr to read',
      fn: async () => {
        expect(new CorpusReadError({
          detail: 'people/gum/page.md at deadbeef',
          cause: 'a plain string',
          commit: 'held',
        },).kind,).toBe('other',);
        expect(new CorpusReadError({
          detail: 'people/gum/page.md at deadbeef',
          cause: null,
          commit: 'held',
        },).kind,).toBe('other',);
        expect(new CorpusReadError({
          detail: 'people/gum/page.md at deadbeef',
          cause: { code: 1, },
          commit: 'held',
        },).kind,).toBe('other',);
      },
    },),

    it({
      name: 'READS a string stderr as its text, so a missing-object phrase lands on the kind at a commit the '
        + 'probe found held, and one that is neither buffer nor string as no stderr at all',
      fn: async () => {
        expect(new CorpusReadError({
          detail: 'people/gum/page.md at deadbeef',
          cause: {
            stderr: "fatal: path 'people/gum/page.md' does not exist in 'deadbeef'",
          },
          commit: 'held',
        },).kind,).toBe('missing-object',);
        expect(new CorpusReadError({
          detail: 'people/gum/page.md at deadbeef',
          cause: { stderr: 42, },
          commit: 'held',
        },).kind,).toBe('other',);
      },
    },),
    it({
      name: 'CLASSIFIES both texts git 2.55 prints for a path absent at a commit the probe found held as a '
        + 'missing object',
      fn: async () => {
        expect(new CorpusReadError({
          detail: 'people/gum/page.en.md at deadbeef',
          cause: { stderr: "fatal: path 'people/gum/page.en.md' does not exist in 'deadbeef'\n", },
          commit: 'held',
        },).kind,).toBe('missing-object',);
        expect(new CorpusReadError({
          detail: 'people/gum/page.en.md at deadbeef',
          cause: { stderr: "fatal: path 'people/gum/page.en.md' exists on disk, but not in 'deadbeef'\n", },
          commit: 'held',
        },).kind,).toBe('missing-object',);
      },
    },),

    it({
      name: 'READS no absent path in any text git 2.55 prints for a revision it cannot resolve or a clone it '
        + 'cannot open, so at a commit the probe found held each is other and the probe alone names them',
      fn: async () => {
        expect([
          "fatal: invalid object name 'abcdef1'.\n",
          'fatal: Not a valid object name abcdef1',
          'fatal: not a tree object',
          "fatal: cannot change to '/no/such/clone': No such file or directory\n",
          'fatal: not a git repository (or any parent up to mount point /)\n'
            + 'Stopping at filesystem boundary (GIT_DISCOVERY_ACROSS_FILESYSTEM not set).\n',
        ].map(function kindOfStderr(stderr,): string {
          return new CorpusReadError({
            detail: 'people/gum/page.md at deadbeef',
            cause: { stderr, },
            commit: 'held',
          },).kind;
        },),).toEqual([
          'other',
          'other',
          'other',
          'other',
          'other',
        ],);
      },
    },),

    it({
      name: 'NAMES the kind a commit the clone lacks as missing-commit, not missing-object, when the probe '
        + 'of the commit says it is not there, whatever stderr git printed',
      fn: async () => {
        /**
         Refusal of a read whose commit the probe found lacking, though
         git printed what it prints for an absent path.
         */
        const lacking = new CorpusReadError({
          detail: 'people/gum/page.md at deadbeef',
          cause: { stderr: "fatal: path 'people/gum/page.md' does not exist in 'deadbeef'\n", },
          commit: 'lacking',
        },);
        expect(lacking.kind,).toBe('missing-commit',);
        expect(isMissingCorpusObject(lacking,),).toBe(false,);
      },
    },),

    it({
      name: 'NAMES the kind unreadable-clone when git could not open the clone for the probe, and other when '
        + 'the probe got no answer, never missing-object, whatever stderr the read printed',
      fn: async () => {
        /**
         Probe answers that say nothing of the path.
         */
        const unanswered = [
          'unopened',
          'unasked',
        ] as const;
        expect(unanswered.map(function kindAfterProbe(commit,): readonly [string, boolean,] {
          /**
           The refusal of a read whose own text says the path is absent.
           */
          const refusal = new CorpusReadError({
            detail: 'people/gum/page.md at deadbeef',
            cause: { stderr: "fatal: path 'people/gum/page.md' does not exist in 'deadbeef'\n", },
            commit,
          },);
          return [
            refusal.kind,
            isMissingCorpusObject(refusal,),
          ];
        },),).toEqual([
          [
            'unreadable-clone',
            false,
          ],
          [
            'other',
            false,
          ],
        ],);
      },
    },),

    it({
      name: 'PRINTS for each kind what was found and the one remedy that fits it, naming only the path and the revision',
      fn: async () => {
        expect([
          new CorpusReadError({
            detail: 'people/gum/page.en.md at deadbeef',
            cause: { stderr: "fatal: path 'people/gum/page.en.md' does not exist in 'deadbeef'\n", },
            commit: 'held',
          },),
          new CorpusReadError({
            detail: 'people/gum/page.en.md at deadbeef',
            cause: { stderr: "fatal: path 'people/gum/page.en.md' does not exist in 'deadbeef'\n", },
            commit: 'lacking',
          },),
          new CorpusReadError({
            detail: 'people/gum/page.en.md at deadbeef',
            cause: { stderr: 'fatal: not a git repository\n', },
            commit: 'unopened',
          },),
          new CorpusReadError({
            detail: 'people/gum/page.en.md at deadbeef',
            cause: { stderr: 'fatal: unable to read the object\n', },
            commit: 'held',
          },),
        ].map(String,),).toEqual([
          'CorpusReadError: corpus read failed for people/gum/page.en.md at deadbeef (missing-object);'
            + ' the commit has no such path: check the path, or pin a commit that has it.',
          'CorpusReadError: corpus read failed for people/gum/page.en.md at deadbeef (missing-commit);'
            + ' the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
          'CorpusReadError: corpus read failed for people/gum/page.en.md at deadbeef (unreadable-clone);'
            + ' git could not open the clone: check that the directory exists, is the top of a git repository, and'
            + ' is one git may read.',
          'CorpusReadError: corpus read failed for people/gum/page.en.md at deadbeef (other);'
            + ' the read failed another way: run the same git read in the clone by hand to see why.',
        ],);
      },
    },),

    it({
      name: 'REFUSES a full hash the clone lacks as missing-commit for a text read, a byte read and a listing, '
        + 'whether the path is on disk or not, and a held commit with an absent path stays a missing object',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        /**
         Pin at a commit the clone lacks.
         */
        const lacked = {
          cloneDir: clone.cloneDir,
          commitSha: LACKED_COMMIT_SHA,
        };
        /**
         Pin at the commit the clone holds.
         */
        const held = {
          cloneDir: clone.cloneDir,
          commitSha: clone.commitSha,
        };
        expect(await failureOf(async function readsPathOnDisk(): Promise<unknown> {
          return await readCorpusFile({
            pin: lacked,
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },),).toEqual({
          kind: 'missing-commit',
          says: `CorpusReadError: corpus read failed for ${LACKED_COMMIT_SHA}:people/${PAIRED_ENTRY}/page.md `
            + '(missing-commit); the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        },);
        expect(await failureOf(async function readsPathOffDisk(): Promise<unknown> {
          return await readCorpusFile({
            pin: lacked,
            relPath: 'people/ghost/page.md',
          },);
        },),).toEqual({
          kind: 'missing-commit',
          says: `CorpusReadError: corpus read failed for ${LACKED_COMMIT_SHA}:people/ghost/page.md (missing-commit);`
            + ' the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        },);
        expect((await failureOf(async function readsBytes(): Promise<unknown> {
          return await readCorpusBytes({
            pin: lacked,
            relPath: `people/${PAIRED_ENTRY}/photos/intro.webp`,
          },);
        },)).kind,).toBe('missing-commit',);
        expect(await failureOf(async function lists(): Promise<unknown> {
          return await listCorpusPeople({ pin: lacked, },);
        },),).toEqual({
          kind: 'missing-commit',
          says: `CorpusReadError: corpus read failed for people/ at ${LACKED_COMMIT_SHA} (missing-commit);`
            + ' the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        },);
        expect(await failureOf(async function readsAbsentPath(): Promise<unknown> {
          return await readCorpusFile({
            pin: held,
            relPath: 'people/ghost/page.md',
          },);
        },),).toEqual({
          kind: 'missing-object',
          says: `CorpusReadError: corpus read failed for ${clone.commitSha}:people/ghost/page.md (missing-object);`
            + ' the commit has no such path: check the path, or pin a commit that has it.',
        },);
      },
    },),

    it({
      name: 'REFUSES an abbreviated hash the clone lacks and a name that is no revision as missing-commit',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        expect((await failureOf(async function readsAbbreviated(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: 'abcdef1',
            },
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },)).kind,).toBe('missing-commit',);
        expect((await failureOf(async function readsNoRevision(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: 'no-such-branch',
            },
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },)).kind,).toBe('missing-commit',);
        expect((await failureOf(async function listsAbbreviated(): Promise<unknown> {
          return await listCorpusPeople({
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: 'abcdef1',
            },
          },);
        },)).kind,).toBe('missing-commit',);
      },
    },),

    it({
      name: 'REFUSES A REVISION THAT NAMES A TREE the clone holds as missing-commit where a page is absent, saying '
        + 'the clone holds no commit by it rather than that it lacks the revision',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        /**
         Id of the tree the one commit records, an object the clone holds that
         is no commit.
         */
        const treeId = (await fixtureGit({
          cloneDir: clone.cloneDir,
          args: [
            'rev-parse',
            `${clone.commitSha}^{tree}`,
          ],
        },))
          .trim();
        expect(await failureOf(async function readsAtTree(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: treeId,
            },
            relPath: 'people/ghost/page.md',
          },);
        },),).toEqual({
          kind: 'missing-commit',
          says: `CorpusReadError: corpus read failed for ${treeId}:people/ghost/page.md (missing-commit); the clone `
            + 'holds no commit by that revision: fetch it, or pin a commit the clone holds.',
        },);
      },
    },),

    it({
      name: 'REFUSES a clone directory that does not exist and a directory that is no repository as '
        + 'unreadable-clone, for a text read, a byte read and a listing, since git could not open either',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        await using bare = await scratchDir({ prefix: 'translation-repair-no-repository-', },);
        /**
         Directories git cannot read a corpus from.
         */
        const unreadable = [
          join(
            clone.cloneDir,
            'no-such-clone',
          ),
          bare.path,
        ];
        for (const cloneDir of unreadable) {
          /* oxlint-disable no-await-in-loop -- two directories, each read once in order */
          expect(await failureOf(async function readsUnreadable(): Promise<unknown> {
            return await readCorpusFile({
              pin: {
                cloneDir,
                commitSha: clone.commitSha,
              },
              relPath: `people/${PAIRED_ENTRY}/page.md`,
            },);
          },),).toEqual({
            kind: 'unreadable-clone',
            says: `CorpusReadError: corpus read failed for ${clone.commitSha}:people/${PAIRED_ENTRY}/page.md `
              + '(unreadable-clone); git could not open the clone: check that the directory exists, is the top '
              + 'of a git repository, and is one git may read.',
          },);
          expect((await failureOf(async function readsUnreadableBytes(): Promise<unknown> {
            return await readCorpusBytes({
              pin: {
                cloneDir,
                commitSha: clone.commitSha,
              },
              relPath: `people/${PAIRED_ENTRY}/photos/intro.webp`,
            },);
          },)).kind,).toBe('unreadable-clone',);
          expect((await failureOf(async function listsUnreadable(): Promise<unknown> {
            return await listCorpusPeople({
              pin: {
                cloneDir,
                commitSha: clone.commitSha,
              },
            },);
          },)).kind,).toBe('unreadable-clone',);
          /* oxlint-enable no-await-in-loop */
        }
      },
    },),

    it({
      name: 'REFUSES A DIRECTORY THAT IS NO REPOSITORY INSIDE ANOTHER as unreadable-clone, for a page the '
        + 'enclosing repository holds, a page it lacks, a commit it lacks and a listing, and when reached through '
        + 'a symbolic link, never reading the enclosing repository in its place',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        /**
         Plain directory inside the throwaway repository, named as a clone.
         */
        const inner = join(
          clone.cloneDir,
          'not-a-clone',
        );
        await mkdir(inner,);
        expect(await failureOf(async function readsEnclosedPage(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: inner,
              commitSha: clone.commitSha,
            },
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },),).toEqual({
          kind: 'unreadable-clone',
          says: `CorpusReadError: corpus read failed for ${clone.commitSha}:people/${PAIRED_ENTRY}/page.md `
            + '(unreadable-clone); git could not open the clone: check that the directory exists, is the top of '
            + 'a git repository, and is one git may read.',
        },);
        expect((await failureOf(async function readsEnclosedAbsentPage(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: inner,
              commitSha: clone.commitSha,
            },
            relPath: 'people/ghost/page.md',
          },);
        },)).kind,).toBe('unreadable-clone',);
        expect((await failureOf(async function readsEnclosedLackedCommit(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: inner,
              commitSha: LACKED_COMMIT_SHA,
            },
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },)).kind,).toBe('unreadable-clone',);
        expect((await failureOf(async function listsEnclosed(): Promise<unknown> {
          return await listCorpusPeople({
            pin: {
              cloneDir: inner,
              commitSha: clone.commitSha,
            },
          },);
        },)).kind,).toBe('unreadable-clone',);
        await using links = await scratchDir({ prefix: 'translation-repair-clone-link-', },);
        /**
         A symbolic link to the throwaway repository, so the directory is
         named through a path git must resolve to compare with its own.
         */
        const linked = join(
          links.path,
          'linked-clone',
        );
        await symlink(
          clone.cloneDir,
          linked,
        );
        expect((await failureOf(async function readsEnclosedThroughLink(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: join(
                linked,
                'not-a-clone',
              ),
              commitSha: clone.commitSha,
            },
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },)).kind,).toBe('unreadable-clone',);
      },
    },),

    it({
      name: 'REFUSES A CLONE DIRECTORY THAT IS ITSELF A SYMBOLIC LINK to a plain directory inside a repository as '
        + 'unreadable-clone, for a page the repository holds, a page it lacks and a listing, while a link to a real '
        + 'clone still reads',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        await using links = await scratchDir({ prefix: 'translation-repair-clone-link-', },);
        /**
         Plain directory inside the throwaway repository.
         */
        const inner = join(
          clone.cloneDir,
          'not-a-clone',
        );
        await mkdir(inner,);
        /**
         Symbolic link to that directory, named as the clone, so the
         directory git starts in has no ancestor in common with the link's
         own parent but the scratch root.
         */
        const linkedInner = join(
          links.path,
          'linked-not-a-clone',
        );
        await symlink(
          inner,
          linkedInner,
        );
        /**
         Symbolic link to the real clone, the control that a linked clone
         still reads.
         */
        const linkedClone = join(
          links.path,
          'linked-clone',
        );
        await symlink(
          clone.cloneDir,
          linkedClone,
        );
        expect(await failureOf(async function readsThroughLinkedDirectory(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: linkedInner,
              commitSha: clone.commitSha,
            },
            relPath: `people/${PAIRED_ENTRY}/page.md`,
          },);
        },),).toEqual({
          kind: 'unreadable-clone',
          says: `CorpusReadError: corpus read failed for ${clone.commitSha}:people/${PAIRED_ENTRY}/page.md `
            + '(unreadable-clone); git could not open the clone: check that the directory exists, is the top of '
            + 'a git repository, and is one git may read.',
        },);
        expect((await failureOf(async function readsAbsentThroughLinkedDirectory(): Promise<unknown> {
          return await readCorpusFile({
            pin: {
              cloneDir: linkedInner,
              commitSha: clone.commitSha,
            },
            relPath: 'people/ghost/page.md',
          },);
        },)).kind,).toBe('unreadable-clone',);
        expect((await failureOf(async function listsThroughLinkedDirectory(): Promise<unknown> {
          return await listCorpusPeople({
            pin: {
              cloneDir: linkedInner,
              commitSha: clone.commitSha,
            },
          },);
        },)).kind,).toBe('unreadable-clone',);
        expect(await readCorpusFile({
          pin: {
            cloneDir: linkedClone,
            commitSha: clone.commitSha,
          },
          relPath: `people/${PAIRED_ENTRY}/page.en.md`,
        },),).toBe('The cat likes the sun.\n',);
      },
    },),

    it({
      name: 'REFUSES AS STATED A CLONE WHOSE PARENT\'S REAL PATH HOLDS A COLON before git reads anything, naming '
        + 'the clone and its parent and saying to move it, for a page, its bytes and a listing, since git splits '
        + 'its ceiling list at colons and would read a repository around the clone in its place',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        /**
         Plain directory inside the throwaway repository, under a parent whose
         name holds a colon.
         */
        const colonParent = join(
          clone.cloneDir,
          'tabby:mittens',
        );
        /**
         The directory named as the clone.
         */
        const inner = join(
          colonParent,
          'not-a-clone',
        );
        await mkdir(
          inner,
          { recursive: true, },
        );
        /**
         The pin over that directory, at the commit the repository around it
         holds.
         */
        const pin = {
          cloneDir: inner,
          commitSha: clone.commitSha,
        };
        /**
         The whole refusal each read must end in.
         */
        const expected = `StatedRefusalError: corpus clone ${inner} lies under ${await realpath(colonParent,)}, a `
          + 'path holding a colon, which git\'s ceiling list cannot carry since git splits it at colons, so git '
          + 'could read a repository around the clone in its place: move the clone to a directory whose path '
          + 'holds no colon.';
        /**
         What each read refused with.
         */
        const refusals = [
          await rejectionOf(async function readsUnderAColon(): Promise<unknown> {
            return await readCorpusFile({
              pin,
              relPath: `people/${PAIRED_ENTRY}/page.md`,
            },);
          },),
          await rejectionOf(async function readsBytesUnderAColon(): Promise<unknown> {
            return await readCorpusBytes({
              pin,
              relPath: `people/${PAIRED_ENTRY}/photos/intro.webp`,
            },);
          },),
          await rejectionOf(async function listsUnderAColon(): Promise<unknown> {
            return await listCorpusPeople({ pin, },);
          },),
        ];
        expect(refusals.map(function facts(refusal,): {
          readonly stated: boolean;
          readonly says: string;
        } {
          return {
            stated: refusal instanceof StatedRefusalError,
            says: String(refusal,),
          };
        },),).toEqual([
          {
            stated: true,
            says: expected,
          },
          {
            stated: true,
            says: expected,
          },
          {
            stated: true,
            says: expected,
          },
        ],);
      },
    },),
  ],
},);
