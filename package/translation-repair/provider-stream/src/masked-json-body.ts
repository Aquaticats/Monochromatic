import {
  credentialsOfHeaders,
  maskCredentials,
} from './credential-mask.ts';

//region Masked JSON body
// READS AN ANSWER'S BODY AS JSON WITH EVERY CREDENTIAL THE REQUEST CARRIED
// MASKED OUT OF ITS TEXT FIRST.
//
// A lookup endpoint may echo the key it was sent, and what a lookup reads is
// kept: a hit's title and highlight, and a fetched page's title and text, go
// into a sheet a model reads and into a cache on disk. The mask is applied to
// the text before it is parsed, so nothing parsed out of it can carry the key,
// and a parse failure's message, which quotes the text it refused, quotes the
// masked text.

/**
 Reads a response body as JSON from its masked text.

 @param response - endpoint's answer, whose body is read once, here

 @param headers - headers the request carried, which name the credentials the
 body must not repeat

 @returns The parsed body, read from text with every credential of the request
 masked out of it

 @throws SyntaxError when the masked text is not JSON, unchanged

 @throws Whatever the body stream raises while it arrives, unchanged, so the
 caller says which endpoint was answering

 @example
 ```ts
 const parsed = await readMaskedJsonBody({ response, headers: { 'x-api-key': apiKey, }, },);
 ```
 */
export async function readMaskedJsonBody(
  {
    response,
    headers,
  }: {
    readonly response: Response;
    readonly headers: Readonly<Record<string, string>>;
  },
): Promise<unknown> {
  /**
   Body as the endpoint sent it.
   */
  const text = await response.text();
  return JSON.parse(maskCredentials({
    text,
    credentials: credentialsOfHeaders({ headers, },),
  },),);
}

//endregion Masked JSON body
