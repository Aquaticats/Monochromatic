/**
 Tests for the coverage probe's read of one entry: both sides when both are
 there, and a skip, logged, when either is not.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CorpusReadError, } from '@monochromatic-dev/translation-repair-corpus-source';
import { readPair, } from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  LACKED_COMMIT_SHA,
  makeCloneHoldingOneCommit,
  PAIRED_ENTRY,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-lacked-commit.test-fixture.ts';
import { makeProbeCorpus, } from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';

await describe({
  name: readPair.name,
  children: [
    it({
      name: 'READS both sides of an entry that holds both, logging nothing',
      fn: async () => {
        await using corpus = await makeProbeCorpus({
          files: {
            'people/Mittens/page.md': 'zh Mittens\n',
            'people/Mittens/page.en.md': 'en Mittens\n',
          },
        },);
        const { logger, lines, } = capturingLoggerPair();

        expect(await readPair({
          pin: corpus.pin,
          entryId: 'Mittens',
          log: logger,
        },),).toEqual({
          kind: 'read',
          source: 'zh Mittens\n',
          target: 'en Mittens\n',
        },);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'SKIPS an entry that lacks its translation, logging the refusal that said so',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/Mittens/page.md': 'zh Mittens\n', }, },);
        const { logger, lines, } = capturingLoggerPair();

        expect(await readPair({
          pin: corpus.pin,
          entryId: 'Mittens',
          log: logger,
        },),).toEqual({ kind: 'missing', },);
        expect(lines,).toEqual([
          `Mittens: skipped, CorpusReadError: corpus read failed for ${corpus.commitSha}:people/Mittens/page.en.md `
            + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.',
        ],);
      },
    },),
    it({
      name: 'SKIPS an entry that lacks its original, logging the refusal that said so',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/Mittens/page.en.md': 'en Mittens\n', }, },);
        const { logger, lines, } = capturingLoggerPair();

        expect(await readPair({
          pin: corpus.pin,
          entryId: 'Mittens',
          log: logger,
        },),).toEqual({ kind: 'missing', },);
        expect(lines,).toEqual([
          `Mittens: skipped, CorpusReadError: corpus read failed for ${corpus.commitSha}:people/Mittens/page.md `
            + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.',
        ],);
      },
    },),
    it({
      name: 'REFUSES a pin whose commit the clone lacks, naming the commit and logging nothing, instead of '
        + 'skipping the entry as one with a single side',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the read of an entry at a commit the clone lacks rejected with.
         */
        const refusal = await rejectionOf({
          promise: readPair({
            pin: {
              cloneDir: clone.cloneDir,
              commitSha: LACKED_COMMIT_SHA,
            },
            entryId: PAIRED_ENTRY,
            log: logger,
          },),
        },);

        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect((refusal instanceof CorpusReadError) && refusal.kind,).toBe('missing-commit',);
        expect(String(refusal,),).toBe(
          `CorpusReadError: corpus read failed for ${LACKED_COMMIT_SHA}:people/${PAIRED_ENTRY}/page.md `
            + '(missing-commit); the clone holds no commit by that revision: fetch it, or pin '
            + 'a commit the clone holds.',
        );
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES a clone directory that does not exist, logging nothing, instead of skipping the entry',
      fn: async () => {
        await using clone = await makeCloneHoldingOneCommit();
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the read of an entry in a clone directory that does not exist
         rejected with.
         */
        const refusal = await rejectionOf({
          promise: readPair({
            pin: {
              cloneDir: `${clone.cloneDir}/no-such-clone`,
              commitSha: clone.commitSha,
            },
            entryId: PAIRED_ENTRY,
            log: logger,
          },),
        },);

        expect(refusal,).toBeInstanceOf(CorpusReadError,);
        expect((refusal instanceof CorpusReadError) && refusal.kind,).toBe('unreadable-clone',);
        expect(String(refusal,),).toBe(
          `CorpusReadError: corpus read failed for ${clone.commitSha}:people/${PAIRED_ENTRY}/page.md `
            + '(unreadable-clone); git could not open the clone: check that the directory exists, is the top of a '
            + 'git repository, and is one git may read.',
        );
        expect(lines,).toEqual([],);
      },
    },),
  ],
},);
