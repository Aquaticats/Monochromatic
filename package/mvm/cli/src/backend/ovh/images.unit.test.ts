/**
 Unit tests for image selection and for the login account: a shorthand is matched
 against normalized catalogue names, only an active image is used, the newest one
 wins, a published image beats a private image of the same name, and the account
 an SSH client logs in as is the image's own `user` field rather than `root`.

 Image bodies here are stand-ins written by hand from OVHcloud's schema
 (`cloud.image.Image`), never a recording, so nothing in this file claims what
 OVHcloud's catalogue really holds.

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
      name: 'resolveImage asks the region own catalogue',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(() => jsonAnswer(IMAGES,),);
        const image = await resolveImage({
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
