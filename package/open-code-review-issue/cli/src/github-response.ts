/**
 `gh api --include` response parsing.
 
 @module
 */

/**
 Lowest valid HTTP status code.
 */
const HTTP_STATUS_MINIMUM = 100;

/**
 Highest valid HTTP status code.
 */
const HTTP_STATUS_MAXIMUM = 599;

/**
 Included GitHub API response with normalized headers.
 
 @example
 ```ts
 const response: IncludedResponse = {
   status: 200,
   headers: { 'content-type': 'application/json' },
   body: {},
 };
 ```
 */
export type IncludedResponse = {
  readonly status: number;
  readonly headers: Readonly<Record<string, string>>;
  readonly body: unknown;
};

/**
 Reports malformed GitHub CLI included output.
 
 @example
 ```ts
 throw new IncludedResponseError('missing HTTP status');
 ```
 */
export class IncludedResponseError extends Error {
  /**
   Creates included-output parse failure.
   
   @param message - Evidence identifying malformed response section.
   
   @example
   ```ts
   const error = new IncludedResponseError('missing headers');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'IncludedResponseError';
  }
}

/**
 Splits header and body at first supported HTTP blank-line delimiter.
 
 @param stdout - Complete captured GitHub CLI standard output.
 
 @returns Header text and body text.
 
 @throws {@link IncludedResponseError} when delimiter is absent.
 
 @example
 ```ts
 splitIncludedOutput('HTTP/2 200 OK\n\n{}');
 ```
 */
function splitIncludedOutput(stdout: string,): readonly [
  headers: string,
  body: string
] {
  /**
   CRLF delimiter position used by HTTP wire-style output.
   */
  const crlfIndex = stdout.indexOf('\r\n\r\n',);
  if (crlfIndex !== (-1)) {
    return [
      stdout.slice(
        0,
        crlfIndex,
      ),
      stdout.slice(crlfIndex + '\r\n\r\n'.length,),
    ];
  }
  /**
   LF delimiter position used by normalized terminal output.
   */
  const lfIndex = stdout.indexOf('\n\n',);
  if (lfIndex !== (-1)) {
    return [
      stdout.slice(
        0,
        lfIndex,
      ),
      stdout.slice(lfIndex + '\n\n'.length,),
    ];
  }
  throw new IncludedResponseError('gh api --include output has no header/body delimiter',);
}

/**
 Parses HTTP status line emitted by GitHub CLI.
 
 @param statusLine - First included-output header line.
 
 @returns Valid numeric status.
 
 @throws {@link IncludedResponseError} when status is absent or invalid.
 
 @example
 ```ts
 parseStatus('HTTP/2.0 200 OK'); // 200
 ```
 */
function parseStatus(statusLine: string,): number {
  /**
   Whitespace-separated status-line components.
   */
  const parts = statusLine.split(' ',)
    .filter(function nonEmpty(part,): boolean {
      return part !== '';
    },);
  /**
   Numeric status token when present.
   */
  const [, statusText,] = parts;
  if (statusText === undefined) {
    throw new IncludedResponseError('gh api --include output has no HTTP status',);
  }
  /**
   Parsed status candidate.
   */
  const status = Number(statusText,);
  if ((!Number.isInteger(status,))
    || (status < HTTP_STATUS_MINIMUM)
    || (status > HTTP_STATUS_MAXIMUM))
    {
    throw new IncludedResponseError(`gh api --include output has invalid HTTP status ${statusText}`,);
  }
  return status;
}

/**
 Parses case-insensitive HTTP header lines.
 
 @param lines - Header lines after status line.
 
 @returns Lowercase header-name record.
 
 @throws {@link IncludedResponseError} when a header lacks colon.
 
 @example
 ```ts
 parseHeaders(['Content-Type: application/json']);
 ```
 */
function parseHeaders(lines: readonly string[],): Readonly<Record<string, string>> {
  /**
   Mutable header record scoped to this parse boundary.
   */
  const headers: Record<string, string> = {};
  lines.forEach(function parseHeader(line,): void {
    /**
     First separator between field name and value.
     */
    const separator = line.indexOf(':',);
    if (separator <= 0) {
      throw new IncludedResponseError(`gh api --include output has invalid header line ${line}`,);
    }
    /**
     Lowercase field name for case-insensitive lookup.
     */
    const name = line.slice(
      0,
      separator,
    )
      .trim()
      .toLowerCase();
    /**
     Trimmed field value.
     */
    const value = line.slice(separator + 1,)
      .trim();
    headers[name] = value;
  },);
  return headers;
}

/**
 Parses JSON body text with response-specific diagnostic.
 
 @param bodyText - Captured text after included headers.
 
 @returns Parsed JSON value.
 
 @throws {@link IncludedResponseError} when body is malformed JSON.
 
 @example
 ```ts
 parseBody('{}');
 ```
 */
function parseBody(bodyText: string,): unknown {
  try {
    return JSON.parse(bodyText,);
  }
  catch (error: unknown) {
    throw new IncludedResponseError(`gh api response body must be JSON: ${String(error,)}`,);
  }
}

/**
 Parses complete captured `gh api --include` standard output.
 
 @param stdout - Captured output containing one HTTP response and JSON body.
 
 @returns Numeric status, normalized headers, and parsed JSON body.
 
 @throws {@link IncludedResponseError} when output or JSON body is malformed.
 
 @example
 ```ts
 parseIncludedResponse({ stdout: 'HTTP/2 200 OK\n\n{}' });
 ```
 */
export function parseIncludedResponse({
  stdout,
}: {
  readonly stdout: string;
},): IncludedResponse {
  /**
   Header and body sections split at HTTP delimiter.
   */
  const [headerText, bodyText,] = splitIncludedOutput(stdout,);
  /**
   Header lines normalized from CRLF or LF input.
   */
  const lines = headerText.replaceAll(
    '\r\n',
    '\n',
  )
    .split('\n',);
  /**
   Required first status line.
   */
  const [statusLine,] = lines;
  if (statusLine === undefined) {
    throw new IncludedResponseError('gh api --include output has no status line',);
  }
  /**
   Parsed response JSON body.
   */
  const body = parseBody(bodyText,);
  return {
    status: parseStatus(statusLine,),
    headers: parseHeaders(lines.slice(1,),),
    body,
  };
}
