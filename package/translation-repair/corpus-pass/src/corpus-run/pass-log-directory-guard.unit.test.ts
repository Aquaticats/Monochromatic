/**
 Tests for the refusal of a default logger pointed into a run folder (ledger B1115). The whole pass is not run here;
 `corpus-pass-run.unit.test.ts` holds that the pass refuses to start.

 A DIRECTORY IS INSIDE THE RUN FOLDER BY WHERE IT REALLY IS: a link into the folder is inside, a directory that does not
 exist yet is judged by its nearest parent that does, and a sibling whose name begins with the folder's name is not.

 Every folder is made under a scratch directory. Fixtures are cat-themed invention.

 @module
 */

import {
  mkdir,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { refuseLogDirectoryInRunFolder, } from '../../dist/final/node/index.mjs';

await describe({
  name: refuseLogDirectoryInRunFolder.name,
  concurrency: 1,
  children: [
    it({
      name: 'ACCEPTS a launch that names no logger directory, an empty one, one beside the run folder whose name begins '
        + 'with the folder\'s own, and one above it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-log-directory-guard-', },);
        /**
         The run folder, and a sibling whose name begins with its name.
         */
        const runsDir = join(
          scratch.path,
          'tabby',
        );
        await mkdir(runsDir,);
        await mkdir(join(
          scratch.path,
          'tabby-other',
        ),);

        await refuseLogDirectoryInRunFolder({
          env: {},
          runsDir,
        },);
        await refuseLogDirectoryInRunFolder({
          env: { MONOCHROMATIC_LOG_DIRECTORY: '', },
          runsDir,
        },);
        await refuseLogDirectoryInRunFolder({
          env: {
            MONOCHROMATIC_LOG_DIRECTORY: join(
              scratch.path,
              'tabby-other',
            ),
          },
          runsDir,
        },);
        await refuseLogDirectoryInRunFolder({
          env: { MONOCHROMATIC_LOG_DIRECTORY: scratch.path, },
          runsDir,
        },);
      },
    },),

    it({
      name: 'REFUSES the run folder, a place in it, a place not made yet and a link into it, naming the variable, the '
        + 'directory and the run folder',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'pass-log-directory-guard-', },);
        /**
         The run folder, with a day directory in it, and a link to the day directory made outside.
         */
        const runsDir = join(
          scratch.path,
          'tabby',
        );
        const day = join(
          runsDir,
          'log',
          '2026-10-31',
        );
        await mkdir(
          day,
          { recursive: true, },
        );
        const link = join(
          scratch.path,
          'link-to-the-day',
        );
        await symlink(
          day,
          link,
        );
        /**
         Directories a logger may be told to write to, each of them in the run folder.
         */
        const inside = [
          runsDir,
          day,
          join(
            runsDir,
            'not',
            'made',
          ),
          link,
        ];
        for (const directory of inside) {
          // oxlint-disable-next-line no-await-in-loop -- one directory after the other, so a failing one is the one named
          const refusal = await rejectionOf(async function namesInside(): Promise<unknown> {
            return await refuseLogDirectoryInRunFolder({
              env: { MONOCHROMATIC_LOG_DIRECTORY: directory, },
              runsDir,
            },);
          },);

          expect(refusal,).toBeInstanceOf(StatedRefusalError,);
          expect(String(refusal,).startsWith(
            `StatedRefusalError: MONOCHROMATIC_LOG_DIRECTORY names ${directory}, which is the run folder ${runsDir} or inside it`,
          ),).toBe(true,);
        }
      },
    },),
  ],
},);
