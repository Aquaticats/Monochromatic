/**
 OVHcloud cloud API request core: the signature, the clock, and error mapping.

 The signature is OVHcloud's own and is built the way its Python client builds
 it: `"/1.0"`-based targets, `$1$` plus the SHA-1 hex of the application
 secret, the consumer key, the HTTP method, the full target URL, the request
 body and a Unix timestamp, joined by `+`
 (`github.com/ovh/python-ovh`, `ovh/client.py`, lines 596-617 of the clone read
 on 2026-11-28; the guide states the same form as
 `"$1$" + SHA1_HEX(AS+"+"+CK+"+"+METHOD+"+"+QUERY+"+"+BODY+"+"+TSTAMP)`).

 Request signatures are only valid for a short time, so the timestamp is
 corrected by the API host's own clock, read once per process from the
 unauthenticated `GET /1.0/auth/time` exactly as that client does
 (`time_delta`, lines 251-268).

 No header value and no key value ever leaves this module in a message, a log
 line or a thrown error: a failed request is reported by its method, its path
 with the project masked, its status and OVHcloud's own message.

 @module
 */

import { createHash, } from 'node:crypto';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { requireCredentials, } from './config.ts';
import type { OvhFailureBody, } from './types.ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Constants

/**
 Path of the unauthenticated clock endpoint, under the legacy `/1.0` base the
 way OVHcloud's own client reaches it.
 */
export const AUTH_TIME_PATH = '/1.0/auth/time';

/**
 Prefix of every signature OVHcloud accepts.
 */
const SIGNATURE_PREFIX = '$1$';

/**
 Text that replaces the project identifier wherever a path is shown.
 */
export const MASKED_PROJECT = '<project>';

/**
 HTTP status for a response with no body.
 */
const HTTP_NO_CONTENT = 204;

/**
 Separator of the six fields the signature covers.
 */
const SIGNATURE_SEPARATOR = '+';

/**
 Algorithms digesting the signature payload; SHA-1 is OVHcloud's, not a choice here.
 */
const SIGNATURE_ALGORITHM = 'sha1';

/**
 Milliseconds in one second, the divisor between the runtime clock and the
 whole seconds OVHcloud signs.
 */
const MILLISECONDS_PER_SECOND = 1_000;

//endregion Constants

//region Masking

/**
 Masks the project identifier inside a path so a logged or thrown path names no
 account's project.
 
 @param path - API path possibly holding the project identifier
 
 @param projectId - identifier to replace
 
 @returns the path with every occurrence of `projectId` replaced
 
 @example
 ```ts
 maskProjectId({ path: '/v1/cloud/project/abc/instance', projectId: 'abc' });
 // => '/v1/cloud/project/<project>/instance'
 ```
 */
export function maskProjectId({
  path,
  projectId,
}: {
  readonly path: string;
  readonly projectId: string;
},): string {
  if (projectId === '') {
    return path;
  }
  return path.replaceAll(
    projectId,
    MASKED_PROJECT,
  );
}

//endregion Masking

//region Signature

/**
 Builds the text OVHcloud digests to check a request.
 
 @param applicationSecret - the `AS`
 
 @param body - request body exactly as it is sent, `''` when there is none
 
 @param consumerKey - the `CK`
 
 @param method - HTTP method, upper-cased before it enters the payload
 
 @param target - complete URL the request goes to
 
 @param timestamp - Unix seconds the request claims
 
 @returns the six fields joined by `+`
 
 @example
 ```ts
 signaturePayload({ applicationSecret: 'as', consumerKey: 'ck', method: 'get', target: 'https://ca.api.ovh.com/v1/cloud/project/p/instance', body: '', timestamp: 1 });
 // => 'as+ck+GET+https://ca.api.ovh.com/v1/cloud/project/p/instance++1'
 ```
 */
export function signaturePayload({
  applicationSecret,
  body,
  consumerKey,
  method,
  target,
  timestamp,
}: {
  readonly applicationSecret: string;
  readonly body: string;
  readonly consumerKey: string;
  readonly method: string;
  readonly target: string;
  readonly timestamp: number;
},): string {
  return [
    applicationSecret,
    consumerKey,
    method.toUpperCase(),
    target,
    body,
    String(timestamp,),
  ].join(SIGNATURE_SEPARATOR,);
}

/**
 Signs one request with OVHcloud's header set.
 
 @param applicationKey - the `AK`, sent as `X-Ovh-Application`
 
 @param applicationSecret - the `AS`, digested and never sent
 
 @param body - request body exactly as it is sent, `''` when there is none
 
 @param consumerKey - the `CK`, sent as `X-Ovh-Consumer` and digested
 
 @param method - HTTP method
 
 @param target - complete URL the request goes to
 
 @param timestamp - Unix seconds the request claims
 
 @returns the four authentication headers, and nothing else
 
 @example
 ```ts
 const headers = signRequest({ applicationKey: 'ak', applicationSecret: 'as', body: '', consumerKey: 'ck', method: 'GET', target: 'https://ca.api.ovh.com/v1/cloud/project/p/instance', timestamp: 1 });
 ```
 */
