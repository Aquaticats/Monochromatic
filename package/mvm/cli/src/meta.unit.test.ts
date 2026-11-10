/**
 Tests of the VM metadata reader's failure paths.

 `readVmMeta` may rebuild metadata from the legacy `image` file only when
 `meta.json` is absent. A file that exists but cannot be read or parsed holds
 metadata this reader cannot see, and rebuilding from the legacy file then
 hands the caller a different guest than the VM was made for: file transfers
 pick a share the VM may not have, and exec picks the wrong shell.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_IMAGE,
  IMAGES,
  type ImageSpec,
  readVmMeta,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 File name the metadata reader looks for first.
 */
const META_FILE_NAME = 'meta.json';

/**
 File name of the legacy image identifier the reader falls back to.
 */
const LEGACY_IMAGE_FILE_NAME = 'image';

/**
 Registry image identifier that differs from {@link DEFAULT_IMAGE}, so a
 fallback to the legacy file is visible as a different OS family.
 */
const LEGACY_IMAGE = 'windows';

/**
 Spec of {@link LEGACY_IMAGE}, which the expectations below derive their
 values from.

 @returns the registry entry the legacy file names

 @throws {Error} when the registry no longer lists {@link LEGACY_IMAGE}
 */
function legacyImageSpec(): ImageSpec {
  /**
   Registry entry the test's legacy file names.
   */
  const spec = IMAGES[LEGACY_IMAGE];
  if (spec === undefined)
    throw new Error(`the image registry no longer lists ${LEGACY_IMAGE}`,);
  return spec;
}

/**
 Disposable VM directory for one test.

 @param prefix - name prefix inside the scratch root

 @returns the directory path and the disposal that removes it
 */
async function scratchVmDirectory(prefix: string,): Promise<{
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   Absolute path of the disposable VM directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function remove(): Promise<void> {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Writes text into a file of the VM directory.

 @param args - directory, file name, and content
 */
async function writeVmFile({
  content,
  fileName,
  vmDirectory,
}: {
  readonly content: string;
  readonly fileName: string;
  readonly vmDirectory: string;
},): Promise<void> {
  await writeFile(
    join(
      vmDirectory,
      fileName,
    ),
    content,
  );
}

/**
 Metadata record every field of, written as `writeVmMeta` spells it.

 @param overrides - fields to leave out or replace for a shape the reader must refuse

 @returns JSON text for `meta.json`
 */
function metaJson(overrides: Readonly<Record<string, unknown>> = {},): string {
  return JSON.stringify({
    createdAt: '2026-11-10T00:00:00.000Z',
    defaultUser: 'ubuntu',
    fileTransfer: 'virtiofs',
    image: DEFAULT_IMAGE,
    osFamily: 'linux',
    shell: '/bin/bash',
    ...overrides,
  },);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: readVmMeta.name,
      children: [
        //region A meta.json the reader cannot use is a failure, not an absence

        it({
          name: 'rejects a meta.json that does not parse instead of rebuilding from the legacy image file',
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-invalid-json-',);
            await writeVmFile({
              content: '{"image": "windows", "osFamily": "wind',
              fileName: META_FILE_NAME,
              vmDirectory: vm.path,
            },);
            await writeVmFile({
              content: LEGACY_IMAGE,
              fileName: LEGACY_IMAGE_FILE_NAME,
              vmDirectory: vm.path,
            },);

            await expect(readVmMeta(vm.path,),).rejects.toThrow(join(
              vm.path,
              META_FILE_NAME,
            ),);
          },
        },),

        it({
          name: 'rejects a meta.json that parses but lacks a required field',
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-incomplete-',);
            await writeVmFile({
              content: metaJson({
                osFamily: undefined,
                shell: undefined,
              },),
              fileName: META_FILE_NAME,
              vmDirectory: vm.path,
            },);
            await writeVmFile({
              content: LEGACY_IMAGE,
              fileName: LEGACY_IMAGE_FILE_NAME,
              vmDirectory: vm.path,
            },);

            await expect(readVmMeta(vm.path,),).rejects.toThrow(join(
              vm.path,
              META_FILE_NAME,
            ),);
          },
        },),

        it({
          name: 'rejects a meta.json whose text is empty',
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-empty-',);
            await writeVmFile({
              content: '',
              fileName: META_FILE_NAME,
              vmDirectory: vm.path,
            },);

            await expect(readVmMeta(vm.path,),).rejects.toThrow(join(
              vm.path,
              META_FILE_NAME,
            ),);
          },
        },),

        it({
          name: 'reports a meta.json it may not read instead of rebuilding from the legacy image file',
          skip: (process.getuid?.() === 0)
            ? 'the process is root, and no mode refuses root a file'
            : false,
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-unreadable-',);
            await writeVmFile({
              content: metaJson(),
              fileName: META_FILE_NAME,
              vmDirectory: vm.path,
            },);
            await writeVmFile({
              content: LEGACY_IMAGE,
              fileName: LEGACY_IMAGE_FILE_NAME,
              vmDirectory: vm.path,
            },);
            /**
             Path of the metadata file sealed below, reopened so disposal can
             remove it.
             */
            const metaPath = join(
              vm.path,
              META_FILE_NAME,
            );
            await chmod(
              metaPath,
              0,
            );
            await using _reopened = {
              [Symbol.asyncDispose]: async function reopen(): Promise<void> {
                await chmod(
                  metaPath,
                  0o600,
                );
              },
            };

            await expect(readVmMeta(vm.path,),).rejects.toThrow(metaPath,);
          },
        },),

        it({
          name: 'reports a meta.json it cannot read as a file instead of rebuilding from the legacy image file',
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-directory-',);
            await mkdir(join(
              vm.path,
              META_FILE_NAME,
            ),);
            await writeVmFile({
              content: LEGACY_IMAGE,
              fileName: LEGACY_IMAGE_FILE_NAME,
              vmDirectory: vm.path,
            },);

            await expect(readVmMeta(vm.path,),).rejects.toThrow(join(
              vm.path,
              META_FILE_NAME,
            ),);
          },
        },),

        //endregion A meta.json the reader cannot use is a failure, not an absence

        //region An absent meta.json still falls back

        it({
          name: 'reads the legacy image file when meta.json is absent',
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-legacy-',);
            await writeVmFile({
              content: LEGACY_IMAGE,
              fileName: LEGACY_IMAGE_FILE_NAME,
              vmDirectory: vm.path,
            },);

            expect(await readVmMeta(vm.path,),).toEqual({
              createdAt: '',
              defaultUser: legacyImageSpec().defaultUser,
              image: LEGACY_IMAGE,
              osFamily: legacyImageSpec().osFamily,
              shell: legacyImageSpec().shell,
            },);
          },
        },),

        it({
          name: 'assumes the default image when neither file is there',
          fn: async () => {
            await using vm = await scratchVmDirectory('mvm-read-meta-none-',);

            expect((await readVmMeta(vm.path,)).image,).toBe(DEFAULT_IMAGE,);
          },
        },),

        //endregion An absent meta.json still falls back
      ],
    },),
  ],
},);
