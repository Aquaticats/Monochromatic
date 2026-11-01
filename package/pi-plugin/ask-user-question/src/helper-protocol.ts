//region Protocol types

/**
 Completion sent by detached answer helper after editor process settles.
 */
export type HelperCompletion =
  | {
    readonly status: 'submitted';
  }
    | {
      readonly status: 'cancelled';
    }
    | {
      readonly status: 'error';
      readonly message: string;
    };

//endregion Protocol types

//region Error

/**
 Reports malformed or unauthenticated helper protocol content.
 
 @example
 ```ts
 new HelperProtocolError('Helper token did not match.');
 ```
 */
export class HelperProtocolError extends Error {
  /**
   Creates a helper protocol diagnostic.
   
   @param message - protocol evidence that prevented completion
   
   @param options - optional source error
   */
  constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'HelperProtocolError';
  }
}

//endregion Error

//region Serialization

/**
 Serializes one terminal helper completion frame.
 
 @param completion - helper completion value
 
 @returns one JSON frame without framing delimiter
 
 @example
 ```ts
 serializeHelperCompletion({ completion: { status: 'submitted' } });
 ```
 */
export function serializeHelperCompletion(
  { completion, }: { readonly completion: HelperCompletion; },
): string {
  return JSON.stringify(completion,);
}

/**
 Parses completion JSON received after authenticated helper handshake.
 
 Empty content means authenticated helper disappeared before submission,
 which is cancellation rather than protocol failure.
 
 @param payload - completion-frame text after authentication line
 
 @returns validated helper completion
 
 @throws {@link HelperProtocolError} when frame is malformed
 
 @example
 ```ts
 parseHelperCompletion({ payload: '{"status":"cancelled"}' });
 ```
 */
export function parseHelperCompletion(
  { payload, }: { readonly payload: string; },
): HelperCompletion {
  if (payload.length === 0)
    return { status: 'cancelled', };
  /**
   Parsed JSON value before structural narrowing.
   */
  const parsed: unknown = parseJson({ payload, },);
  if (!hasStatus(parsed,))
    throw new HelperProtocolError('Answer helper completion must be an object with a status.',);
  /**
   Status field copied after structural narrowing.
   */
  const { status, } = parsed;
  if (status === 'submitted')
    return { status, };
  if (status === 'cancelled')
    return { status, };
  if (status !== 'error')
    throw new HelperProtocolError('Answer helper completion status is not recognized.',);
  if (!('message' in parsed))
    throw new HelperProtocolError('Answer helper error completion must include a nonempty message.',);
  /**
   Error message before string narrowing.
   */
  const { message, } = parsed;
  if (((typeof message) !== 'string') || (message.length === 0))
    throw new HelperProtocolError('Answer helper error completion must include a nonempty message.',);
  return {
    status,
    message,
  };
}

/**
 Determines whether decoded value exposes completion status.
 
 @param value - decoded JSON candidate
 
 @returns whether status property is readable
 */
function hasStatus(
  value: unknown,
): value is Readonly<Record<'status', unknown>> {
  if (value === null)
    return false;
  if ((typeof value) !== 'object')
    return false;
  return 'status' in value;
}

/**
 Parses JSON while retaining source failure as error cause.
 
 @param payload - raw completion payload
 
 @returns decoded JSON value
 
 @throws {@link HelperProtocolError} when payload is not JSON
 */
function parseJson(
  { payload, }: { readonly payload: string; },
): unknown {
  try {
    return JSON.parse(payload,);
  }
  catch (error: unknown) {
    throw new HelperProtocolError(
      'Answer helper completion is not valid JSON.',
      { cause: error, },
    );
  }
}

//endregion Serialization