export function signRequest({
  applicationKey,
  applicationSecret,
  body,
  consumerKey,
  method,
  target,
  timestamp,
}: {
  readonly applicationKey: string;
  readonly applicationSecret: string;
  readonly body: string;
  readonly consumerKey: string;
  readonly method: string;
  readonly target: string;
  readonly timestamp: number;
},): Readonly<Record<string, string>> {
  /**
   Hex SHA-1 of the payload, the part OVHcloud compares against its own digest.
   */
  const digest = createHash(SIGNATURE_ALGORITHM,)
    .update(
    signaturePayload({
      applicationSecret,
      body,
      consumerKey,
      method,
      target,
      timestamp,
    },),
    'utf8',
    )
    .digest('hex',);
  return {
    'X-Ovh-Application': applicationKey,
    'X-Ovh-Consumer': consumerKey,
    'X-Ovh-Signature': `${SIGNATURE_PREFIX}${digest}`,
    'X-Ovh-Timestamp': String(timestamp,),
  };
}

//endregion Signature

//region Clock

/**
 Clock differences this process has read, one per API root. A `Map` rather than a
 module-level mutable binding, so the cache cannot leak between API hosts.
 */
const CLOCK_OFFSETS = new Map<string, Promise<number>>();

/**
 Asks the API host for its time and computes the difference.
 
 @param endpointRoot - API root to ask
 
 @returns host time minus local time, in whole seconds
 
 @throws {@link OvhApiError} when the endpoint cannot be read
 
 @example
 ```ts
 const offset = await readClockOffset({ endpointRoot: 'https://ca.api.ovh.com' });
 ```
 */
async function readClockOffset({
  endpointRoot,
}: {
  readonly endpointRoot: string;
},): Promise<number> {
  /**
   Logger scoped to the clock read so its failure is attributable.
   */
  const rl = tagged({
    tag: readClockOffset.name,
    l,
  },);
  /**
   Host's own answer: a bare Unix timestamp, with no authentication.
   */
  const answer = await fetch(`${endpointRoot}${AUTH_TIME_PATH}`,);
  if (!answer.ok) {
    throw new OvhApiError({
      method: 'GET',
      path: AUTH_TIME_PATH,
      status: answer.status,
      message: `could not read the API host clock: ${answer.statusText}`,
    },);
  }
  /**
   Parsed body; the endpoint answers with a JSON number.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the clock endpoint answers a bare JSON number
  const seconds = await answer.json() as number;
  /**
   Difference to add to this machine's clock, in whole seconds.
   */
  const offset = seconds - Math.floor(Date.now() / MILLISECONDS_PER_SECOND,);
  rl.debug(`API host clock is ${String(offset,)}s from this machine's`,);
  return offset;
}

/**
 Reads the API host's clock once and returns the difference, as OVHcloud's own
 client does. Signatures are valid for a short window, so a wrong local clock
 fails every request without this correction.
 
 @param endpointRoot - API root to ask
 
 @returns host time minus local time, in whole seconds
 
 @throws {@link OvhApiError} when the clock endpoint cannot be read
 
 @example
 ```ts
 const offset = await serverClockOffset({ endpointRoot: 'https://ca.api.ovh.com' });
 ```
 */
export function serverClockOffset({
  endpointRoot,
}: {
  readonly endpointRoot: string;
},): Promise<number> {
  /**
   Difference already read for this API root, when one was.
   */
  const cached = CLOCK_OFFSETS.get(endpointRoot,);
  if (cached !== undefined) {
    return cached;
  }
  /**
   Difference being read now; stored before it resolves so concurrent requests
   share one call.
   */
  const pending = readClockOffset({ endpointRoot, },);
  CLOCK_OFFSETS.set(
    endpointRoot,
    pending,
  );
  return pending;
}

/**
 Drops the cached clock difference. Called by the tests, whose stand-in answers
 the clock endpoint with one fixed time per file, and by nothing in production.
 
 @example
 ```ts
 forgetServerClockOffset();
 ```
 */
export function forgetServerClockOffset(): void {
  CLOCK_OFFSETS.clear();
}

//endregion Clock

//region Errors

/**
 A request OVHcloud refused or could not answer.

 Carries the method, the path with the project masked, the status and
 OVHcloud's own message, and nothing that was sent or received as a header.

 @example
 ```ts
 try { await ovhRequest({ method: 'GET', path: '/v1/cloud/project/p/instance' }); }
 catch (error) { if (error instanceof OvhApiError) console.error(error.status); }
 ```
 */
export class OvhApiError extends Error {
  /**
   HTTP method of the request.
   */
  readonly method: string;
  /**
   Path of the request with the project identifier masked.
   */
  readonly path: string;
  /**
   HTTP status OVHcloud answered with.
   */
  readonly status: number;
  /**
   Machine-readable code OVHcloud sent, when the body carried one.
   */
  readonly errorCode: string;

