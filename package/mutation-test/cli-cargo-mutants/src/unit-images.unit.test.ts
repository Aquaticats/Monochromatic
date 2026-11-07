import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import {
  argumentSeparator,
  callTerminator,
} from './fake-programs.ts';
import {
  createFakePodman,
  type FakePodman,
} from './fake-podman-fixture.ts';
import { parseLauncherOptions } from './options.ts';
import { planUnits } from './plan.ts';
import { ensureUnitImages } from './unit-images.ts';

/**
 Repository the cases give the launcher.
 */
const repository = 'localhost/product-test';

/**
 Tag of the gate image in the stand-in registry.
 */
const gateTag = 'development';

/**
 Image ID of the gate image.
 */
const gateImage = 'a'.repeat(64);

/**
 Image ID of some other image, such as an older gate.
 */
const otherImage = 'd'.repeat(64);

/**
 Reference of a tag in the cases' repository.

 @param tag - image tag

 @returns `<repository>:<tag>`
 */
function referenceOf(tag: string): string {
  return `${repository}:${tag}`;
}

/**
 Write the stand-in registry the fake `podman` answers from.

 @param fake - the fixture
 @param images - image reference to image ID

 @returns nothing, once the file is written
 */
async function writeRegistry({
  fake,
  images,
}: {
  readonly fake: FakePodman;
  readonly images: Readonly<Record<string, string>>;
}): Promise<void> {
  await writeFile(
    join(
      fake.directory,
      'images.json',
    ),
    JSON.stringify(images),
  );
}

/**
 Read the registry back after a launch.

 @param fake - the fixture

 @returns image reference to image ID
 */
async function readRegistry(fake: FakePodman): Promise<Readonly<Record<string, string>>> {
  return JSON.parse(await readFile(
    join(
      fake.directory,
      'images.json',
    ),
    'utf8',
  )) as Readonly<Record<string, string>>;
}

/**
 Every `podman` call the fake recorded, one argument list per call.

 @param fake - the fixture

 @returns the calls in order
 */
async function podmanCalls(fake: FakePodman): Promise<readonly (readonly string[])[]> {
  return (await readFile(
    join(
      fake.directory,
      'calls.log',
    ),
    'utf8',
  ))
    .split(callTerminator)
    .filter(function isCall(call) {
      return call !== '';
    })
    .map(function argumentsOf(call) {
      return call.split(argumentSeparator);
    });
}

/**
 Plan a launch whose podman is the fake.

 @param fake - the fixture
 @param argv - launcher options between the required ones and the label
 @param labelAndFiles - the label and files

 @returns options and plans
 */
function launch({
  fake,
  argv,
  labelAndFiles,
}: {
  readonly fake: FakePodman;
  readonly argv: readonly string[];
  readonly labelAndFiles: readonly string[];
}) {
  const options = parseLauncherOptions({
    argv: [
      '--task',
      '//p:t',
      '--tag-env',
      'IMG_TAG',
      '--image-repository',
      repository,
      '--setenv',
      `FAKE_DIR=${fake.directory}`,
      '--root',
      fake.workingDirectory,
      ...argv,
      ...labelAndFiles,
    ],
    environment: fake.environment,
    home: fake.directory,
    packageDirectory: '/repo/package/mutation-test/cli-cargo-mutants',
  });
  return {
    options,
    plans: planUnits(options),
  };
}

/**
 Run the preparation and return what it threw.

 @param call - arguments of `ensureUnitImages`

 @returns the thrown value
 */
async function refusal(call: Parameters<typeof ensureUnitImages>[0]): Promise<unknown> {
  try {
    await ensureUnitImages(call);
  }
  catch (error) {
    return error;
  }
  throw new Error('ensureUnitImages returned although it had to refuse');
}

/**
 Podman words that write: anything other than reading an image.

 @param calls - recorded calls

 @returns the calls that are not `image inspect`
 */
function writes(calls: readonly (readonly string[])[]): readonly (readonly string[])[] {
  return calls.filter(function isWrite(call) {
    return !((call[0] === 'image') && (call[1] === 'inspect'));
  });
}

