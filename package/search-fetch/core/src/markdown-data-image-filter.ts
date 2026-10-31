/**
 Markdown data-image filtering for fetched page responses.
 
 @module
 */

import {
  findBase64ImageRange,
  findLinkedImageRange,
  isUnparseableMarkdown,
  lineOwnedRemovalRange,
} from './markdown-data-image-parser.ts';

//region Constants

/**
 Markdown inline-image opening token.
 */
const IMAGE_OPEN = '![';

//endregion Constants

//region Types

/**
 Filtered Markdown and removal count.
 */
type MarkdownDataImageFilterResult = {
  /**
   Markdown after base64-backed inline images are removed.
   */
  readonly markdown: string;
  /**
   Number of removed image constructs.
   */
  readonly removedImageCount: number;
};

/**
 Fetch response after Markdown filtering.
 */
type FetchResponseDataImageFilterResult = {
  /**
   Original response or shallow copy carrying filtered Markdown.
   */
  readonly linkupResponse: unknown;
  /**
   Number of removed image constructs.
   */
  readonly removedImageCount: number;
};

//endregion Types

//region Public API

/**
 Remove inline base64 image constructs from Markdown in a fetched response.
 
 A construct that owns its physical lines removes those complete lines.
 Inline constructs remove only their Markdown span so surrounding prose survives.
 Non-Markdown response shapes and responses without matching images retain identity.
 
 @param response - parsed provider response
 
 @returns response for model-visible output and removal count
 
 @example
 ```ts
 filterFetchResponseDataImages({ markdown: 'Before\n![logo](data:image/png;base64,AAAA)\nAfter' });
 ```
 */
function filterFetchResponseDataImages(response: unknown,): FetchResponseDataImageFilterResult {
  if (!hasMarkdownString(response,))
    return {
      linkupResponse: response,
      removedImageCount: 0,
    };

  /**
   Markdown-specific filter result.
   */
  const filtered = filterMarkdownDataImages(response.markdown,);
  if (filtered.removedImageCount === 0)
    return {
      linkupResponse: response,
      removedImageCount: 0,
    };

  return {
    linkupResponse: {
      ...response,
      markdown: filtered.markdown,
    },
    removedImageCount: filtered.removedImageCount,
  };
}

/**
 Remove Markdown inline images whose destinations are base64 image data URLs.
 
 @param markdown - fetched page Markdown
 
 @returns filtered Markdown and removed image count
 
 @example
 ```ts
 filterMarkdownDataImages('Text ![logo](data:image/png;base64,AAAA) remains.');
 ```
 */
function filterMarkdownDataImages(markdown: string,): MarkdownDataImageFilterResult {
  return (function scanMarkdownDataImages(): MarkdownDataImageFilterResult {
    /**
     Retained text chunks joined once after scanning.
     */
    const retainedChunks: string[] = [];
    /**
     Start of text not yet copied into retained chunks.
     */
    let retainedStart = 0;
    /**
     Search start for next inline-image opener.
     */
    let searchStart = 0;
    /**
     Number of accepted image constructs removed so far.
     */
    let removedImageCount = 0;
    /**
     Next possible inline-image opener.
     */
    let imageStart = markdown.indexOf(
      IMAGE_OPEN,
      searchStart,
    );

    while (imageStart !== (-1)) {
      /**
       Parsed base64 image token, or rejection sentinel.
       */
      const imageRange = findBase64ImageRange({
        markdown,
        imageStart,
      },);
      if (isUnparseableMarkdown(imageRange,)) {
        searchStart = imageStart + IMAGE_OPEN.length;
      }
      else {
        /**
         Full linked-image wrapper when image is tightly wrapped by one.
         */
        const constructRange = findLinkedImageRange({
          markdown,
          imageRange,
        },);
        /**
         Smallest safe range, widened to physical lines only when construct owns them.
         */
        const removalRange = lineOwnedRemovalRange({
          markdown,
          constructRange,
        },);
        retainedChunks.push(markdown.slice(
          retainedStart,
          removalRange.start,
        ),);
        retainedStart = removalRange.end;
        searchStart = removalRange.end;
        removedImageCount += 1;
      }
      imageStart = markdown.indexOf(
        IMAGE_OPEN,
        searchStart,
      );
    }

    retainedChunks.push(markdown.slice(retainedStart,),);
    return {
      markdown: retainedChunks.join('',),
      removedImageCount,
    };
  })();
}

//endregion Public API

//region Value helpers

/**
 Return whether value is a response record carrying Markdown text.
 
 @param value - candidate provider response
 
 @returns whether Markdown can be filtered and copied
 */
function hasMarkdownString(value: unknown,): value is Readonly<Record<string, unknown>> & {
  readonly markdown: string;
} {
  return (value !== null)
    && ((typeof value) === 'object')
    && (!Array.isArray(value,))
    && ('markdown' in value)
    && ((typeof value.markdown) === 'string');
}

//endregion Value helpers

export {
  filterFetchResponseDataImages,
  filterMarkdownDataImages,
};
export type {
  FetchResponseDataImageFilterResult,
  MarkdownDataImageFilterResult,
};
