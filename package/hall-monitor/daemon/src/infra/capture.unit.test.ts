/**
 Handoff tests for the screenshot capture path.

 `captureScreenshot` hands a PNG from Spectacle to ffmpeg through a file on
 disk, so the file's location decides who can influence it. These cases prove
 the handoff lives in a directory this process just created for itself, private
 to this account and unpredictable to any other, that the parent it is created
 in is never a shared one, and that releasing the handoff removes it again.

 Every case releases the directory it created, so no case leaves one behind.

 @module
 */

import { stat, } from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createScreenshotHandoff,
  handoffParentDirectory,
  type ScreenshotHandoff,
} from '../../dist/final/node/testing.mjs';

//region Fixtures

/**
 Permission bits a private handoff directory must carry.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 Permission bits a path exposes once the file-type bits are masked away.
 */
const PERMISSION_BITS = 0o777;

/**
 Name the handoff file must carry inside its directory.
 */
const EXPECTED_HANDOFF_FILE_NAME = 'screen.png';

/**
 Parent directory cases create their handoff directories in, so no fixture lands
 in the system temporary directory.
 */
const FIXTURE_PARENT = scratchRoot();

/**
 Releases one handoff when its scope ends, so a case needs no `try`/`finally`.

 @param handoff - handoff to release

 @returns disposable releasing the handoff directory
 */
function releasingHandoff({ handoff, }: Readonly<{ handoff: ScreenshotHandoff; }>,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await handoff.release();
    },
  };
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: handoffParentDirectory.name,
      children: [
        it({
          name: 'returns the per-user runtime directory it is given',
          fn: async () => {
            expect(handoffParentDirectory({
              environment: { XDG_RUNTIME_DIR: '/run/user/1000', },
            },),).toBe('/run/user/1000',);
          },
        },),

        it({
          name: 'refuses a missing runtime directory rather than falling back to a shared one',
          fn: async () => {
            expect(() => handoffParentDirectory({ environment: {}, },),).toThrow();
          },
        },),

        it({
          name: 'refuses an empty runtime directory',
          fn: async () => {
            expect(() => handoffParentDirectory({
              environment: { XDG_RUNTIME_DIR: '', },
            },),).toThrow();
          },
        },),
      ],
    },),

    describe({
      name: createScreenshotHandoff.name,
      children: [
        it({
          name: 'puts the handoff file in a directory only this account can reach',
          fn: async () => {
            /**
             Handoff under test, released when this case ends.
             */
            const handoff = await createScreenshotHandoff({ parentDirectory: FIXTURE_PARENT, },);
            await using _handoff = releasingHandoff({ handoff, },);

            /**
             Directory holding the handoff file.
             */
            const directory = dirname(handoff.pngPath,);
            /**
             Directory metadata the privacy claim rests on.
             */
            const metadata = await stat(directory,);

            expect(metadata.isDirectory(),).toBe(true,);
            expect((metadata.mode & PERMISSION_BITS),).toBe(PRIVATE_DIRECTORY_MODE,);
            expect(dirname(directory,),).toBe(FIXTURE_PARENT,);
            expect(directory,).not.toBe(FIXTURE_PARENT,);
          },
        },),

        it({
          name: 'creates a fresh directory per handoff, so no path can be pre-planted',
          fn: async () => {
            /**
             First handoff, released when this case ends.
             */
            const first = await createScreenshotHandoff({ parentDirectory: FIXTURE_PARENT, },);
            await using _first = releasingHandoff({ handoff: first, },);
            /**
             Second handoff, released when this case ends.
             */
            const second = await createScreenshotHandoff({ parentDirectory: FIXTURE_PARENT, },);
            await using _second = releasingHandoff({ handoff: second, },);

            expect(dirname(first.pngPath,),).not.toBe(dirname(second.pngPath,),);
          },
        },),

        it({
          name: 'names the handoff file inside that directory, with nothing there yet',
          fn: async () => {
            /**
             Handoff under test, released when this case ends.
             */
            const handoff = await createScreenshotHandoff({ parentDirectory: FIXTURE_PARENT, },);
            await using _handoff = releasingHandoff({ handoff, },);

            expect(basename(handoff.pngPath,),).toBe(EXPECTED_HANDOFF_FILE_NAME,);
            expect(dirname(handoff.pngPath,),).not.toBe(FIXTURE_PARENT,);
            await expect(stat(handoff.pngPath,),).rejects.toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'removes the whole handoff directory when released',
          fn: async () => {
            /**
             Handoff under test, released inside this case.
             */
            const handoff = await createScreenshotHandoff({ parentDirectory: FIXTURE_PARENT, },);
            /**
             Directory the handoff file was placed in.
             */
            const directory = dirname(handoff.pngPath,);

            await handoff.release();

            await expect(stat(directory,),).rejects.toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'releases a handoff directory whose file was never written',
          fn: async () => {
            /**
             Handoff under test, released when this case ends; no capture ever
             writes its file, which is the path a failing Spectacle leaves.
             */
            const handoff = await createScreenshotHandoff({ parentDirectory: FIXTURE_PARENT, },);
            await using _handoff = releasingHandoff({ handoff, },);

            expect(join(dirname(handoff.pngPath,), EXPECTED_HANDOFF_FILE_NAME,),).toBe(handoff.pngPath,);
          },
        },),
      ],
    },),
  ],
},);
