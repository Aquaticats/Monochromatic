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
 Codes a transport failure carries in the text Pi's adapters report for it.

 Each is the `code` undici or Node sets on the failure that stopped a request before the member
 produced anything: a host that never answered within the connect limit, a refused connection, a
 reset connection, and a name the resolver could not answer. Pi's own retry classifier lists the same
 family as transient (`@earendil-works/pi-ai` dist/utils/retry.js, `RETRYABLE_PROVIDER_ERROR_PATTERN`),
 which is why the pool must consume such a refusal before Pi restarts at the same member. A member's
 own HTTP refusal is not one of these, however its body reads: it carries a status, and the status
 decides what it is.
 */
export const TRANSPORT_FAILURE_CODES = [
  'UND_ERR_CONNECT_TIMEOUT',
  'ECONNREFUSED',
  'ECONNRESET',
  'UND_ERR_SOCKET',
  'ENOTFOUND',
  'EAI_AGAIN',
] as const;

/**
 Code a failed request carries when it is no transport failure; no code is the empty string, so a
 reader never has to model absence as a nullish value.
 */
export const NO_TRANSPORT_CODE = '';

/**
 Whether one character can extend a code into a longer identifier.

 `@earendil-works/pi-ai` reads its own codes the same way, refusing a code next to any lower-case
 letter, digit or underscore (dist/utils/retry.js, `RETRYABLE_PROVIDER_ERROR_PATTERN`), so a request
 id or a byte count that happens to contain one is no code.

 @param character - case-folded character to classify

 @returns whether the character belongs to an identifier

 @example
 ```ts
 isIdentifierCharacter('x'); // true
 ```
 */
function isIdentifierCharacter(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '_');
}

/**
 Whether the characters around a span keep it a token of its own.

 @param text - case-folded text holding the code

 @param start - offset the code begins at

 @param end - offset just past the code

 @returns whether neither neighbour extends it into a longer identifier
 */
function standsAlone({
  text,
  start,
  end,
}: {
  readonly text: string;
  readonly start: number;
  readonly end: number;
},): boolean {
  /**
   Character before the code, absent at the start of the text.
   */
  const before = text[start - 1];
  /**
   Character after the code, absent at the end of the text.
   */
  const after = text[end];
  return ((before === undefined) || (!isIdentifierCharacter(before,)))
    && ((after === undefined) || (!isIdentifierCharacter(after,)));
}

/**
 Code a transport failure reports, in either of the two spellings Pi's adapters produce.

 The adapter names the deepest cause of a failed fetch and appends a code the message does not
 already spell (`@earendil-works/pi-ai` dist/utils/error-body.js, `withCauseDetail`): a resolver or
 socket error whose own message carries the code reads `getaddrinfo ENOTFOUND host`, and one whose
 message does not reads `other side closed (UND_ERR_SOCKET)`. Both are read as tokens.

 @param value - terminal error text from an adapter

 @returns the first code of the set the text names, or {@link NO_TRANSPORT_CODE} when it names none

 @example
 ```ts
 transportFailureCode('generate ECONNREFUSED 127.0.0.1:443'); // 'ECONNREFUSED'
 ```
 */
export function transportFailureCode(value: string,): string {
  /**
   Case-folded terminal message, so a code a body or a transport wrote in lower case still reads.
   */
  const text = foldProviderText(value,);
  for (const code of TRANSPORT_FAILURE_CODES) {
    /**
     Case-folded spelling to search for.
     */
    const needle = foldProviderText(code,);
    for (
      let start = text.indexOf(needle,); start !== (-1);
      start = text.indexOf(
        needle,
        start + needle.length,
      )
    ) {
      if (standsAlone({
        text,
        start,
        end: start + needle.length,
      },))
        return code;
    }
  }
  return NO_TRANSPORT_CODE;
}

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
