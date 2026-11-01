/**
 Tests for the walk that decides which entries a pass may work on.

 WHAT THESE PIN is the difference between an entry missing a side, which is
 an ordinary state of this corpus and is named, and any other read failure,
 which is a fault in the run and propagates. Until this module every read
 failure was stepped past as a missing side, so a clone that had gone away
 read as a corpus with no entries.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { devNull, } from 'node:os';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { spawnKeyless, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import { collectEligiblePairs, } from '../../dist/final/node/index.mjs';
import { LACKED_COMMIT_SHA, } from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-lacked-commit.test-fixture.ts';
import { fixtureGit, REAL_GIT, } from '@monochromatic-dev/translation-repair-adjudication/ts/hermetic-git-run.test-fixture.ts';
import { scratchDirWith, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Pages the throwaway clone carries, by entry then side.
 */
const PAGES: Readonly<Record<string, Readonly<Record<'page.md' | 'page.en.md', string>>>> = {
  whiskers: {
    'page.md': '## 简介\n\n猫猫喜欢晒太阳。\n',
    'page.en.md': '## Introduction\n\nWhiskers likes to sun herself.\n',
  },
  mittens: {
    'page.md': '## 简介\n\n猫猫在窗台上睡觉。\n',
    'page.en.md': '## Introduction\n\nMittens sleeps on the sill.\n',
  },
};

/**
 Entry the clone carries only the original of.
 */
const HALF_ENTRY = 'tabby';

/**
 Makes a throwaway corpus clone with two complete entries and one that has
 only its original, at one commit.

 @returns Clone directory and commit, removed on dispose

 @example
 ```ts
 await using corpus = await throwawayCorpus();
 ```
 */
