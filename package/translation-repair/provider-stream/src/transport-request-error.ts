//region Transport request error
// A REQUEST THE RUNTIME REJECTED BEFORE ANY ANSWER CAME BACK, stated by an
// authored account. The runtime's own rejection can quote a header value (a
// `TypeError` for a value `fetch` will not send does), so it stays unmarked and
// every log line names its class alone, which says nothing about what failed.
// The transport catches it where it is raised and throws this class, keeping
// the runtime's rejection as the cause.

/**
 Request the runtime rejected with no abort standing.

 THE ACCOUNT HOLDS FOR EVERY CAUSE IT COVERS: a network that could not be
 reached, a name that did not resolve, a connection refused or reset, and a
 request the runtime will not send, which is what a header value with a line
 break or another character a header cannot carry does. The rejection cannot
 tell these apart without reading its message, which may quote the value, so the
 sentence names the possibilities and picks none.

 @example
 ```ts
 throw new TransportRequestFailedError({ label: 'hf:whiskers', cause: error, },);
 ```
 */
export class TransportRequestFailedError extends Error {
  /**
   Declares this message safe to forward: it names the call's label and writes
   the rest itself.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure naming what the call was for.

   @param label - what the call was for, a model id or an endpoint name

   @param cause - the runtime's rejection, kept for a reader who owns the
   process and may read it

   @example
   ```ts
   new TransportRequestFailedError({ label: 'hf:whiskers', cause: new TypeError('fetch failed',), },);
   ```
   */
  public constructor(
    {
      label,
      cause,
    }: {
      readonly label: string;
      readonly cause: unknown;
    },
  ) {
    super(
      `${label}: the request failed before any answer came back, either because the network could not be reached `
        + 'or because the transport refused to send it (a header value it cannot carry is one such refusal); '
        + 'check the connection and the key',
      { cause, },
    );
    this.name = 'TransportRequestFailedError';
  }
}

//endregion Transport request error
