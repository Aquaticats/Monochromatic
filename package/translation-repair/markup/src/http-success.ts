//region Http success
// WHAT COUNTS AS A SUCCESSFUL REPLY, in one place rather than in each client.
//
// Both providers answer over plain HTTP and both clients ask the same question
// of the status before they try to read a body. The check lived inside the
// first client, so the second would have had to restate the two boundary
// numbers, and a pair of magic numbers restated in two files is a pair that
// can disagree.

/**
 Lowest HTTP status treated as success.
 */
const HTTP_SUCCESS_MIN = 200;

/**
 First HTTP status past the success family.
 */
const HTTP_SUCCESS_MAX_EXCLUSIVE = 300;

/**
 Whether a status says the body is worth reading as an answer.

 A REDIRECT IS NOT A SUCCESS HERE. Neither provider redirects an API call,
 and a 3xx body carries no completion, so admitting one would send an empty
 or HTML body into a parser that reports it as a provider contract violation
 rather than as the misrouted request it is.

 @param status - HTTP status the transport reported

 @returns Whether the reply carries an answer rather than a failure

 @example
 ```ts
 if (!isSuccessStatus({ status: reply.status, },)) throw new ProviderHttpError({ provider: 'hyper', status, bodyText, },);
 ```
 */
export function isSuccessStatus(
  { status, }: { readonly status: number; },
): boolean {
  return (status >= HTTP_SUCCESS_MIN) && (status < HTTP_SUCCESS_MAX_EXCLUSIVE);
}

//endregion Http success