async function throwawayCorpus(): Promise<
  AsyncDisposable & {
    readonly cloneDir: string;
    readonly commitSha: string;
  }
  > {
  // Where the clone lives.
  return await scratchDirWith({
    prefix: 'pass-eligibility-',
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

      /**
       Every path written, for explicit staging.
       */
      const written: string[] = [];
      for (const [entryId, pages,] of Object.entries(PAGES,)) {
        for (const [name, text,] of Object.entries(pages,)) {
          /**
           Path within the clone.
           */
          const relPath = `people/${entryId}/${name}`;
          written.push(relPath,);
          /* oxlint-disable-next-line no-await-in-loop -- fixture setup writes a handful of files in order */
          await mkdir(
            join(
              cloneDir,
              'people',
              entryId,
            ),
            { recursive: true, },
          );
          /* oxlint-disable-next-line no-await-in-loop -- fixture setup writes a handful of files in order */
          await writeFile(
            join(
              cloneDir,
              relPath,
            ),
            text,
            'utf8',
          );
        }
      }
      await mkdir(
        join(
          cloneDir,
          'people',
          HALF_ENTRY,
        ),
        { recursive: true, },
      );
      await writeFile(
        join(
          cloneDir,
          'people',
          HALF_ENTRY,
          'page.md',
        ),
        '## 简介\n\n猫猫有自己的碗。\n',
        'utf8',
      );
      written.push(`people/${HALF_ENTRY}/page.md`,);
      await fixtureGit({
        cloneDir,
        args: [
          'add',
          ...written,
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
          'add cats',
          '--no-gpg-sign',
          '--',
          ...written,
        ],
      },);

      /**
       Commit the entries sit at.
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

await describe({
  name: collectEligiblePairs.name,
  children: [
    it({
      name:
        'SORTS the walk into complete pairs to run, settled sizes, named gaps and every complete pair at the '
          + 'pin: a settled entry is sized and counted a pair but not run, and the entry missing its translation '
          + 'is reported by name and side rather than dropped in silence',
      fn: async () => {
        await using corpus = await throwawayCorpus();

        /**
         The walk over every entry, with one already settled.
         */
        const walked = await collectEligiblePairs({
          ids: [
            'whiskers',
            'mittens',
            HALF_ENTRY,
          ],
          done: new Set(['mittens',],),
          pin: {
            cloneDir: corpus.cloneDir,
            commitSha: corpus.commitSha,
          },
        },);
        expect(walked.eligible
          .map(function toId(pair,): string {
            return pair.id;
          },),).toEqual(['whiskers',],);
        expect(walked.eligible[0]?.targetText,).toContain('Whiskers',);
        expect(walked.settled
          .map(function toId(entry,): string {
            return entry.id;
          },),).toEqual(['mittens',],);
        expect(walked.settled[0]?.sourceBytes,).toBeGreaterThan(0,);
        expect(walked.incomplete
          .map(function toGap(gap,): string {
            return `${gap.id}:${gap.side}`;
          },),).toEqual([`${HALF_ENTRY}:target`,],);
        expect(walked.incomplete[0]?.detail,).toContain('missing-object',);
        expect(walked.paired,).toEqual([
          'whiskers',
          'mittens',
        ],);
      },
    },),
    it({
      name: 'SIZES A SETTLED ENTRY WHOSE ENGLISH PAGE THE PIN LACKS for ranking and counts it no complete pair, '
        + 'naming no gap for an entry that runs no more',
      fn: async () => {
        await using corpus = await throwawayCorpus();

        /**
         The walk over a settled entry whose English page the pin lacks.
         */
        const walked = await collectEligiblePairs({
          ids: [
            'whiskers',
            HALF_ENTRY,
          ],
          done: new Set([HALF_ENTRY,],),
          pin: {
            cloneDir: corpus.cloneDir,
            commitSha: corpus.commitSha,
          },
        },);
        expect(walked.settled
          .map(function toId(entry,): string {
            return entry.id;
          },),).toEqual([HALF_ENTRY,],);
        expect(walked.incomplete,).toEqual([],);
        expect(walked.paired,).toEqual(['whiskers',],);
      },
    },),
    it({
      name:
        'NAMES an entry whose original is absent as missing its source side, without reading the translation',
      fn: async () => {
        await using corpus = await throwawayCorpus();

        /**
         The walk over an entry the clone never carried.
         */
        const walked = await collectEligiblePairs({
          ids: ['calico',],
          done: new Set(),
          pin: {
            cloneDir: corpus.cloneDir,
            commitSha: corpus.commitSha,
          },
        },);
        expect(walked.eligible,).toEqual([],);
        expect(walked.incomplete
          .map(function toGap(gap,): string {
            return `${gap.id}:${gap.side}`;
          },),).toEqual(['calico:source',],);
      },
    },),
    it({
      name:
        'PROPAGATES any other read failure, since an unreadable clone is a fault in the run and a walk that '
          + 'stepped past it would report a smaller corpus than exists: this is the drop the register found',
      fn: async () => {
        /**
         What the walk over a clone that does not exist raised.
         */
        let caught: unknown;
        try {
          await collectEligiblePairs({
            ids: ['whiskers',],
            done: new Set(),
            pin: {
              cloneDir: join(
                scratchRoot(),
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
      name: 'REFUSES a pin whose commit the clone lacks, naming the commit and the first page, instead of '
        + 'naming every entry as missing its source side',
      fn: async () => {
        await using corpus = await throwawayCorpus();

        /**
         What the walk over a commit the clone lacks raised.
         */
        let caught: unknown;
        try {
          await collectEligiblePairs({
            ids: [
              'whiskers',
              'mittens',
            ],
            done: new Set(),
            pin: {
              cloneDir: corpus.cloneDir,
              commitSha: LACKED_COMMIT_SHA,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CorpusReadError,);
        expect((caught instanceof CorpusReadError) && caught.kind,).toBe('missing-commit',);
        expect(String(caught,),).toBe(
          `CorpusReadError: corpus read failed for ${LACKED_COMMIT_SHA}:people/whiskers/page.md (missing-commit); `
            + 'the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        );
      },
    },),
  ],
},);
