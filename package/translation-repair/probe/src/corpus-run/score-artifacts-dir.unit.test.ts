/**
 Tests for the check the artifact-reading `score-*` runners make before they
 read: that the artifacts directory can be listed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { requireArtifactsDir, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: requireArtifactsDir.name,
  children: [
    it({
      name: 'ACCEPTS a directory that holds no artifact, leaving that refusal to the pool',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'score-artifacts-dir-', },);

        await expect(requireArtifactsDir({ artifactsDir: scratch.path, },),).resolves.toBeUndefined();
      },
    },),

    it({
      name: 'REFUSES a directory that is not there, naming its path and ENOENT',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'score-artifacts-dir-', },);

        /**
         Path no run ever wrote.
         */
        const artifactsDir = join(
          scratch.path,
          'nowhere',
        );

        /**
         What the check raised.
         */
        const refusal = await rejectionOf(async function checksAbsentDirectory(): Promise<void> {
          await requireArtifactsDir({ artifactsDir, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot list ${artifactsDir} (ENOENT): name a runs directory that holds an `
            + '`artifacts` directory with TRANSLATION_REPAIR_RUNS_DIR',
        );
      },
    },),

    it({
      name: 'REFUSES a path that is a file, naming its path and ENOTDIR',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'score-artifacts-dir-', },);

        /**
         File standing where the directory should be.
         */
        const artifactsDir = join(
          scratch.path,
          'artifacts',
        );
        await writeFile(
          artifactsDir,
          'not a directory\n',
          'utf8',
        );

        /**
         What the check raised.
         */
        const refusal = await rejectionOf(async function checksFile(): Promise<void> {
          await requireArtifactsDir({ artifactsDir, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot list ${artifactsDir} (ENOTDIR): name a runs directory that holds an `
            + '`artifacts` directory with TRANSLATION_REPAIR_RUNS_DIR',
        );
      },
    },),
  ],
},);
