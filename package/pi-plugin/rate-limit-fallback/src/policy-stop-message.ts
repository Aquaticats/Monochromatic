/**
 Preserve policy advisories while preventing Pi from treating them as transient failures.
 @module
 */

//region Pi AI 1.5.2 transient-error grammar, from dist/utils/retry.js

/**
 Literal retry triggers after expanding fixed alternatives and folding ASCII case.
 */
const RETRY_PHRASES = [
  'overloaded',
  'server_busy',
  'servers are currently busy',
  'currently experiencing high demand',
  'model is at capacity',
  'too many requests',
  'exceeded request buffer limit while retrying upstream',
  'unable to forward upstream',
  'upstream_stream_read_error',
  'stream was interrupted',
  'http2 stream failed',
  'http/2 stream failed',
  'other side closed',
  'fetch failed',
  'getaddrinfo',
  'enotfound',
  'eai_again',
  'reset before headers',
  'socket hang up',
  'socket connection was closed',
  'time out',
  'timed out',
  'timeout',
  'terminated',
  'ended without',
  'stream ended before message_stop',
  'stream ended before a terminal response event',
  'http2 request did not get a response',
  'stream disconnected before completion',
  'cut short after the reply had begun',
  'pending stream has been canceled',
  'retry delay',
  'you can retry your request',
  'try your request again',
  'please retry your request',
  'resourceexhausted',
  'subscription_sharing_usage_unavailable',
  'subscription_sharing_user_unavailable',
] as const;

/**
 Phrases whose parts may be adjacent or separated by one non-line-ending code unit.
 */
const SEPARATED_PHRASES = [
  [
    'rate',
    'limit',
  ],
  [
    'service',
    'unavailable',
  ],
  [
    'service',
    'is',
    'unavailable',
  ],
  [
    'service',
    'temporarily',
    'unavailable',
  ],
  [
    'service',
    'is',
    'temporarily',
    'unavailable',
  ],
  [
    'server',
    'error',
  ],
  [
    'internal',
    'error',
  ],
  [
    'provider',
    'returned',
    'error',
  ],
  [
    'provider',
    'returned',
    'an',
    'error',
  ],
  [
    'upstream',
    'request',
    'failed',
  ],
  [
    'network',
    'error',
  ],
  [
    'connection',
    'error',
  ],
  [
    'connection',
    'refused',
  ],
  [
    'connection',
    'lost',
  ],
  [
    'upstream',
    'connect',
  ],
  [
    'websocket',
    'closed',
  ],
  [
    'websocket',
    'error',
  ],
] as const;

/**
 Retryable status tokens must not be embedded in ASCII letters or digits.
 */
const RETRY_STATUSES = [
  '429',
  '500',
  '502',
  '503',
  '504',
  '520',
  '524',
] as const;

/**
 Literal framing for the node-fetch failure with a non-whitespace destination.
 */
const FETCH_HEAD = 'request to ';

/** Literal suffix following the destination token in a node-fetch failure. */
const FETCH_TAIL = ' failed, reason:';

//endregion Pi AI 1.5.2 transient-error grammar, from dist/utils/retry.js

//region Index scans keep the original text and its offsets intact

/**
 Fold only ASCII letters, as Pi's case-insensitive non-Unicode patterns do for ASCII tokens.

 @param char - code point from provider text

 @returns folded point of the same width
 */
function foldAscii(char: string,): string {
  return (char >= 'A') && (char <= 'Z') ? char.toLowerCase() : char;
}

/**
 Check the word boundary used by Pi's status tokens.

 @param char - case-folded code unit beside a status token

 @returns whether the character joins a surrounding word or number to the status
 */
function asciiAlphanumeric(char = '',): boolean {
  return ((char >= 'a') && (char <= 'z')) || ((char >= '0') && (char <= '9'));
}

/**
 Find the longest match of a fixed phrase with an optional one-character separator at each join.
 The phrase lengths and number of joins are bounded by the pinned Pi vocabulary.

 @param value - case-folded provider text

 @param start - first offset of the candidate

 @param parts - fixed phrase components

 @returns exclusive end of the match, or start if none
 */
