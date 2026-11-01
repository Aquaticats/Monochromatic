//region Body-cut response
// A RESPONSE THAT ANSWERS 200 AND THEN FAILS WHILE ITS BODY STREAMS, for cases
// that read what a lookup does when the connection drops after the status
// line came back.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The two lookups' cases both build it.

/**
 Builds a 200 response whose body stream errors on its first read.

 @param failure - what the connection fails with while the body arrives

 @returns The response, whose `json()` rejects with the failure

 @example
 ```ts
 const response = bodyCutBy({ failure: new TypeError('terminated',), },);
 ```
 */
export function bodyCutBy({ failure, }: { readonly failure: Error; },): Response {
  return new Response(
    new ReadableStream({
      start(controller,): void {
        controller.error(failure,);
      },
    },),
    { status: 200, },
  );
}

/**
 The rejection the runtime's `fetch` raises for a header value it will not
 send, which quotes the value and carries no cause.

 @returns The rejection

 @example
 ```ts
 throw invalidHeaderRejection();
 ```
 */
export function invalidHeaderRejection(): TypeError {
  return new TypeError('Headers.append: "fake\nvalue" is an invalid header value.',);
}

//endregion Body-cut response
