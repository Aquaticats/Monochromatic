import type { ChatTextRequest, } from './chat-contract.ts';
import { imagePixelSizeOf, } from './neuralwatt-image-size.ts';
import { promptTokensOfBody, } from './neuralwatt-estimate.ts';

//region NeuralWatt prompt tokens of a request
// PROMPT TOKENS A REQUEST STANDS FOR, A PICTURE COUNTED AS A PICTURE, for the
// energy estimate (`neuralwatt-estimate.ts`).
//
// WHAT WENT WRONG. The estimate read prompt tokens off the body's bytes, four
// to a token, and a picture travels as a base64 data URI inside that body:
// `image0.webp` (1271156 bytes, 2400 by 2616) made a body of 1.7 million bytes
// and 423985 tokens, the top bucket (0.9089 Wh for `glm-5.3-flash`), while the
// provider reported 8188 prompt tokens and the call measured 0.2311 Wh
// (XingZ6016, 2026-10-21, `SPEND` at log line 533 and the settlement line
// after it). The same seat read two 576 by 1280 pictures for 1156 reported
// tokens each against 26957 and 32466 estimated (`zheermao19` to `zheermao21`).
//
// NEURALWATT DOCUMENTS NO IMAGE ACCOUNTING. Its chat completions page, models
// page and OpenAPI file (fetched 2026-10-21) say only that an `image_url` part
// is read by a model with `vision` and cap the images per turn (20 for this
// model). The count here is therefore fitted to what the provider reported, on
// two pictures: round(width / 28) times round(height / 28) tokens each, and
// 190 tokens of text and framing around them. 2400 by 2616 is 86 by 93, 7998,
// and 7998 plus 190 is the reported 8188; 576 by 1280 is 21 by 46, 966, and 966
// plus 190 is the reported 1156. THE 190 WAS FITTED ON THE FIRST PICTURE AND
// THE SECOND IS THE CHECK; it is a two-point fit and a guess for any other
// size, and for the DeepSeek seat, which has never read a picture here. The
// estimate only sizes the hold a call has in flight, because the measured
// energy replaces it at settlement.
//
// A PICTURE OF UNREADABLE SIZE IS HELD AT A CEILING: 8192 tokens, a little
// above the largest count reported (8188 for the 2400 by 2616 picture, about
// 190 of it text), so an unreadable header over-holds a little instead of
// under-holding. A picture larger than that one is held at its patches, which
// the provider may cap or downscale; nothing here was measured above 2400 by
// 2616.

/**
 Prompt tokens held for a picture whose size cannot be read, and for one that
 is not inline.
 */
export const NEURALWATT_UNSIZED_PICTURE_TOKENS = 8_192;

/**
 Pixels along one side that make a token's patch: the grid the fit described in this module's note found.
 */
const PICTURE_PATCH_PIXELS = 28;

/**
 What stands between an inline picture's media type and its base64 payload.
 */
const BASE64_MARKER = ';base64,';

/**
 Tokens one picture stands for, by the size its header states.

 @param width - pixels across

 @param height - pixels down

 @returns Patch tokens, at least one

 @example
 ```ts
 pictureTokensOfSize({ width: 576, height: 1280, },); // 966
 ```
 */
export function pictureTokensOfSize(
  {
    width,
    height,
  }: {
    readonly width: number;
    readonly height: number;
  },
): number {
  /**
   Patches across.
   */
  const across = Math.max(
    1,
    Math.round(width / PICTURE_PATCH_PIXELS,),
  );

  /**
   Patches down.
   */
  const down = Math.max(
    1,
    Math.round(height / PICTURE_PATCH_PIXELS,),
  );
  return across * down;
}

/**
 Tokens one picture part stands for.

 @param url - the part's `image_url.url`: a data URI or a remote address

 @returns Patch tokens when the inline file's header gives a size, else the ceiling

 @example
 ```ts
 const tokens = pictureTokensOfUrl({ url, },);
 ```
 */
export function pictureTokensOfUrl({ url, }: { readonly url: string; },): number {
  if (!url.includes(BASE64_MARKER,))
    return NEURALWATT_UNSIZED_PICTURE_TOKENS;

  /**
   The file, decoded from the payload that follows the marker.
   */
  const bytes = Buffer.from(
    url.slice(url.indexOf(BASE64_MARKER,) + BASE64_MARKER.length,),
    'base64',
  );

  /**
   Size the file's header states.
   */
  const size = imagePixelSizeOf({ bytes, },);
  if (size.kind === 'unsized')
    return NEURALWATT_UNSIZED_PICTURE_TOKENS;
  return pictureTokensOfSize({
    width: size.width,
    height: size.height,
  },);
}

/**
 Every picture URL a conversation carries, in order.

 @param messages - conversation the body carries

 @returns The `image_url.url` of each picture part

 @example
 ```ts
 const urls = pictureUrlsOf({ messages, },);
 ```
 */
function pictureUrlsOf({ messages, }: { readonly messages: ChatTextRequest['messages']; },): readonly string[] {
  return messages.flatMap(function urlsOfMessage(message,): readonly string[] {
    if ((typeof message.content) === 'string')
      return [];
    return (message.content as Exclude<typeof message.content, string>).flatMap(function urlOfPart(part,): readonly string[] {
      if (part.type !== 'image_url')
        return [];
      return [part.image_url
        .url,];
    },);
  },);
}

/**
 Sum of some numbers.

 @param values - the numbers

 @returns Their sum, zero for none

 @example
 ```ts
 sumOf({ values: [1, 2,], },); // 3
 ```
 */
function sumOf({ values, }: { readonly values: readonly number[]; },): number {
  return values.reduce(
    function total(
      sum,
      value,
    ): number {
      return sum + value;
    },
    0,
  );
}

/**
 Prompt tokens a request body stands for: its text bytes over four, plus each
 picture by its patches.

 @param messages - conversation the body carries

 @param bodyBytes - size of the JSON body about to be sent, pictures included

 @returns Prompt tokens, rounded up so a body never reads as lighter than it is

 @example
 ```ts
 const promptTokens = promptTokensOfRequest({ messages, bodyBytes: Buffer.byteLength(bodyJson,), },);
 ```
 */
export function promptTokensOfRequest(
  {
    messages,
    bodyBytes,
  }: {
    readonly messages: ChatTextRequest['messages'];
    readonly bodyBytes: number;
  },
): number {
  /**
   Every picture the conversation carries.
   */
  const urls = pictureUrlsOf({ messages, },);

  /**
   Bytes the inline pictures take in the body: a data URI is ASCII, one byte a character.
   */
  const inlineBytes = sumOf({
    values: urls
      .filter(function isInline(url,): boolean {
        return url.startsWith('data:',);
      },)
      .map(function bytesOf(url,): number {
        return url.length;
      },),
  },);

  /**
   Tokens the pictures stand for.
   */
  const pictureTokens = sumOf({
    values: urls.map(function tokensOf(url,): number {
      return pictureTokensOfUrl({ url, },);
    },),
  },);

  /**
   Text bytes: the body less the pictures, never below nothing.
   */
  const textBytes = Math.max(
    0,
    bodyBytes - inlineBytes,
  );
  return promptTokensOfBody({ bodyBytes: textBytes, },) + pictureTokens;
}

//endregion NeuralWatt prompt tokens of a request
