import { mapIterableAsync, } from '@monochromatic-dev/async-iter-core/ts';
import { readFile, } from 'node:fs/promises';
import {
  dirname,
  resolve,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  DOT_ENV_ABSENT,
  DOT_ENV_PATH,
} from './opmls.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the opml-text module.
 */
const l = tagged({
  tag: 'opml-text',
  l: parentLogger,
},);

//region OPML text fetching: Retrieves raw OPML content from HTTP and file URLs

/**
 Fetches OPML file contents from all configured source URLs.
 Handles HTTP(S) and file:// protocols, discarding unreachable sources with warnings.
 Relative `file://` paths resolve against {@link DOT_ENV_PATH}, throwing when
 it is {@link DOT_ENV_ABSENT}.
 
 @param opmls - Validated OPML source URLs
 
 @returns Array of raw OPML XML strings
 
 @mutates opmls - `mapIterableAsync` may invoke caller-owned iterator capabilities and passes reachable source values to `fetchOpml`.
 
 @example
 ```ts
 const texts = await getOPMLTexts(getOpmls());
 ```
 */
export async function getOPMLTexts(
  opmls: string[],
): Promise<string[]> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: getOPMLTexts.name,
    l,
  },);
  /**
   Unique sentinel returned for fetch/read failures so the filter step can drop them.
   */
  const DISCARD = Symbol('opml fetch or read failed',);
  /**
   Successfully fetched OPML texts left after dropping DISCARD entries.
   */
  const result = (await mapIterableAsync({
    fn: async function fetchOpml(
      opmlLink: string,
    ): Promise<string | typeof DISCARD> {
      if (opmlLink.startsWith('http',)) {
        /**
         Single Response held so status check and text read share one network round trip.
         */
        const response = await fetch(opmlLink,);
        if (!response.ok) {
          innerL.warn(`${opmlLink} responded ${String(response.status,)}`,);
          return DISCARD;
        }
        try {
          return await response.text();
        }
        catch (error) {
          innerL.warn(
            `text conversion failed for ${opmlLink}: ${JSON.stringify(error,)}`,
          );
          return DISCARD;
        }
      }
      if (opmlLink.startsWith('file',)) {
        if (opmlLink.startsWith('file:///',)) {
          try {
            return await readFile(
              fileURLToPath(opmlLink,),
              'utf8',
            );
          }
          catch (error) {
            innerL.warn(`failed reading ${opmlLink}: ${JSON.stringify(error,)}`,);
            return DISCARD;
          }
        }
        if (DOT_ENV_PATH === DOT_ENV_ABSENT)
          throw new Error(
            'cannot resolve relative file:// OPML path without a discoverable .env',
          );
        /**
         Relative file path resolved against the .env directory so config-local paths work.
         */
        const absPath = resolve(
          dirname(DOT_ENV_PATH,),
          opmlLink
            .slice('file://'.length,),
        );
        try {
          return await readFile(
            absPath,
            'utf8',
          );
        }
        catch (error) {
          innerL.warn(
            `failed reading ${opmlLink} at ${absPath}: ${JSON.stringify(error,)}`,
          );
          return DISCARD;
        }
      }
      innerL.warn(`${opmlLink} uses unsupported protocol`,);
      return DISCARD;
    },
    iterable: opmls,
  },))
    .filter(function notDiscard(text,): text is string {
      return text !== DISCARD;
    },);
  innerL.debug(`fetched ${String(result.length,)} OPML texts`,);
  return result;
}

//endregion OPML text fetching
