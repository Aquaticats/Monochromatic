/**
 Static asset serving handler for bundled JS and CSS from dist/client/.
 */
import {
  defineHandler,
  type EventHandlerWithFetch,
  serveStatic,
  type ServeStaticOptions,
} from 'h3';
import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

//region Logging

/**
 Logger for this module: a static asset that cannot be stat-ed gets no cache metadata.
 */
const l = tagged({
  tag: 'done-variant-static',
  l: logger,
},);

//endregion Logging

/**
 Resolved cache-metadata shape h3's `getMeta` contract returns; carries the
 `undefined` "missing file" case without a literal `T | undefined` annotation.
 */
type StaticMeta = Awaited<ReturnType<ServeStaticOptions['getMeta']>>;

/**
 Reads a static asset's text from disk relative to the working directory.
 
 Reads as UTF-8 (the handler serves only bundled JS and CSS, both text), so
 the result is a `string` (a `BodyInit`) rather than a `Buffer`, whose modern
 `Buffer<ArrayBufferLike>` typing is not assignable to h3's `getContents` contract.
 
 @param id - Request path resolved by h3, relative to the project root
 
 @returns File contents as a UTF-8 string
 */
function readContents(id: string,): Promise<string> {
  return readFile(
    join(
      '.',
      id,
    ),
    'utf8',
  );
}

/**
 Resolves size/mtime cache metadata for a static asset.
 
 @param id - Request path resolved by h3, relative to the project root
 
 @returns Cache metadata, or `undefined` when the file is missing or inaccessible
 */
async function getMetadata(id: string,): Promise<StaticMeta> {
  try {
    /**
     Filesystem stats for the requested asset; drives both the is-file check and meta payload.
     */
    const stats = await stat(
      join(
        '.',
        id,
      ),
    );
    if (!stats.isFile())
      return undefined;
    return {
      size: stats.size,
      mtime: stats.mtimeMs,
    };
  }
  catch (error) {
    l.debug(`static asset could not be stat-ed, so it gets no cache metadata: ${caughtValueText(error,)}`,);
    return undefined;
  }
}

/**
 h3 handler that serves static files from the `dist/` directory.
 Returns the file contents with size/mtime metadata for caching.
 
 @example
 ```ts
 app.get('/dist/client/**', staticHandler);
 ```
 */
export const staticHandler: EventHandlerWithFetch = defineHandler(
  /**
   Serves one event through h3 static-response state.
   
   @param event - Incoming h3 event.
   
   @returns static response when an asset exists.
   
   @mutates event - `h3@2.0.1-rc.24 . serveStatic` may affect event through bundled path and header operations.
   
   @example
   ```ts
   await handleStaticAsset(event);
   ```
   */
  function handleStaticAsset(event,) {
    return serveStatic(
      event,
      {
        getContents: readContents,
        getMeta: getMetadata,
      },
    );
  },
);
