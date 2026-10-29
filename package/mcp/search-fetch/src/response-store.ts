/**
 Bounded index of truncated full responses, exposed to hosts as MCP resources.

 Truncated tool output points at a temp file holding the full response.
 This store gives each recorded file a stable `search-fetch://response/<id>` URI so hosts
 without file access can still read the tail through `resources/read`.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ResourceContents,
  ResourceDescriptor,
  ResourceProvider,
} from '@monochromatic-dev/mcp-stdio/ts';

//region Logger

/**
 Logger root for the search-fetch MCP server.
 */
const searchFetchMcpLogger = tagged({ tag: 'search-fetch-mcp', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'response-store',
  l: searchFetchMcpLogger,
},);

//endregion Logger

//region Types

/**
 One recorded full response reachable through a resource URI.
 */
export type RecordedResponse = {
  /**
   Resource URI hosts fetch this response by.
   */
  readonly uri: string;
  /**
   Short resource name advertised in `resources/list`.
   */
  readonly name: string;
  /**
   Resource description advertised in `resources/list`.
   */
  readonly description: string;
  /**
   Temp file path holding the full response text.
   */
  readonly filePath: string;
};

/**
 Input accepted by {@link ResponseStore.record}.
 */
export type RecordResponseInput = {
  /**
   Temp file path holding the full response text.
   */
  readonly filePath: string;
  /**
   Short resource name advertised in `resources/list`.
   */
  readonly name: string;
  /**
   Resource description advertised in `resources/list`.
   */
  readonly description: string;
};

/**
 Store of recorded full responses doubling as the server's {@link ResourceProvider}.
 */
export type ResponseStore = ResourceProvider & {
  /**
   Records one full response and returns its resource identity.
   */
  readonly record: (input: RecordResponseInput) => RecordedResponse;
};

/**
 Error thrown when a resource URI is not recorded or its file cannot be read.
 */
export class ResourceReadError extends Error {
  /**
   Build a read failure naming the resource URI and the underlying cause.

   @param uri - resource URI that could not be served

   @param cause - underlying read failure text
   */
  constructor(
    uri: string,
    cause: string,
  ) {
    super(`resource ${uri} is not readable: ${cause}`,);
    this.name = 'ResourceReadError';
  }
}

//endregion Types

//region Public API

/**
 Create a bounded response store.

 @param capacity - maximum number of responses kept in the index

 @returns store recording responses and answering resource listing and reads

 @example
 ```ts
 const store = createResponseStore({ capacity: 32 });
 store.record({ filePath, name: 'full response', description: 'truncated search output' });
 ```
 */
export function createResponseStore(
  { capacity, }: {
    /**
     Maximum number of responses kept in the index before oldest entries are dropped.
     */
    readonly capacity: number;
  },
): ResponseStore {
  /**
   Recorded responses in insertion order, capped at `capacity`.
   */
  const responses = new Map<string, RecordedResponse>();
  /**
   Monotonic sequence giving each recorded response a stable URI suffix.
   */
  const sequence = { next: 0, };

  /**
   Record one full response, dropping the oldest entry when over capacity.

   @param input - temp file path with resource name and description

   @returns recorded response identity

   @example
   ```ts
   store.record({ filePath, name: 'full response', description: 'truncated fetch output' });
   ```
   */
  function record(input: RecordResponseInput,): RecordedResponse {
    sequence.next += 1;
    /**
     Resource URI for this response, unique within the process.
     */
    const uri = `search-fetch://response/${String(sequence.next,)}`;
    /**
     Recorded response stored under its URI.
     */
    const recorded: RecordedResponse = {
      uri,
      name: input.name,
      description: input.description,
      filePath: input.filePath,
    };
    responses.set(
      uri,
      recorded,
    );
    while (responses.size > capacity) {
      /**
       Oldest recorded URI, evicted to keep the index bounded.
       */
      const oldestUri = responses
        .keys()
        .next()
        .value;
      if (oldestUri === undefined)
        break;
      responses.delete(oldestUri,);
    }
    l.debug(`recorded full response ${uri} at ${input.filePath}`,);
    return recorded;
  }

  /**
   List every currently recorded response.

   @returns resource descriptors for `resources/list`

   @example
   ```ts
   store.list();
   ```
   */
  function list(): readonly ResourceDescriptor[] {
    return [...responses.values(),]
      .map(function toDescriptor(recorded,) {
        return {
          uri: recorded.uri,
          name: recorded.name,
          description: recorded.description,
          mimeType: 'text/plain',
        };
      },);
  }

  /**
   Read one recorded response by URI.

   @param uri - resource URI previously returned by record

   @returns resource contents holding the full response text

   @throws ResourceReadError when the URI is unknown or its file cannot be read

   @example
   ```ts
   await store.read('search-fetch://response/1');
   ```
   */
  async function read(uri: string,): Promise<ResourceContents> {
    /**
     Recorded response for this URI, if any.
     */
    const recorded = responses.get(uri,);
    if (recorded === undefined)
      throw new ResourceReadError(
        uri,
        'no response is recorded under that URI',
      );

    // Deliberate catch-and-rethrow: the read failure names the resource URI so hosts see
    // which recorded response is unreachable rather than a bare file-system error.
    try {
      /**
       Full response text read back from the temp file.
       */
      const text = await readFile(
        recorded.filePath,
        'utf8',
      );
      return {
        uri,
        text,
        mimeType: 'text/plain',
      };
    }
    catch (error: unknown) {
      throw new ResourceReadError(
        uri,
        caughtValueText(error,),
      );
    }
  }

  return {
    record,
    list,
    read,
  };
}

//endregion Public API
