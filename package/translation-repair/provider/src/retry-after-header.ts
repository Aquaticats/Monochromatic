import { isUnsignedNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Retry-after header
// THE WAIT A REPLY'S `retry-after` HEADER NAMES, in milliseconds. The other providers state their wait in the body
// (`retry-stated-wait.ts`); Anthropic's 429 and 529 replies state it as a header of whole seconds, which the fetch
// transport hands back when the exchange names it (`captureHeaders`). The ladder takes the longer of the two.

/**
 Milliseconds in the seconds the header counts in.
 */
const MS_PER_SECOND = 1_000;

/**
 Letter that opens an exponent in a number's spelling, in lower case.
 */
const EXPONENT_MARK = 'e';

/**
 Name of the header, in the lower case the transport hands back.
 */
export const RETRY_AFTER_HEADER = 'retry-after';

/**
 Wait the `retry-after` header of a reply names.

 SECONDS ONLY, WRITTEN AS PLAIN DECIMAL DIGITS: the HTTP date form is not read, since Anthropic documents seconds,
 and a hex or exponent spelling is refused rather than read by `Number`. An unreadable value names no wait, which
 leaves the ladder on its own jitter.

 @param headers - the reply's captured headers, absent where none were

 @returns Milliseconds the header asks the caller to wait, or zero

 @example
 ```ts
 const ms = retryAfterHeaderMsOf({ headers: { 'retry-after': '3', }, },);
 // => 3000
 ```
 */
export function retryAfterHeaderMsOf(
  { headers, }: { readonly headers?: Readonly<Record<string, string>>; },
): number {
  /**
   The header as the reply carried it.
   */
  const raw = headers?.[RETRY_AFTER_HEADER];
  // A PLAIN DECIMAL ONLY: a hex or exponent spelling (`0x10`, `1e3`), which `Number` would read, is not a figure
  // Anthropic writes, and a signed or spelled one names no wait.
  if ((raw === undefined)
    || (!isUnsignedNumberText({ text: raw, },))
    || raw.toLowerCase()
      .includes(EXPONENT_MARK,))
    return 0;
  /**
   The header read as a number of seconds.
   */
  const seconds = Number(raw,);
  if (!(Number.isFinite(seconds,) && (seconds > 0)))
    return 0;
  return Math.round(seconds * MS_PER_SECOND,);
}

//endregion Retry-after header
