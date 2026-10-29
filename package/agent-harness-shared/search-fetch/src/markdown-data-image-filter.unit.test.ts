/**
 Unit tests for Markdown data-image post-filtering.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  filterFetchResponseDataImages,
  filterMarkdownDataImages,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Single-line base64 image Markdown.
 */
const SINGLE_LINE_IMAGE = '![Logo](data:image/png;base64,QUFBQQ==)';

/**
 Linked multiline image shaped like rendered fetch output from Microsoft pages.
 */
const MULTILINE_LINKED_IMAGE = [
  ' [![Microsoft',
  ' Logo](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA',
  ' ALgAAAAmCAYAAAB3X1H0AAAA)]()',
].join('\n',);

/**
 Unsupported or malformed image cases that must survive unchanged.
 */
const RETAINED_MARKDOWN_CASES = [
  {
    name: 'ordinary remote image',
    markdown: '![Logo](https://example.com/logo.png)',
  },
  {
    name: 'non-image data URL',
    markdown: '![Text](data:text/plain;base64,QUFBQQ==)',
  },
  {
    name: 'non-base64 image data URL',
    markdown: '![Pixel](data:image/svg+xml,%3Csvg%3E)',
  },
  {
    name: 'empty base64 payload',
    markdown: '![Empty](data:image/png;base64,)',
  },
  {
    name: 'invalid base64 payload character',
    markdown: '![Invalid](data:image/png;base64,AAAA_123)',
  },
  {
    name: 'unclosed base64 image',
    markdown: '![Broken](data:image/png;base64,QUFBQQ==',
  },
  {
    name: 'reference image syntax',
    markdown: '![Logo][logo-reference]',
  },
] as const;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: filterMarkdownDataImages.name,
      children: [
        it({
          name: 'removes complete physical line owned by base64 image',
          fn: async () => {
            /**
             Filtered line-isolated image.
             */
            const result = filterMarkdownDataImages([
              'Before',
              SINGLE_LINE_IMAGE,
              'After',
            ].join('\n',),);

            expect(result.markdown,).toBe('Before\nAfter',);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
        it({
          name: 'removes multiline linked image without leaving wrapper lines',
          fn: async () => {
            /**
             Filtered Microsoft-shaped linked image.
             */
            const result = filterMarkdownDataImages([
              'Before',
              MULTILINE_LINKED_IMAGE,
              'After',
            ].join('\n',),);

            expect(result.markdown,).toBe('Before\nAfter',);
            expect(result.markdown,).not.toContain('Microsoft',);
            expect(result.markdown,).not.toContain('iVBOR',);
            expect(result.markdown,).not.toContain(']()',);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
        it({
          name: 'removes tight linked image span while retaining same-line prose',
          fn: async () => {
            /**
             Filtered inline linked image.
             */
            const result = filterMarkdownDataImages(
              'Before [![Logo](data:image/png;base64,QUFBQQ==)](https://example.com/a_(b)) after.',
            );

            expect(result.markdown,).toBe('Before  after.',);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
        it({
          name: 'removes angle-wrapped image with nested and escaped alt text',
          fn: async () => {
            /**
             Filtered angle-wrapped data URL.
             */
            const result = filterMarkdownDataImages(
              String.raw`Prefix ![nested [label\] text]](<data:image/gif;base64,R0lGODlhAQAB>) suffix`,
            );

            expect(result.markdown,).toBe('Prefix  suffix',);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
        it({
          name: 'removes several inline images without discarding neighboring text',
          fn: async () => {
            /**
             Filtered Markdown containing independent image constructs.
             */
            const result = filterMarkdownDataImages(
              `A ${SINGLE_LINE_IMAGE} B ${SINGLE_LINE_IMAGE} C`,
            );

            expect(result.markdown,).toBe('A  B  C',);
            expect(result.removedImageCount,).toBe(2,);
          },
        },),
        it({
          name: 'removes line-isolated image at end of text without terminal newline',
          fn: async () => {
            /**
             Filtered final image line.
             */
            const result = filterMarkdownDataImages(`Before\n${SINGLE_LINE_IMAGE}`,);

            expect(result.markdown,).toBe('Before\n',);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
        it({
          name: 'keeps inner-image removal bounded when outer link wrapper is malformed',
          fn: async () => {
            /**
             Filtered image with unclosed outer link destination.
             */
            const result = filterMarkdownDataImages(
              `Before [${SINGLE_LINE_IMAGE}](https://example.com after`,
            );

            expect(result.markdown,).toBe('Before [](https://example.com after',);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
        ...RETAINED_MARKDOWN_CASES.map(function createRetainedMarkdownTest(testCase,) {
          return it({
            name: `retains ${testCase.name}`,
            fn: async () => {
              /**
               Filter result for unsupported or malformed Markdown.
               */
              const result = filterMarkdownDataImages(testCase.markdown,);

              expect(result.markdown,).toBe(testCase.markdown,);
              expect(result.removedImageCount,).toBe(0,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: filterFetchResponseDataImages.name,
      children: [
        it({
          name: 'preserves response identity when no Markdown is filterable',
          fn: async () => {
            /**
             Non-Markdown response fixture.
             */
            const response = { content: SINGLE_LINE_IMAGE, };
            /**
             Filtered response metadata.
             */
            const result = filterFetchResponseDataImages(response,);

            expect(result.linkupResponse,).toBe(response,);
            expect(result.removedImageCount,).toBe(0,);
          },
        },),
        it({
          name: 'preserves response identity when Markdown has no matching image',
          fn: async () => {
            /**
             Ordinary Markdown response fixture.
             */
            const response = { markdown: '# Heading', };
            /**
             Filtered response metadata.
             */
            const result = filterFetchResponseDataImages(response,);

            expect(result.linkupResponse,).toBe(response,);
            expect(result.removedImageCount,).toBe(0,);
          },
        },),
        it({
          name: 'copies response with filtered Markdown while retaining extra fields',
          fn: async () => {
            /**
             Metadata-bearing Markdown response fixture.
             */
            const response = {
              markdown: `Before\n${SINGLE_LINE_IMAGE}\nAfter`,
              title: 'Fetched title',
            };
            /**
             Filtered response metadata.
             */
            const result = filterFetchResponseDataImages(response,);

            expect(result.linkupResponse,).not.toBe(response,);
            expect(result.linkupResponse,).toEqual({
              markdown: 'Before\nAfter',
              title: 'Fetched title',
            },);
            expect(result.removedImageCount,).toBe(1,);
          },
        },),
      ],
    },),
  ],
},);
