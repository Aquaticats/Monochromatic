/**
 Tests for extension-based media types.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  IMAGE_MEDIA_TYPES,
  mediaTypeForPath,
  OCTET_STREAM,
} from '@monochromatic-dev/config-lfs-r2-worker';

await describe({
  name: '',
  children: [
    describe({
      name: 'IMAGE_MEDIA_TYPES',
      children: [
        it({
          name: 'covers every raster extension .gitattributes routes through git-lfs',
          fn: async () => {
            expect([...IMAGE_MEDIA_TYPES.keys(),].toSorted(),).toEqual([
              'avif',
              'gif',
              'heif',
              'jpeg',
              'jpg',
              'jxl',
              'png',
              'webp',
            ],);
          },
        },),
        it({
          name: 'maps only to image types',
          fn: async () => {
            for (const mediaType of IMAGE_MEDIA_TYPES.values()) {
              expect(mediaType.startsWith('image/',),).toBe(true,);
            }
          },
        },),
      ],
    },),
    describe({
      name: mediaTypeForPath.name,
      children: [
        ...[...IMAGE_MEDIA_TYPES.entries(),].map(function mapExtension([extension, mediaType,],) {
          return it({
            name: `maps .${extension} to ${mediaType}`,
            fn: async () => {
              expect(mediaTypeForPath(`package/music-player/asset/readme/shot.${extension}`,),).toBe(mediaType,);
            },
          },);
        },),
        it({
          name: 'lowercases the extension before lookup',
          fn: async () => {
            expect(mediaTypeForPath('cover.PNG',),).toBe('image/png',);
          },
        },),
        it({
          name: 'falls back to octet-stream for an empty suffix',
          fn: async () => {
            expect(mediaTypeForPath('',),).toBe(OCTET_STREAM,);
          },
        },),
        it({
          name: 'falls back to octet-stream without an extension',
          fn: async () => {
            expect(mediaTypeForPath('asset/readme/shot',),).toBe(OCTET_STREAM,);
          },
        },),
        it({
          name: 'ignores dots in directory segments',
          fn: async () => {
            expect(mediaTypeForPath('dir.v2/name',),).toBe(OCTET_STREAM,);
          },
        },),
        it({
          name: 'falls back to octet-stream for a trailing dot',
          fn: async () => {
            expect(mediaTypeForPath('shot.',),).toBe(OCTET_STREAM,);
          },
        },),
        it({
          name: 'falls back to octet-stream for an unknown extension',
          fn: async () => {
            expect(mediaTypeForPath('notes.txt',),).toBe(OCTET_STREAM,);
          },
        },),
      ],
    },),
  ],
},);
