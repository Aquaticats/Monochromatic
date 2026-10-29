/**
 Tests for writing an artifact no concurrent reader can catch half-written.

 The window is small and the consequence is a silently wrong denominator: a
 partial file is classified as malformed, the pool keeps malformed files on
 purpose so the reader that reports them still sees them, and a later reader
 parses the now-complete file and counts it without the generation checks it
 should have faced.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { writeFileAtomic, } from '../../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Characters in the uuid a temporary file's name carries.
 */
const UUID_LENGTH = 36;

await describe({
  name: writeFileAtomic.name,
  concurrency: 1,
  children: [
    it({
      name: 'writes the content under the name asked for, which is the whole '
        + 'contract a caller sees: the temporary name is an implementation '
        + 'detail no reader may ever meet',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        const dir = scratch.path;

        /**
         Path the artifact takes.
         */
        const path = join(
          dir,
          'Mittens.json',
        );

        await writeFileAtomic({
          path,
          text: '{"id":"Mittens"}\n',
        },);

        expect(await readFile(
          path,
          'utf8',
        ),).toBe('{"id":"Mittens"}\n',);
      },
    },),

    it({
      name: 'LEAVES NO partial file behind, since a leftover would sit in the '
        + 'artifacts directory the readers glob and would be counted by every '
        + 'listing that keys on a name rather than on content',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        const dir = scratch.path;

        await writeFileAtomic({
          path: join(
            dir,
            'Mittens.json',
          ),
          text: '{"id":"Mittens"}\n',
        },);

        expect(await readdir(dir,),).toEqual(['Mittens.json',],);
      },
    },),

    it({
      name: 'replaces an existing artifact rather than appending to it or '
        + 'refusing, which is what a re-settled entry needs',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        const dir = scratch.path;

        /**
         Path holding an older artifact for the same entry.
         */
        const path = join(
          dir,
          'Mittens.json',
        );
        await writeFile(
          path,
          '{"id":"Mittens","status":"stale"}\n',
        );

        await writeFileAtomic({
          path,
          text: '{"id":"Mittens","status":"repaired"}\n',
        },);

        expect(await readFile(
          path,
          'utf8',
        ),).toBe('{"id":"Mittens","status":"repaired"}\n',);
        expect(await readdir(dir,),).toEqual(['Mittens.json',],);
      },
    },),

    it({
      name: 'writes bytes the artifacts a real pass produces are made of, '
        + 'including multi-byte text, so the rename path is exercised on a '
        + 'payload of the shape it actually carries rather than on ASCII alone',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        const dir = scratch.path;

        /**
         Path the artifact takes.
         */
        const path = join(
          dir,
          'Pepper.json',
        );

        /**
         Artifact body carrying text outside the ASCII range.
         */
        const text = `${JSON.stringify({
          id: 'Pepper',
          note: 'ペッパーは窓辺で眠る',
        },)}\n`;

        await writeFileAtomic({
          path,
          text,
        },);

        expect(await readFile(
          path,
          'utf8',
        ),).toBe(text,);
      },
    },),

    it({
      name: 'LEAVES NOTHING BESIDE THE PATH when the rename is refused, a directory standing where the file '
        + 'belongs, and rejects with the rename\'s own refusal, where the whole temporary file stayed in the '
        + 'directory for every later listing to step over',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        const dir = scratch.path;

        /**
         Path the artifact takes, a directory standing there.
         */
        const path = join(
          dir,
          'Mittens.json',
        );
        await mkdir(path,);

        /**
         What the write rejected with.
         */
        const refusal = await rejectionOf(async function overADirectory(): Promise<void> {
          await writeFileAtomic({
            path,
            text: '{"id":"Mittens"}\n',
          },);
        },);

        // Read by code, call and destination: its message names the temporary
        // file, whose name no caller is given.
        expect((Error.isError(refusal,) && ('code' in refusal) && ('syscall' in refusal) && ('dest' in refusal))
          ? {
            code: refusal.code,
            syscall: refusal.syscall,
            dest: refusal.dest,
          }
          : refusal,).toEqual({
            code: 'EISDIR',
            syscall: 'rename',
            dest: path,
          },);
        expect(await readdir(dir,),).toEqual(['Mittens.json',],);
      },
    },),

    it({
      name: 'LANDS BOTH OF TWO WRITES OF ONE PATH STARTED AT ONCE in one process, one text whole at the path and '
        + 'nothing beside it, where the two shared one temporary file, so the later rename found it gone and '
        + 'the bytes of the two could interleave',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        const dir = scratch.path;

        /**
         Path both writes take.
         */
        const path = join(
          dir,
          'Pepper.json',
        );

        /**
         The two texts, of different lengths so an interleave would show.
         */
        const texts = [
          `${JSON.stringify({
            id: 'Pepper',
            naps: Array.from(
              { length: 4_000, },
              function napAt(_unused, index,): number {
                return index;
              },
            ),
          },)}\n`,
          '{"id":"Pepper"}\n',
        ];

        /**
         How each write settled.
         */
        const settled = await Promise.allSettled(texts.map(async function write(text,): Promise<void> {
          await writeFileAtomic({
            path,
            text,
          },);
        },),);

        /**
         What the path holds once both settled.
         */
        const landed = await readFile(
          path,
          'utf8',
        );

        expect(settled.map(function statusOf(outcome,): string {
          return outcome.status;
        },),).toEqual(['fulfilled', 'fulfilled',],);
        expect(texts.includes(landed,),).toBe(true,);
        expect(await readdir(dir,),).toEqual(['Pepper.json',],);
      },
    },),

    it({
      name: 'REJECTS WITH THE WRITE\'S OWN REFUSAL and warns that the temporary file could not be removed, a name '
        + 'too long for the filesystem refusing the write and the removal alike',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'atomic-write-', },);
        /**
         Path whose file name leaves no room for the temporary name's suffix.
         */
        const path = join(
          scratch.path,
          `${'whiskers'.repeat(30,)}.json`,
        );

        const {
          result: refusal,
          warned,
        } = await warnLinesDuring({
          run: async function overlong(): Promise<unknown> {
            return await rejectionOf(async function writeOverlong(): Promise<void> {
              await writeFileAtomic({
                path,
                text: '{"id":"Whiskers"}\n',
              },);
            },);
          },
        },);

        expect((Error.isError(refusal,) && ('code' in refusal) && ('syscall' in refusal))
          ? {
            code: refusal.code,
            syscall: refusal.syscall,
          }
          : refusal,).toEqual({
            code: 'ENAMETOOLONG',
            syscall: 'open',
          },);
        expect(warned.length,).toBe(1,);
        /**
         What the warning says before and after the unique temporary name the write chose: this process's
         id and a uuid, which the case cannot know.
         */
        const before = `[translation-repair] [removePartial] ${path}.${String(process.pid,)}.`;
        const after = '.partial is left after a failed write and could not be removed (ENAMETOOLONG)';
        const [line = '',] = warned;
        expect({
          opening: line.slice(
            0,
            before.length,
          ),
          closing: line.slice(line.length - after.length,),
          uniqueName: line.slice(
            before.length,
            line.length - after.length,
          ).length,
        },).toEqual({
          opening: before,
          closing: after,
          uniqueName: UUID_LENGTH,
        },);
      },
    },),
  ],
},);
