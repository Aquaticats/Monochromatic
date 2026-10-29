/**
 Tests for the content-block translation.

 THE DATA URI CASES ARE THE POINT. Pictures reach the models as data URIs
 built by `encodeImageAsset`, and the Messages API wants that URI taken apart
 into a media type and a payload. Reassembling it wrongly does not fail
 loudly: the provider answers, and the model describes a picture it could not
 see. So every way the split can go wrong is checked here rather than left to
 be noticed in an answer.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contentBlocksFor,
  MalformedImageUriError,
  readImageSource,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readImageSource.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS a data URI into the media type and payload the Messages API takes, which is '
            + 'exactly what encodeImageAsset builds',
          fn: async () => {
            expect(readImageSource({ url: 'data:image/webp;base64,d2hpc2tlcnM=', },),).toEqual({
              type: 'base64',
              media_type: 'image/webp',
              data: 'd2hpc2tlcnM=',
            },);
          },
        },),

        it({
          name: 'READS a media type carrying further parameters, since the encoding parameter is not '
            + 'required to sit in any particular place',
          fn: async () => {
            expect(readImageSource({ url: 'data:image/png;charset=utf-8;base64,cGF3cw==', },),).toEqual({
              type: 'base64',
              media_type: 'image/png',
              data: 'cGF3cw==',
            },);
          },
        },),

        it({
          name: 'SPLITS ON THE FIRST separator, so a payload carrying one is not truncated',
          fn: async () => {
            expect(readImageSource({ url: 'data:image/png;base64,cGF3,cw==', },),).toEqual({
              type: 'base64',
              media_type: 'image/png',
              data: 'cGF3,cw==',
            },);
          },
        },),

        it({
          name: 'FORWARDS a remote address as a remote source rather than reading it as inline, '
            + 'because the shared content type permits a URL even though nothing here sends one',
          fn: async () => {
            expect(readImageSource({ url: 'https://example.invalid/cat.webp', },),).toEqual({
              type: 'url',
              url: 'https://example.invalid/cat.webp',
            },);
          },
        },),

        it({
          name: 'REFUSES a data URI with no separator between its metadata and payload',
          fn: async () => {
            expect(() => {
              readImageSource({ url: 'data:image/png;base64', },);
            },).toThrow(MalformedImageUriError,);
          },
        },),

        it({
          name: 'REFUSES an empty media type, which would send the provider a picture it cannot decode',
          fn: async () => {
            expect(() => {
              readImageSource({ url: 'data:;base64,cGF3cw==', },);
            },).toThrow(MalformedImageUriError,);
          },
        },),

        it({
          name: 'REFUSES an encoding other than base64, which is the only inline one this protocol takes',
          fn: async () => {
            expect(() => {
              readImageSource({ url: 'data:image/svg+xml,%3Csvg%2F%3E', },);
            },).toThrow(MalformedImageUriError,);
          },
        },),

        it({
          name: 'REFUSES an empty payload rather than sending a picture with no bytes in it',
          fn: async () => {
            expect(() => {
              readImageSource({ url: 'data:image/png;base64,', },);
            },).toThrow(MalformedImageUriError,);
          },
        },),
      ],
    },),

    describe({
      name: contentBlocksFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRAPS a plain-text message in a block array, so the vision half and the text half '
            + 'take one path rather than two',
          fn: async () => {
            expect(contentBlocksFor({
              message: {
                role: 'user',
                content: 'Count the toebeans.',
              },
            },),).toEqual([
              {
                type: 'text',
                text: 'Count the toebeans.',
              },
            ],);
          },
        },),

        it({
          name: 'KEEPS the order of parts, because a caption before a picture and one after it are '
            + 'different questions',
          fn: async () => {
            expect(contentBlocksFor({
              message: {
                role: 'user',
                content: [
                  {
                    type: 'text',
                    text: 'Before.',
                  },
                  {
                    type: 'image_url',
                    image_url: { url: 'data:image/webp;base64,d2hpc2tlcnM=', },
                  },
                  {
                    type: 'text',
                    text: 'After.',
                  },
                ],
              },
            },).map(function named(block,): string {
              return block.type;
            },),).toEqual(['text', 'image', 'text',],);
          },
        },),

        it({
          name: 'CARRIES an empty message through as an empty text block rather than dropping it, '
            + 'since a dropped turn changes what alternates with what',
          fn: async () => {
            expect(contentBlocksFor({
              message: {
                role: 'assistant',
                content: '',
              },
            },),).toEqual([
              {
                type: 'text',
                text: '',
              },
            ],);
          },
        },),

        it({
          name: 'FORWARDS an unreadable picture as a refusal rather than sending the message without '
            + 'it, which would ask a model to read an image it was never shown',
          fn: async () => {
            expect(() => {
              contentBlocksFor({
                message: {
                  role: 'user',
                  content: [
                    {
                      type: 'image_url',
                      image_url: { url: 'data:image/png;base64,', },
                    },
                  ],
                },
              },);
            },).toThrow(MalformedImageUriError,);
          },
        },),
      ],
    },),
  ],
},);
