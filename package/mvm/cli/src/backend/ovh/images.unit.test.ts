/**
 Unit tests for image selection and for the login account: a shorthand is matched
 against normalized catalogue names, only an active image is used, the newest one
 wins, a published image beats a private image of the same name, a shorthand
 never stands for an image with preinstalled software (a GPU driver image above
 all) or for a variant of another image, an image the flavor cannot boot is left
 out, and the account an SSH client logs in as is the image's own `user` field
 rather than `root`.

 Image bodies here are stand-ins written by hand from OVHcloud's schema
 (`cloud.image.Image`), never a recording. `CATALOGUE` copies the names, the
 tags and the order of the creation times a live project's catalogue showed on
 2026-11-28, because that order is what made a GPU driver image the newest match.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  instanceImageUser,
  instanceLoginUser,
  normalizeImageName,
  type OvhImage,
  resolveImage,
  selectImage,
} from '../../../dist/final/node/index.mjs';
import {
  cloudPath,
  errorFrom,
  flavorAnswer,
  imageAnswer,
  installEnv,
  installStandIn,
  instanceAnswer,
  jsonAnswer,
  matching,
  throwMessage,
} from './fixture.ovh-api-stand-in.ts';

//region Fixtures

/**
 Images of one region: two Ubuntu releases, a third that is not active, a
 private snapshot named after a distribution, and a Debian one.
 */
const IMAGES = [
  imageAnswer({
    creationDate: '2025-01-01T00:00:00Z',
    id: 'ubuntu-24-04',
    name: 'Ubuntu 24.04',
  },),
  imageAnswer({
    creationDate: '2026-01-01T00:00:00Z',
    id: 'ubuntu-26-04',
    name: 'Ubuntu 26.04',
  },),
  imageAnswer({
    creationDate: '2027-01-01T00:00:00Z',
    id: 'ubuntu-28-04',
    name: 'Ubuntu 28.04',
    status: 'saving',
  },),
  imageAnswer({
    creationDate: '2027-06-01T00:00:00Z',
    id: 'private-ubuntu-snapshot',
    name: 'Ubuntu 24.04',
    user: 'cloud',
    visibility: 'private',
  },),
  imageAnswer({
    creationDate: '2025-06-01T00:00:00Z',
    id: 'debian-12',
    name: 'Debian 12',
    user: 'debian',
  },),
] as const satisfies readonly OvhImage[];

/**
 The flavor every selection here is made for: the smallest a live project
 offered, by its name, with the stand-in's disk and memory.
 */
const FLAVOR = flavorAnswer({ name: 'd2-2', },);

/**
 A catalogue shaped like a live one: a release, its UEFI variant and a GPU
 driver image built on it, each published after the plain release, and the
 same for an older release. The tags are the ones OVHcloud puts on an image
 with preinstalled software.
 */
const CATALOGUE = [
  imageAnswer({
    creationDate: '2026-08-20T17:30:31Z',
    id: 'ubuntu-26-04',
    name: 'Ubuntu 26.04',
  },),
  imageAnswer({
    creationDate: '2026-08-20T17:31:57Z',
    id: 'ubuntu-26-04-uefi',
    name: 'Ubuntu 26.04 - UEFI',
  },),
  imageAnswer({
    creationDate: '2026-08-20T22:00:21Z',
    id: 'ubuntu-26-04-nvidia',
    name: 'Ubuntu 26.04 - NVIDIA - v595',
    tags: ['application', 'nvidia_ngc',],
  },),
  imageAnswer({
    creationDate: '2026-08-20T17:27:36Z',
    id: 'ubuntu-24-04',
    name: 'Ubuntu 24.04',
  },),
  imageAnswer({
    creationDate: '2026-08-20T21:48:05Z',
    id: 'ubuntu-24-04-nvidia',
    name: 'Ubuntu 24.04 - NVIDIA - v580',
    tags: ['application', 'nvidia_ngc',],
  },),
] as const satisfies readonly OvhImage[];

/**
 The catalogue's images that carry preinstalled software, which is every GPU
 driver image in it.
 */
const WITH_SOFTWARE = CATALOGUE.filter(function hasTags(image,) {
  return 'tags' in image;
},);

