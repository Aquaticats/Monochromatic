/**
 Tests for the listing of settled artifacts both sampling commands start from.

 WHAT THE LISTING OWES AN OPERATOR who named the wrong runs directory: a
 refusal in words that say which directory is missing and which variable names
 it, rather than the filesystem's own error reported as a fault in the command.
 Only regular files named like an artifact are names, as the listing's module
 promises.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { listSettledNames, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: listSettledNames.name,
  children: [
    it({
      name: 'LISTS the regular files named like an artifact and nothing else the directory holds',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sample-names-', },);
        const artifactsDir = join(
          scratch.path,
          'artifacts',
        );
        await mkdir(join(
          artifactsDir,
          'backup.json',
        ),{ recursive: true, },);
        await writeFile(
          join(
            artifactsDir,
            'mittens.json',
          ),
          '{}',
          'utf8',
        );
        await writeFile(
          join(
            artifactsDir,
            'notes.txt',
          ),
          'purr',
          'utf8',
        );

        expect(await listSettledNames({ runsDir: scratch.path, },),).toEqual(['mittens.json',],);
      },
    },),
    it({
      name: 'LISTS nothing for an artifacts directory that holds no file',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sample-names-', },);
        await mkdir(join(
          scratch.path,
          'artifacts',
        ),);

        expect(await listSettledNames({ runsDir: scratch.path, },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES as stated, naming the missing directory and the variable, when the runs directory holds none',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sample-names-', },);

        const refusal = await rejectionOf(async function listAbsent(): Promise<void> {
          await listSettledNames({ runsDir: scratch.path, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: there is no artifacts directory at ${join(scratch.path, 'artifacts',)}; name the `
            + 'runs directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR',
        );
      },
    },),
    it({
      name: 'REFUSES as stated, naming the filesystem code, when the artifacts directory is a file',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'sample-names-', },);
        await writeFile(
          join(
            scratch.path,
            'artifacts',
          ),
          'purr',
          'utf8',
        );

        const refusal = await rejectionOf(async function listBeneathAFile(): Promise<void> {
          await listSettledNames({ runsDir: scratch.path, },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: the artifacts directory at ${join(scratch.path, 'artifacts',)} could not be listed `
            + '(ENOTDIR); name the runs directory a pass settled entries into with TRANSLATION_REPAIR_RUNS_DIR',
        );
      },
    },),
  ],
},);
