import { mapIterableAsync, } from '@monochromatic-dev/async-iter-core/ts';
import type { Dirent, } from 'node:fs';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { IGNORE_PATH, } from './path.ts';
import type { DeepReadonly, } from './types.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the ignore module.
 */
const l = tagged({
  tag: 'ignore',
  l: parentLogger,
},);

/**
 Sentinel returned when the ignore directory does not exist.
 Distinct non-nullish value so {@link readIgnoreDir} never widens to a banned `T | undefined`.
 */
const IGNORE_DIR_ABSENT = Symbol('ignore directory missing on disk',);

/**
 Reads the ignore directory entries, returning {@link IGNORE_DIR_ABSENT} when it is missing.
 Module-scoped because it captures nothing from any caller.
 
 @returns Directory entries, or {@link IGNORE_DIR_ABSENT} when the directory does not exist
 
 @example
 ```ts
 const entries = await readIgnoreDir();
 ```
 */
async function readIgnoreDir(): Promise<Dirent[] | typeof IGNORE_DIR_ABSENT> {
  try {
    return await readdir(
      IGNORE_PATH,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    /**
     Inner logger tagged with this function name for traceable log lines.
     */
    const innerL = tagged({
      tag: readIgnoreDir.name,
      l,
    },);
    innerL.debug(`ignore directory not readable: ${JSON.stringify(error,)}`,);
    return IGNORE_DIR_ABSENT;
  }
}

/**
 Reads one ignore directory entry's file contents as UTF-8 text.
 Module-scoped because it captures nothing from any caller.
 
 @param dirent - Directory entry whose backing file is read
 
 @returns File contents as UTF-8 text
 
 @example
 ```ts
 const text = await readIgnoreFile(dirent);
 ```
 */
function readIgnoreFile(dirent: DeepReadonly<Dirent>,): Promise<string> {
  return readFile(
    join(
      dirent.parentPath,
      dirent.name,
    ),
    'utf8',
  );
}

//region Ignore content loading: Reads raw JSONL content for salt derivation and link filtering

/**
 Reads the raw text content of all ignore JSONL files.
 Used both for content-derived memoize salt and for link filtering.
 Lists entries via {@link readIgnoreDir} and reads each one with {@link readIgnoreFile}.
 
 @returns Concatenated raw text from all ignore files, or empty string if directory is missing
 
 @example
 ```ts
 const content = await getIgnoreContent();
 ```
 */
export async function getIgnoreContent(): Promise<string> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: getIgnoreContent.name,
    l,
  },);
  /**
   Directory entries, or the absent sentinel when the ignore directory is missing.
   */
  const filesInDir = await readIgnoreDir();
  if (filesInDir === IGNORE_DIR_ABSENT) {
    innerL.debug('ignore directory not found',);
    return '';
  }
  /**
   Per-file contents read in parallel, joined back into one stream for callers.
   */
  const contents = await mapIterableAsync({
    fn: readIgnoreFile,
    iterable: filesInDir,
  },);
  return contents.join('',);
}

/**
 Parses ignored links from raw ignore file content.
 
 @param content - Raw JSONL text from ignore files
 
 @returns Set of link URLs that should be excluded from rendering
 
 @example
 ```ts
 const links = parseIgnoredLinks('{"link":"https://example.com"}\n');
 ```
 */
export function parseIgnoredLinks(content: string,): Set<string> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: parseIgnoredLinks.name,
    l,
  },);
  /**
   Accumulator for unique link URLs so duplicate ignore entries collapse.
   */
  const links = new Set<string>();
  /**
   Trimmed, non-empty lines so the JSON parser does not choke on whitespace.
   */
  const lines = content
    .split('\n',)
    .map(function trimLine(line,) {
      return line.trim();
    },)
    .filter(function nonEmpty(line,) {
      return line.length
        > 0;
    },);
  for (const line of lines) {
    try {
      /**
       Parsed entry kept as unknown so the shape check below narrows it explicitly.
       */
      const parsed: unknown = JSON.parse(line,);
      if ((parsed !== null) && ((typeof parsed) === 'object')
        && ('link' in parsed)) {
        /**
         Destructured link field so the type guard runs on a named binding.
         */
        const { link, } = parsed;
        if (((typeof link) === 'string') && (link !== ''))
          links.add(link,);
      }
    }
    catch (error) {
      innerL.warn(`invalid JSON in ignore file: ${line} ${JSON.stringify(error,)}`,);
    }
  }
  innerL.debug(`${String(links.size,)} ignored links`,);
  return links;
}

//endregion Ignore content loading
