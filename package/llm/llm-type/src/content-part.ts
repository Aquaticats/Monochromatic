/**
 Multimodal content parts for OpenAI-compatible vision chat-completion requests.
 
 A multimodal message replaces a plain-text `content` string with an ordered
 list of these parts (interleaved text and image URLs). Consumers compose
 their own message envelope, for example
 `{ readonly role: 'user'; readonly content: readonly ContentPart[]; }`,
 rather than redeclaring the part union.
 
 @example
 ```ts
 import type { ContentPart } from '@monochromatic-dev/llm-llm-type';
 
 const parts: readonly ContentPart[] = [
   { type: 'text', text: 'Describe this image.', },
   { type: 'image_url', image_url: { url: 'data:image/png;base64,...', }, },
 ];
 ```
 
 @module
 */

/**
 One entry in a multimodal message body: either a text run or an image URL.
 
 `image_url.url` accepts both remote `https:` URLs and inline base64 data URIs,
 matching the OpenAI vision request contract.
 
 @example
 ```ts
 const textPart: ContentPart = { type: 'text', text: 'Caption:', };
 const imagePart: ContentPart = {
   type: 'image_url',
   image_url: { url: 'https://example.test/cat.png', },
 };
 ```
 */
export type ContentPart =
  | {
    /**
     Discriminant marking a plain-text content run.
     */
    readonly type: 'text';
    /**
     Text body of this part.
     */
    readonly text: string;
  }
    | {
    /**
     Discriminant marking an image content part.
     */
      readonly type: 'image_url';
    /**
     Image reference; `url` is a remote URL or inline data URI.
     */
      readonly image_url: {
      /**
       Remote URL or base64 data URI of the image.
       */
        readonly url: string;
      };
    };
