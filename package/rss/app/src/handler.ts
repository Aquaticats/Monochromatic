import {
  appendFile,
  mkdir,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { indexHtmlStart, } from './asset.ts';
import { INDEX_HTML_END, } from './html.ts';
import {
  type IgnoreRecord,
  IgnoreRequestBodyError,
  readIgnoreRecord,
} from './ignore-request.ts';
import { IGNORE_PATH, } from './path.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the handler module.
 */
const l = tagged({
  tag: 'handler',
  l: parentLogger,
},);

/**
 File inside the ignore directory holding one JSON record per line.
 */
const RECORD_FILE = 'api.jsonl';

/**
 Byte size of a file, or zero when it does not exist yet.
 
 @param filePath - path whose size is wanted
 
 @returns byte size, zero for a missing file
 
 @throws when the path exists but cannot be inspected
 */
async function sizeOrZero(filePath: string,): Promise<number> {
  try {
    /**
     File stats whose size decides whether the next record needs a separator.
     */
    const stats = await stat(filePath,);
    return stats.size;
  }
  catch (error) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT'))
      return 0;
    throw error;
  }
}

/**
 Reads the ignore record, or the response that rejects the body.
 
 @param options - request whose body names one ignored item, and the logger
 reporting a rejection
 
 @returns validated record, or the 400 response naming the broken rule
 
 @mutates options - Emits one warn line per rejected body.
 */
async function readRecordOrRejection(options: {
  readonly request: Request;
  readonly l: Logger;
},): Promise<IgnoreRecord | Response> {
  /**
   Destructured inputs so the catch branch reads the logger directly.
   */
  const {
    request,
    l: innerL,
  } = options;
  try {
    return await readIgnoreRecord(request,);
  }
  catch (error) {
    if (!(error instanceof IgnoreRequestBodyError))
      throw error;
    innerL.warn(`rejected ignore request: ${caughtValueText(error,)}`,);
    return Response.json(
      { error: error.message, },
      { status: 400, },
    );
  }
}

//region HTTP handlers: Serve rendered HTML and persist ignored items

/**
 Serves the full rendered HTML page with inlined assets and feed body,
 wrapping the body between {@link indexHtmlStart} and {@link INDEX_HTML_END}.
 
 @param options - Contains the async function that returns the rendered HTML body
 
 @returns Response containing the complete HTML document
 
 @example
 ```ts
 const response = await serveIndex({ getHtmlBody: async () => '<p>feeds</p>' });
 ```
 */
export async function serveIndex(options: {
  readonly getHtmlBody: () => Promise<string>;
},): Promise<Response> {
  /**
   Destructured renderer so the call site reads without `options.` prefix.
   */
  const { getHtmlBody, } = options;
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: serveIndex.name,
    l,
  },);
  innerL.debug('serving index',);
  /**
   Awaited HTML body so the response constructor receives a string, not a Promise.
   */
  const body = await getHtmlBody();
  return new Response(
    `${indexHtmlStart}${body}${INDEX_HTML_END}`,
    {
      status: 200,
      headers: { 'content-type': 'text/html', },
    },
  );
}

/**
 Records an ignored feed item to the JSONL ignore file inside the ignore
 directory ({@link IGNORE_PATH} unless a caller names its own). Creates the
 directory and file if they do not exist, and rejects a body that breaks the
 documented record shape or the size bound before anything reaches disk.
 
 @param options - Incoming request naming one ignored item, plus the optional
 ignore directory to write into
 
 @returns Response with file stats after appending, or a 400 naming the rule a
 rejected body broke
 
 @mutates options - Creates the named ignore directory and appends one record to the JSONL file inside it.
 
 @example
 ```ts
 const response = await ignore({
   request: new Request('http://localhost', {
     method: 'POST',
     body: '{"link":"https://example.com"}',
   }),
 });
 ```
 */
export async function ignore(options: {
  readonly request: Request;
  readonly directory?: string;
},): Promise<Response> {
  /**
   Destructured inputs so the body reads without the options prefix.
   */
  const {
    request,
    directory = IGNORE_PATH,
  } = options;
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: ignore.name,
    l,
  },);
  /**
   Validated record, or the response rejecting the body before anything touches
   disk so a refused request leaves the ignore file unchanged.
   */
  const readResult = await readRecordOrRejection({
    request,
    l: innerL,
  },);
  if (readResult instanceof Response)
    return readResult;
  /**
   Validated record stored below.
   */
  const record = readResult;
  innerL.debug(`ignore ${JSON.stringify(record,)}`,);

  /**
   Resolved path to the JSONL log so create-on-miss and append share one target.
   */
  const filePath = join(
    directory,
    RECORD_FILE,
  );
  /**
   Byte size already on disk, so the separator sits between records instead of
   leaving an empty first line in a fresh file.
   */
  const existingBytes = await sizeOrZero(filePath,);
  if (existingBytes === 0) {
    innerL.debug(`creating ${RECORD_FILE}`,);
    await mkdir(
      directory,
      { recursive: true, },
    );
    await writeFile(
      filePath,
      '',
      'utf8',
    );
  }
  await appendFile(
    filePath,
    `${existingBytes === 0 ? '' : '\n'}${JSON.stringify(record,)}`,
  );

  /**
   Final file stats returned to the client as proof of the append.
   */
  const stats = await stat(filePath,);
  return Response.json(stats,);
}

//endregion HTTP handlers
