/**
 Tests for the lifetime of the files one draw writes.

 A draw produces three outputs that only mean anything together, and a final
 path is refused once it exists. A fault between writes therefore leaves a
 partial set that the refusal then blocks the next draw on, and the obstacle
 is a file nobody ever graded. These cases pin that a failed draw leaves
 nothing behind and a finished one leaves everything.

 Every case runs against a throwaway directory from `mkdtemp`, never the real
 runs directory, because the behaviour under test is file REMOVAL.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { trackDrawOutputs, } from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: trackDrawOutputs.name,
  children: [
    it({
      name: 'removes what an UNCOMMITTED draw wrote, so a fault between the '
        + 'three writes leaves no partial set. The overwrite guard refuses a '
        + 'final path that exists, so a half-written set would block every '
        + 'later draw on files nobody had graded',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);
        /**
         Sheet the draw got as far as writing.
         */
        const sheet = join(
          scratch.path,
          'grading-sheet.md',
        );

        {
          await using outputs = trackDrawOutputs({ enabled: true, },);
          await writeFile(
            sheet,
            'partial',
          );
          outputs.record({ path: sheet, },);
          // Scope ends WITHOUT commit, standing in for a throw partway.
        }

        expect(await readdir(scratch.path,),).toEqual([],);
      },
    },),

    it({
      name: 'keeps everything a COMMITTED draw wrote, since a complete set is '
        + 'the record of one draw and the manifest inside it is the only thing '
        + 'that can ever join a human grade to a machine verdict',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);

        {
          await using outputs = trackDrawOutputs({ enabled: true, },);
          /**
           Every output of the completed set.
           */
          const paths = [
            'grading-sheet.md',
            'repair-sheet.md',
            'sample-manifest.json',
          ].map(function toPath(name,) {
            return join(
              scratch.path,
              name,
            );
          },);
          await Promise.all(paths.map(function writeOne(path,) {
            return writeFile(
              path,
              'done',
            );
          },),);
          for (const path of paths)
            outputs.record({ path, },);
          outputs.commit();
        }

        expect((await readdir(scratch.path,)).toSorted(),).toEqual([
          'grading-sheet.md',
          'repair-sheet.md',
          'sample-manifest.json',
        ],);
      },
    },),

    it({
      name: 'removes only what it RECORDED, leaving files it never wrote. The '
        + 'runs directory holds earlier rounds\' graded sheets, and a cleanup '
        + 'that swept the directory rather than its own list would destroy '
        + 'exactly the work the overwrite guard exists to protect',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);
        /**
         Graded sheet from an earlier round, untouched by this draw.
         */
        const earlier = join(
          scratch.path,
          'grading-sheet-round-two.md',
        );
        await writeFile(
          earlier,
          'hours of human grading',
        );

        {
          await using outputs = trackDrawOutputs({ enabled: true, },);
          /**
           This draw's own output.
           */
          const mine = join(
            scratch.path,
            'grading-sheet-round-three.md',
          );
          await writeFile(
            mine,
            'partial',
          );
          outputs.record({ path: mine, },);
        }

        expect(await readdir(scratch.path,),).toEqual([
          'grading-sheet-round-two.md',
        ],);
      },
    },),

    it({
      name: 'disposes cleanly when a recorded file is already gone, which is '
        + 'the ordinary shape of the failure: the write that threw is the one '
        + 'whose file may never have been created',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);

        await using outputs = trackDrawOutputs({ enabled: true, },);
        outputs.record({
          path: join(
            scratch.path,
            'never-written.md',
          ),
        },);

        // Disposing must not throw: reaching the `readdir` expectation after it is the assertion.
        await outputs[Symbol.asyncDispose]();
        expect(await readdir(scratch.path,),).toEqual([],);
      },
    },),
    it({
      name: 'records nothing when DISABLED, which is what a preliminary draw '
        + 'needs. Preliminary outputs are written with `w` and replace whatever '
        + 'was there, so removing one on failure would delete a file this '
        + 'invocation never created; and they carry no overwrite guard, so a '
        + 'partial one is replaced by the next draw rather than blocking it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);
        /**
         Preliminary sheet standing in for one an earlier draw left behind.
         */
        const sheet = join(
          scratch.path,
          'grading-sheet-preliminary.md',
        );

        {
          await using outputs = trackDrawOutputs({ enabled: false, },);
          await writeFile(
            sheet,
            'replaced contents',
          );
          outputs.record({ path: sheet, },);
          // Ends without commit, exactly as the failing case does.
        }

        expect(await readdir(scratch.path,),).toEqual([
          'grading-sheet-preliminary.md',
        ],);
      },
    },),

    it({
      name: 'removes a file recorded BEFORE its write, which is the only order '
        + 'that catches a write that created the file and then failed. '
        + 'Recording on success alone would leave exactly that file behind, and '
        + 'it is the one the overwrite guard then refuses to replace',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);
        /**
         Path recorded first, then written only partially.
         */
        const sheet = join(
          scratch.path,
          'grading-sheet.md',
        );

        {
          await using outputs = trackDrawOutputs({ enabled: true, },);
          outputs.record({ path: sheet, },);
          await writeFile(
            sheet,
            'truncated before the failure',
          );
        }

        expect(await readdir(scratch.path,),).toEqual([],);
      },
    },),
    it({
      name: 'keeps a file it RELEASED, which an exclusive create found already standing, and removes the rest',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-draw-', },);
        /**
         Sheet some other draw made, which this draw's exclusive create found taken.
         */
        const taken = join(
          scratch.path,
          'grading-sheet.md',
        );
        /**
         Sheet this draw made before the next one failed.
         */
        const own = join(
          scratch.path,
          'repair-sheet.md',
        );
        await writeFile(
          taken,
          'graded by someone else',
        );

        {
          await using outputs = trackDrawOutputs({ enabled: true, },);
          outputs.record({ path: own, },);
          await writeFile(
            own,
            'this draw\'s',
          );
          outputs.record({ path: taken, },);
          outputs.release({ path: taken, },);
        }

        expect(await readdir(scratch.path,),).toEqual(['grading-sheet.md',],);
      },
    },),
  ],
},);
