import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import type { VisionMessage, } from './chat-contract.ts';

//region Anthropic content
// ONE MESSAGE, TRANSLATED INTO THE BLOCKS THE MESSAGES API TAKES.
//
// The pipeline speaks the OpenAI content-part shape everywhere above the client
// seam, because that is what the first provider takes, and it was widened for
// pictures. Anthropic takes a different shape for the same three things: a run
// of text, an inline image, an image behind a URL. This file is the whole of
// that difference, so no stage learns which provider it is talking to.
//
// PICTURES ARRIVE AS DATA URIS, always, because `encodeImageAsset` builds one
// and nothing else produces a content part in this pipeline. The remote branch
// exists because the shared type permits a URL and a silent mis-send would be
// worse than a branch nobody exercises.

/**
 Scheme prefix an inline data URI opens with.
 */
const DATA_PREFIX = 'data:';

/**
 Separator between a data URI's metadata and its payload.
 */
const PAYLOAD_MARK = ',';

/**
 Separator between the media type and the parameters after it.
 */
const PARAMETER_MARK = ';';

/**
 Encoding parameter the Messages API can take inline.
 */
const BASE64 = 'base64';

/**
 Refusal raised when a picture's data URI cannot be read.

 THROWN RATHER THAN DROPPED. A picture that reaches a model unreadable is a
 defect in our own encoder, not an unreliable answer, and dropping it would
 ask a model to transcribe an image it was never shown.

 @example
 ```ts
 throw new MalformedImageUriError({ detail: 'no payload separator', },);
 ```
 */
export class MalformedImageUriError extends Error {
  /**
   Declares this message safe to forward: it names which part of the data URI is wrong, never its payload.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming what could not be read.

   @param detail - which part of the data URI was missing or unexpected

   @example
   ```ts
   new MalformedImageUriError({ detail: 'encoding is not base64', },);
   ```
   */
  public constructor(
    { detail, }: { readonly detail: string; },
  ) {
    super(`Image content part is not a readable data URI: ${detail}`,);
    this.name = 'MalformedImageUriError';
  }
}

/**
 Where the Messages API is told to find one picture.

 @example
 ```ts
 const source: AnthropicImageSource = { type: 'base64', media_type: 'image/webp', data, };
 ```
 */
export type AnthropicImageSource =
  | {
    /**
     Discriminator marking a picture carried inline.
     */
    readonly type: 'base64';

    /**
     Media type the payload is encoded from.
     */
    readonly media_type: string;

    /**
     Base64 payload, without the URI metadata around it.
     */
    readonly data: string;
  }
    | {
    /**
     Discriminator marking a picture the provider fetches itself.
     */
      readonly type: 'url';

    /**
     Address the picture is fetched from.
     */
      readonly url: string;
    };

/**
 One run of content inside an Anthropic message.

 @example
 ```ts
 const block: AnthropicContentBlock = { type: 'text', text: 'Count the toebeans.', };
 ```
 */
export type AnthropicContentBlock =
  | {
    /**
     Discriminator marking a run of text.
     */
    readonly type: 'text';

    /**
     Text of this run.
     */
    readonly text: string;
  }
    | {
    /**
     Discriminator marking a picture.
     */
      readonly type: 'image';

    /**
     Where the picture is.
     */
      readonly source: AnthropicImageSource;
    };

/**
 Reads one picture reference into the source shape the Messages API takes.

 @param url - remote address or inline data URI a content part carried

 @returns Inline source for a data URI, remote source for anything else

 @throws {@link MalformedImageUriError} where a data URI carries no payload
 separator, an empty media type, an encoding other than base64, or no payload

 @example
 ```ts
 const source = readImageSource({ url: encoded.dataUri, },);
 ```
 */
export function readImageSource(
  { url, }: { readonly url: string; },
): AnthropicImageSource {
  if (!url.startsWith(DATA_PREFIX,))
    return {
      type: 'url',
      url,
    };

  /**
   Where the metadata ends and the payload begins.
   */
  const mark = url.indexOf(PAYLOAD_MARK,);

  if (mark === (-1))
    throw new MalformedImageUriError({ detail: 'no separator between metadata and payload', },);

  /**
   Media type and its parameters, between the scheme and the payload.
   */
  const parameters = url
    .slice(
    DATA_PREFIX.length,
    mark,
    )
    .split(PARAMETER_MARK,);

  /**
   Media type, which every part after the first qualifies.

   DEFAULTED because a split always yields a first element and the type cannot
   say so. An empty media type is refused by the `mediaType.length === 0` check either way, so the default
   changes no outcome and asserts nothing.
   */
  const [mediaType = '',] = parameters;

  if (mediaType.length === 0)
    throw new MalformedImageUriError({ detail: 'media type is empty', },);

  if (!parameters.includes(BASE64,))
    throw new MalformedImageUriError({
      detail: 'encoding is not base64, which is the only inline encoding the Messages API takes',
    },);

  /**
   Payload after the separator.
   */
  const data = url.slice(mark + 1,);

  if (data.length === 0)
    throw new MalformedImageUriError({ detail: 'payload is empty', },);

  return {
    type: BASE64,
    media_type: mediaType,
    data,
  };
}

/**
 Content blocks one message becomes.

 A PLAIN-TEXT MESSAGE STILL BECOMES A BLOCK ARRAY rather than a bare string.
 The Messages API takes either, and one shape means one path to test and no
 branch that only the vision half exercises.

 @param message - message in whichever shape the caller built

 @returns Blocks in the order the model reads them

 @throws {@link MalformedImageUriError} where a picture cannot be read

 @example
 ```ts
 const content = contentBlocksFor({ message, },);
 ```
 */
export function contentBlocksFor(
  { message, }: { readonly message: ChatMessage | VisionMessage; },
): readonly AnthropicContentBlock[] {
  if ((typeof message.content) === 'string')
    return [
      {
        type: 'text',
        text: message.content,
      },
    ];

  return message
    .content
    .map(function toBlock(part,): AnthropicContentBlock {
      if (part.type === 'text')
        return {
          type: 'text',
          text: part.text,
        };

      return {
        type: 'image',
        source: readImageSource({
          url: part
            .image_url
            .url,
        },),
      };
    },);
}

//endregion Anthropic content
