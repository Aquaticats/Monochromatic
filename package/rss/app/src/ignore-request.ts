/**
 The ignore endpoint's request-body contract.

 The endpoint is unauthenticated and appends to a file on disk, so a body is
 checked against the documented record shape and a size bound before any write
 happens.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import * as v from 'valibot';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the ignore-request module.
 */
const l = tagged({
  tag: 'ignore-request',
  l: parentLogger,
},);

/**
 Largest request body the endpoint accepts.
 
 The documented body is one JSON object naming a feed link or an item's
 rendered metadata (issue 1602 asks for the bound): a few hundred bytes in
 practice, so 8 KiB leaves room for a long link or a verbose metadata card
 while stopping one client from appending megabytes to the ignore file.
 */
const MAXIMUM_BODY_BYTES = 8_192;

/**
 UTF-8 encoder measuring a body that arrived without a declared length.
 */
const textEncoder = new TextEncoder();

/**
 Record the endpoint stores: the item's link when the reader resolved one, or
 the rendered metadata of an item whose link it could not read.
 */
export type IgnoreRecord = { readonly link: string; }
  | { readonly metadataOuterHtml: string; };

/**
 Validates an ignore request body against {@link IgnoreRecord}, dropping every
 undeclared field so only the documented record reaches the file.
 */
const IGNORE_RECORD_SCHEMA: v.GenericSchema<unknown, IgnoreRecord> = v.union([
  v.object({
    link: v.pipe(
      v.string(),
      v.nonEmpty(),
    ),
  },),
  v.object({
    metadataOuterHtml: v.pipe(
      v.string(),
      v.nonEmpty(),
    ),
  },),
],);

/**
 Rejection of a request body the ignore endpoint refuses to store.
 
 Carries the message the HTTP response reports, so a client learns which rule
 its body broke.
 */
export class IgnoreRequestBodyError extends Error {
  /**
   Builds one rejection carrying the reason a client reads.
   
   @param message - rejection reason reported to the client
   
   @param cause - underlying failure, when one exists
   */
  constructor(
    message: string,
    cause?: unknown,
  ) {
    super(
      message,
      { cause, },
    );
    this.name = 'IgnoreRequestBodyError';
  }
}

/**
 Parses request text as JSON.
 
 @param body - raw request body text
 
 @returns parsed value, unknown until a schema narrows it
 
 @throws when the text is not valid JSON
 */
function parseRequestBodyJson(body: string,): unknown {
  try {
    /**
     Parsed JSON value, unknown until {@link IGNORE_RECORD_SCHEMA} narrows it.
     */
    const parsed: unknown = JSON.parse(body,);
    return parsed;
  }
  catch (error) {
    throw new IgnoreRequestBodyError(
      `body is not JSON: ${caughtValueText(error,)}`,
      error,
    );
  }
}

/**
 Parses and validates one ignore request body against {@link IGNORE_RECORD_SCHEMA}.
 
 @param body - raw request body text
 
 @returns validated record holding only documented fields
 
 @throws when the text is not JSON or carries no documented field
 */
function parseIgnoreRecord(body: string,): IgnoreRecord {
  /**
   Parsed body, unknown until the schema narrows it.
   */
  const parsedBody = parseRequestBodyJson(body,);
  /**
   Validation result kept so a rejected shape reports one message instead of a
   ValiError the HTTP layer would treat as a server fault.
   */
  const validated = v.safeParse(
    IGNORE_RECORD_SCHEMA,
    parsedBody,
  );
  if (!validated.success) {
    throw new IgnoreRequestBodyError(
      `body must carry a non-empty link or metadataOuterHtml: ${v.summarize(validated.issues,)}`,
    );
  }
  return validated.output;
}

/**
 Reads and validates an ignore request body, refusing one that breaks the
 documented contract or exceeds {@link MAXIMUM_BODY_BYTES}.
 
 @param request - incoming request whose body names one ignored item
 
 @returns validated record holding only documented fields
 
 @throws when the body is not JSON, carries no documented field, or exceeds
 the accepted size
 
 @example
 ```ts
 const record = await readIgnoreRecord(new Request('http://localhost', {
   method: 'POST',
   body: '{"link":"https://example.com"}',
 }));
 ```
 */
export async function readIgnoreRecord(request: Request,): Promise<IgnoreRecord> {
  /**
   Declared length in bytes, absent when the client streams its body.
   */
  const declaredLength = request
    .headers
    .get('content-length',);
  if ((declaredLength !== null) && (Number(declaredLength,) > MAXIMUM_BODY_BYTES)) {
    l.warn(`rejected ignore request: body declares ${declaredLength} bytes`,);
    throw new IgnoreRequestBodyError(
      `body exceeds ${String(MAXIMUM_BODY_BYTES,)} bytes`,
    );
  }
  /**
   Raw body kept only long enough to validate and re-serialize.
   */
  const body = await request.text();
  /**
   Byte length of the body as UTF-8, so a chunked body is bounded too.
   */
  const bodyBytes = textEncoder
    .encode(body,)
    .byteLength;
  if (bodyBytes > MAXIMUM_BODY_BYTES) {
    l.warn(`rejected ignore request: body measured ${String(bodyBytes,)} bytes`,);
    throw new IgnoreRequestBodyError(
      `body exceeds ${String(MAXIMUM_BODY_BYTES,)} bytes`,
    );
  }
  return parseIgnoreRecord(body,);
}