//endregion Fixtures

await describe({
  name: 'ovh images',
  concurrency: 1,
  children: [
    //region Names

    it({
      name: 'normalizeImageName lower-cases and turns whitespace runs into one hyphen',
      fn: async () => {
        expect(normalizeImageName('Ubuntu 24.04',),).toBe('ubuntu-24.04',);
        expect(normalizeImageName('  Debian   12 ',),).toBe('debian-12',);
      },
    },),

    //endregion Names

    //region Selection

    it({
      name: 'selectImage takes the newest active match for a distribution shorthand',
      fn: async () => {
        expect(
          selectImage({
            flavor: FLAVOR,
            images: IMAGES,
            region: 'bhs5',
            shorthand: 'ubuntu',
          },).id,
        ).toBe('ubuntu-26-04',);
      },
    },),
    it({
      name: 'selectImage matches a shorthand that names a release exactly',
      fn: async () => {
        expect(
          selectImage({
            flavor: FLAVOR,
            images: IMAGES,
            region: 'bhs5',
            shorthand: 'ubuntu-24.04',
          },).id,
        ).toBe('ubuntu-24-04',);
      },
    },),
    it({
      name: 'selectImage prefers a published image over a private one of the same name',
      fn: async () => {
        expect(
          selectImage({
            flavor: FLAVOR,
            images: IMAGES,
            region: 'bhs5',
            shorthand: 'ubuntu-24.04',
          },).user,
        ).toBe('ubuntu',);
      },
    },),
    it({
      name: 'selectImage never takes an image that is not active',
      fn: async () => {
        const message = throwMessage({
          run: () => selectImage({
            flavor: FLAVOR,
            images: IMAGES,
            region: 'bhs5',
            shorthand: 'ubuntu-28',
          },),
        },);
        expect(message,).toContain('ubuntu-28',);
      },
    },),
    it({
      name: 'selectImage names the region and the shorthand when nothing matches',
      fn: async () => {
        const message = throwMessage({
          run: () => selectImage({
            flavor: FLAVOR,
            images: IMAGES,
            region: 'bhs5',
            shorthand: 'plan9',
          },),
        },);
        expect(message,).toContain('bhs5',);
        expect(message,).toContain('plan9',);
      },
    },),
    it({
      name: 'selectImage never takes a GPU driver image, or any image with preinstalled software, for a shorthand',
      fn: async () => {
        expect(
          selectImage({
            flavor: FLAVOR,
            images: CATALOGUE,
            region: 'bhs5',
            shorthand: 'ubuntu',
          },).id,
        ).toBe('ubuntu-26-04',);
        expect(
          selectImage({
            flavor: FLAVOR,
            images: CATALOGUE,
            region: 'bhs5',
            shorthand: 'ubuntu-24.04',
          },).id,
        ).toBe('ubuntu-24-04',);
      },
    },),
    it({
      name: 'selectImage takes the plain release over a newer variant of it',
      fn: async () => {
        /**
         The release and its UEFI variant only, so no tag decides the choice.
         */
        const images = CATALOGUE.filter(function isUntagged(image,) {
          return !('tags' in image);
        },);
        expect(
          selectImage({
            flavor: FLAVOR,
            images,
            region: 'bhs5',
            shorthand: 'ubuntu-26.04',
          },).id,
        ).toBe('ubuntu-26-04',);
      },
    },),
    it({
      name: 'selectImage takes an image that is named whole, with preinstalled software or not',
      fn: async () => {
        expect(
          selectImage({
            flavor: FLAVOR,
            images: CATALOGUE,
            region: 'bhs5',
            shorthand: 'Ubuntu 26.04 - NVIDIA - v595',
          },).id,
        ).toBe('ubuntu-26-04-nvidia',);
        expect(
          selectImage({
            flavor: FLAVOR,
            images: CATALOGUE,
            region: 'bhs5',
            shorthand: 'Ubuntu 26.04 - UEFI',
          },).id,
        ).toBe('ubuntu-26-04-uefi',);
      },
    },),
    it({
      name: 'selectImage refuses when only images with preinstalled software match, naming them and the option',
      fn: async () => {
        const message = throwMessage({
          run: () => selectImage({
            flavor: FLAVOR,
            images: WITH_SOFTWARE,
            region: 'bhs5',
            shorthand: 'ubuntu',
          },),
        },);
        expect(message,).toContain('--image',);
        for (const image of WITH_SOFTWARE) {
          expect(message,).toContain(image.name,);
        }
      },
    },),
    it({
      name: 'selectImage leaves out an image the flavor cannot boot',
      fn: async () => {
        /**
         Three newer releases the flavor cannot boot (too little disk, too
         little memory, another flavor family) and an older one it can.
         */
        const images = [
          imageAnswer({
            creationDate: '2030-01-01T00:00:00Z',
            id: 'needs-more-disk',
            minDisk: FLAVOR.disk + 1,
            name: 'Ubuntu 30.04',
          },),
          imageAnswer({
            creationDate: '2029-01-01T00:00:00Z',
            id: 'needs-more-memory',
            minRam: (FLAVOR.ram * 1_024) + 1,
            name: 'Ubuntu 29.04',
          },),
          imageAnswer({
            creationDate: '2028-01-01T00:00:00Z',
            flavorType: 't1,t2',
            id: 'for-another-family',
            name: 'Ubuntu 28.04',
          },),
          imageAnswer({
            creationDate: '2026-01-01T00:00:00Z',
            id: 'boots',
            name: 'Ubuntu 26.04',
          },),
        ];
        expect(
          selectImage({
            flavor: FLAVOR,
            images,
            region: 'bhs5',
            shorthand: 'ubuntu',
          },).id,
        ).toBe('boots',);
      },
    },),
    it({
      name: 'selectImage takes an image that lists the flavor own family',
      fn: async () => {
        expect(
          selectImage({
            flavor: FLAVOR,
            images: [
              imageAnswer({
                flavorType: 'b2, d2',
                id: 'for-this-family',
                name: 'Ubuntu 26.04',
              },),
            ],
            region: 'bhs5',
            shorthand: 'ubuntu',
          },).id,
        ).toBe('for-this-family',);
      },
    },),
    it({
      name: 'selectImage names the flavor and both options when the flavor boots no match',
      fn: async () => {
        const message = throwMessage({
          run: () => selectImage({
            flavor: FLAVOR,
            images: [
              imageAnswer({
                minDisk: FLAVOR.disk + 1,
                name: 'Ubuntu 26.04',
              },),
            ],
            region: 'bhs5',
            shorthand: 'ubuntu',
          },),
        },);
        expect(message,).toContain(FLAVOR.name,);
        expect(message,).toContain('--image',);
        expect(message,).toContain('--server-type',);
      },
    },),
    it({
      name: 'resolveImage asks the region own catalogue',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(IMAGES,),);
        const image = await resolveImage({
          flavor: FLAVOR,
          region: 'bhs5',
          shorthand: 'debian',
        },);
        expect(image.id,).toBe('debian-12',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'GET',
            path: `${cloudPath('/image',)}?region=bhs5`,
          },).length,
        ).toBe(1,);
      },
    },),

    //endregion Selection

    //region Login account

    it({
      name: 'instanceImageUser reads the account the image names, never root',
      fn: async () => {
        const instance = instanceAnswer({ imageUser: 'debian', },);
        expect(await instanceImageUser({ instance, },),).toBe('debian',);
      },
    },),
    it({
      name: 'instanceLoginUser asks the project for the image when the instance does not carry it',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(imageAnswer({ user: 'rocky', },),),);
        /**
         The instance as the API reports it when it omits its image.
         */
        const instance = instanceAnswer({ withImage: false, },);
        expect(await instanceLoginUser({ instance, },),).toBe('rocky',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'GET',
            path: cloudPath('/image/image-ubuntu',),
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'instanceLoginUser refuses an image that names no account',
      fn: async () => {
        const error = await errorFrom({
          run: async function connect() {
            await instanceLoginUser({ instance: instanceAnswer({ imageUser: '', },), },);
          },
        },);
        expect(Error.isError(error,),).toBe(true,);
        expect(caughtValueText(error,),).toContain('names no login account',);
      },
    },),

    //endregion Login account
  ],
},);
