/**
 Transport failures a member reports when its request never reached a provider, each with the text
 the installed Pi adapters produce for it.
 @module
 */

//region The failure each code arrives as

/**
 One transport failure: the token the pool reads, the text the installed adapters produce, and the
 transport that produces it in an in-memory test.
 */
export type TransportFailure = {
  /**
   What a person saw happen, used as the case name in the suites.
   */
  readonly name: string;
  /**
   Code the adapter's text carries, as undici or Node spells it.
   */
  readonly code: string;
  /**
   Whole terminal text Pi's adapter reports for this failure, byte for byte.
   */
  readonly text: string;
  /**
   Transport that fails this way, for a driver that stands in for the member's provider.
   */
  readonly respond: () => Promise<Response>;
};

/**
 Build the rejection undici's fetch gives when it cannot establish the request.

 undici rejects with `TypeError: fetch failed` whose deepest cause carries the socket or resolver
 error. `@earendil-works/pi-ai` passes that chain to `withCauseDetail`
 (`dist/utils/error-body.js`), which appends the deepest cause's message and, when the message does
 not already spell it, its `code` in parentheses.

 @param message - deepest cause's message, as the transport reports it

 @param code - that cause's `code` property

 @returns fetch rejection with the cause chain the adapter reads

 @example
 ```ts
 throw fetchFailed({ message: 'read ECONNRESET', code: 'ECONNRESET' });
 ```
 */
function fetchFailed({
  message,
  code,
}: {
  readonly message: string;
  readonly code: string;
}): TypeError {
  return new TypeError(
    'fetch failed',
    {
      cause: Object.assign(
        new Error(message,),
        { code, },
      ),
    },
  );
}

/**
 Build the stream error undici gives when a response body ends mid-read, after a successful header.

 `@earendil-works/pi-ai` lets this value reach the adapter's own catch, where `withCauseDetail`
 composes `terminated: <cause message> (<cause code>)`.

 @returns body read failure with the cause chain the adapter reads

 @example
 ```ts
 controller.error(socketClosed());
 ```
 */
function socketClosed(): TypeError {
  return new TypeError(
    'terminated',
    {
      cause: Object.assign(
        new Error('other side closed',),
        { code: 'UND_ERR_SOCKET', },
      ),
    },
  );
}

/**
 Response whose body fails as a socket closed mid-stream does.

 @returns response with a 200 header and a body that never yields a byte

 @example
 ```ts
 await failure.respond(); // rejects the reader
 ```
 */
function resetAfterHeader(): Promise<Response> {
  /**
   Body that fails as soon as Pi reads it, which is what a reset connection does.
   */
  const body = new ReadableStream({
    start(controller,): void {
      controller.error(socketClosed(),);
    },
  },);
  return Promise.resolve(new Response(
    body,
    {
      status: 200,
      headers: { 'content-type': 'text/event-stream', },
    },
  ),);
}

/**
 A host that never answered within the connect limit the transport applies.
 */
export const CONNECT_TIMEOUT: TransportFailure = {
  name: 'a connect timeout',
  code: 'UND_ERR_CONNECT_TIMEOUT',
  text: 'Request timed out.: Connect Timeout Error (attempted address: stand-in.example:443, timeout: 10000ms) (UND_ERR_CONNECT_TIMEOUT)',
  respond: function connectTimeout(): Promise<Response> {
    return Promise.reject(fetchFailed({
      message: 'Connect Timeout Error (attempted address: stand-in.example:443, timeout: 10000ms)',
      code: 'UND_ERR_CONNECT_TIMEOUT',
    },),);
  },
};

/**
 A connection the host refused.
 */
export const REFUSED_CONNECTION: TransportFailure = {
  name: 'a refused connection',
  code: 'ECONNREFUSED',
  text: 'Connection error.: connect ECONNREFUSED 127.0.0.1:443',
  respond: function refused(): Promise<Response> {
    return Promise.reject(fetchFailed({
      message: 'connect ECONNREFUSED 127.0.0.1:443',
      code: 'ECONNREFUSED',
    },),);
  },
};

/**
 A connection the host reset before any response header arrived.
 */
export const RESET_BEFORE_HEADER: TransportFailure = {
  name: 'a reset before any response header',
  code: 'ECONNRESET',
  text: 'Connection error.: read ECONNRESET',
  respond: function resetBeforeHeader(): Promise<Response> {
    return Promise.reject(fetchFailed({
      message: 'read ECONNRESET',
      code: 'ECONNRESET',
    },),);
  },
};

/**
 A connection the host reset after a successful response header, when the member had shown nothing.
 */
export const RESET_AFTER_HEADER: TransportFailure = {
  name: 'a reset after a successful response header',
  code: 'UND_ERR_SOCKET',
  text: 'terminated: other side closed (UND_ERR_SOCKET)',
  respond: resetAfterHeader,
};

/**
 A name the resolver answered was not there.
 */
export const NAME_NOT_FOUND: TransportFailure = {
  name: 'a name that does not resolve',
  code: 'ENOTFOUND',
  text: 'Connection error.: getaddrinfo ENOTFOUND no-such-host.invalid',
  respond: function nameNotFound(): Promise<Response> {
    return Promise.reject(fetchFailed({
      message: 'getaddrinfo ENOTFOUND no-such-host.invalid',
      code: 'ENOTFOUND',
    },),);
  },
};

/**
 A name the resolver could not answer for at this moment.
 */
export const NAME_TEMPORARILY_UNRESOLVABLE: TransportFailure = {
  name: 'a name the resolver cannot answer for now',
  code: 'EAI_AGAIN',
  text: 'Connection error.: getaddrinfo EAI_AGAIN no-such-host.invalid',
  respond: function nameTemporarilyUnresolvable(): Promise<Response> {
    return Promise.reject(fetchFailed({
      message: 'getaddrinfo EAI_AGAIN no-such-host.invalid',
      code: 'EAI_AGAIN',
    },),);
  },
};

/**
 Every transport failure the pool hands to the next member.

 The text of each entry is what `src/adapter-transport-failure.unit.test.ts` reads back from the two
 installed adapters, and `doc/captures.md` records the transport each one was captured from.
 */
export const TRANSPORT_FAILURES: readonly TransportFailure[] = [
  CONNECT_TIMEOUT,
  REFUSED_CONNECTION,
  RESET_BEFORE_HEADER,
  RESET_AFTER_HEADER,
  NAME_NOT_FOUND,
  NAME_TEMPORARILY_UNRESOLVABLE,
];

//endregion The failure each code arrives as