await describe({
  name: ensureUnitImages.name,
  children: [
    describe({
      name: 'with --from-tag',
      children: [
        it({
          name: 'tags the source image once for every unit, then finds every tag on that image',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: { [referenceOf(gateTag)]: gateImage },
            });
            const { options, plans } = launch({
              fake,
              argv: ['--from-tag', gateTag],
              labelAndFiles: ['w5', 'a.rs', 'b.rs', 'c.rs'],
            });
            const lines = await ensureUnitImages({ options, plans });
            expect(await readRegistry(fake)).toEqual({
              [referenceOf(gateTag)]: gateImage,
              [referenceOf('w50')]: gateImage,
              [referenceOf('w51')]: gateImage,
              [referenceOf('w52')]: gateImage,
            });
            expect(writes(await podmanCalls(fake))).toEqual(plans.map(function tagCall(plan) {
              return [
                'tag',
                gateImage,
                referenceOf(plan.tag),
              ];
            }));
            expect(lines.join('\n')).toContain(gateImage.slice(
              0,
              12,
            ));
          },
        }),
        it({
          name: 'writes nothing but podman tag, and never removes an image or a tag',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: {
                [referenceOf(gateTag)]: gateImage,
                [referenceOf('w50')]: otherImage,
              },
            });
            const { options, plans } = launch({
              fake,
              argv: ['--from-tag', gateTag],
              labelAndFiles: ['w5', 'a.rs', 'b.rs'],
            });
            await ensureUnitImages({ options, plans });
            expect(new Set((await podmanCalls(fake)).map(function verbOf(call) {
              return call
                .slice(
                0,
                2,
                )
                .join(' ');
            }))
              .isSubsetOf(new Set([
                'image inspect',
                `tag ${gateImage}`,
              ]))).toBe(true);
          },
        }),
        it({
          name: 'names a tag it moved from another image, with both IDs',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: {
                [referenceOf(gateTag)]: gateImage,
                [referenceOf('w50')]: otherImage,
              },
            });
            const { options, plans } = launch({
              fake,
              argv: ['--from-tag', gateTag],
              labelAndFiles: ['w5', 'a.rs', 'b.rs'],
            });
            const lines = (await ensureUnitImages({ options, plans })).join('\n');
            expect(lines).toContain(referenceOf('w50'));
            expect(lines).toContain(otherImage);
            expect(lines).not.toContain(referenceOf('w51'));
            expect((await readRegistry(fake))[referenceOf('w50')]).toBe(gateImage);
          },
        }),
        it({
          name: 'leaves a unit whose tag is the source tag alone',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: { [referenceOf('w50')]: gateImage },
            });
            const { options, plans } = launch({
              fake,
              argv: ['--from-tag', 'w50'],
              labelAndFiles: ['w5', 'a.rs'],
            });
            await ensureUnitImages({ options, plans });
            expect(writes(await podmanCalls(fake))).toEqual([]);
          },
        }),
        it({
          name: 'refuses when the source image does not exist, names it, and tags nothing',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: {},
            });
            const { options, plans } = launch({
              fake,
              argv: ['--from-tag', gateTag],
              labelAndFiles: ['w5', 'a.rs'],
            });
            const thrown = await refusal({ options, plans });
            expect(thrown).toBeInstanceOf(CargoMutantsToolingError);
            expect((thrown as Error).message).toContain(referenceOf(gateTag));
            expect((thrown as Error).message).toContain('--from-tag');
            expect(writes(await podmanCalls(fake))).toEqual([]);
          },
        }),
        it({
          name: 'refuses when a tag lands on another image than the source, naming the tag, both IDs and the command',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: { [referenceOf(gateTag)]: gateImage },
            });
            const { options, plans } = launch({
              fake,
              argv: [
                '--from-tag',
                gateTag,
                '--setenv',
                `FAKE_TAG_LANDS_ON=${otherImage}`,
              ],
              labelAndFiles: ['w5', 'a.rs'],
            });
            const message = ((await refusal({ options, plans })) as Error).message;
            expect(message).toContain(referenceOf('w50'));
            expect(message).toContain(gateImage);
            expect(message).toContain(otherImage);
            expect(message).toContain(`podman tag ${gateImage} ${referenceOf('w50')}`);
          },
        }),
        it({
          name: 'refuses with the podman text when podman tag fails',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: { [referenceOf(gateTag)]: gateImage },
            });
            const { options, plans } = launch({
              fake,
              argv: [
                '--from-tag',
                gateTag,
                '--setenv',
                'FAKE_FAIL=tag',
              ],
              labelAndFiles: ['w5', 'a.rs'],
            });
            const message = ((await refusal({ options, plans })) as Error).message;
            expect(message).toContain(referenceOf('w50'));
            expect(message).toContain('fake podman failure: tag');
          },
        }),
      ],
    }),
    describe({
      name: 'without --from-tag',
      children: [
        it({
          name: 'starts nothing and tags nothing when every unit tag resolves',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: {
                [referenceOf('w50')]: gateImage,
                [referenceOf('w51')]: otherImage,
              },
            });
            const { options, plans } = launch({
              fake,
              argv: [],
              labelAndFiles: ['w5', 'a.rs', 'b.rs'],
            });
            await ensureUnitImages({ options, plans });
            expect(writes(await podmanCalls(fake))).toEqual([]);
            expect((await podmanCalls(fake)).length).toBe(plans.length);
          },
        }),
        it({
          name: 'refuses when a tag is missing, naming it with the command that fixes it, and writes nothing',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: { [referenceOf('w50')]: gateImage },
            });
            const { options, plans } = launch({
              fake,
              argv: [],
              labelAndFiles: ['w5', 'a.rs', 'b.rs', 'c.rs'],
            });
            const thrown = await refusal({ options, plans });
            expect(thrown).toBeInstanceOf(CargoMutantsToolingError);
            const message = (thrown as Error).message;
            expect(message).toContain(referenceOf('w51'));
            expect(message).toContain(referenceOf('w52'));
            expect(message).not.toContain(referenceOf('w50'));
            expect(message).toContain(`podman tag ${referenceOf('<gate-tag>')} ${referenceOf('w51')}`);
            expect(message).toContain('--from-tag');
            expect(writes(await podmanCalls(fake))).toEqual([]);
          },
        }),
        it({
          name: 'counts a tag whose answer is not a full image ID as missing',
          fn: async () => {
            await using fake = await createFakePodman();
            await writeRegistry({
              fake,
              images: { [referenceOf('w50')]: 'abc' },
            });
            const { options, plans } = launch({
              fake,
              argv: [],
              labelAndFiles: ['w5', 'a.rs'],
            });
            expect(((await refusal({ options, plans })) as Error).message).toContain(referenceOf('w50'));
          },
        }),
      ],
    }),
  ],
});
