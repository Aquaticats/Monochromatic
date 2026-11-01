import { ProviderHttpError, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { retryAfterHeaderMsOf, } from './retry-after-header.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Request size refusal
// The gateway caps request bodies and reports a body over that cap as a JSON
// parse failure at a byte offset. That message describes the serialisation we
// sent, which is well-formed, so a caller reading it looks for a defect in our
// encoder and finds none. Re-raising the one case where the cap explains it is
// what makes the failure legible.
//
// `doc/troubleshooting/synthetic-request-body-size-cap.md` holds the
// measurement this rests on.

/**
 Largest request body measured to reach the gateway intact.

 EXACT, UNLIKE ITS COUNTERPART. This size was sent and accepted. The failing
 size opposite it is reported as approximate and the boundary between them has
 never been bisected, so this is the only number here that may be compared
 against.
 */
const PASSING_BODY_BYTES = 10_485_760;

/**
 Opening of the message the gateway returns for a body over its cap.

 MATCHED BY PREFIX rather than whole, because the tail carries the byte offset
 where the truncated body stopped parsing and that offset differs per request.
 */
const PARSE_FAILURE_MARK = 'Could not parse request as valid JSON';

/**
 Bad Request: the status an oversize body comes back as.
 */
const HTTP_BAD_REQUEST = 400;

/**
 Signals a request Synthetic's gateway refused for its size while naming
 something else.

 SYNTHETIC'S ALONE, as its name says. The cap and the parse-failure words were
 measured on that gateway only; whether Charm Hyper, OpenRouter or Bedrock
 refuse at any size, and in what words, is unmeasured, so no other provider's
 reply is read as this.

 A SUBCLASS RATHER THAN A SIBLING, so every caller branching on
 {@link ProviderHttpError} or reading its `status` keeps working. This is one
 kind of HTTP failure, not a separate failure mode, and a caller that does not
 care why a `400` arrived should not have to learn about this to keep catching
 it.

 @example
 ```ts
 throw new SyntheticRequestTooLargeError({
   status: 400,
   bodyText: 'Could not parse request as valid JSON. Unterminated string ...',
   bodyBytes: 11_185_335,
 },);
 ```
 */
export class SyntheticRequestTooLargeError extends ProviderHttpError {
  /**
   Declares this message safe to forward: it counts bytes.
   */
  readonly messageNamesOnly: true = true;

  /**
   Bytes the request body occupied on the wire, measured rather than
   estimated.
   */
  public readonly bodyBytes: number;

  /**
   Bytes a body is known to survive at, so a reader has both halves of the
   comparison without opening this file.
   */
  public readonly passingBodyBytes: number;

  /**
   Builds failure naming size, replacing the message its parent composed.

   @param status - status returned, carried through so `instanceof` callers
   branching on it see what they always saw

   @param bodyText - raw response body, excerpted by the parent constructor

   @param bodyBytes - measured wire size of what was sent

   @example
   ```ts
   new SyntheticRequestTooLargeError({
     status: 400,
     bodyText: 'Could not parse request as valid JSON.',
     bodyBytes: 11_185_335,
   },);
   ```
   */
  public constructor(
    {
      status,
      bodyText,
      bodyBytes,
    }: {
      readonly status: number;
      readonly bodyText: string;
      readonly bodyBytes: number;
    },
  ) {
    super({
      provider: 'synthetic',
      status,
      bodyText,
      // THE GATEWAY'S OWN WORDS ARE KEPT OFF THIS MESSAGE. This class declares
      // its message quote-free, and a gateway that echoed request bytes would
      // otherwise print corpus text at every CLI boundary; the words
      // stay on `bodyExcerpt`, which `exchangeFailureLogText` prints on the
      // lost-voice log lines, where the searchable byte position still is.
      excerpt: 'withheld',
      summary: `The Synthetic API refused a request body of ${String(bodyBytes,)} ${
        wordForCount({
          count: bodyBytes,
          one: 'byte',
          many: 'bytes',
        },)
      }, `
        + `${String(bodyBytes - PASSING_BODY_BYTES,)} above the ${String(PASSING_BODY_BYTES,)} `
        + `measured to pass. It answered HTTP ${String(status,)} naming a JSON parse failure, `
        + `which describes the body we serialised rather than its size, and that body is `
        + `well-formed. Send less in one call: fewer or smaller pictures, a shorter `
        + `instruction, or a smaller slice. `
        + `doc/troubleshooting/synthetic-request-body-size-cap.md records the measurement; `
        + `the gateway's own words are in the run log, not here`,
    },);
    this.name = 'SyntheticRequestTooLargeError';
    this.bodyBytes = bodyBytes;
    this.passingBodyBytes = PASSING_BODY_BYTES;
  }
}

/**
 Names why a non-success reply failed, re-reading the gateway's size refusal.

 FOUR SIGNALS TOGETHER, and the conjunction is the point. Status alone would
 catch every malformed request we ever send; the message alone would catch a
 body we genuinely broke; the size alone would catch an oversize request the
 gateway happened to accept and then fail for its own reasons; and only
 Synthetic's gateway was measured giving the other three for a body over its
 cap. Only all four describe a body refused for being too big, and any one of
 them missing leaves a plain {@link ProviderHttpError} saying exactly what it
 always said.

 AFTER THE FACT, NEVER BEFORE IT. Nothing here refuses a request. Only the
 passing size is exact, so a client-side guard at that number would reject
 bodies between it and the true boundary that the gateway may well carry.
 Reading an answer that already arrived cannot cost a call that would have
 worked.

 @param provider - provider that served the call, carried onto the plain failure; only Synthetic's reply is read
 for its size, since no other gateway's cap was measured

 @param status - status returned

 @param bodyText - raw response body, read for its message and excerpted into
 whichever failure is built

 @param requestBodyBytes - wire size of what was sent, which must be measured
 in bytes: this corpus is Chinese, and character counts run about a third of
 the bytes their UTF-8 costs

 @param headers - the reply's captured headers, absent where none were asked for; a `retry-after` among them rides
 on the failure for the budget layer to read

 @returns Failure to throw, size-naming only where all four signals agree

 @example
 ```ts
 throw failureForReply({
   provider: 'synthetic',
   status: reply.status,
   bodyText: reply.bodyText,
   requestBodyBytes: Buffer.byteLength(bodyJson,),
 },);
 ```
 */
export function failureForReply(
  {
    provider,
    status,
    bodyText,
    requestBodyBytes,
    headers,
  }: {
    readonly provider: ProviderName;
    readonly status: number;
    readonly bodyText: string;
    readonly requestBodyBytes: number;
    readonly headers?: Readonly<Record<string, string>>;
  },
): ProviderHttpError {
  /**
   Whether the status is the one an oversize body draws.
   */
  const refusedAsBadRequest = status === HTTP_BAD_REQUEST;

  /**
   Whether the gateway blamed our JSON, which is what it says about a body
   it stopped reading partway through.
   */
  const blamedOurJson = bodyText.includes(PARSE_FAILURE_MARK,);

  /**
   Whether what went out was larger than anything measured to arrive.
   */
  const overPassingSize = requestBodyBytes > PASSING_BODY_BYTES;

  /**
   Whether the reply came from the one gateway the cap was measured on.
   */
  const servedBySynthetic = provider === 'synthetic';

  if (
    refusedAsBadRequest
      && blamedOurJson
      && overPassingSize
      && servedBySynthetic
  ) {
    return new SyntheticRequestTooLargeError({
      status,
      bodyText,
      bodyBytes: requestBodyBytes,
    },);
  }

  /**
   Wait the headers named, zero where none was read.
   */
  const retryAfterMs = retryAfterHeaderMsOf((headers === undefined) ? {} : { headers, },);

  return new ProviderHttpError({
    provider,
    status,
    bodyText,
    ...((retryAfterMs === 0) ? {} : { retryAfterMs, }),
  },);
}

//endregion Request size refusal