  /**
   @param errorCode - OVHcloud's `errorCode`, when the answer carried one
   
   @param message - OVHcloud's own message, or the status text when the body said nothing
   
   @param method - HTTP method of the request
   
   @param path - request path, already masked
   
   @param status - HTTP status OVHcloud answered with
   */
  constructor({
    errorCode = '',
    message,
    method,
    path,
    status,
  }: {
    readonly errorCode?: string;
    readonly message: string;
    readonly method: string;
    readonly path: string;
    readonly status: number;
  },) {
    super(
      `OVHcloud ${method} ${path} answered ${String(status,)}: ${message}${
        errorCode === '' ? '' : ` (errorCode ${errorCode})`
      }`,
    );
    this.name = 'OvhApiError';
    this.method = method;
    this.path = path;
    this.status = status;
    this.errorCode = errorCode;
  }
}

/**
 Reads OVHcloud's own message and code out of a failed answer, falling back to
 the status text when the body is empty or not the shape its client reads.
 
 @param res - failed response
 
 @returns OVHcloud's message and code, either possibly empty
 
 @example
 ```ts
 const { message } = await parseErrorBody(res);
 ```
 */
async function parseErrorBody(res: Response,): Promise<{
  readonly message: string;
  readonly errorCode?: string;
}> {
  try {
    /**
     Decoded body, typed as the shape OVHcloud's client reads.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- OVHcloud error envelope
    const body = await res.json() as OvhFailureBody;
    return {
      message: ((typeof body.message) === 'string') ? body.message : res.statusText,
      ...(((typeof body.errorCode) === 'string') ? { errorCode: body.errorCode, } : {}),
    };
  }
  catch (error: unknown) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    return { message: res.statusText, };
  }
}

//endregion Errors

//region Request

/**
 Issues one authenticated request against the cloud API.
 
 @param body - JSON body to send, absent for reads and deletes
 
 @param method - HTTP method
 
 @param path - path from the API root, including its version segment, beginning with `/`
 
 @returns the parsed JSON answer, or `undefined` for a body-less one
 
 @throws {@link OvhApiError} when the answer's status is not ok
 
 @throws Error naming the first missing key when the environment lacks one
 
 @example
 ```ts
 const instances = await ovhRequest<readonly OvhInstance[]>({ method: 'GET', path: '/v1/cloud/project/p/instance' });
 ```
 */
export async function ovhRequest<T>({
  body,
  method,
  path,
}: {
  readonly body?: unknown;
  readonly method: string;
  readonly path: string;
},): Promise<T> {
  /**
   Logger scoped to this request so its lines are namespaced.
   */
  const rl = tagged({
    tag: ovhRequest.name,
    l,
  },);
  /**
   Keys, API root and project of this process.
   */
  const credentials = requireCredentials();
  /**
   Request path with the project masked, the only form that may be shown.
   */
  const maskedPath = maskProjectId({
    path,
    projectId: credentials.projectId,
  },);
  /**
   Serialized body; the signature covers exactly these bytes, so it is built once.
   */
  const serialized = (body === undefined) ? '' : JSON.stringify(body,);
  /**
   Complete target URL, the sixth field of the signature.
   */
  const target = `${credentials.endpointRoot}${path}`;
  /**
   Seconds this machine's clock is behind the host's, read before the stamp so a
   second ticking while the clock is read cannot make the stamp earlier than the
   host's own time, which a signature window may refuse.
   */
  const offset = await serverClockOffset({ endpointRoot: credentials.endpointRoot, },);
  /**
   Corrected Unix timestamp this request claims.
   */
  const timestamp = Math.floor(Date.now() / MILLISECONDS_PER_SECOND,) + offset;
  rl.debug(`${method} ${maskedPath}`,);

  /**
   Raw answer; inspected for ok-ness before anything is parsed.
   */
  const res = await fetch(
    target,
    {
      method,
      headers: {
        ...signRequest({
          applicationKey: credentials.applicationKey,
          applicationSecret: credentials.applicationSecret,
          body: serialized,
          consumerKey: credentials.consumerKey,
          method,
          target,
          timestamp,
        },),
        ...((body === undefined) ? {} : { 'Content-Type': 'application/json', }),
      },
      ...((body === undefined) ? {} : { body: serialized, }),
    },
  );

  if (!res.ok) {
    /**
     OVHcloud's own message and code for this refusal.
     */
    const {
      errorCode,
      message,
    } = await parseErrorBody(res,);
    throw new OvhApiError({
      ...(errorCode === undefined ? {} : { errorCode, }),
      message,
      method,
      path: maskedPath,
      status: res.status,
    },);
  }

  if (res.status === HTTP_NO_CONTENT) {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- 204 has no body
    return undefined as T;
  }
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted OVHcloud API JSON answer
  return await res.json() as T;
}

//endregion Request
