/**
 Index scans for the ASCII tokens reported in provider error messages and reset headers.

 @module
 */

/**
 First ASCII decimal digit.
 */
const ZERO = '0';

/**
 Last ASCII decimal digit.
 */
const NINE = '9';

/**
 HTTP status indicating a rate limit.
 */
export const HTTP_RATE_LIMIT_STATUS = 429;

/**
 HTTP status a provider uses for capacity it cannot serve now.

 NeuralWatt documents 503 as fleet capacity rather than a permanent refusal
 (https://docs.neuralwatt.com/guides/rate-limits.md), and the owner's decision of 2026-11-19 moves the
 request to the next member when it arrives, like a 429.
 */
const HTTP_UNAVAILABLE_STATUS = 503;

/**
 First Cloudflare origin status, "Web server is returning an unknown error".
 */
const CLOUDFLARE_ORIGIN_STATUS_START = 520;

/**
 Last Cloudflare origin status, "Error 526: invalid SSL certificate".

 Cloudflare documents 520 to 526 as its own origin failures
 (https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-5xx-errors/),
 and the owner's decision of 2026-11-19 moves the request to the next member for all of them.
 */
const CLOUDFLARE_ORIGIN_STATUS_END = 526;

/**
 Status token a Responses adapter prints inside its failure label, after the provider word.
 */
const RESPONSES_LABEL = ' api error (';

/**
 Fold the ASCII comparisons the wire grammars make, including the Unicode equivalents of `k` and `s`.

 @param value - provider message to compare without case

 @returns comparison text

 @example
 ```ts
 foldProviderText('HTTP 429'); // 'http 429'
 ```
 */
export function foldProviderText(value: string,): string {
  return value.toLowerCase()
    .replaceAll(
    'ſ',
    's',
    );
}

/**
 Find the end of the ASCII decimal digits starting at an offset.

 @param value - text containing an unsigned whole-number component

 @param offset - first character to inspect

 @returns exclusive end, or offset when there are no digits

 @example
 ```ts
 digitsEnd({ value: '12ms', offset: 0 }); // 2
 ```
 */
export function digitsEnd({
  value,
  offset,
}: {
  readonly value: string;
  readonly offset: number;
},): number {
  for (let cursor = offset; cursor < value.length; cursor += 1) {
    /**
     Character to compare against the ordered ASCII digit range.
     */
    const digit = value[cursor];
    if ((digit === undefined) || (digit < ZERO)
      || (digit > NINE))
      return cursor;
  }
  return value.length;
}

/**
 Parse the unsigned whole number starting at an offset.

 @param value - text holding the digits

 @param offset - first digit

 @returns the number, or NaN when no digit starts there

 @example
 ```ts
 numberAt({ value: '429: busy', offset: 0 }); // 429
 ```
 */
function numberAt({
  value,
  offset,
}: {
  readonly value: string;
  readonly offset: number;
},): number {
  /**
   Exclusive end of the digit run.
   */
  const end = digitsEnd({
    value,
    offset,
  },);
  return end === offset ? Number.NaN : Number(value.slice(
    offset,
    end,
  ),);
}

/**
 Whether one status means "this provider cannot serve the request now" rather than "its answer is the
 request's answer".

 @param status - HTTP status an adapter exposed or a message named

 @returns whether the pool may try the next member

 @example
 ```ts
 isTransientStatus(523); // true
 ```
 */
export function isTransientStatus(status: number,): boolean {
  return (status === HTTP_UNAVAILABLE_STATUS)
    || ((status >= CLOUDFLARE_ORIGIN_STATUS_START) && (status <= CLOUDFLARE_ORIGIN_STATUS_END));
}

/**
 Status a message opens with when it opens with none; no HTTP status is zero.
 */
const NO_STATUS = 0;

/**
 Read the HTTP status a provider message opens with, in the forms the installed adapters print.

 A Completions refusal opens `<status>: <body>`; a Responses refusal opens
 `<provider> API error (<status>): <body>` (`@earendil-works/pi-ai` dist/utils/error-body.js,
 `formatProviderError`); a recorded refusal may also begin `<status> "<body>"`. A number that merely
 starts with a status, such as `4290 tokens`, is not one.

 @param value - terminal error text from an adapter

 @returns the leading status, or {@link NO_STATUS} when the text opens with none

 @example
 ```ts
 leadingStatus('523: <html>'); // 523
 ```
 */
export function leadingStatus(value: string,): number {
  /**
   Case-folded terminal message, with no leading whitespace removed.
   */
  const message = foldProviderText(value,);
  // An anchored status is the refusal's own, so a body that quotes a label cannot change it.
  /**
   Status at the very start of the text, before any body.
   */
  const leading = numberAt({
    value: message,
    offset: 0,
  },);
  if (!Number.isNaN(leading,)) {
    /**
     Digits of that status, as the message carries them.
     */
    const leadingDigits = String(leading,);
    /**
     Character after the status token, absent when the status is the whole message.
     */
    const boundary = message[leadingDigits.length];
    if ((boundary === undefined)
      || (boundary === ':')
      || (boundary === ' ')
      || (boundary === '\t'))
      return leading;
  }
  /**
   Offset where a Responses adapter's failure label begins, or minus one when absent.
   */
  const labelAt = message.indexOf(RESPONSES_LABEL,);
  if (labelAt <= 0)
    return NO_STATUS;
  // The adapter's label is one word, so whitespace before it means the text is not one.
  for (let index = 0; index < labelAt; index += 1) {
    /**
     Case-folded character at the current label position.
     */
    const character = message[index];
    /**
     Whether that character is any whitespace a label cannot hold.
     */
    const whitespace = (character === ' ')
      || (character === '\t')
      || (character === '\n')
      || (character === '\r');
    if (whitespace)
      return NO_STATUS;
  }
  /**
   Offset of the first status digit inside the label.
   */
  const digitsAt = labelAt + RESPONSES_LABEL.length;
  /**
   Status the label carries.
   */
  const status = numberAt({
    value: message,
    offset: digitsAt,
  },);
  if (Number.isNaN(status,))
    return NO_STATUS;
  /**
   Digits of that status, as the label carries them.
   */
  const statusDigits = String(status,);
  /**
   Offset just past the digits.
   */
  const closeAt = digitsAt + statusDigits.length;
  return (message[closeAt] === ')') && (message[closeAt + 1] === ':') ? status : NO_STATUS;
}