function separatedEnd({
  value,
  start,
  parts: [head, ...tail],
}: {
  readonly value: string;
  readonly start: number;
  readonly parts: readonly [
    string,
    ...string[]
  ];
},): number {
  if (!value.startsWith(
    head,
    start,
  ))
    return start;
  /**
   Offsets reachable at the end of the last matched component.
   */
  const matches = { ends: [start + head.length,], };
  for (const part of tail) {
    /**
     Offsets reachable after this component, with or without a separator.
     */
    const next: number[] = [];
    for (const end of matches.ends) {
      if (value.startsWith(
        part,
        end,
      ))
        next.push(end + part.length,);
      /**
       Pi's wildcard does not match a line ending.
       */
      const separator = value[end];
      if ((separator !== undefined) && (!'\r\n\u2028\u2029'.includes(separator,))
        && value.startsWith(
          part,
          end + 1,
        ))
        next.push(end + 1
          + part.length,);
    }
    if (next.length === 0)
      return start;
    matches.ends = next;
  }
  return Math.max(...matches.ends,);
}

/**
 Scan for the full node-fetch prefix, destination token, and closing phrase.
 Each destination is reached only after its own whitespace-delimited prefix.

 @param value - folded provider text

 @param start - candidate prefix offset

 @returns exclusive end of the match, or start if none
 */
function fetchFailureEnd({
  value,
  start,
}: {
  readonly value: string;
  readonly start: number
},): number {
  if (!value.startsWith(
    FETCH_HEAD,
    start,
  ))
    return start;
  /**
   Start of the destination token.
   */
  const destination = start + FETCH_HEAD.length;
  for (let end = destination; end < value.length; end += 1) {
    if (value.charAt(end,)
      .trim()
      !== '')
      continue;
    return (end > destination) && value.startsWith(
      FETCH_TAIL,
      end,
    ) ? end + FETCH_TAIL.length : start;
  }
  return start;
}

/**
 Find a retry trigger starting at this offset without matching a status within a reference code.

 @param value - folded provider text

 @param start - next offset of the scan

 @returns exclusive end of the longest trigger at this position, or start if none
 */
function retryTriggerEnd({
  value,
  start,
}: {
  readonly value: string;
  readonly start: number
},): number {
  /**
   Furthest matching end at the current start.
   */
  let end = fetchFailureEnd({
    value,
    start,
  },);
  for (const phrase of RETRY_PHRASES) {
    if (value.startsWith(
      phrase,
      start,
    ))
      end = Math.max(
        end,
        start + phrase.length,
      );
  }
  for (const parts of SEPARATED_PHRASES)
    end = Math.max(
      end,
      separatedEnd({
        value,
        start,
        parts,
      },),
    );
  if (!asciiAlphanumeric(value[start - 1],)) {
    for (const status of RETRY_STATUSES) {
      if (value.startsWith(
        status,
        start,
      ) && (!asciiAlphanumeric(value[start + status.length],)))
        end = Math.max(
          end,
          start + status.length,
        );
    }
  }
  return end;
}

/**
 Attribute the policy stop and mask only the spans that Pi checks for retries.
 Pi exposes a boolean classifier, not the matching spans. Keep the scan in step with Pi on upgrade.

 @param memberLabel - physical provider and model that stopped

 @param message - original provider policy message

 @returns attributed advisory with transient-error wording omitted

 @example
 ```ts
 policyStopMessage({ memberLabel: 'openai/model', message: '429: blocked by content policy' });
 ```
 */
export function policyStopMessage({
  memberLabel,
  message,
}: {
  readonly memberLabel: string;
  readonly message: string;
},): string {
  /**
   One-to-one folded comparison string; slices come from the untouched message.
   */
  const value = Array.from(
    message,
    foldAscii,
  )
    .join('',);
  /**
   Untouched spans and markers; the marker does not join words into another retry trigger.
   */
  const spans: string[] = [];
  /**
   Next untouched offset in the original advisory.
   */
  const scan = { retained: 0, };
  for (let start = 0; start < value.length;) {
    /**
     Exclusive end of any Pi retry wording at the current offset.
     */
    const end = retryTriggerEnd({
      value,
      start,
    },);
    if (end === start) {
      start += 1;
      continue;
    }
    spans.push(
      message.slice(
        scan.retained,
        start,
      ),
      '[omitted]',
    );
    scan.retained = end;
    start = end;
  }
  spans.push(message.slice(scan.retained,),);
  return `${memberLabel} stopped the request under its content policy: ${spans.join('',)}`;
}

//endregion Index scans keep the original text and its offsets intact
