/**
 Tests for the readers of the optional files beside a graded detection sheet.

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
import {
  preGradeName,
  readNamedOrBeside,
  readOptional,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: 'score-agreement-read',
  children: [
    describe({
      name: preGradeName.name,
      children: [
        it({
          name: 'NAMES the pre-grades file of a draw after its seed',
          fn: async () => {
            expect(preGradeName({ seed: 'round-cats', },),).toBe('pre-grades-round-cats.json',);
          },
        },),
      ],
    },),

    describe({
      name: readOptional.name,
      children: [
        it({
          name: 'READS a file that is there',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-read-', },);

            /**
             File the case wrote.
             */
            const path = join(
              scratch.path,
              'pre-grades.json',
            );
            await writeFile(
              path,
              '[]',
              'utf8',
            );

            expect(await readOptional({ path, label: 'pre-grades file', },),).toStrictEqual({
              found: true,
              text: '[]',
            },);
          },
        },),

        it({
          name: 'NAMES a file that is not there as absent',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-read-', },);

            expect(await readOptional({
              path: join(
                scratch.path,
                'pre-grades.json',
              ),
              label: 'pre-grades file',
            },),).toStrictEqual({ found: false, },);
          },
        },),

        it({
          name: 'REFUSES a path that is a directory in its own words, naming its path and EISDIR',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-read-', },);

            /**
             What reading the directory raised.
             */
            const refusal = await rejectionOf(async function readsDirectory(): Promise<void> {
              await readOptional({ path: scratch.path, label: 'pre-grades file', },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: cannot read the pre-grades file at ${scratch.path} (EISDIR)`,
            );
          },
        },),
      ],
    },),

    describe({
      name: readNamedOrBeside.name,
      children: [
        it({
          name: 'READS a named file that is there',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-read-', },);

            /**
             File the case wrote.
             */
            const path = join(
              scratch.path,
              'manifest.json',
            );
            await writeFile(
              path,
              '{}',
              'utf8',
            );

            expect(await readNamedOrBeside({
              path,
              label: 'sample manifest',
              flag: 'manifest',
              named: true,
            },),).toStrictEqual({
              found: true,
              text: '{}',
            },);
          },
        },),

        it({
          name: 'NAMES a file nobody named and nothing is there as absent',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-read-', },);

            expect(await readNamedOrBeside({
              path: join(
                scratch.path,
                'manifest.json',
              ),
              label: 'sample manifest',
              flag: 'manifest',
              named: false,
            },),).toStrictEqual({ found: false, },);
          },
        },),

        it({
          name: 'REFUSES a named file that is not there, naming the flag and the path',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-agreement-read-', },);

            /**
             Path nothing was written to.
             */
            const path = join(
              scratch.path,
              'manifest.json',
            );

            /**
             What reading it raised.
             */
            const refusal = await rejectionOf(async function readsAbsentNamed(): Promise<void> {
              await readNamedOrBeside({
                path,
                label: 'sample manifest',
                flag: 'manifest',
                named: true,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: --manifest names ${path}, which is not there; name a sample manifest that exists, `
                + 'or leave the flag out to look beside the sheet',
            );
          },
        },),
      ],
    },),
  ],
},);
