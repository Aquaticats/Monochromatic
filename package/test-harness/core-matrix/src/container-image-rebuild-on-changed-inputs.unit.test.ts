/**
 Tests for rebuilding a container image when the inputs it was built from have changed.
 A recording engine stands in for `podman`, so no test starts a container build.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildContainerImage,
  ensureContainerImage,
  IMAGE_BUILD_INPUTS_LABEL_NAME,
  inputsDigestOf,
  isContainerImageCurrent,
  type ContainerEngine,
  type ContainerImageInputs,
} from '../dist/final/node/index.mjs';

/**
 Name the fixture image is tagged with.
 */
const FIXTURE_IMAGE = 'localhost/fixture/image';

/**
 Name of the fixture image the build starts from.
 */
const FIXTURE_BASE_IMAGE = 'localhost/fixture/base';

/**
 Inputs of one fixture build, from which tests derive variants.
 */
const FIXTURE_INPUTS: ContainerImageInputs = {
  image: FIXTURE_IMAGE,
  containerfile: 'Containerfile',
  contextDirectory: '.',
  buildArguments: ['--build-arg', 'ZIG_ARCHIVE_SHA256=first',],
  boundingFlags: ['--memory=4g',],
  baseImages: [FIXTURE_BASE_IMAGE,],
};

/**
 What a recording engine holds about its local images and the builds it was asked to run.
 */
type RecordingEngine = ContainerEngine & {
  /**
   Arguments of every build the engine ran, in order.
   */
  readonly builds: (readonly string[])[];
};

/**
 Builds an engine whose local images are the given identities and labels.

 @param images - identity and labels of each local image by name

 @returns the engine, which records builds and stores the labels a build applies

 @example
 ```ts
 const engine = engineWith({ [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, }, },);
 ```
 */
function engineWith(images: Readonly<Record<string, { readonly id: string; readonly labels: Readonly<Record<string, string>>; }>>,): RecordingEngine {
  /**
   Local images by name, which a build adds to.
   */
  const local = new Map(Object.entries(images,),);
  /**
   Arguments of every build.
   */
  const builds: (readonly string[])[] = [];
  return {
    builds,
    hasImage: async function hasImage(image,) {
      return local.has(image,);
    },
    idOf: async function idOf(image,) {
      return local.get(image,)?.id ?? '';
    },
    labelOf: async function labelOf({ image, name, },) {
      return local.get(image,)?.labels[name] ?? '';
    },
    build: async function build(buildCommandArguments,) {
      builds.push(buildCommandArguments,);
      /**
       Position of the label flag whose next word is the label.
       */
      const labelFlag = buildCommandArguments.indexOf('--label',);
      /**
       The `name=value` word the build applies.
       */
      const label = buildCommandArguments[labelFlag + 1] ?? '';
      /**
       Position of the separator between the label name and value.
       */
      const separator = label.indexOf('=',);
      local.set(FIXTURE_IMAGE, {
        id: `sha256:built-${String(builds.length,)}`,
        labels: { [label.slice(0, separator,)]: label.slice(separator + 1,), },
      },);
    },
  };
}

await describe({
  name: 'container image rebuild on changed inputs',
  children: [
    describe({
      name: inputsDigestOf.name,
      children: [
        it({
          name: 'is the same for the same arguments and base image, and differs when an argument or a base image changes',
          fn: async () => {
            /**
             Engine whose base image has the first identity.
             */
            const engine = engineWith({ [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, }, },);
            /**
             Digest of the fixture inputs.
             */
            const digest = await inputsDigestOf({ inputs: FIXTURE_INPUTS, engine, },);
            expect(await inputsDigestOf({ inputs: { ...FIXTURE_INPUTS, boundingFlags: ['--memory=8g',], }, engine, },),).toBe(digest,);
            expect(await inputsDigestOf({
              inputs: { ...FIXTURE_INPUTS, buildArguments: ['--build-arg', 'ZIG_ARCHIVE_SHA256=second',], },
              engine,
            },),).not.toBe(digest,);
            expect(await inputsDigestOf({
              inputs: FIXTURE_INPUTS,
              engine: engineWith({ [FIXTURE_BASE_IMAGE]: { id: 'sha256:2', labels: {}, }, },),
            },),).not.toBe(digest,);
            expect(await inputsDigestOf({ inputs: FIXTURE_INPUTS, engine: engineWith({},), },),).not.toBe(digest,);
          },
        },),
      ],
    },),
    describe({
      name: buildContainerImage.name,
      children: [
        it({
          name: 'passes the bounding flags, build arguments, the digest label, the tag and the Containerfile to the engine',
          fn: async () => {
            /**
             Engine with the base image present.
             */
            const engine = engineWith({ [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, }, },);
            await buildContainerImage({ inputs: FIXTURE_INPUTS, engine, },);
            expect(engine.builds,).toEqual([
              [
                '--memory=4g',
                '--build-arg',
                'ZIG_ARCHIVE_SHA256=first',
                '--label',
                `${IMAGE_BUILD_INPUTS_LABEL_NAME}=${await inputsDigestOf({ inputs: FIXTURE_INPUTS, engine, },)}`,
                '--tag',
                FIXTURE_IMAGE,
                '--file',
                'Containerfile',
                '.',
              ],
            ],);
          },
        },),
      ],
    },),
    describe({
      name: isContainerImageCurrent.name,
      children: [
        it({
          name: 'is false for a missing image, an unlabelled image and an image built from other arguments',
          fn: async () => {
            expect(await isContainerImageCurrent({
              inputs: FIXTURE_INPUTS,
              engine: engineWith({ [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, }, },),
            },),).toBe(false,);
            expect(await isContainerImageCurrent({
              inputs: FIXTURE_INPUTS,
              engine: engineWith({
                [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, },
                [FIXTURE_IMAGE]: { id: 'sha256:old', labels: {}, },
              },),
            },),).toBe(false,);
            expect(await isContainerImageCurrent({
              inputs: FIXTURE_INPUTS,
              engine: engineWith({
                [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, },
                [FIXTURE_IMAGE]: { id: 'sha256:old', labels: { [IMAGE_BUILD_INPUTS_LABEL_NAME]: 'stale-digest', }, },
              },),
            },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: ensureContainerImage.name,
      children: [
        it({
          name: 'builds a missing image once, leaves a current image alone, and builds again after an argument or a base image changes',
          fn: async () => {
            /**
             Engine with the base image present and no fixture image.
             */
            const engine = engineWith({ [FIXTURE_BASE_IMAGE]: { id: 'sha256:1', labels: {}, }, },);
            expect(await ensureContainerImage({ inputs: FIXTURE_INPUTS, engine, },),).toBe(true,);
            expect(await ensureContainerImage({ inputs: FIXTURE_INPUTS, engine, },),).toBe(false,);
            expect(engine.builds.length,).toBe(1,);
            /**
             Inputs whose Zig checksum moved.
             */
            const movedInputs = { ...FIXTURE_INPUTS, buildArguments: ['--build-arg', 'ZIG_ARCHIVE_SHA256=second',], };
            expect(await ensureContainerImage({ inputs: movedInputs, engine, },),).toBe(true,);
            expect(await ensureContainerImage({ inputs: movedInputs, engine, },),).toBe(false,);
            expect(engine.builds.length,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
