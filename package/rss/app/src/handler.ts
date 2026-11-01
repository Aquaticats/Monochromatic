import {
  access,
  appendFile,
  mkdir,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { indexHtmlStart, } from './asset.ts';
import { INDEX_HTML_END, } from './html.ts';
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
 Records an ignored feed item to the JSONL ignore file under {@link IGNORE_PATH}.
 Creates the ignore directory and file if they do not exist.
 
 @param request - Incoming request with JSON body containing a `link` property
 
 @returns Response with file stats after appending
 
 @example
 ```ts
 const response = await ignore(new Request('http://localhost', {
   method: 'POST',
   body: '{"link":"https://example.com"}',
 }));
 ```
 */
export async function ignore(request: Request,): Promise<Response> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: ignore.name,
    l,
  },);
  /**
   Raw request body persisted verbatim so the ignore file matches the wire payload.
   */
  const body = await request.text();
  innerL.debug(`ignore ${body}`,);

  /**
   Resolved path to the JSONL log so create-on-miss and append share one target.
   */
  const filePath = join(
    IGNORE_PATH,
    'api.jsonl',
  );
  try {
    await access(filePath,);
  }
  catch (error) {
    innerL.debug(`creating api.jsonl: ${JSON.stringify(error,)}`,);
    await mkdir(
      IGNORE_PATH,
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
    `\n${body}`,
  );

  /**
   Final file stats returned to the client as proof of the append.
   */
  const stats = await stat(filePath,);
  return Response.json(stats,);
}

//endregion HTTP handlers
